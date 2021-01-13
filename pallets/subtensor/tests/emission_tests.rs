use pallet_subtensor::{NeuronMetadata};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;

use frame_system as system;
use substrate_fixed::types::U64F64;

fn random_neuron_with_stake(hotkey:u64, stake_to_init: u64, ip:u128, port:u16, ip_type:u8, modality: u8, coldkey:u64) -> NeuronMetadata<u64> {
    let _ = SubtensorModule::subscribe(<<Test as system::Trait>::Origin>::signed(hotkey), ip, port, ip_type, modality, coldkey);
    let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey);

    // Let's give this neuron an initial stake.
    SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, stake_to_init); // Add the stake.
    assert_eq!(stake_to_init, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.
    neuron
}

// Tests that the self emission returns the correct values with a single node.
#[test]
fn test_emit_does_not_exist() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let neuron = NeuronMetadata::<u64> {
            ip: 0,
            port: 0,
            ip_type: 0,
            uid: 0,
            modality: 0,
            hotkey: 0,
            coldkey: 0,
        };
        // Let's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
	});
}

// Tests that the self emission returns the correct values with a single node.
#[test]
fn test_self_emission() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey:u64 = 1;
        let stake:u64 = 1000000000;
        let neuron = random_neuron_with_stake(hotkey, stake, ipv4(8,8,8,8), 1, 4, 0, 1);

        // Let's set this neuron's weights. (0,0) = 1
		let weight_uids = vec![neuron.uid];
		let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (weight_uids, weight_values)); // Check the weights are set.

        // Left's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(stake, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number to trigger emit. It starts at block 0.
        run_to_block(1);
        
        // Let's call an emit. Causes the new node to mint 500000000 to himself.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(1500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number to trigger emit. It starts at block 0.
        run_to_block(2);

        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(2000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.
          
	});
}


// Tests that emitting twice in the same block does not increase the stake emitted.
#[test]
fn test_multiemit_per_block() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey:u64 = 1;
        let stake:u64 = 1000000000;
        let neuron = random_neuron_with_stake(hotkey, stake, ipv4(8,8,8,8), 1, 4, 0, 1);

        // Let's set this neuron's weights. (0,0) = 1
		let weight_uids = vec![neuron.uid];
		let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (weight_uids, weight_values)); // Check the weights are set.

        // Let's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // NOTE: not rolling block forward!!

        // Let's call emit again.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

	});
}


// Tests that one node emitting to another produces the correct ammount.
#[test]
fn test_emission_to_other() {
	new_test_ext().execute_with(|| {
        // Let's subscribe two neurons.
        let hotkey_one:u64 = 1;
        let hotkey_two:u64 = 2;
        let stake:u64 = 1000000000;
        let neuron_one = random_neuron_with_stake(hotkey_one, stake, ipv4(8,8,8,8), 1, 4, 0, 1);
        let neuron_two = random_neuron_with_stake(hotkey_two, 0, ipv4(8,8,8,9), 1, 4, 0, 1);

        // Let's set this neuron's weights. (0,0) = 1
		let weight_uids = vec![neuron_two.uid];
		let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey_one), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron_one), (weight_uids, weight_values)); // Check the weights are set.

        // Let's call an emit at block 1
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron_one);
        assert_eq!(total_emission, 0);
        assert_eq!(0, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_two.uid)); // Check that the stake is there.

        // Increase the block number by 1.
        run_to_block(1);
        
        // Let's call an emit. Causes the new node to mint 500000000 to the other guy.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron_one);
        assert_eq!(total_emission, 500000000);
        assert_eq!(500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_two.uid)); // Check that the stake is there.

        // Increase the block number by 2.
        run_to_block(3);

         // Let's call an emit. Causes the new node to mint 500000000 to the otehr guy.
         let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron_one);
         assert_eq!(total_emission, 666666666); // because neuron 1 only has 2/3 of the stake.
         assert_eq!(1166666666, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_two.uid)); // Check that the stake is there.
	});
}

// Tests that removing the weights causes a zero emit.
#[test]
fn test_self_weight() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey:u64 = 1;
        let stake:u64 = 1000000000;
        let neuron = random_neuron_with_stake(hotkey, stake, ipv4(8,8,8,8), 1, 4, 0, 1);

        // Let's set this neuron's weights. (0,0) = 1
        let weight_uids = vec![neuron.uid];
        let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (weight_uids, weight_values)); // Check the weights are set.

        // Let's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number by 1
        run_to_block(1);
  
        // Let's call emit again.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(1500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number by 1
        run_to_block(2);

        // Let's call emit again.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(2000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

	});
}



