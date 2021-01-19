use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;

/********************************************
	subscribing::subscribe() tests
*********************************************/


#[test]
fn test_subscribe_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id));
		let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);

		// Check uid setting functionality
		assert_eq!(neuron.uid, 0);

		// Check if metadata is set correctly
		assert_eq!(neuron.ip, ip);
		assert_eq!(neuron.ip_type, ip_type);
		assert_eq!(neuron.port, port);
		assert_eq!(neuron.coldkey, coldkey_account_id);

		// Check if this function works
		assert_eq!(SubtensorModule::is_uid_active(neuron.uid), true);

		// Check neuron count increment functionality
        assert_eq!(SubtensorModule::get_neuron_count(), 1);

		// Check if weights are set correctly. Only self weight
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (vec![neuron.uid], vec![u32::MAX]));

		// Check if the neuron has a hotkey account
		assert_eq!(SubtensorModule::has_hotkey_account(&neuron.uid), true);

		// Check if the balance of this hotkey account == 0
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);
	});
}

#[test]
fn test_subscriptions_per_block() {
	new_test_ext().execute_with(|| {
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667; // N
		for i in 0..= 24 {
			assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(i), ip, port, ip_type, modality, coldkey_account_id));
		}
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(25), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::ToManySubscriptionsThisBlock.into()));
		run_to_block(1);
		for i in 0..= 24 {
			assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(26 + i), ip, port, ip_type, modality, coldkey_account_id));
		}
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(52), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::ToManySubscriptionsThisBlock.into()));
	});
}


#[test]
fn test_invalid_modality() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 1;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::InvalidModality.into()));
	});
}

/// This test tests the following
/// Given an already subscribed neuron, resubscribing should not
/// change the last emit data.
#[test]
fn test_subsribe_resubrice_emit_does_not_change() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 1;
		let coldkey_id = 2;

		// Move the block_nr to some point in the future
		run_to_block(10);

		let mut neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);
		// The last_emit_block should be 10

		assert_eq!(SubtensorModule::get_last_emit_for_neuron(neuron.uid), 10);

		// Let's move the block counter again to simulate a jump
		run_to_block(100);

		// A subsequent call to subscribe should not change the last emit
		neuron = subscribe_ok_neuron(hotkey_id, coldkey_id);
		assert_eq!(SubtensorModule::get_last_emit_for_neuron(neuron.uid), 10);
	});
}

#[test]
fn test_subscribe_update_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id));
		let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);

		// Check uid setting functionality
		assert_eq!(neuron.uid, 0);

		// Check if metadata is set correctly
		assert_eq!(neuron.ip, ip);
		assert_eq!(neuron.ip_type, ip_type);
		assert_eq!(neuron.port, port);
		assert_eq!(neuron.coldkey, coldkey_account_id);

		// Check neuron count increment functionality
        assert_eq!(SubtensorModule::get_neuron_count(), 1);

		// Check if weights are set correctly. Only self weight
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (vec![neuron.uid], vec![u32::MAX]));

		// Check if the neuron has a hotkey account
		assert_eq!(SubtensorModule::has_hotkey_account(&neuron.uid), true);

		// Check if the balance of this hotkey account == 0
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

		// Subscribe again, this time an update. hotkey and cold key are the same.
 		let new_ip = ipv6(0,0,0,0,0,0,1,1);  // off by one.
		let new_ip_type = 6; // change to 6.
		let new_port = port + 1; // off by one.
		let new_modality = modality; // off by once
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), new_ip, new_port, new_ip_type, new_modality, coldkey_account_id));
		let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);

		// UID, coldkey and hotkey are the same.
		assert_eq!(neuron.uid, 0);
		assert_eq!(neuron.hotkey, hotkey_account_id);
		assert_eq!(neuron.coldkey, coldkey_account_id);

		// metadata has changed
		assert_eq!(neuron.ip, new_ip);
		assert_eq!(neuron.ip_type, new_ip_type);
		assert_eq!(neuron.port, new_port);
		assert_eq!(neuron.modality, new_modality);

		// Check neuron count increment functionality
		assert_eq!(SubtensorModule::get_neuron_count(), 1);

		// Check the weights are unchanged.
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (vec![neuron.uid], vec![u32::MAX]));

		// Check the neuron still exists.
		assert_eq!(SubtensorModule::has_hotkey_account(&neuron.uid), true);

		// Check the stake is unchanged.
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

	});
}

