// On spliting modules : https://stackoverflow.com/questions/56902167/in-substrate-is-there-a-way-to-use-storage-and-functions-from-one-custom-module


#![cfg_attr(not(feature = "std"), no_std)]

// --- Frame imports.
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, debug, traits::{Currency, WithdrawReasons, WithdrawReason, ExistenceRequirement}, Printable};
use frame_support::weights::{DispatchClass, Pays};
use codec::{Decode, Encode};
use frame_system::{self as system, ensure_signed};
use substrate_fixed::types::U32F32;
use sp_std::convert::TryInto;
use sp_std::{
	prelude::*
};

mod weights;
mod staking;
mod subscribing;
mod emission;

use frame_support::debug::RuntimeLogger;


/// --- Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// --- Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// --- Currency type that will be used to place deposits on neurons
	type Currency: Currency<Self::AccountId>;
}

// ---- Create account types for the NeuronMetadata struct.
// Account is of type system::Trait::AccoountId.
type AccountIdOf<T> = <T as system::Trait>::AccountId;
type NeuronMetadataOf<T> = NeuronMetadata<AccountIdOf<T>>;

#[derive(Encode, Decode, Default)]
pub struct NeuronMetadata <AccountId> {
	/// ---- The endpoint's u128 encoded ip address of type v6 or v4.  
	ip: u128,

	/// ---- The endpoint's u16 encoded port. 
	port: u16,
	
	/// ---- The endpoint's ip type, 4 for ipv4 and 6 for ipv6. 
	ip_type: u8,

	/// ---- The associated coldkey account. 
	/// Staking and unstaking transactions must be made by this account.
	/// The hotkey account (in the Neurons map) has permission to call emit
	/// subscribe and unsubscribe.
	coldkey: AccountId,
}

// The pallet's runtime storage items.
decl_storage! {
	trait Store for Module<T: Trait> as SubtensorModule {
	
		/// ---- Maps between a neuron's hotkey account address and that neurons
		/// weights, a.k.a is row_weights in the square matrix W. The vector of keys
		/// and the vector of weights must be the same length and if they exist
		/// their values must be positive and sum to the largest u32 value.
		pub WeightKeys: map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;
		pub WeightVals: map hasher(blake2_128_concat) T::AccountId => Vec<u32>;

		/// ---- Maps between a neuron's hotkey account address and the block number
		/// when that peer last called an emission. The last emit time is used to determin
		/// the proportion of inflation remaining to emit during the next emit call.
		pub LastEmit get(fn block): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;
		
		/// ----  Maps between a neuron's hotkey account address and additional 
		/// metadata associated with that neuron. Specifically, the ip,port, and coldkey address.
		pub Neurons get(fn neuron): map hasher(blake2_128_concat) T::AccountId => NeuronMetadataOf<T>;

		/// ----  Maps between a neuron's hotkey account address and the number of
		/// staked tokens under that key.
		pub Stake get(fn stake): map hasher(blake2_128_concat) T::AccountId => u32;

		/// ---- Stores the amount of currently staked token.
		TotalStake: u32;

		/// ---- Stores the number of active neurons.
		NeuronCount: u32;
	}
}

// Subtensor events.
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// ---- Event created when a caller successfully set's their weights
		/// on the chain.
		WeightsSet(AccountId),

		/// --- Event created when a new neuron account has been added to the 
		/// active set.
		NeuronAdded(AccountId),

		/// --- Event created when the neuron information associated with a hotkey
		/// is changed, for instance, when the ip/port changes.
		NeuronUpdated(AccountId),

		/// --- Event created when a new neuron account has been removed from the
		/// active set.
		NeuronRemoved(AccountId),

		/// --- Event created during when stake has been transfered from 
		/// the coldkey onto the hotkey staking account.
		StakeAdded(AccountId, u32),

		/// -- Event created when stake has been removed from 
		/// the staking account into the coldkey account.
		StakeRemoved(AccountId, u32),

		/// ---- Event created when a transaction triggers and incentive
		/// mechanism emission.
		Emission(AccountId, u32),
		
	}
);

