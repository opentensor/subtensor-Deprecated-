use pallet_subtensor::{ChargeTransactionPayment, Error, CallType};
use frame_support::{assert_ok};

mod mock;

use mock::*;
use frame_support::weights::{DispatchInfo, Pays};
use frame_support::weights::PostDispatchInfo;
use sp_std::marker::PhantomData;
use sp_runtime::traits::SignedExtension;
use pallet_subtensor::Call as SubtensorCall;
use pallet_balances::{Call as BalanceCall};
use pallet_sudo::{Call as SudoCall};
use sp_runtime::transaction_validity::{InvalidTransaction, ValidTransaction};
use frame_support::dispatch::GetDispatchInfo;

#[test]
fn fee_from_emission_works() {
    new_test_ext().execute_with(|| {
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert!(ChargeTransactionPayment::<Test>(PhantomData).validate(&1, &call, &info, len).is_ok());
    });
}

#[test]
fn fee_from_emission_priority_no_neuron() {
    new_test_ext().execute_with(|| {
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&1, &call, &info, len).unwrap().priority, 0);
    });
}

#[test]
fn fee_from_emission_priority_with_neuron() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
    });
}


#[test]
fn fee_from_emission_priority_with_neuron_and_weights() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        let weight_uids = vec![neuron.uid];
        let weight_values = vec![u32::MAX];
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weight_uids.clone(), weight_values.clone()));

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
    });
}

#[test]
fn fee_from_emission_priority_with_neuron_and_weights_and_stake() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        let weight_uids = vec![neuron.uid];
        let weight_values = vec![u32::MAX];
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weight_uids.clone(), weight_values.clone()));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 100000000); // Add the stake.

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
    });
}



#[test]
fn test_emission_low_priority_but_emission_goes_to_user() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id_1 = 1;
        let _neuron_1 = subscribe_neuron(hotkey_account_id_1, 10, 666, 4, 0, 66);

        let hotkey_account_id_2 = 2;
        let neuron_2 = subscribe_neuron(hotkey_account_id_2, 10, 666, 4, 0, 66);

        let hotkey_account_id_3 = 3;
        let neuron_3 = subscribe_neuron(hotkey_account_id_3, 10, 666, 4, 0, 66);

        let weight_uids = vec![neuron_3.uid];
        let weight_values = vec![u32::MAX];
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id_2), weight_uids.clone(), weight_values.clone()));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron_2.uid, 100000000); // Add the stake.

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id_2, &call, &info, len).unwrap().priority, 0);
        run_to_block(1);
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id_2, &call, &info, len).unwrap().priority, 0);
        let _total_emission: u64 = SubtensorModule::emit_for_neuron(&neuron_2); // actually do the emission.
        assert_eq!(500000000, SubtensorModule::get_neuron_stake(neuron_3.uid));
    });
}

#[test]
fn fee_from_emission_priority_with_neuron_and_adam() {
    new_test_ext().execute_with(|| {
        let adam_account_id = 0;
        let _adam = subscribe_neuron(adam_account_id, 10, 666, 4, 0, 66);
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        let weight_uids = vec![neuron.uid];
        let weight_values = vec![u32::MAX];
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weight_uids.clone(), weight_values.clone()));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1_000_000_000); // Add the stake.

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
        run_to_block(1);

        // Priority should not change over time
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
        assert_eq!(1_000_000_000, SubtensorModule::get_neuron_stake(neuron.uid)); // Check that his stake has not increased.

        let _total_emission: u64 = SubtensorModule::emit_for_neuron(&neuron); // actually do the emission.
        assert_eq!(1_500_000_000, SubtensorModule::get_neuron_stake(neuron.uid)); // Check that his stake has increased (he is *not* adam)
    });
}

/************************************************************	
	ChargeTransactionPayment::can_process_set_weights() tests
************************************************************/
#[test]
fn test_charge_transaction_payment_can_processes_set_weights_ok() {
    let hotkey_id = 0;
    let stake = 1_000_000_000;
    let coldkey_id = 2;
    let transaction_payment:u64 = 10;

    // @todo Figure out whu test_ext_with_stake does no work
    test_ext_with_stake(vec![(0, stake)]).execute_with(|| {
        let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id); // Now has self-weight
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, stake);

        let result = ChargeTransactionPayment::<Test>::can_process_set_weights(&hotkey_id, transaction_payment);
        assert_eq!(result, Ok(10));
    });
}

