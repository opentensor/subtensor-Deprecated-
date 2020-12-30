use pallet_subtensor::{Error};
use frame_support::{assert_ok};
use frame_system::Trait;
mod mock;
use mock::*;

use frame_system as system;

#[test]
fn test_self_emssion() {
	new_test_ext().execute_with(|| {
        // Let's subscribe a new neuron to the chain.
        let hotkey_pub:u64 = 666;
        let ip:u128 = 1;
        let port:u16 = 1;
        let ip_type:u8 = 1;
        let coldkey:u64 = 1;
        let _ = SubtensorModule::subscribe(<<Test as system::Trait>::Origin>::signed(hotkey_pub), ip, port, ip_type, coldkey);
        let neuron = SubtensorModule::get_neuron_metadata_for_hotkey(&hotkey_pub);

        // Let's give this neuron an initial stake of 1 token.
        SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, 1000000000); // Add the stake.
        assert_eq!(1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Let's set this neuron's weights. (0,0) = 1
		let weight_uids = vec![neuron.uid];
		let weight_values = vec![u32::MAX]; 
        assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(hotkey_pub), weight_uids.clone(), weight_values.clone()));
        assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (weight_uids, weight_values)); // Check the weights are set.

        // Let's call an emit.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 0);
        assert_eq!(1000000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.

        // Increase the block number to trigger emit. It starts at block 0.
        run_to_block(1);
        
        // Let's call an emit. Causes the new node to mint 500000000 to himself.
        let total_emission:u64 = SubtensorModule::emit_for_neuron(&neuron);
        assert_eq!(total_emission, 500000000);
        assert_eq!(1500000000, SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid)); // Check that the stake is there.
	});
}
