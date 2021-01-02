use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;

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


/********************************************
	subscribing::subscribe() tests
*********************************************/

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

		let ip = ipv6(0,0,0,0,0,0,1,1); // Ipv6 localhost, invalid
		let ip_type = 6;
		let port = 1337;
		let coldkey_account_id = 667;


        let result = SubtensorModule::subscribe(<<Test as Trait>::Origin>::none(), ip, port, ip_type, coldkey_account_id);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

/********************************************
	subscribing::get_next_uid() tests
*********************************************/
#[test]
fn test_unsubscribe_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
		let ip = ipv6(0,0,0,0,0,0,0,8); // Ipv6 localhost, invalid
		let ip_type = 6;
		let port = 1337;
		let coldkey_account_id = 667;


		// Setup neuron
		let _ = subscribe_neuron(hotkey_account_id, ip, port, ip_type, coldkey_account_id);

		assert_ok!(SubtensorModule::unsubscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id)));
	});
}

#[test]
fn test_unsubscribe_err_not_active() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 777; // Not existing
		let result = SubtensorModule::unsubscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id));
		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}




/********************************************
	subscribing::increase_neuron_count() tests
*********************************************/
#[test]
fn test_incease_neuron_count_ok() {
	new_test_ext().execute_with(|| {
        assert_eq!(SubtensorModule::get_neuron_count(), 0); // Neuron count should be zero at first
		SubtensorModule::increase_neuron_count();
		assert_eq!(SubtensorModule::get_neuron_count(), 1);
	});
}

#[test]
fn test_decrease_neuron_count_ok() {
	new_test_ext().execute_with(|| {
        SubtensorModule::set_neuron_count(4);
		SubtensorModule::decrease_neuron_count();
		assert_eq!(SubtensorModule::get_neuron_count(), 3);
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
		let coldkey = 66;

        let neuron = subscribe_neuron(account_id, ip, port, ip_type,coldkey);
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
		let uid = 666;

		let neuron = SubtensorModule::add_neuron_to_metagraph(ip,port, ip_type, coldkey, &account_id, uid);

		assert_eq!(neuron.ip, ip);
		assert_eq!(neuron.port, port);
		assert_eq!(neuron.ip_type, ip_type);
		assert_eq!(neuron.coldkey, coldkey);
		assert_eq!(neuron.uid, 666);

		let neuron_in_storage = SubtensorModule::get_neuron_metadata_for_hotkey(&55);
		assert_eq!(neuron_in_storage.ip, ip);
		assert_eq!(neuron_in_storage.port, port);
		assert_eq!(neuron_in_storage.ip_type, ip_type);
		assert_eq!(neuron_in_storage.coldkey, coldkey);
		assert_eq!(neuron_in_storage.uid, 666);
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