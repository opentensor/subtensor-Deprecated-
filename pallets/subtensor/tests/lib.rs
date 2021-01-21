use pallet_subtensor::{FeeFromSelfEmission};
use frame_support::{assert_ok};
mod mock;
use mock::*;
use frame_support::weights::DispatchInfo;
use sp_std::marker::PhantomData;
use sp_runtime::traits::SignedExtension;
use pallet_subtensor::Call as SubtensorCall;

#[test]
fn fee_from_emission_works() {
    new_test_ext().execute_with(|| {

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;
        assert!( FeeFromSelfEmission::<Test>(PhantomData).validate(&1, &call, &info, len).is_ok() );
    });
}

#[test]
fn fee_from_emission_priority_no_neuron() {
    new_test_ext().execute_with(|| {

        let call = SubtensorCall::set_weights(vec![0], vec![0]).into();
        let info = DispatchInfo::default();
        let len = 10;

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&1, &call, &info, len).unwrap().priority, 0);
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

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
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

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
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

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);
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

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 0);

        run_to_block(1);

        assert_eq!( FeeFromSelfEmission::<Test>(PhantomData).validate(&hotkey_account_id, &call, &info, len).unwrap().priority, 50000000);
    });
}