// Subtensor Errors.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// ---- Thrown when the caller attempts to set the weight keys
		/// and values but these vectors have different size.
		WeightVecNotEqualSize,

		/// ---- Thrown when the caller triggers an emit but the computed amount
		/// to emit is zero.
		NothingToEmit,

		/// ---- Thrown when the caller requests setting or removing data from
		/// a neuron which does not exist in the active set.
		NotActive,

		/// ---- Thrown when the caller requests subscribing a neuron which 
		/// already exists in the active set.
		AlreadyActive,

		/// ---- Thrown when a stake or unstake request is made by a coldkey
		/// which is not associated with the hotkey account. 
		/// See: fn add_stake and fn remove_stake 
		NonAssociatedColdKey,

		/// ---- Thrown when the caller requests removing more stake then there exists 
		/// in the staking account. See: fn remove_stake.
		NotEnoughStaketoWithdraw,
	}
}

impl<T: Trait> Printable for Error<T> {
    fn print(&self) {
        match self {
            Error::AlreadyActive => "The node with the supplied public key is already active".print(),
            Error::NotActive => "The node with the supplied piblic key is not active".print(),
			Error::NothingToEmit => "There is nothing to emit".print(),
			Error::WeightVecNotEqualSize => "The vec of keys and the vec of values are not of the same size".print(),
			Error::NonAssociatedColdKey => "The used cold key is not associated with the hot key acccount".print(),
            _ => "Invalid Error Case".print(),
        }
    }
}

// Subtensor Dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;


		/// --- Sets weights to public keys on the neuron
		/// The dests parameter is a vector of public keys
		/// The values parameter is a vector of unsigned 32 bit integers
		/// These 32 bit integers should represent a decimal number where when all bits
		/// are set, this represents 1.0.
		///
		/// The function normalizes all integers to u32_max. This means that if the sum of all
		/// elements is larger or smaller than the amount of elements * u32_max, all elements
		/// will be corrected for this deviation. See the unit tests on the bottom of this file
		/// for more information.
		///
		/// After normalizing the weights, they are pushed on the chain and an event is issued.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn set_weights(origin,
				dests: Vec<T::AccountId>,
				values: Vec<u32>) -> dispatch::DispatchResult {


			Self::do_set_weights(origin, dests, values)
		}




		/// --- Adds stake to a neuron account. The call is made from the
		/// coldkey account linked in the neurons's NeuronMetadata.
		/// Only the associated coldkey is allowed to make staking and
		/// unstaking requests. This protects the neuron against
		/// attacks on its hotkey running in production code.
		///
		/// # Args:
		/// 	* `origin`: (<T as frame_system::Trait>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* `hotkey` (T::AccountId):
		/// 		- The hotkey account to add stake to.
		///
		/// 	* `ammount_staked` (u32):
		/// 		- The ammount to transfer from the balances account of the cold key
		/// 		into the staking account of the hotkey.
		///
		/// # Emits:
		/// 	`StakeAdded`:
		/// 		- On the successful staking of funds.
		///
		/// # Raises:
		/// 	* `NotActive`:
		/// 		- If the hotkey account is not active (has not subscribed)
		///
		/// 	* `NonAssociatedColdKey`:
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* `InsufficientBalance`:
		/// 		- When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
		#[weight = (0, DispatchClass::Operational, Pays::No)] // TODO(const): should be a normal transaction fee.
		fn add_stake(origin, hotkey: T::AccountId, ammount_staked: u32) -> dispatch::DispatchResult {
			Self::do_add_stake(origin, hotkey, ammount_staked)
		}

		/// ---- Remove stake from the staking account. The call must be made
		/// from the coldkey account attached to the neuron metadata. Only this key
		/// has permission to make staking and unstaking requests.
		///
		/// # Args:
		/// 	* `origin``: (<T as frame_system::Trait>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* `hotkey` (T::AccountId):
		/// 		- The hotkey account to withdraw stake from.
		///
		/// 	* `ammount_unstaked` (u32):
		/// 		- The ammount to transfer from the staking account into the balance
		/// 		of the coldkey.
		///
		/// # Emits:
		/// 	* `StakeRemoved`:
		/// 		- On successful withdrawl.
		///
		/// # Raises:
		/// 	* `NonAssociatedColdKey`:
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* `NotEnoughStaketoWithdraw`:
		/// 		- When the amount to unstake exceeds the quantity staked in the
		/// 		associated hotkey staking account.
		///
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn remove_stake(origin, hotkey: T::AccountId, ammount_unstaked: u32) -> dispatch::DispatchResult {
			Self::do_remove_stake(origin, hotkey, ammount_unstaked)
		}

		/// ---- Subscribes the caller to the Neuron set with given metadata. If the caller
		/// already exists in the active set, the metadata is updated but the cold key remains unchanged.
		/// If the caller does not exist they make a link between this hotkey account
		/// and the passed coldkey account. Only the cold key has permission to make add_stake/remove_stake calls.
		///
		/// # Args:
		/// 	* `origin`: (<T as frame_system::Trait>Origin):
		/// 		- The caller, a hotkey associated with the subscribing neuron.
		///
		/// 	* `ip` (u128):
		/// 		- The u32 encoded IP address of type 6 or 4.
		///
		/// 	* `port` (u16):
		/// 		- The port number where this neuron receives RPC requests.
		///
		/// 	* `ip_type` (u8):
		/// 		- The ip type one of (4,6).
		///
		/// 	* `coldkey` (T::AccountId):
		/// 		- The associated coldkey to be attached to the account.
		///
		/// # Emits:
		/// 	* `NeuronAdded`:
		/// 		- On subscription of a new neuron to the active set.
		///
		/// 	* `NeuronUpdated`:
		/// 		- On subscription of new metadata attached to the calling hotkey.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn subscribe(origin, ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {
			Self::do_subscribe(origin, ip, port, ip_type, coldkey)
		}

		/// ---- Unsubscribes the caller from the active Neuron. The call triggers
		/// an emit call before unstaking the current stake balance into the coldkey account.
		///
		/// # Args:
		/// 	* `origin`: (<T as frame_system::Trait>Origin):
		/// 		- The caller, a hotkey associated with the subscribing neuron.
		///
		/// # Emits:
		/// 	* `NeuronRemoved`:
		/// 		- On subscription of a new neuron to the active set.
		///
		/// 	* `NeuronUpdated`:
		/// 		- On subscription of new metadata attached to the calling hotkey.
		///
		/// # Raises:
		/// 	* `NotActive`:
		/// 		- Raised if the unsubscriber does not exist.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn unsubscribe(origin) -> dispatch::DispatchResult {
			Self::do_unsubscribe(origin)
		}
	}
}

