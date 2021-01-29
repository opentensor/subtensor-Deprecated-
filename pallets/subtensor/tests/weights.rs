use pallet_subtensor::{Module, NeuronMetadata};
mod test_xt;
use sp_runtime::testing::{Header, Block};
use sp_runtime::{generic::Era, Perbill, DispatchError, traits::{BlakeTwo256, IdentityLookup}, transaction_validity::{UnknownTransaction, TransactionValidityError}, ApplyExtrinsicResultWithInfo};
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types, impl_outer_dispatch,
	weights::{Weight, RuntimeDbWeight},
};
use frame_system::{self as system, ChainContext};
use pallet_balances::Call as BalancesCall;
use pallet_balances as balances;
use sp_runtime::traits::{ValidateUnsigned, Applyable, Dispatchable, SignedExtension, DispatchInfoOf, PostDispatchInfoOf};
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidity};
use frame_support::traits::OnRuntimeUpgrade;
use frame_support::codec::{Codec};

use frame_support::{assert_ok};


const TEST_KEY: &[u8] = &*b":test:key:";

type SubtensorModule = Module<Runtime>;


type System = frame_system::Module<Runtime>;
type Balances = pallet_balances::Module<Runtime>;
type AccountId = u64;



	impl_outer_origin! {
		pub enum Origin for Runtime { }
	}

	impl_outer_event!{
		pub enum MetaEvent for Runtime {
			system<T>,
			balances<T>,
		}
	}
	impl_outer_dispatch! {
		pub enum Call for Runtime where origin: Origin {
			frame_system::System,
			pallet_balances::Balances,
			self::SubtensorModule,
		}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Runtime;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const MaximumExtrinsicWeight: Weight = 500;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub const BlockExecutionWeight: Weight = 10;
		pub const ExtrinsicBaseWeight: Weight = 5;
		pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
			read: 10,
			write: 100,
		};
	}
	impl frame_system::Trait for Runtime {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Index = u64;
		type Call = Call;
		type BlockNumber = u64;
		type Hash = sp_core::H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<u64>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = DbWeight;
		type BlockExecutionWeight = BlockExecutionWeight;
		type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
		type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
		type AvailableBlockRatio = AvailableBlockRatio;
		type MaximumBlockLength = MaximumBlockLength;
		type Version = RuntimeVersion;
		type PalletInfo = ();
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
	}

	type Balance = u64;
	parameter_types! {
		pub const ExistentialDeposit: Balance = 1;
	}
	impl pallet_balances::Trait for Runtime {
		type Balance = Balance;
		type Event = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type MaxLocks = ();
		type WeightInfo = ();
	}

	impl pallet_subtensor::Trait for Runtime {
		type Event = ();
		type Currency = Balances;

	}

	parameter_types! {
		pub const TransactionByteFee: Balance = 0;
	}
	// impl pallet_transaction_payment::Trait for Runtime {
	// 	type Currency = Balances;
	// 	type OnTransactionPayment = ();
	// 	type TransactionByteFee = TransactionByteFee;
	// 	type WeightToFee = IdentityFee<Balance>;
	// 	type FeeMultiplierUpdate = ();
	// }

	impl ValidateUnsigned for Runtime {
		type Call = Call;
		//
		// fn pre_dispatch(_call: &Self::Call) -> Result<(), TransactionValidityError> {
		// 	Ok(())
		// }
		//
		//
		//
		fn validate_unsigned(
			_source: TransactionSource,
			call: &Self::Call,
		) -> TransactionValidity {
			match call {
				Call::Balances(BalancesCall::set_balance(_, _, _)) => Ok(Default::default()),
				_ => UnknownTransaction::NoUnsignedValidator.into(),
			}
		}
	}

	pub struct RuntimeVersion;
	impl frame_support::traits::Get<sp_version::RuntimeVersion> for RuntimeVersion {
		fn get() -> sp_version::RuntimeVersion {
			RUNTIME_VERSION.with(|v| v.borrow().clone())
		}
	}

	thread_local! {
		pub static RUNTIME_VERSION: std::cell::RefCell<sp_version::RuntimeVersion> =
			Default::default();
	}

	type SignedExtra = (
		frame_system::CheckEra<Runtime>,
		frame_system::CheckNonce<Runtime>,
		frame_system::CheckWeight<Runtime>,
		pallet_subtensor::ChargeTransactionPayment<Runtime>,
		pallet_subtensor::FeeFromSelfEmission<Runtime>
	);
	type AllModules = (System, Balances, SubtensorModule);
	// type TestXt = mock::TestXt<Call, SignedExtra>;

	type TestXt = test_xt::TestXt<Call, SignedExtra>;



	struct CustomOnRuntimeUpgrade;
	impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
		fn on_runtime_upgrade() -> Weight {
			sp_io::storage::set(TEST_KEY, "custom_upgrade".as_bytes());
			// sp_io::storage::set(CUSTOM_ON_RUNTIME_KEY, &true.encode());
			0
		}
	}

	type Executive = frame_executive::Executive<
		Runtime,
		Block<TestXt>,
		ChainContext<Runtime>,
		Runtime,
		AllModules,
		CustomOnRuntimeUpgrade
	>;

	fn extra(nonce: u64) -> SignedExtra {
		(
			frame_system::CheckEra::from(Era::Immortal),
			frame_system::CheckNonce::from(nonce),
			frame_system::CheckWeight::new(),
			pallet_subtensor::ChargeTransactionPayment::new(),
			pallet_subtensor::FeeFromSelfEmission::new()
		)
	}

	fn sign_extra(who: u64, nonce: u64) -> Option<(u64, SignedExtra)> {
		Some((who, extra(nonce)))
	}


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
}