// Tests a k graph emission.
#[test]
fn test_many_with_weights() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        
        let n = 25;
        let mut neurons: Vec<NeuronMetadata<u64>> = vec![];
        for i in 0..n {
			neurons.push(subscribe_neuron(i as u64, 10, 666, 4, 0, 66));
        }
        let mut stakes = vec![];
        for (_, _) in neurons.iter().enumerate(){
			stakes.push(1000000000);
        }
        println!("{:?}", stakes);
        let mut weight_uids = vec![];
        for (_, _) in neurons.iter().enumerate(){
            let mut uids = vec![];
            for (i, _) in neurons.iter().enumerate(){
                uids.push(neurons[ i ].uid);
            }
            weight_uids.push(uids);
        }
        let mut weight_vals = vec![];
        for (_, _) in neurons.iter().enumerate() {
            let mut vals = vec![];
            for (_, _) in neurons.iter().enumerate(){
                vals.push(u32::MAX / n as u32 );
            }
            weight_vals.push(vals);
		}
		for (i, neuron) in neurons.iter().enumerate() {
			SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, stakes[i]);
        }
        for (i, neuron) in neurons.iter().enumerate() {
		    assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(neuron.uid), weight_uids[i].clone(), weight_vals[i].clone()));
        }
        let mut emission_per_neuron = vec![];
        for (_, neuron) in neurons.iter().enumerate() {
		    emission_per_neuron.push(SubtensorModule::emit_for_neuron(&neuron));
        }
        for (i, _) in neurons.iter().enumerate() {
            assert_eq!(emission_per_neuron[i], 0);
        }
        for (i, neuron) in neurons.iter().enumerate(){
			assert_eq!(stakes[i], SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid));
        }
        run_to_block(2 * n);
        let mut emission_per_neuron = vec![];
        for (_, neuron) in neurons.iter().enumerate() {
		    emission_per_neuron.push(SubtensorModule::emit_for_neuron(&neuron));
        }
        for (i, _) in neurons.iter().enumerate() {
            assert!( close( emission_per_neuron[i], 1000000000, 100 ));
        }
        for (i, neuron) in neurons.iter().enumerate(){
			assert!( close( stakes[i] + 1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 100 ));
        }
        
	});
}


/************************************************************
	emission::can_emission_proceed() tests
************************************************************/
#[test]
fn test_can_emission_proceed_ok() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(40);
        let weight_uids  = vec![1,2];
        let weight_vals =vec![54434,90099];

        assert_eq!(SubtensorModule::can_emission_proceed(&emission, &weight_uids, &weight_vals), true)
	});
}

#[test]
fn test_can_emission_proceed_fails_no_emission() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(0);
        let weight_uids  = vec![1,2];
        let weight_vals =vec![54434,90099];

        assert_eq!(SubtensorModule::can_emission_proceed(&emission, &weight_uids, &weight_vals), false)

	});
}

#[test]
fn test_can_emission_proceed_fails_no_uids() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(0);
        let weight_uids  = vec![]; // Empty, illegal state
        let weight_vals =vec![54434,90099];

        assert_eq!(SubtensorModule::can_emission_proceed(&emission, &weight_uids, &weight_vals), false)
	});
}

#[test]
fn test_can_emission_proceed_fails_no_vals() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(40);
        let weight_uids  = vec![1,2];
        let weight_vals =vec![]; // Empty, illegal state

        assert_eq!(SubtensorModule::can_emission_proceed(&emission, &weight_uids, &weight_vals), false)

	});
}

/************************************************************
	emission::calculate_stake_increment() tests
************************************************************/
#[test]
fn test_calculate_stake_increment_ok() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(2);
        let weight = U64F64::from_num(0.5);

        assert_eq!(SubtensorModule::calculate_stake_increment(emission, weight), 1);
	});
}

#[test]
#[should_panic]
fn test_calculate_stake_increment_fails_weight_out_of_range() {
	new_test_ext().execute_with(|| {
        let emission = U64F64::from_num(2);
        let weight = U64F64::from_num(1.5); // Out of range

        assert_eq!(SubtensorModule::calculate_stake_increment(emission, weight), 1);
	});
}


/************************************************************
	emission::get_pending_emission_for_neuron() tests
************************************************************/
#[test]
fn test_get_pending_emission_for_neuron_ok() {
	new_test_ext().execute_with(|| {
        let uid = 55;
        let pending_emission = 50;

        // Set it up
        SubtensorModule::update_pending_emission_for_neuron(uid, pending_emission);
        assert_eq!(SubtensorModule::get_pending_emission_for_neuron(uid), pending_emission);
	});
}



/************************************************************
	emission::drain_pending_emission_for_neuron() tests
************************************************************/


/************************************************************
	emission::update_last_emit_for_neuron() tests
************************************************************/


/************************************************************
	emission::calculate_new_emission() tests
************************************************************/

/************************************************************
	emission::update_pending_emission_for_neuron() tests
************************************************************/


pub fn close(x:u64, y:u64, d:u64) -> bool {
    if x > y {
        if x - y < d {
            return true;
        }
        else {
            println!("{:?} - {:?} >= {:?}", y, x, d);
            return false;
        }
    }
    if y > x {
        if y - x < d {
            return true;
        }
        else {
            println!("{:?} - {:?} >= {:?}", x, y, d);
            return false;
        }
    }
    true
}