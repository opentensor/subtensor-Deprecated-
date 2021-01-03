use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;

#[test]
fn test_add_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 533453;
		let ip = ipv4(8,8,8,8);
		let port = 66;
		let ip_type = 4;
		let coldkey_account_id = 55453;


		// Subscribe neuron
		let neuron = subscribe_neuron(hotkey_account_id, ip,port,ip_type, coldkey_account_id);

		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_stake_to_coldkey_account(&coldkey_account_id, 10000);

		// Check we have zero staked before transfer
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 0);

		// Also total stake should be zero
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Transfer to hotkey account, and check if the result is ok
		assert_ok!(SubtensorModule::do_add_stake(<<Test as Trait>::Origin>::signed(coldkey_account_id), hotkey_account_id, 10000));

		// Check if stake has increased
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), 10000);

		// Check if balance has  decreased
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Check if total stake has increased accordingly.
		assert_eq!(SubtensorModule::get_total_stake(), 10000);
	});
}

#[test]
fn test_add_stake_err_with_emission() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id = 533453;
		let ip = ipv4(8,8,8,8);
		let port = 66;
		let ip_type = 4;
		let coldkey_account_id = 55453;

		let transfer_amount:u64 = 10000;
		let initial_stake:u64 = 5000;

		// Subscribe neuron
		let neuron = subscribe_neuron(hotkey_account_id, ip,port,ip_type, coldkey_account_id);

		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_stake_to_coldkey_account(&coldkey_account_id, transfer_amount.into());

		// Add some stake to the hotkey account, so we can test for emission before the transfer takes place
		SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stake);

		// Run a couple of blocks to check if emission works
		run_to_block(5);

		// Initiate transfer
		assert_ok!(SubtensorModule::do_add_stake(<<Test as Trait>::Origin>::signed(coldkey_account_id), hotkey_account_id, transfer_amount));

		// Check if the stake is bigger than the inital stake + transfer due to emission.
		assert!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid) > initial_stake + transfer_amount);

		// Check if the balance has been reduced by the transfer amount
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Check if the total stake is bigger than the sum of initial stake + transferred amount
		assert!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid) > initial_stake + transfer_amount);
	});
}

#[test]
fn test_add_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 654; // bogus
		let amount = 20000 ; // Not used

		let result = SubtensorModule::add_stake(<<Test as Trait>::Origin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_add_stake_err_not_active() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 435445; // Not active id
		let hotkey_account_id = 54544;
		let amount = 1337;

		let result = SubtensorModule::add_stake(<<Test as Trait>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
	});
}


#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;

		let _neuron = subscribe_neuron(hotkey_id, ipv4(8, 8, 8, 8), 66, 4, coldkey_id);

		// Perform the request which is signed by a different cold key
		let result = SubtensorModule::add_stake(<<Test as Trait>::Origin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_add_stake_err_not_enough_belance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;


		let _neuron = subscribe_neuron(hotkey_id, ipv4(8, 8, 8, 8), 66, 4, coldkey_id);

		// Lets try to stake with 0 balance in cold key account
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), 0);
		let result = SubtensorModule::add_stake(<<Test as Trait>::Origin>::signed(coldkey_id), hotkey_id, 60000);

		assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
	});
}


/********************************************
	subscribing::remove_stake_from_coldkey_account() tests
*********************************************/


#[test]
fn test_remove_stake_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Put some $$ on the bank
		SubtensorModule::add_stake_to_coldkey_account(&coldkey_account_id, ammount);

		let result = SubtensorModule::remove_stake_from_coldkey_account(&coldkey_account_id,ammount);
		assert_eq!(result, true);



	});
}

#[test]
fn test_remove_stake_from_coldkey_account_failed() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Try to remove stake from the coldkey account. This should fail,
		// as there is no balance, nor does the account exist
		let result = SubtensorModule::remove_stake_from_coldkey_account(&coldkey_account_id,ammount);
		assert_eq!(result, false);
	});
}


