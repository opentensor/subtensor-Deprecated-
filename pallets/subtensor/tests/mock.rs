use pallet_subtensor::{Module, Trait, NeuronMetadata};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};


/// The AccountId alias in this test module.
pub(crate) type AccountId = u64;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;


use frame_system as system;
use pallet_balances as balances;
use frame_support::traits::{OnInitialize, OnFinalize};

impl_outer_event! {
	pub enum MetaEvent for Test {
		system<T>,
		balances<T>,
	}
}

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

pub type Balances = pallet_balances::Module<Test>;
pub type System = frame_system::Module<Test>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const MaxLocks: u32 = 1024;
}

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = AccountIndex;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl Trait for Test {
	type Event = ();
	type Currency = Balances;
}

impl pallet_balances::Trait for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ();
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}

// impl pallet_balances::Config for Test {
// 	type MaxLocks = MaxLocks;
// 	type Balance = Balance;
// 	type Event = MetaEvent;
// 	type DustRemoval = ();
// 	type ExistentialDeposit = ExistentialDeposit;
// 	type AccountStore = System;
// 	type WeightInfo = ();
// }

pub type SubtensorModule = Module<Test>;
// type AccountIdOf<Test> = <Test as system::Trait>::AccountId;
// type NeuronMetadataOf<Test> = <pallet_subtensor::Module<Test> as Trait>::NeuronMetadata;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[allow(dead_code)]
pub fn subscribe_neuron(hotkey_account_id : u64, ip: u128, port: u16, ip_type : u8, coldkey_acount_id : u64) -> NeuronMetadata<u64> {
	let _ = SubtensorModule::subscribe(<<Test as system::Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, coldkey_acount_id);
	let neuron = SubtensorModule::get_neuron_metadata_for_hotkey(&hotkey_account_id);
	neuron
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        SubtensorModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SubtensorModule::on_initialize(System::block_number());
    }
}