#[test]
fn test_charge_transaction_payment_can_process_set_weights_failed_no_stake() {
    let hotkey_id = 0;
    let coldkey_id = 2;
    let transaction_payment:u64 = 10;

	new_test_ext().execute_with(|| {
        let _neuron = subscribe_ok_neuron(hotkey_id, coldkey_id); // Now has self-weight
        // Neuron has no stake by default
        let result = ChargeTransactionPayment::<Test>::can_process_set_weights(&hotkey_id, transaction_payment);
        assert_eq!(result, Err(InvalidTransaction::Payment.into()));
	});
}


#[test]
fn test_charge_transaction_payment_can_process_set_weights_failed_no_slots() {
    let hotkey_id = 0;
    let coldkey_id = 2;
    let transaction_payment:u64 = 10;
    let stake = 1_000_000_000;

	new_test_ext().execute_with(|| {
        // Fill 100 slots
        for i in 0..100 {
            SubtensorModule::fill_set_weights_slot(i, 5_000);
        }

        let neuron = subscribe_ok_neuron(hotkey_id, coldkey_id); // Now has self-weight

        // Make sure we have enough stake, so that this won't be a problem.
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, stake);

        let result = ChargeTransactionPayment::<Test>::can_process_set_weights(&hotkey_id, transaction_payment);
        assert_eq!(result, Err(InvalidTransaction::Payment.into()));
	});
}


/************************************************************
	ChargeTransactionPayment::can_process_add_stake() tests
************************************************************/
#[test]
fn test_charge_transaction_payment_can_process_add_stake_ok() {
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 1_000_000_000)]).execute_with(|| {
        let result = ChargeTransactionPayment::<Test>::can_process_add_stake(&coldkey_id, len);
        assert_eq!(result, Ok(20000));
    });
}

#[test]
fn test_charge_transaction_payment_can_process_add_stake_err() {
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 0)]).execute_with(|| {
        let result = ChargeTransactionPayment::<Test>::can_process_add_stake(&coldkey_id, len);
        assert_eq!(result, Err(InvalidTransaction::Payment.into()));
    });
}

/************************************************************
	ChargeTransactionPayment::can_process_remove_stake() tests
************************************************************/
#[test]
fn test_charge_transaction_payment_can_process_remove_stake_ok_enough_balance() {
    let coldkey_id = 0;
    let hotkey_id = 1;
    let len = 200;
    test_ext_with_balances(vec![(coldkey_id, 100_000)]).execute_with(|| {
        let result = ChargeTransactionPayment::<Test>::can_process_remove_stake(&coldkey_id, &hotkey_id, len);
        assert_eq!(result, Ok(20000));
    });
}

#[test]
fn test_charge_transaction_payment_can_process_remove_stake_ok_enough_stake() {
    let coldkey_id = 0;
    let hotkey_id = 1;
    let len = 200;

    new_test_ext().execute_with(|| {
        let adam = subscribe_ok_neuron(hotkey_id, coldkey_id);
        let _ = SubtensorModule::add_stake_to_neuron_hotkey_account(adam.uid, 100_000);
        let result = ChargeTransactionPayment::<Test>::can_process_remove_stake(&coldkey_id, &hotkey_id, len);
        assert_eq!(result, Ok(20000));
    });
}


#[test]
fn test_charge_transaction_payment_can_process_remove_stake_err_insufficient_funds() {
    let hotkey_id = 0;
    let coldkey_id = 1;
    let len = 200;

    new_test_ext().execute_with(|| {
        let adam = subscribe_ok_neuron(hotkey_id, coldkey_id);

        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), 0);
        assert_eq!(SubtensorModule::get_neuron_stake(adam.uid), 0);

        let result = ChargeTransactionPayment::<Test>::can_process_remove_stake(&coldkey_id, &hotkey_id, len);
        assert_eq!(result, Err(InvalidTransaction::Payment.into()));
    });
}

/************************************************************
	ChargeTransactionPayment::can_process_subscribe() tests
************************************************************/
#[test]
fn test_charge_transaction_payment_subscribe_ok() {
    new_test_ext().execute_with(|| {
        let result = ChargeTransactionPayment::<Test>::can_process_subscribe();
        assert_eq!(result, Ok(0));
    });
}

/************************************************************
	ChargeTransactionPayment::can_process_other() tests
************************************************************/
#[test]
fn test_charge_transaction_payment_can_process_other_ok() {
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 100_000_000)]).execute_with(|| {
        let info = DispatchInfo {
            weight: 0,
            class: Default::default(),
            pays_fee: Pays::Yes,
        };

        let result = ChargeTransactionPayment::<Test>::can_process_other(&info, &coldkey_id, len);
        assert_eq!(result, Ok(20000));
    });
}

#[test]
fn test_test_charge_transaction_payment_can_process_other_ok_does_not_pay() {
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 0)]).execute_with(|| {
        let info = DispatchInfo {
            weight: 0,
            class: Default::default(),
            pays_fee: Pays::No,
        };

        let result = ChargeTransactionPayment::<Test>::can_process_other(&info, &coldkey_id, len);
        assert_eq!(result, Ok(20000));
    });
}

