use sp_runtime::{generic::Era, Perbill, traits::{BlakeTwo256, IdentityLookup}, transaction_validity::{UnknownTransaction}, ApplyExtrinsicResultWithInfo};
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types, impl_outer_dispatch,
	weights::{Weight, RuntimeDbWeight},
};
use frame_system::{self as system, ChainContext};
use pallet_balances::Call as BalancesCall;
use pallet_balances as balances;
use frame_support::traits::{OnRuntimeUpgrade, OnFinalize, OnInitialize};

use frame_support::{assert_ok};
use pallet_subtensor::{NeuronMetadata, Module};
use std::net::{Ipv6Addr, Ipv4Addr};

use serde::{Serialize, Serializer, Deserialize, de::Error as DeError, Deserializer};
use std::{fmt::{self, Debug}, ops::Deref, cell::RefCell};
use sp_runtime::codec::{Codec, Encode, Decode};
use sp_runtime::traits::{
	self, Checkable, Applyable, OpaqueKeys,
	SignedExtension, Dispatchable, DispatchInfoOf, PostDispatchInfoOf,
};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::{generic, KeyTypeId, CryptoTypeId};
pub use sp_core::{H256, sr25519};
use sp_core::{crypto::{CryptoType, Dummy, key_types, Public}, U256};
use sp_runtime::transaction_validity::{TransactionValidity, TransactionSource, TransactionValidityError};
use sp_runtime::testing::Header;
use frame_support::weights::{DispatchInfo, GetDispatchInfo};


const TEST_KEY: &[u8] = &*b":test:key:";


type System = frame_system::Module<Test>;
type Balances = pallet_balances::Module<Test>;

#[allow(dead_code)]
pub type AccountId = u64;

impl_outer_origin! {
	pub enum Origin for Test { }
}

impl_outer_event!{
	pub enum MetaEvent for Test {
		system<T>,
		balances<T>,
	}
}
impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		pallet_balances::Balances,
		self::SubtensorModule,
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
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
impl frame_system::Trait for Test {
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

type Balance = u128;
parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Trait for Test {
	type Balance = Balance;
	type Event = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
}

impl pallet_subtensor::Trait for Test {
	type Event = ();
	type Currency = Balances;

}

parameter_types! {
	pub const TransactionByteFee: Balance = 0;
}
// impl pallet_transaction_payment::Trait for Test {
// 	type Currency = Balances;
// 	type OnTransactionPayment = ();
// 	type TransactionByteFee = TransactionByteFee;
// 	type WeightToFee = IdentityFee<Balance>;
// 	type FeeMultiplierUpdate = ();
// }

impl ValidateUnsigned for Test {
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
	frame_system::CheckEra<Test>,
	frame_system::CheckNonce<Test>,
	frame_system::CheckWeight<Test>,
	pallet_subtensor::ChargeTransactionPayment<Test>,
	pallet_subtensor::FeeFromSelfEmission<Test>
);

#[allow(dead_code)]
type AllModules = (System, Balances, SubtensorModule);



#[allow(dead_code)]
pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	fn on_runtime_upgrade() -> Weight {
		sp_io::storage::set(TEST_KEY, "custom_upgrade".as_bytes());
		// sp_io::storage::set(CUSTOM_ON_RUNTIME_KEY, &true.encode());
		0
	}
}

#[allow(dead_code)]
pub type Executive = frame_executive::Executive<
	Test,
	Block<TestXt<Call, SignedExtra>>,
	ChainContext<Test>,
	Test,
	AllModules,
	CustomOnRuntimeUpgrade
>;

#[allow(dead_code)]
fn extra(nonce: u64) -> SignedExtra {
	(
		frame_system::CheckEra::from(Era::Immortal),
		frame_system::CheckNonce::from(nonce),
		frame_system::CheckWeight::new(),
		pallet_subtensor::ChargeTransactionPayment::new(),
		pallet_subtensor::FeeFromSelfEmission::new()
	)
}

#[allow(dead_code)]
pub fn sign_extra(who: u64, nonce: u64) -> Option<(u64, SignedExtra)> {
	Some((who, extra(nonce)))
}


