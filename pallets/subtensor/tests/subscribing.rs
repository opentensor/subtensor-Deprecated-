use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;
use sp_runtime::DispatchError;

#[test]
fn test_subscribe_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		assert_ok!(SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_account_id));
		let neuron = SubtensorModule::get_neuron_metadata_for_hotkey(&hotkey_account_id);

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
	});
}


#[test]
fn test_subscribe_failed_already_active() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
		let ip = ipv4(8,8,8,8);
		let ip_type = 4;
		let port = 1337;
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let mut result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_account_id);
		assert_ok!(result);

		// The second should fail when using the same hotkey account id
		result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::AlreadyActive.into()))
	});
}

#[test]
fn test_subscribe_failed_invalid_ip_type() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 1;
		let ip = ipv4(127,0,0,1);
		let ip_type = 10;  // Not 4 or 6
		let port = 1337;
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_account_id);
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
		let coldkey_account_id = 667;

		// This first subscription should succeed without problems
		let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_account_id);
		assert_eq!(result, Err(Error::<Test>::InvalidIpAddress.into()));
	});
}

#[test]
fn test_subscribe_failed_no_signature() {
	new_test_ext().execute_with(|| {
        
	});
}