#[test]
fn test_charge_transaction_payment_can_process_other_err_no_balance() {
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 0)]).execute_with(|| {
        let info = DispatchInfo {
            weight: 0,
            class: Default::default(),
            pays_fee: Pays::Yes,
        };

        let result = ChargeTransactionPayment::<Test>::can_process_other(&info, &coldkey_id, len);
        assert_eq!(result, Err(InvalidTransaction::Payment.into()));
    });
}



/************************************************************
	ChargeTransactionPayment::get_priority_vanilla() tests
************************************************************/

#[test]
fn test_charge_transaction_payment_get_priority_vanilla() {
    new_test_ext().execute_with(|| {
        assert_eq!(ChargeTransactionPayment::<Test>::get_priority_vanilla(), u64::max_value());
    });
}


/************************************************************
	ChargeTransactionPayment::validate() tests
************************************************************/

#[test]
fn test_charge_transaction_payment_validate_set_weights_ok() {
    let uid = 0;
    let coldkey_id = 0;
    let len = 200;

    test_ext_with_pending_emissions(vec![(uid, 100_000)]).execute_with(|| {
        let _adam = subscribe_ok_neuron(0, coldkey_id);

        let call: mock::Call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = call.get_dispatch_info();

        let result = ChargeTransactionPayment::<Test>(PhantomData).validate(&coldkey_id, &call, &info, len);
        assert_eq!(result, Ok(ValidTransaction {
            priority: 0, // Priority for set_weights function is always 0
            longevity: 1,
            ..Default::default()
        }))
    });
}

#[test]
fn test_charge_transaction_payment_validate_add_stake_ok() {
    let coldkey_id = 0;
    let hotkey_id = 33;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 100_000)]).execute_with(|| {
        let _adam = subscribe_ok_neuron(0, coldkey_id);

        let call: mock::Call = SubtensorCall::add_stake(hotkey_id, 5_000).into();
        let info = call.get_dispatch_info();

        let result = ChargeTransactionPayment::<Test>(PhantomData).validate(&coldkey_id, &call, &info, len);
        assert_eq!(result, Ok(ValidTransaction {
            priority: u64::max_value(),
            longevity: u64::max_value(), // Forevah
            ..Default::default()
        }))
    });
}

#[test]
fn test_charge_transaction_payment_validate_remove_stake_ok() {
    let coldkey_id = 0;
    let hotkey_id = 33;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 100_000)]).execute_with(|| {
        let _adam = subscribe_ok_neuron(0, coldkey_id);

        let call: mock::Call = SubtensorCall::remove_stake(hotkey_id, 5_000).into();
        let info = call.get_dispatch_info();

        let result = ChargeTransactionPayment::<Test>(PhantomData).validate(&coldkey_id, &call, &info, len);
        assert_eq!(result, Ok(ValidTransaction {
            priority: u64::max_value(),
            longevity: u64::max_value(), // Forevah
            ..Default::default()
        }))
    });
}

#[test]
fn test_charge_transaction_payment_validate_subscribe_ok() {
    let coldkey_id = 0;
    let len = 200;
    let balance = 0; // Zero balance, subscribe is free

    test_ext_with_balances(vec![(coldkey_id, balance)]).execute_with(|| {
        let call: mock::Call = SubtensorCall::subscribe(ipv4(8, 8, 8, 8), 666, 4, 0, coldkey_id).into();
        let info = call.get_dispatch_info();

        let result = ChargeTransactionPayment::<Test>(PhantomData).validate(&coldkey_id, &call, &info, len);
        assert_eq!(result, Ok(ValidTransaction {
            priority: u64::max_value(),
            longevity: u64::max_value(), // Forevah
            ..Default::default()
        }))
    });
}

#[test]
fn test_charge_transaction_payment_validate_other_ok() {
    let coldkey_id = 0;
    let dest_id = 4332;
    let len = 200;

    test_ext_with_balances(vec![(coldkey_id, 100_000)]).execute_with(|| {
        let call: mock::Call = BalanceCall::transfer(dest_id, 5_000).into();
        let info = call.get_dispatch_info();

        let result = ChargeTransactionPayment::<Test>(PhantomData).validate(&coldkey_id, &call, &info, len);
        assert_eq!(result, Ok(ValidTransaction {
            priority: u64::max_value(),
            longevity: u64::max_value(), // Forevah
            ..Default::default()
        }))
    });
}


