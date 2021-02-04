use frame_support::{assert_ok};
use frame_system::{Trait};
mod mock;
use mock::*;
use mock::{TestXt};
use frame_support::sp_runtime::DispatchError;
use pallet_subtensor::{Call as SubtensorCall, Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};

use pallet_balances::{Call as BalanceCall};

#[test]
fn test_temp_transaction_test() {
    let source_account_id = 0;

	// test_ext_with_balances(vec![(source_account_id, 1_000_000_000)]).execute_with(|| {
    //     // let call = Call::SubtensorModule(SubtensorCall::add_stake(test_neuron_hot_key, 500_000_000));
    //     // let Call::Balances(BalanceCall::transfer(..);
    //
    //     let call = Call::Balances(BalanceCall::transfer(1, Balance::from(100 as u128)));
    //
    //     let xt = TestXt::new(call, mock::sign_extra(source_account_id, 0));
    //     let result = mock::Executive::apply_extrinsic(xt);
    //
	// 	assert_ok!(result);
	// });
}