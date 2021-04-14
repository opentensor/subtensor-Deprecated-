mod mock;
use mock::*;
use mock::{TestXt};
use pallet_subtensor::{Call as SubtensorCall, Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;
use fixed::types::U64F64;


/***************************
  pub fn set_weights() tests
*****************************/

// This does not produce the expected result
#[test]
fn test_set_weights_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let w_uids = vec![1, 1];
		let w_vals = vec![1, 1];

		let call = Call::SubtensorModule(SubtensorCall::set_weights(w_uids, w_vals));

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}


#[test]
fn test_set_weights_transaction_fee_pool_and_neuron_receive_funds() {
	new_test_ext().execute_with(|| {
		let w_uids = vec![1, 2]; // When applied to neuron_1, this will set 50% to himself and 50% to neuron_2
		let w_vals = vec![50, 50]; // The actual numbers are irrelevant for this test though

		let neuron_1_id = 1;
		let neuron_2_id = 2;

		let block_reward = U64F64::from_num(500_000_000);
		let neuron_1_stake = 1_000_000_000;  // This is the stake that will be given to neuron 1
		let expected_transaction_fee_pool = (block_reward as U64F64 * U64F64::from_num(0.01)).round().to_num::<u64>(); // This is the stake to be expected after applying set_weights
		let expected_neuron_1_stake = neuron_1_stake + (block_reward * U64F64::from_num(0.99)).round().to_num::<u64>();


		// let bleh = U64F64::from_num(500_000_000) * U64F64::from_num(0.99);
		// assert_eq!(4444,bleh);


		let _adam    = subscribe_ok_neuron(0, 666);
		let _neuron1 = subscribe_ok_neuron(neuron_1_id, 666); // uid 1
		let _neuron2 = subscribe_ok_neuron(neuron_2_id, 666);

		// Add 1 Tao to neuron 1. He now hold 100% of the stake, so will get the full emission,
		// also he only has a self_weight.

		SubtensorModule::add_stake_to_neuron_hotkey_account(_neuron1.uid, neuron_1_stake);

		// Move to block, to build up pending emission
		mock::run_to_block(1); // This will emit .5 TAO to neuron 1, since he has 100% of the total stake
		// Verify transacion fee pool == 0
		assert_eq!(SubtensorModule::get_transaction_fee_pool(), 0);

		// Define the call
		let call = Call::SubtensorModule(SubtensorCall::set_weights(w_uids, w_vals));

		// Setup the extrinsic
		let xt = TestXt::new(call, mock::sign_extra(_neuron1.uid,0)); // Apply t

		// Execute. This will trigger the set_weights function to emit, before the new weights are set.
		// Resulting in neuron1 getting 99% of the block reward and 1% going to the transaction pool
		let result = mock::Executive::apply_extrinsic(xt);

		// Verfify success
		assert_ok!(result);

		let transaction_fees_pool = SubtensorModule::get_transaction_fee_pool();
		let neuron_1_new_stake = SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_1_id);

		assert_eq!(transaction_fees_pool, expected_transaction_fee_pool);
		assert_eq!(neuron_1_new_stake, expected_neuron_1_stake);  // Neuron 1 maintains his original stake + 99% of the block reward
	});
}




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

		let weights_keys : Vec<AccountId> = vec![];
		let weight_values : Vec<u32> = vec![];

		// == Expectations ==

		let expect_keys = vec![]; // keys should not change
		let expect_values = vec![]; // Value should be normalized for u32:max
		let expect_stake = 10000; // The stake for the neuron should remain the same
		let expect_total_stake = 10000; // The total stake should remain the same


		// Let's subscribe a new neuron to the chain
		let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);

		// Let's give it some stake.
		SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stake);

		// Dispatch a signed extrinsic, setting weights.
		assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weights_keys, weight_values));
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (expect_keys, expect_values));
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), expect_stake);
		assert_eq!(SubtensorModule::get_total_stake(), expect_total_stake);
	});
}

#[test]
fn set_weights_ok_with_weights() {
	new_test_ext().execute_with(|| {
		let neurons = vec![
			subscribe_neuron(55, 10, 666, 4, 0, 66),
			subscribe_neuron(66, 10, 666, 4, 0, 66),
			subscribe_neuron(77, 10, 666, 4, 0, 66)
		];

		let initial_stakes = vec![10000,0,0];

		let weight_uids = vec![neurons[1].uid, neurons[2].uid];
		let weight_values = vec![u32::MAX / 2, u32::MAX / 2]; // Set equal weights to ids 2,3

		// Expectations
		let expect_weight_uids = vec![neurons[1].uid, neurons[2].uid];
		let expect_weight_values = vec![u32::MAX / 2, u32::MAX / 2];

		// Dish out the stake for all neurons
		for (i, neuron) in neurons.iter().enumerate() {
			SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stakes[i]);
		}

		// Perform tests

		// First call to set the weights. An emit is triggered, but since there are no weights, no emission occurs
		assert_ok!(SubtensorModule::set_weights(Origin::signed(55), weight_uids.clone(), weight_values.clone()));

		// Increase the block number to trigger emit. It starts at block 0
		run_to_block(1);

		// Second set weights. This should cause inflation to be distributed and end up in hotkey accounts.
		assert_ok!(SubtensorModule::set_weights(Origin::signed(55), weight_uids.clone(), weight_values.clone()));
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neurons[0]), (expect_weight_uids, expect_weight_values));

		let mut stakes: Vec<u64> = vec![];
		for neuron in neurons {
			stakes.push(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid));
		}

		assert_eq!(stakes[0], initial_stakes[0]); // Stake of sender should remain unchanged
		assert!(stakes[1] >  initial_stakes[1]); // The stake of destination 1 should have increased
		assert!(stakes[2] >  initial_stakes[2]); // The stake destination 2 should habe increased
		assert_eq!(stakes[1], stakes[2]); // The stakes should have increased the same
	});
}

#[test]
fn test_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);

		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5]; // Uneven sizes

		let result = SubtensorModule::set_weights(Origin::signed(666), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

#[test]
fn test_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);
		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];

		let result = SubtensorModule::set_weights(Origin::signed(666), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

#[test]
fn test_no_signature() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<AccountId> = vec![];
		let weight_values: Vec<u32> = vec![];

		let result = SubtensorModule::set_weights(Origin::none(), weights_keys, weight_values);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

#[test]
fn test_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6];

		let result = SubtensorModule::set_weights(Origin::signed(1), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}


#[test]
fn test_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(55, 33, 55, 4, 0, 66);
		let weight_keys : Vec<AccountId> = vec![9999999999]; // Does not exist
		let weight_values : Vec<u32> = vec![88]; // random value

		let result = SubtensorModule::set_weights(Origin::signed(55), weight_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));

	});
}