// @todo extend the pre_dispatch tests
#[test]
fn pre_dispatch_set_weights_success() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1_000_000_000); // Add the stake.
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;

        let result = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap();
        assert_eq!(result.0, CallType::SetWeights);
        assert_eq!(result.1, 0);
        assert_eq!(result.2, hotkey_account_id);
    });
}

#[test]
fn post_dispatch_works() {
    new_test_ext().execute_with(|| {
        let adam_id = 0;
        let hotkey_account_id = 1;

        let _adam = subscribe_ok_neuron(adam_id, 667);
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), vec![neuron.uid], vec![u32::MAX]));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        run_to_block(1);

        let result = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len);
        assert_ok!(result);

        let pre = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap();
        // assert!(ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo {actual_weight: Some(0), pays_fee: Default::default()}, len, &Ok(())).is_ok());
        assert!(ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo { actual_weight: Some(1000000000), pays_fee: Default::default() }, len, &Ok(())).is_ok());
    });
}

#[test]
fn test_post_dispatch_does_not_deposit_to_adam_on_error() {
    new_test_ext().execute_with(|| {
        let adam_id = 0;
        let _adam = subscribe_ok_neuron(adam_id, 667);

        // Adam should have no stake before operation
        let result = SubtensorModule::get_neuron_stake(adam_id);
        assert_eq!(result, 0);

        let pre_dispatch_result = Err(Error::<Test>::DuplicateUids.into());
        let pre_dispatch_data = (pallet_subtensor::CallType::SetWeights, 0, 0);

        // let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let post_dispatch_info = PostDispatchInfo { actual_weight: Some(0), pays_fee: Default::default() };
        let len = 100;

        let post_dispatch_result = ChargeTransactionPayment::<Test>::post_dispatch(pre_dispatch_data, &info, &post_dispatch_info, len, &pre_dispatch_result);
        assert_ok!(post_dispatch_result);
        assert_eq!(SubtensorModule::get_neuron_stake(adam_id), 0);
    });
}

#[test]

// @todo reimplement these
// fn post_dispatch_deposit_to_transaction_fee_pool_works() {
//     let hotkey_account_id = 1;
//     let coldkey_account_id = 2;
//     let stake = 1_000_000_000;
//
//     test_ext_with_stake(vec![(hotkey_account_id, stake)]).execute_with(|| {
//         let neuron = subscribe_ok_neuron(hotkey_account_id, coldkey_account_id);
//         let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
//         let info = DispatchInfo::default();
//         let len = 10;
//
//         let pre = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap();
//         let result = ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo { actual_weight: Some(0), pays_fee: Default::default() }, len, &Ok(()));
//         assert_ok!(return);
//
//         assert_eq!(SubtensorModule::get_transaction_fee_pool(), 5_000_000); // 1%
//     });
// }

/************************************************************
    These tests test if the sudo call and other calls
    are handled correctly with regard to transaction fees
************************************************************/

#[test]
fn test_calls_not_from_this_pallet_pay_transacion_fees_when_pays_is_yes() {
    let source_wallet = 0;
    let dest_wallet = 1;

    let initial_balance = 1_000_000_000;
    let amount = 500_000_000;

    test_ext_with_balances(vec![(source_wallet, initial_balance)]).execute_with(|| {
        // let _adam = subscribe_ok_neuron(0, 667); // Register Adam


        let call = Call::Balances(BalanceCall::transfer(dest_wallet, Balance::from(amount as u128)));
        let xt = TestXt::new(call, mock::sign_extra(source_wallet, 0));
        let result = mock::Executive::apply_extrinsic(xt);

        assert_ok!(result);

        assert!(SubtensorModule::get_transaction_fee_pool() > 0);
        assert!(SubtensorModule::get_coldkey_balance(&source_wallet) < initial_balance - amount);
        assert!(SubtensorModule::get_coldkey_balance(&dest_wallet) == amount);
    });
}

#[test]
fn test_sudo_call_does_not_pay_transaction_fee() {
    let source_key_id = 8888;
    let dest_key_id = 99889;
    let balance = 1_000_000_000;
    let amount = 500_000_000;
    let sudo_key = 1;

    test_ext_with_balances(vec![(source_key_id, balance)]).execute_with(|| {
        let adam = subscribe_ok_neuron(0, 7778);

        let call = Box::new(Call::SubtensorModule(SubtensorCall::add_stake(dest_key_id, amount)));

        let sudo_call = Call::Sudo(SudoCall::sudo_unchecked_weight(call, 1000));

        let xt = TestXt::new(sudo_call, mock::sign_extra(sudo_key, 0));
        let result = mock::Executive::apply_extrinsic(xt);
        assert_ok!(result);

        // Verify adam has not received any monies
        assert_eq!(SubtensorModule::get_neuron_stake(adam.uid), 0);
    });
}