pub type SubtensorModule = Module<Test>;



/************************************************************
			HELPER FUNCTIONS
************************************************************/


// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[allow(dead_code)]
pub fn test_ext_with_balances(balances : Vec<(u64, u128)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}

#[allow(dead_code)]
pub fn subscribe_neuron(hotkey_account_id : u64, ip: u128, port: u16, ip_type : u8, modality: u8, coldkey_acount_id : u64) -> NeuronMetadata<u64> {
	let result = SubtensorModule::subscribe(<<Test as system::Trait>::Origin>::signed(hotkey_account_id), ip, port, ip_type, modality, coldkey_acount_id);
	assert_ok!(result);
	let neuron = SubtensorModule::get_neuron_for_hotkey(&hotkey_account_id);
	neuron
}

#[allow(dead_code)]
pub fn subscribe_ok_neuron(hotkey_account_id : u64,  coldkey_account_id : u64) -> NeuronMetadata<u64> {
	return subscribe_neuron(hotkey_account_id, ipv4(8,8,8,8), 66, 4, 0, coldkey_account_id );
}

#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        SubtensorModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SubtensorModule::on_initialize(System::block_number());
    }
}

// Generates an ipv6 address based on 8 ipv6 words and returns it as u128
#[allow(dead_code)]
pub fn ipv6(a: u16, b : u16, c : u16, d : u16, e : u16 ,f: u16, g: u16,h :u16) -> u128 {
	return Ipv6Addr::new(a,b,c,d,e,f,g,h).into();
}

// Generate an ipv4 address based on 4 bytes and returns the corresponding u128, so it can be fed
// to the module::subscribe() function
#[allow(dead_code)]
pub fn ipv4(a: u8 ,b: u8,c : u8,d : u8) -> u128 {
	let ipv4 : Ipv4Addr =  Ipv4Addr::new(a, b, c, d);
	let integer : u32 = ipv4.into();
	return u128::from(integer);
}




/************************************************************
	TEST EXTRINSIC
************************************************************/



/// A dummy type which can be used instead of regular cryptographic primitives.
///
/// 1. Wraps a `u64` `AccountId` and is able to `IdentifyAccount`.
/// 2. Can be converted to any `Public` key.
/// 3. Implements `RuntimeAppPublic` so it can be used instead of regular application-specific
///    crypto.
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, Debug, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct UintAuthorityId(pub u64);

impl From<u64> for UintAuthorityId {
	fn from(id: u64) -> Self {
		UintAuthorityId(id)
	}
}

impl From<UintAuthorityId> for u64 {
	fn from(id: UintAuthorityId) -> u64 {
		id.0
	}
}

impl UintAuthorityId {
	/// Convert this authority id into a public key.
	pub fn to_public_key<T: Public>(&self) -> T {
		let bytes: [u8; 32] = U256::from(self.0).into();
		T::from_slice(&bytes)
	}
}

impl CryptoType for UintAuthorityId {
	type Pair = Dummy;
}

impl AsRef<[u8]> for UintAuthorityId {
	fn as_ref(&self) -> &[u8] {
		// Unsafe, i know, but it's test code and it's just there because it's really convenient to
		// keep `UintAuthorityId` as a u64 under the hood.
		unsafe {
			std::slice::from_raw_parts(&self.0 as *const u64 as *const _, std::mem::size_of::<u64>())
		}
	}
}

thread_local! {
	/// A list of all UintAuthorityId keys returned to the runtime.
	static ALL_KEYS: RefCell<Vec<UintAuthorityId>> = RefCell::new(vec![]);
}

impl UintAuthorityId {
	/// Set the list of keys returned by the runtime call for all keys of that type.
	pub fn set_all_keys<T: Into<UintAuthorityId>>(keys: impl IntoIterator<Item=T>) {
		ALL_KEYS.with(|l| *l.borrow_mut() = keys.into_iter().map(Into::into).collect())
	}
}

