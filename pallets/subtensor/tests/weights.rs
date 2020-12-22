use pallet_subtensor::{Error, Module};
use frame_support::{assert_ok, assert_noop};
use frame_system::Trait;
// use crate::{mock};
mod mock;
use mock::{new_test_ext, SubtensorModule, Test};




#[test]
fn set_weights_ok() {
	new_test_ext().execute_with(|| {
		let weights_keys : Vec<<Test as Trait>::AccountId> = vec![1,2,3,4,5,6];
		let weight_values : Vec<u32> = vec![1,2,3,4,5,6];



		// Dispatch a signed extrinsic.
		assert_ok!(SubtensorModule::set_weights(<<Test as Trait>::Origin>::signed(1), weights_keys, weight_values));
		// Read pallet storage and assert an expected result.
		// assert_eq!(SubtensorModule::something(), Some(42));
	});
}
//
// #[test]
// fn correct_error_for_none_value() {
// 	mock::new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			SubtensorModule::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
//
//
// #[test]
// fn simple_test() {
// 	assert_eq!(1,1);
// }
//