impl<T: Trait> Module<T> {

	/// Returns the bitcoin inflation rate at block number. We use a mapping between the bitcoin blocks and substrate.
	/// Substrate blocks mint 100x faster and so the halving time and emission rate need to correspondingly changed.
	/// Each block produces 0.5 x 10^12 tokens, or semantically, as 0.5 full coins every 6 seconds. The halving
	/// is occurs every 21 million blocks. Like bitcoin, this ensures there are only every 21 million full tokens
	/// created. In our case this ammount is furthre limited since inflation is only triggered by calling peers.
	/// The inflation is therefore not continuous, and we lose out when peers fail to emit with their stake, or
	/// fail to emit before a halving.
	///
	/// # Args:
	///  	* `now` (&T::BlockNumber):
	/// 		- The block number we wish to return the block reward at (tau)
	///
	/// # Returns
	/// 	* block_reward (U32F32):
	/// 		- The number of tokens to emit at this block as a fixed point.
	///
	fn block_reward(now: &<T as system::Trait>::BlockNumber) -> U32F32 {

		// --- We convert the block number to u32 and then to a fixed point for further
		// calculations.
		let elapsed_blocks_u32 = TryInto::try_into(*now).ok().expect("blockchain will not exceed 2^32 blocks; QED.");
		let elapsed_blocks_u32_f32 = U32F32::from_num(elapsed_blocks_u32);

		// --- We get the initial block reward.
		// TODO(const): shoudl be 0.5 x 10 ^ 12.
		// Bitcoin block reward started at 50 tokens per block.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (50 blocks) / (10 min * 60 sec / 6 sec) =  (50) / (100) = (0.5 tokens per block)
		let block_reward = U32F32::from_num(0.5);

		// --- We calculate the number of halvings since the chain was initialized
		// Bitcoin block halving rate was 210,000 blocks at block every 10 minutes.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (210,000 blocks) * (10 min * 60 sec / 6 sec) =  (210,000) * (100) = (21,000,000 blocks)
		let block_halving = U32F32::from_num(21000000);
		let fractional_halvings = elapsed_blocks_u32_f32 / block_halving;
		let floored_halvings = fractional_halvings.floor().to_num::<u32>();
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("floored_halvings: {:?}", floored_halvings);

		// --- We shit the block reward for each halving to get the actual reward at this block.
		// NOTE: Underflow occurs in 21,000,000 * (16 + 4) blocks essentially never.
		let block_reward_shift = block_reward.overflowing_shr(floored_halvings).0;

		// --- We return the result.
		block_reward_shift
	}

	pub fn u32_to_balance(input: u32) -> <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance
	{
		input.into()
	}






}





