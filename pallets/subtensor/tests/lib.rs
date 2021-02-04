use pallet_subtensor::{ChargeTransactionPayment, Error};
use frame_support::{assert_ok};
mod mock;
use mock::*;
use frame_support::weights::DispatchInfo;
use frame_support::weights::PostDispatchInfo;
use sp_std::marker::PhantomData;
use sp_runtime::traits::SignedExtension;
use pallet_subtensor::Call as SubtensorCall;
use sp_runtime::DispatchError;
use sp_runtime::transaction_validity::TransactionValidityError;

#[test]
fn fee_from_emission_works() {
    new_test_ext().execute_with(|| {
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert!( ChargeTransactionPayment::<Test>(PhantomData).validate(&1, &call, &info, len).is_ok() );
    });
}

#[test]
fn fee_from_emission_priority_no_neuron() {
    new_test_ext().execute_with(|| {
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&1, &call, &info, len).unwrap().priority, 0);
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
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
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
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
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
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
    });
}


#[test]
fn fee_from_emission_priority_with_neuron_and_weights_and_stake_and_run_to_block() {
    new_test_ext().execute_with(|| {

        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        let weight_uids = vec![neuron.uid];
        let weight_values = vec![u32::MAX];
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weight_uids.clone(), weight_values.clone()));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
        run_to_block(1);
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 50000000);
        assert_eq!( 1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that his stake has not increased.
        let _total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron); // actually do the emission.

        // This step now takes places in the post dispatch
        // assert_eq!( 1500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that his stake has increased (he is adam)
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
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id_2, &call, &info, len).unwrap().priority, 0);
        run_to_block(1);
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id_2, &call, &info, len).unwrap().priority, 0);
        let _total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron_2); // actually do the emission.
        assert_eq!( 500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_3.uid));
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
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
        run_to_block(1);
        assert_eq!( ChargeTransactionPayment::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 50000000);
        assert_eq!( 1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that his stake has not increased.

        let _total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron); // actually do the emission.
        assert_eq!( 1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that his stake has increased (he is *not* adam)
        // assert_eq!( 500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam.uid)); // Check that his stake has increased (he is adam)

        // This takes place in post-dispatch. Make necessary adaptations
    });
}

#[test]
fn pre_dispatch_works() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), vec![neuron.uid], vec![u32::MAX]));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap(), 0);
        run_to_block(1);
        assert_eq!(ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap(), 500000000);
    });
}

#[test]
fn post_dispatch_works() {
    new_test_ext().execute_with(|| {
        let adam_id = 0;
        let hotkey_account_id = 1;

        let adam = subscribe_ok_neuron(0, 667);
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), vec![neuron.uid], vec![u32::MAX]));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        run_to_block(1);

        let result = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len);
        assert_ok!(result);

        let pre = result.unwrap();
        assert!(ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo {actual_weight: Some(0), pays_fee: Default::default()}, len, &Ok(())).is_ok());
        assert!(ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo {actual_weight: Some(1000000000), pays_fee: Default::default()}, len, &Ok(())).is_ok());
    });
}


#[test]
fn test_post_dispath_works_only_on_set_weights_function() {
	new_test_ext().execute_with(|| {
        assert!(true == false);
	});
}

#[test]
fn test_post_dispatch_does_not_deposit_to_adam_on_error() {
	new_test_ext().execute_with(|| {
        let adam_id = 0;
        let adam = subscribe_ok_neuron(adam_id, 667);

        // Adam should have no stake before operation
        let mut result = SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam_id);
        assert_eq!(result, 0);

        let pre_dispatch_result = Err(Error::<Test>::DuplicateUids.into());
        let self_emission = 1000000;

        // let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let post_dispatch_info = PostDispatchInfo {actual_weight: Some(0), pays_fee: Default::default()};
        let len = 100;

        let post_dispatch_result  = ChargeTransactionPayment::<Test>::post_dispatch(self_emission, &info, &post_dispatch_info, len, &pre_dispatch_result);
        assert_ok!(post_dispatch_result);

        assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam_id), 0);
	});
}

#[test]
fn post_dispatch_deposit_to_adam_works() {
    new_test_ext().execute_with(|| {
        let adam_account_id = 0;
        let adam = subscribe_neuron(adam_account_id, 10, 666, 4, 0, 66);
        let hotkey_account_id = 1;
        let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
        assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), vec![neuron.uid], vec![u32::MAX]));
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.
        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        run_to_block(1);
        let pre = ChargeTransactionPayment::<Test>(PhantomData).pre_dispatch(&hotkey_account_id, &call, &info, len).unwrap();
        assert!(ChargeTransactionPayment::<Test>::post_dispatch(pre, &info, &PostDispatchInfo {actual_weight: Some(0), pays_fee: Default::default()}, len, &Ok(())).is_ok());
        assert_eq!(500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam.uid)); // Check that adam has more stake now.
    });
}
