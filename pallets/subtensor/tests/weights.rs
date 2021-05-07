mod mock;
use mock::*;
use mock::{TestXt};
use pallet_subtensor::{Call as SubtensorCall, Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;





#[test]
fn test_set_weights_slots_are_cleared_every_block() {
	new_test_ext().execute_with(|| {
	    assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
		// Fill some slots
		for i in 0..5{
			SubtensorModule::fill_set_weights_slot(i, 5_000);
		}

		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 5);
		run_to_block(1);

		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
	});
}


/***************************
  pub fn set_weights() tests
*****************************/

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
fn test_set_weights_focus_on_transaction_fees() {
	new_test_ext().execute_with(|| {
		let w_uids = vec![1, 2]; // When applied to neuron_1, this will set 50% to himself and 50% to neuron_2
		let w_vals = vec![50, 50]; // The actual numbers are irrelevant for this test though

		let neuron_1_id = 1;
		let neuron_2_id = 2;

		let block_reward = 500_000_000;
		let neuron_1_stake = 1_000_000_000;  // This is the stake that will be given to neuron 1
		let expected_transaction_fee_pool = 0; // This operation is free, so no stake should be added to the transaction fee pool
		let expected_neuron_1_stake = neuron_1_stake + block_reward;

		let _adam    = subscribe_ok_neuron(0, 666);
		let _neuron1 = subscribe_ok_neuron(neuron_1_id, 666); // uid 1
		let _neuron2 = subscribe_ok_neuron(neuron_2_id, 666);

		// Add 1 Tao to neuron 1. He now hold 100% of the stake, so will get the full emission,
		// also he only has a self_weight.

		SubtensorModule::add_stake_to_neuron(_neuron1.uid, neuron_1_stake);

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
		let neuron_1_new_stake = SubtensorModule::get_neuron_stake(neuron_1_id);

		assert_eq!(transaction_fees_pool, expected_transaction_fee_pool);
		assert_eq!(neuron_1_new_stake, expected_neuron_1_stake);  // Neuron 1 maintains his original stake + 100% of the block reward
	});
}


/*********************************
  pub fn set_weights_v1_1_0 tests
**********************************/
#[test]
fn test_set_weights_v1_1_0_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
	    let w_uids = vec![1, 1];
		let w_vals = vec![1, 1];
		let transaction_fee = 100;

		let call = Call::SubtensorModule(SubtensorCall::set_weights_v1_1_0(w_uids, w_vals, transaction_fee));

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_set_weights_v1_1_0_focus_on_transaction_fees() {
	new_test_ext().execute_with(|| {
		let w_uids = vec![1, 2]; // When applied to neuron_1, this will set 50% to himself and 50% to neuron_2
		let w_vals = vec![50, 50]; // The actual numbers are irrelevant for this test though

		let neuron_1_id = 1;
		let neuron_2_id = 2;

		let block_reward = 500_000_000;
		let neuron_1_stake = 1_000_000_000;  // This is the stake that will be given to neuron 1
		let transaction_fee = 1000; // The fee that will be paid for the operation
		let expected_transaction_fee_pool = transaction_fee; // This operation is free, so no stake should be added to the transaction fee pool
		let expected_neuron_1_stake = neuron_1_stake + block_reward - transaction_fee;

		let _adam    = subscribe_ok_neuron(0, 666);
		let _neuron1 = subscribe_ok_neuron(neuron_1_id, 666); // uid 1
		let _neuron2 = subscribe_ok_neuron(neuron_2_id, 666);

		// Add 1 Tao to neuron 1. He now hold 100% of the stake, so will get the full emission,
		// also he only has a self_weight.

		SubtensorModule::add_stake_to_neuron(_neuron1.uid, neuron_1_stake);

		// Move to block, to build up pending emission
		mock::run_to_block(1); // This will emit .5 TAO to neuron 1, since he has 100% of the total stake
		// Verify transacion fee pool == 0
		assert_eq!(SubtensorModule::get_transaction_fee_pool(), 0);

		// Define the call
		let call = Call::SubtensorModule(SubtensorCall::set_weights_v1_1_0(w_uids, w_vals, transaction_fee));

		// Setup the extrinsic
		let xt = TestXt::new(call, mock::sign_extra(_neuron1.uid,0)); // Apply t

		// Execute. This will trigger the set_weights function to emit, before the new weights are set.
		// Resulting in neuron1 getting 99% of the block reward and 1% going to the transaction pool
		let result = mock::Executive::apply_extrinsic(xt);

		// Verfify success
		assert_ok!(result);

		let transaction_fees_pool = SubtensorModule::get_transaction_fee_pool();
		let neuron_1_new_stake = SubtensorModule::get_neuron_stake(neuron_1_id);

		assert_eq!(transaction_fees_pool, expected_transaction_fee_pool);
		assert_eq!(neuron_1_new_stake, expected_neuron_1_stake);  // Neuron 1 maintains his original stake + 100% of the block reward
	});
}





/*****************************
  pub fn do_set_weights tests
*****************************/





/**
* This test the situation where user tries to set weights, but the vecs are empty.
* After setting the weights, the wi
*/
#[test]
fn do_set_weights_ok_no_weights() {
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
		SubtensorModule::add_stake_to_neuron(neuron.uid, initial_stake);

		// Dispatch a signed extrinsic, setting weights.
		assert_ok!(SubtensorModule::do_set_weights(Origin::signed(hotkey_account_id), weights_keys, weight_values, 0));
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (expect_keys, expect_values));
		assert_eq!(SubtensorModule::get_neuron_stake(neuron.uid), expect_stake);
		assert_eq!(SubtensorModule::get_total_stake(), expect_total_stake);
	});
}