#[test]
fn test_subscribe_update_coldkey_modality_not_changed_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id));

		// Subscribe again, this time an update. hotkey and cold key are the same.
		let new_coldkey_account_id = 668; // The other neighbor, much nicer guy this one.
 		let new_ip = ipv6(0,0,0,0,0,0,1,1);  // off by one.
		let new_ip_type = 6; // change to 6.
		let new_port = port + 1; // off by one.
		let new_modality = modality; // has to be modality 0.
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), new_ip, new_port, new_ip_type, new_modality, new_coldkey_account_id));
		let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);

		// UID, modality, coldkey and hotkey are the same.
		assert_eq!(neuron.uid, 0);
		assert_eq!(neuron.hotkey, hotkey_account_id);
		assert_eq!(neuron.coldkey, coldkey_account_id);
		assert_eq!(neuron.modality, modality);

		// metadata has changed
		assert_eq!(neuron.ip, new_ip);
		assert_eq!(neuron.ip_type, new_ip_type);
		assert_eq!(neuron.port, new_port);

		// Check neuron count increment functionality
		assert_eq!(SubtensorModule::get_neuron_count(), 1);

		// Check the weights are unchanged.
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (vec![neuron.uid], vec![u32::MAX]));

		// Check the neuron still exists.
		assert_eq!(SubtensorModule::has_hotkey_account(&neuron.uid), true);

		// Check the stake is unchanged.
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

	});
}



#[test]
fn test_subscribe_already_active() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id);
		assert_ok!(result);

		// The second should fail when using the same hotkey account id
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id));
	});
}

#[test]
fn test_subscribe_failed_invalid_ip_type() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(127,0,0,1);
		let ip_type = 10;  // Not 4 or 6
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::InvalidIpType.into()));
	});
}

#[test]
fn test_subscribe_failed_invalid_ip_address() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv6(0,0,0,0,0,0,0,1); // Ipv6 localhost, invalid
		let ip_type = 6;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::InvalidIpAddress.into()));
	});
}

#[test]
fn test_subscribe_failed_no_signature() {
	new_test_ext().execute_with(|| {

		let ip = ipv6(0,0,0,0,0,0,1,1); // Ipv6 localhost, valid
		let ip_type = 6;
		let port = 1337;
		let modality = 0;
		let coldkey_account_id = 667;


        let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::none(), ip, port, ip_type, modality, coldkey_account_id);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}



/********************************************
	subscribing::init_weight_matrix_for_neuron() tests
*********************************************/
#[test]
fn test_init_weight_matrix_for_neuron() {
	new_test_ext().execute_with(|| {
		let account_id = 55;
		let ip = ipv4(8,8,8,8);
		let port = 55;
		let ip_type = 4;
		let modality = 0;
		let coldkey = 66;

        let neuron = subscribe_neuron(account_id, ip, port, ip_type, modality, coldkey);
		SubtensorModule::init_weight_matrix_for_neuron(&neuron);
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (vec![neuron.uid], vec![u32::MAX]));
	});
}


/********************************************
	subscribing::add_neuron_to_metagraph() tests
*********************************************/
#[test]
fn test_add_neuron_to_metagraph_ok() {
	new_test_ext().execute_with(|| {
        let account_id = 55;
		let ip = ipv4(8,8,8,8);
		let port = 55;
		let ip_type = 4;
		let coldkey = 66;
		let modality = 0;

		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(account_id), ip, port, ip_type, modality, coldkey));
		let neuron = SubtensorModule::get_neuron_for_hotkey(&account_id);
		assert_eq!(neuron.ip, ip);
		assert_eq!(neuron.port, port);
		assert_eq!(neuron.ip_type, ip_type);
		assert_eq!(neuron.coldkey, coldkey);
		assert_eq!(neuron.modality, modality);
	});
}

/********************************************
	subscribing::get_next_uid() tests
*********************************************/
#[test]
fn test_get_next_uid() {
	new_test_ext().execute_with(|| {
        assert_eq!(SubtensorModule::get_next_uid(), 0); // We start with id 0
		assert_eq!(SubtensorModule::get_next_uid(), 1); // One up
		assert_eq!(SubtensorModule::get_next_uid(), 2) // One more
	});
}