impl sp_application_crypto::RuntimeAppPublic for UintAuthorityId {
	const ID: KeyTypeId = key_types::DUMMY;
	const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"dumm");

	type Signature = TestSignature;

	fn all() -> Vec<Self> {
		ALL_KEYS.with(|l| l.borrow().clone())
	}

	fn generate_pair(_: Option<Vec<u8>>) -> Self {
		use rand::RngCore;
		UintAuthorityId(rand::thread_rng().next_u64())
	}

	fn sign<M: AsRef<[u8]>>(&self, msg: &M) -> Option<Self::Signature> {
		Some(TestSignature(self.0, msg.as_ref().to_vec()))
	}

	fn verify<M: AsRef<[u8]>>(&self, msg: &M, signature: &Self::Signature) -> bool {
		traits::Verify::verify(signature, msg.as_ref(), &self.0)
	}

	fn to_raw_vec(&self) -> Vec<u8> {
		AsRef::<[u8]>::as_ref(self).to_vec()
	}
}

impl OpaqueKeys for UintAuthorityId {
	type KeyTypeIdProviders = ();

	fn key_ids() -> &'static [KeyTypeId] {
		&[key_types::DUMMY]
	}

	fn get_raw(&self, _: KeyTypeId) -> &[u8] {
		self.as_ref()
	}

	fn get<T: Decode>(&self, _: KeyTypeId) -> Option<T> {
		self.using_encoded(|mut x| T::decode(&mut x)).ok()
	}
}

impl sp_runtime::BoundToRuntimeAppPublic for UintAuthorityId {
	type Public = Self;
}

impl traits::IdentifyAccount for UintAuthorityId {
	type AccountId = u64;

	fn into_account(self) -> Self::AccountId {
		self.0
	}
}

/// A dummy signature type, to match `UintAuthorityId`.
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TestSignature(pub u64, pub Vec<u8>);

impl traits::Verify for TestSignature {
	type Signer = UintAuthorityId;

	fn verify<L: traits::Lazy<[u8]>>(&self, mut msg: L, signer: &u64) -> bool {
		signer == &self.0 && msg.get() == &self.1[..]
	}
}

/// Digest item
#[allow(dead_code)]
pub type DigestItem = generic::DigestItem<H256>;


/// An opaque extrinsic wrapper type.
#[derive(PartialEq, Eq, Clone, Debug, Encode, Decode, parity_util_mem::MallocSizeOf)]
pub struct ExtrinsicWrapper<Xt>(Xt);

impl<Xt> traits::Extrinsic for ExtrinsicWrapper<Xt>
where Xt: parity_util_mem::MallocSizeOf
{
	type Call = ();
	type SignaturePayload = ();

	fn is_signed(&self) -> Option<bool> {
		None
	}
}

impl<Xt: Encode> serde::Serialize for ExtrinsicWrapper<Xt> {
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
		self.using_encoded(|bytes| seq.serialize_bytes(bytes))
	}
}

impl<Xt> From<Xt> for ExtrinsicWrapper<Xt> {
	fn from(xt: Xt) -> Self {
		ExtrinsicWrapper(xt)
	}
}

impl<Xt> Deref for ExtrinsicWrapper<Xt> {
	type Target = Xt;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Testing block
#[derive(PartialEq, Eq, Clone, Serialize, Debug, Encode, Decode, parity_util_mem::MallocSizeOf)]
pub struct Block<Xt> {
	/// Block header
	pub header: Header,
	/// List of extrinsics
	pub extrinsics: Vec<Xt>,
}

impl<Xt: 'static + Codec + Sized + Send + Sync + Serialize + Clone + Eq + Debug + traits::Extrinsic> traits::Block
	for Block<Xt>
{
	type Extrinsic = Xt;
	type Header = Header;
	type Hash = <Header as traits::Header>::Hash;

	fn header(&self) -> &Self::Header {
		&self.header
	}
	fn extrinsics(&self) -> &[Self::Extrinsic] {
		&self.extrinsics[..]
	}
	fn deconstruct(self) -> (Self::Header, Vec<Self::Extrinsic>) {
		(self.header, self.extrinsics)
	}
	fn new(header: Self::Header, extrinsics: Vec<Self::Extrinsic>) -> Self {
		Block { header, extrinsics }
	}
	fn encode_from(header: &Self::Header, extrinsics: &[Self::Extrinsic]) -> Vec<u8> {
		(header, extrinsics).encode()
	}
}

impl<'a, Xt> Deserialize<'a> for Block<Xt> where Block<Xt>: Decode {
	fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Self, D::Error> {
		let r = <Vec<u8>>::deserialize(de)?;
		Decode::decode(&mut &r[..])
			.map_err(|e| DeError::custom(format!("Invalid value passed into decode: {}", e.what())))
	}
}

/// Test transaction, tuple of (sender, call, signed_extra)
/// with index only used if sender is some.
///
/// If sender is some then the transaction is signed otherwise it is unsigned.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct TestXt<Call, Extra> {
	/// Signature of the extrinsic.
	pub signature: Option<(u64, Extra)>,
	/// Call of the extrinsic.
	pub call: Call,
}