#[test]
fn do_set_weights_ok_with_weights() {
	let hotkey_account_1 = 1;
	let hotkey_account_2 = 2;
	let hotkey_account_3 = 3;
	let coldkey_account = 66;

	new_test_ext().execute_with(|| {
		let neurons = vec![
			subscribe_ok_neuron(hotkey_account_1, coldkey_account),
			subscribe_ok_neuron(hotkey_account_2, coldkey_account),
			subscribe_ok_neuron(hotkey_account_3, coldkey_account)
			// subscribe_neuron(55, 10, 666, 4, 0, 66),
			// subscribe_neuron(66, 10, 666, 4, 0, 66),
			// subscribe_neuron(77, 10, 666, 4, 0, 66)
		];

		let initial_stakes = vec![10000,0,0];

		let weight_uids = vec![neurons[1].uid, neurons[2].uid];
		let weight_values = vec![u32::MAX / 2, u32::MAX / 2]; // Set equal weights to ids 2,3

		// Expectations
		let expect_weight_uids = vec![neurons[1].uid, neurons[2].uid];
		let expect_weight_values = vec![u32::MAX / 2, u32::MAX / 2];

		// Dish out the stake for all neurons
		for (i, neuron) in neurons.iter().enumerate() {
			SubtensorModule::add_stake_to_neuron(neuron.uid, initial_stakes[i]);
		}

		// Perform tests

		// First call to set the weights. An emit is triggered, but since there are no weights, no emission occurs
		assert_ok!(SubtensorModule::do_set_weights(Origin::signed(hotkey_account_1), weight_uids.clone(), weight_values.clone(), 0));

		// Increase the block number to trigger emit. It starts at block 0
		run_to_block(1);

		// Second set weights. This should cause inflation to be distributed and end up in hotkey accounts.
		assert_ok!(SubtensorModule::do_set_weights(Origin::signed(hotkey_account_1), weight_uids.clone(), weight_values.clone(),0));
		assert_eq!(SubtensorModule::get_weights_for_neuron(&neurons[0]), (expect_weight_uids, expect_weight_values));

		let mut stakes: Vec<u64> = vec![];
		for neuron in neurons {
			stakes.push(SubtensorModule::get_neuron_stake(neuron.uid));
		}

		assert_eq!(stakes[0], initial_stakes[0]); // Stake of sender should remain unchanged
		assert!(stakes[1] >  initial_stakes[1]); // The stake of destination 1 should have increased
		assert!(stakes[2] >  initial_stakes[2]); // The stake destination 2 should habe increased
		assert_eq!(stakes[1], stakes[2]); // The stakes should have increased the same
	});
}

#[test]
fn test_do_set_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);

		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5]; // Uneven sizes

		let result = SubtensorModule::do_set_weights(Origin::signed(666), weights_keys, weight_values, 0);

		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

#[test]
fn test_do_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);
		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];

		let result = SubtensorModule::do_set_weights(Origin::signed(666), weights_keys, weight_values, 0);

		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

#[test]
fn test_do_set_weights_no_signature() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<AccountId> = vec![];
		let weight_values: Vec<u32> = vec![];

		let result = SubtensorModule::do_set_weights(Origin::none(), weights_keys, weight_values, 0);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

#[test]
fn test_do_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6];

		let result = SubtensorModule::do_set_weights(Origin::signed(1), weights_keys, weight_values, 0);

		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}


#[test]
fn test_do_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {
        let _neuron = subscribe_neuron(55, 33, 55, 4, 0, 66);
		let weight_keys : Vec<AccountId> = vec![9999999999]; // Does not exist
		let weight_values : Vec<u32> = vec![88]; // random value

		let result = SubtensorModule::do_set_weights(Origin::signed(55), weight_keys, weight_values, 0);

		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));

	});
}


/*********************************************
  pub fn has_available_set_weights_slot tests
*********************************************/
#[test]
fn test_has_available_set_weights_slot_yes() {
	new_test_ext().execute_with(|| {
	    assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
		assert!(SubtensorModule::has_available_set_weights_slot());
	});
}

#[test]
fn test_has_available_set_weights_slot_no() {
	new_test_ext().execute_with(|| {
		// Fill 100 slots
		for i in 0..100 {
			SubtensorModule::fill_set_weights_slot(i, 5_000);
		}

		assert!(!SubtensorModule::has_available_set_weights_slot());
	});
}

/********************************************
  pub fn inc_set_weights_slot_counter tests
********************************************/

#[test]
fn test_inc_set_weights_slot_counter() {
	new_test_ext().execute_with(|| {
		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
		SubtensorModule::inc_set_weights_slot_counter();
		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 1);
	});
}


/************************************
  pub fn fill_set_weights_slot tests
*************************************/
#[test]
fn test_fill_set_weights_slot() {
	new_test_ext().execute_with(|| {
		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
		SubtensorModule::fill_set_weights_slot(30, 5_000);
		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 1);
	});
}


/*************************************
  pub fn clear_set_weights_slots tests
**************************************/
#[test]
fn test_clear_set_weights_slots() {
	new_test_ext().execute_with(|| {
		for i in 0..5 {
			SubtensorModule::fill_set_weights_slot(i, 5_000);
		}

		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 5);

		SubtensorModule::clear_set_weights_slots();
		assert_eq!(SubtensorModule::get_set_weights_slot_counter(), 0);
	});
}



