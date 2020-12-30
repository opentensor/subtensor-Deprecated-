use pallet_subtensor::{Error, NeuronMetadata};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;

use frame_system as system;

fn random_neuron_with_stake(hotkey:u64, stake_to_init: u64, ip:u128, port:u16, ip_type:u8, coldkey:u64) -> NeuronMetadata<u64> {
    let _ = SubtensorModule::subscribe(<<Test as system::Trait>::Origin>::signed(hotkey), ip, port, ip_type, coldkey);
    let neuron = SubtensorModule::get_neuron_metadata_for_hotkey(&hotkey);

    // Let's give this neuron an initial stake.
    SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, stake_to_init); // Add the stake.
    assert_eq!(stake_to_init, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.
    neuron
}

// Tests that the self emission returns the correct values with a single node.
#[test]
fn test_self_emission() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey:u64 = 1;
        let stake:u64 = 1000000000;
        let neuron = random_neuron_with_stake(hotkey, stake, 1, 1, 1, 1);

        // Let's set this neuron's weights. (0,0) = 1
		let weight_uids = vec![neuron.uid];
		let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (weight_uids, weight_values)); // Check the weights are set.

        // Let's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(stake, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number to trigger emit. It starts at block 0.
        run_to_block(1);
        
        // Let's call an emit. Causes the new node to mint 500000000 to himself.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(1500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.
	});
}


// Tests that emitting twice in the same block does not increase the stake emitted.
#[test]
fn test_multiemit_per_block() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey:u64 = 1;
        let stake:u64 = 1000000000;
        let neuron = random_neuron_with_stake(hotkey, stake, 1, 1, 1, 1);

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
        let neuron_one = random_neuron_with_stake(hotkey_one, stake, 1, 1, 1, 1); 
        let neuron_two = random_neuron_with_stake(hotkey_two, 0, 1, 1, 1, 1);

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