mod mock;
use mock::ipv4;

pub fn subscribe_neuron(hotkey_account_id : u64, ip: u128, port: u16, ip_type : u8, modality: u8, coldkey_acount_id : u64) -> NeuronMetadata<u64> {
	let _result = SubtensorModule::subscribe(Origin::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_acount_id);
	let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}

pub fn subscribe_ok_neuron(hotkey_account_id : u64,  coldkey_account_id : u64) -> NeuronMetadata<u64> {
	return subscribe_neuron(hotkey_account_id, ipv4(8,8,8,8), 66, 4, 0, coldkey_account_id );
}

use pallet_subtensor::Call as SubtensorCall;
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use sp_std::fmt::Debug;
use sp_runtime::generic::CheckedExtrinsic;

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
fn test_set_weights_adam_receives_funds() {
	new_test_ext().execute_with(|| {
		let w_uids = vec![1, 2]; // When applied to neuron_1, this will set 50% to himself and 50% to neuron_2
		let w_vals = vec![50, 50];

		let adam_id = 0;
		let neuron_1_id = 1;

		let _neuron_adam =  subscribe_ok_neuron(adam_id, 666); // uid 0
		let _neuron1 = subscribe_ok_neuron(neuron_1_id, 666); // uid 1

		// Add 1 Tao to neuron 1. He now hold 100% of the stake, so will get the full emission,
		// also he only has a self_weight.
		SubtensorModule::add_stake_to_neuron_hotkey_account(1, 1_000_000_000);

		// Move to block, to build up pending emission
		mock::run_to_block(1); // This will release 1 Tao. 0.5 for block 0, 0.5 for block 1

		// Verify adam's stake == 0
		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam_id), 0);

		// Define the call
		let call = Call::SubtensorModule(SubtensorCall::set_weights(w_uids, w_vals));

		// Setup the extrinsic
		let xt = TestXt::new(call, sign_extra(1,0)); // Apply t

		// Execute. This will trigger the set_weights function to emit, before the new weights are set.
		// Resulting in Adam getting his full emission.
		let result = Executive::apply_extrinsic(xt);

		// Verfify success
		assert_ok!(result);

		// let trans_err = TransactionValidityError::Unknown(UnknownTransaction::CannotLookup);
		// let res : Result<Result<(), DispatchError>, TransactionValidityError> = Result::Err(trans_err);

		let adam_new_stake = SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(adam_id);
		let neuron_1_new_stake = SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron_1_id);

		assert_eq!(adam_new_stake, 500_000_000);
		assert_eq!(neuron_1_new_stake, 1_000_000_000);  // Neuron 1 maintains its stake

		// let api = TestApi::empty();

	});
}