#[allow(dead_code)]
impl<Call, Extra> TestXt<Call, Extra> {
	/// Create a new `TextXt`.
	pub fn new(call: Call, signature: Option<(u64, Extra)>) -> Self {
		Self { call, signature }
	}
}

// Non-opaque extrinsics always 0.
parity_util_mem::malloc_size_of_is_0!(any: TestXt<Call, Extra>);

impl<Call, Extra> Serialize for TestXt<Call, Extra> where TestXt<Call, Extra>: Encode {
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error> where S: Serializer {
		self.using_encoded(|bytes| seq.serialize_bytes(bytes))
	}
}

impl<Call, Extra> Debug for TestXt<Call, Extra> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "TestXt({:?}, ...)", self.signature.as_ref().map(|x| &x.0))
	}
}

impl<Call: Codec + Sync + Send, Context, Extra> Checkable<Context> for TestXt<Call, Extra> {
	type Checked = Self;
	fn check(self, _: &Context) -> Result<Self::Checked, TransactionValidityError> { Ok(self) }
}

impl<Call: Codec + Sync + Send, Extra> traits::Extrinsic for TestXt<Call, Extra> {
	type Call = Call;
	type SignaturePayload = (u64, Extra);

	fn is_signed(&self) -> Option<bool> {
		Some(self.signature.is_some())
	}

	fn new(c: Call, sig: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(TestXt { signature: sig, call: c })
	}
}

impl<Origin, Call, Extra> Applyable for TestXt<Call, Extra> where
	Call: 'static + Sized + Send + Sync + Clone + Eq + Codec + Debug + Dispatchable<Origin=Origin>,
	Extra: SignedExtension<AccountId=u64, Call=Call>,
	Origin: From<Option<u64>>,
{
	type Call = Call;

	/// Checks to see if this is a valid *transaction*. It returns information on it if so.
	fn validate<U: ValidateUnsigned<Call=Self::Call>>(
		&self,
		_source: TransactionSource,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		Ok(Default::default())
	}

	/// Executes all necessary logic needed prior to dispatch and deconstructs into function call,
	/// index and sender.
	fn apply<U: ValidateUnsigned<Call=Self::Call>>(
		self,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> ApplyExtrinsicResultWithInfo<PostDispatchInfoOf<Self::Call>> {
		let (maybe_who, pre) = if let Some((id, extra)) = self.signature {
			let pre = Extra::pre_dispatch(extra, &id, &self.call, info, len)?;
			(Some(id), pre)
		} else {
			let pre = Extra::pre_dispatch_unsigned(&self.call, info, len)?;
			U::pre_dispatch(&self.call)?;
			(None, pre)
		};
		let res = self.call.dispatch(Origin::from(maybe_who));
		let post_info = match res {
			Ok(info) => info,
			Err(err) => err.post_info,
		};
		Extra::post_dispatch(pre, info, &post_info, len, &res.map(|_| ()).map_err(|e| e.error))?;
		Ok(res)
	}
}

/// Implementation for unchecked extrinsic.
impl<Call, Extra> GetDispatchInfo
	for TestXt<Call, Extra>
where
	Call: GetDispatchInfo,
	Extra: SignedExtension,
{
	fn get_dispatch_info(&self) -> DispatchInfo {
		self.call.get_dispatch_info()
	}
}

