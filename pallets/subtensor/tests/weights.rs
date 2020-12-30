use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;


/**
* This test the situation where user tries to set weights, but the vecs are empty.
* After setting the weights, the wi
*/
#[test]
fn set_weights_ok_no_weights() {
	new_test_ext().execute_with(|| {

		// == Intial values ==
		let hotkey_account_id:u64 = 55; // Arbitrary number
		let initial_stake = 10000;

		let weights_keys : Vec<<Test as Trait>::AccountId> = vec![];
		let weight_values : Vec<u32> = vec![];

		// == Expectations ==

		let expect_keys = vec![]; // keys should not change
		let expect_values = vec![]; // Value should be normalized for u32:max
		let expect_stake = 10000;
		let expect_total_stake = 10000;


		// Let's subscribe a new neuron to the chain
		let _ = SubtensorModule::subscribe(<<Test as Trait>::Origin>::signed(hotkey_account_id), 1, 666, 4, 55);

		// Get its metadata. It will be the first neuron, so it's uid is 0;
		let neuron = SubtensorModule::get_neuron_metadata_for_hotkey(&hotkey_account_id);

		// Let's give it some stake.
		SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stake);

		// Also increase to total stake
		SubtensorModule::increase_total_stake(initial_stake);


		// Dispatch a signed extrinsic, setting weights.
		assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey_account_id), weights_keys, weight_values));
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (expect_keys, expect_values));
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account(neuron.uid), expect_stake);
		assert_eq!(SubtensorModule::get_total_stake(), expect_total_stake);
	});
}


#[test]
fn set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<<Test as Trait>::AccountId> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6];

		let result = SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(1), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::NotActive.into()));

	});
}