//
// /**
// * This test the situation where user tries to set weights, but the vecs are empty.
// * After setting the weights, the wi
// */
// #[test]
// fn set_weights_ok_no_weights() {
// 	new_test_ext().execute_with(|| {
//
// 		// == Intial values ==
// 		let hotkey_account_id:u64 = 55; // Arbitrary number
// 		let initial_stake = 10000;
//
// 		let weights_keys : Vec<AccountId> = vec![];
// 		let weight_values : Vec<u32> = vec![];
//
// 		// == Expectations ==
//
// 		let expect_keys = vec![]; // keys should not change
// 		let expect_values = vec![]; // Value should be normalized for u32:max
// 		let expect_stake = 10000; // The stake for the neuron should remain the same
// 		let expect_total_stake = 10000; // The total stake should remain the same
//
//
// 		// Let's subscribe a new neuron to the chain
// 		let neuron = subscribe_neuron(hotkey_account_id, 10, 666, 4, 0, 66);
//
// 		// Let's give it some stake.
// 		SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stake);
//
// 		// Dispatch a signed extrinsic, setting weights.
// 		assert_ok!(SubtensorModule::set_weights(Origin::signed(hotkey_account_id), weights_keys, weight_values));
// 		assert_eq!(SubtensorModule::get_weights_for_neuron(&neuron), (expect_keys, expect_values));
// 		assert_eq!(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), expect_stake);
// 		assert_eq!(SubtensorModule::get_total_stake(), expect_total_stake);
// 	});
// }
//
// #[test]
// fn set_weights_ok_with_weights() {
// 	new_test_ext().execute_with(|| {
// 		let neurons = vec![
// 			subscribe_neuron(55, 10, 666, 4, 0, 66),
// 			subscribe_neuron(66, 10, 666, 4, 0, 66),
// 			subscribe_neuron(77, 10, 666, 4, 0, 66)
// 		];
//
// 		let initial_stakes = vec![10000,0,0];
//
// 		let weight_uids = vec![neurons[1].uid, neurons[2].uid];
// 		let weight_values = vec![u32::MAX / 2, u32::MAX / 2]; // Set equal weights to ids 2,3
//
// 		// Expectations
// 		let expect_weight_uids = vec![neurons[1].uid, neurons[2].uid];
// 		let expect_weight_values = vec![u32::MAX / 2, u32::MAX / 2];
//
// 		// Dish out the stake for all neurons
// 		for (i, neuron) in neurons.iter().enumerate() {
// 			SubtensorModule::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stakes[i]);
// 		}
//
// 		// Perform tests
//
// 		// First call to set the weights. An emit is triggered, but since there are no weights, no emission occurs
// 		assert_ok!(SubtensorModule::set_weights(Origin::signed(55), weight_uids.clone(), weight_values.clone()));
//
// 		// Increase the block number to trigger emit. It starts at block 0
// 		run_to_block(1);
//
// 		// Second set weights. This should cause inflation to be distributed and end up in hotkey accounts.
// 		assert_ok!(SubtensorModule::set_weights(Origin::signed(55), weight_uids.clone(), weight_values.clone()));
// 		assert_eq!(SubtensorModule::get_weights_for_neuron(&neurons[0]), (expect_weight_uids, expect_weight_values));
//
// 		let mut stakes: Vec<u64> = vec![];
// 		for neuron in neurons {
// 			stakes.push(SubtensorModule::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid));
// 		}
//
// 		assert_eq!(stakes[0], initial_stakes[0]); // Stake of sender should remain unchanged
// 		assert!(stakes[1] >  initial_stakes[1]); // The stake of destination 1 should have increased
// 		assert!(stakes[2] >  initial_stakes[2]); // The stake destination 2 should habe increased
// 		assert_eq!(stakes[1], stakes[2]); // The stakes should have increased the same
// 	});
// }
//
// #[test]
// fn test_weights_err_weights_vec_not_equal_size() {
// 	new_test_ext().execute_with(|| {
//         let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);
//
// 		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
// 		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5]; // Uneven sizes
//
// 		let result = SubtensorModule::set_weights(Origin::signed(666), weights_keys, weight_values);
//
// 		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
// 	});
// }
//
// #[test]
// fn test_weights_err_has_duplicate_ids() {
// 	new_test_ext().execute_with(|| {
//         let _neuron = subscribe_neuron(666, 5, 66, 4, 0, 77);
// 		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
// 		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];
//
// 		let result = SubtensorModule::set_weights(Origin::signed(666), weights_keys, weight_values);
//
// 		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
// 	});
// }
//
// #[test]
// fn test_no_signature() {
// 	new_test_ext().execute_with(|| {
// 		let weights_keys: Vec<AccountId> = vec![];
// 		let weight_values: Vec<u32> = vec![];
//
// 		let result = SubtensorModule::set_weights(Origin::none(), weights_keys, weight_values);
// 		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
// 	});
// }
//
// #[test]
// fn test_set_weights_err_not_active() {
// 	new_test_ext().execute_with(|| {
// 		let weights_keys: Vec<AccountId> = vec![1, 2, 3, 4, 5, 6];
// 		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
//
// 		let result = SubtensorModule::set_weights(Origin::signed(1), weights_keys, weight_values);
//
// 		assert_eq!(result, Err(Error::<Test>::NotActive.into()));
// 	});
// }
//
//
// #[test]
// fn test_set_weights_err_invalid_uid() {
// 	new_test_ext().execute_with(|| {
//         let _neuron = subscribe_neuron(55, 33, 55, 4, 0, 66);
// 		let weight_keys : Vec<AccountId> = vec![9999999999]; // Does not exist
// 		let weight_values : Vec<u32> = vec![88]; // random value
//
// 		let result = SubtensorModule::set_weights(Origin::signed(55), weight_keys, weight_values);
//
// 		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));
//
// 	});
// }
//
//
//
