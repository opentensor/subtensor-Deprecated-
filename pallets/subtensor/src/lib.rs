#![cfg_attr(not(feature = "std"), no_std)]

// --- Frame imports.bug
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, dispatch::IsSubType, ensure, debug, IterableStorageMap, traits::{Currency, WithdrawReasons, WithdrawReason, ExistenceRequirement}, Printable};
use frame_support::weights::{DispatchClass, Pays};
use codec::{Decode, Encode};
use frame_system::{self as system, ensure_signed};
use substrate_fixed::types::U64F64;
use sp_std::convert::TryInto;
use sp_std::{
    prelude::*
};
use sp_std::marker::PhantomData;
use frame_support::{
    weights::{
        Weight, DispatchInfo, PostDispatchInfo,
    },
    dispatch::DispatchResult,
};
use sp_runtime::{
    traits::{
        SignedExtension, DispatchInfoOf, PostDispatchInfoOf,
    },
    transaction_validity::{
        TransactionValidityError, TransactionValidity, InvalidTransaction,
    },
};

use sp_runtime::traits::{Dispatchable};
use frame_support::traits::Get;
use frame_support::sp_runtime::transaction_validity::ValidTransaction;

mod weights;
mod staking;
mod subscribing;
mod emission;
mod block_reward;

/// --- Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// --- Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    /// --- Currency type that will be used to place deposits on neurons
    type Currency: Currency<Self::AccountId>;

    /// - The transaction fee in RAO per byte
    type TransactionByteFee: Get<BalanceOf<Self>>;
}

// ---- Create account types for the NeuronMetadata struct.
type AccountIdOf<T> = <T as system::Trait>::AccountId;
type NeuronMetadataOf<T> = NeuronMetadata<AccountIdOf<T>>;


// ---- Neuron endpoint information
#[derive(Encode, Decode, Default)]
pub struct NeuronMetadata<AccountId> {
    /// ---- The endpoint's u128 encoded ip address of type v6 or v4.
    pub ip: u128,

    /// ---- The endpoint's u16 encoded port.
    pub port: u16,

    /// ---- The endpoint's ip type, 4 for ipv4 and 6 for ipv6.
    pub ip_type: u8,

    /// ---- The endpoint's unique identifier. The chain can have
    /// 18,446,744,073,709,551,615 neurons before we overflow. However
    /// by this point the chain would be 10 terabytes just from metadata
    /// alone.
    pub uid: u64,

    /// ---- The neuron modality. Modalities specify which datatype
    /// the neuron endpoint can process. This information is non
    /// verifiable. However, neurons should set this correctly
    /// in order to be detected by others with this datatype.
    /// The initial modality codes are:
    /// TEXT: 0
    /// IMAGE: 1
    /// TENSOR: 2
    pub modality: u8,

    /// ---- The associated hotkey account.
    /// Subscribing, emitting and changing weights can be made by this
    /// account. Subscription can never change the associated coldkey
    /// account.
    pub hotkey: AccountId,

    /// ---- The associated coldkey account.
    /// Staking and unstaking transactions must be made by this account.
    /// The hotkey account (in the Neurons map) has permission to call emit
    /// subscribe and unsubscribe.
    pub coldkey: AccountId,
}

// ---- Subtensor storage items.
decl_storage! {
	trait Store for Module<T: Trait> as SubtensorModule {

		/// ---- Active set map between a hotkey account and network uids.
		/// Used by subtensor for checking peer existence.
		pub Active get(fn uid): map hasher(blake2_128_concat) T::AccountId => u64;

		/// ----  Maps between a neuron's hotkey account address and additional 
		/// metadata associated with that neuron. All other maps, map between the with a uid. 
		/// The metadata contains that uid, the ip, port, and coldkey address.
		pub Neurons get(fn neuron): map hasher(identity) u64 => NeuronMetadataOf<T>;

		/// ---- Maps between a neuron's hotkey uid and the block number
		/// when that peer last called an emission/subscribe.
		pub LastEmit get(fn last_emit): map hasher(identity) u64 => T::BlockNumber;

		/// --- Maps between a neuron's hotkey uid and this peer's pending emission.
		/// pending emission is the quantity 
		pub PendingEmission get(fn pending_emission ): map hasher(identity) u64 => u64;

		/// ---- List of values which map between a neuron's uid an that neuron's
		/// weights, a.k.a is row_weights in the square matrix W. Each outward edge
		/// is represented by a (u64, u64) tuple determining the endpoint and weight
		/// value respectively. Each giga byte of chain storage can hold history for
		/// 83 million weights. 
		pub WeightUids: map hasher(identity) u64 => Vec<u64>;
		pub WeightVals: map hasher(identity) u64 => Vec<u32>;
		
		/// ----  Maps between a neuron's hotkey uid and the number of
		/// staked tokens under that key.
		pub Stake get(fn stake): map hasher(identity) u64 => u64;

		/// ---- Stores the amount of currently staked token.
		TotalStake: u64;

		/// ---- The next uid allocated to a subscribing neuron. Or a count of how many peers
		/// have ever subscribed.
		NextUID: u64;

		/// ---- The number of subscriptions this block, used in conjunction with 
		SubscriptionsThisBlock: u32;
		LastSubscriptionBlock: T::BlockNumber;
	}

	add_extra_genesis {
        config(pending_emissions): Vec<(u64, u64)>;
        config(stake): Vec<(u64, u64)>;
        build(|config| {
            for (uid, emission) in &config.pending_emissions {
                PendingEmission::insert(uid, emission);
            };

            for (uid, stake) in &config.stake {
                Stake::insert(uid, stake);
            };
        })
    }
}

// ---- Subtensor events.
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// ---- Event created when a caller successfully set's their weights
		/// on the chain.
		WeightsSet(AccountId),

		/// --- Event created when a new neuron account has been subscribed to 
		/// the neuron set.
		NeuronAdded(u64),

		/// --- Event created when the neuron information associated with a hotkey
		/// is changed, for instance, when the ip/port changes.
		NeuronUpdated(u64),

		/// --- Event created during when stake has been transfered from 
		/// the coldkey onto the hotkey staking account.
		StakeAdded(AccountId, u64),

		/// --- Event created when stake has been removed from 
		/// the staking account into the coldkey account.
		StakeRemoved(AccountId, u64),

		/// --- Event created when a transaction triggers and incentive
		/// mechanism emission.
		Emission(AccountId, u64),
	}
);

// ---- Subtensor Errors.
decl_error! {
	pub enum Error for Module<T: Trait> {
	    /// ---- Thrown when the user tries to subscribe a neuron which is not of type
	    /// 4 (IPv4) or 6 (IPv6).
		InvalidIpType,

		/// --- Thrown when an invalid IP address is passed to the subscribe function.
		InvalidIpAddress,

		/// --- Thrown when an invalid modality attempted on subscribe.
		/// Currently the chain only accepts modality TEXT = 0.
		InvalidModality,

		/// --- Thrown when subscriptions this block have exeeded the number of 
		/// allowed.
		ToManySubscriptionsThisBlock,

		/// ---- Thrown when the caller attempts to set the weight keys
		/// and values but these vectors have different size.
		WeightVecNotEqualSize,

		/// ---- Thrown when the caller attempts to set weights with duplicate uids
		/// in the weight matrix.
		DuplicateUids,

		/// ---- Thrown when a caller attempts to set weight to at least one uid that
		/// does not exist in the metagraph.
		InvalidUid,

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
		/// See: fn add_stake and fn remove_stake.
		NonAssociatedColdKey,

		/// ---- Thrown when the caller requests removing more stake then there exists 
		/// in the staking account. See: fn remove_stake.
		NotEnoughStaketoWithdraw,

		///  ---- Thrown when the caller requests adding more stake than there exists
		/// in the cold key account. See: fn add_stake
		NotEnoughBalanceToStake,

		/// ---- Thrown when the caller tries to add stake, but for some reason the requested
		/// amount could not be withdrawn from the coldkey account
		BalanceWithdrawalError,

		/// ---- Thrown when the dispatch attempts to convert between a u64 and T::balance 
		/// but the call fails.
		CouldNotConvertToBalance
	}
}


impl<T: Trait> Printable for Error<T> {
    fn print(&self) {
        match self {
            Error::AlreadyActive => "The node with the supplied public key is already active".print(),
            Error::NotActive => "The node with the supplied public key is not active".print(),
            Error::NothingToEmit => "There is nothing to emit".print(),
            Error::WeightVecNotEqualSize => "The vec of keys and the vec of values are not of the same size".print(),
            Error::NonAssociatedColdKey => "The used cold key is not associated with the hot key acccount".print(),
            _ => "Invalid Error Case".print(),
        }
    }
}

// ---- Subtensor Dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;
		// type Balance = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;


		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		const TransactionByteFee: BalanceOf<T> = T::TransactionByteFee::get();


		/// --- Sets the caller weights for the incentive mechanism. The call can be
		/// made from the hotkey account so is potentially insecure, however, the damage
		/// of changing weights is minimal if caught early. This function includes all the
		/// checks that the passed weights meet the requirements. Stored as u64s they represent
		/// rational values in the range [0,1] which sum to 1 and can be interpreted as
		/// probabilities. The specific weights determine how inflation propagates outward
		/// from this peer. Because this function changes the inflation distribution it
		/// triggers an emit before values are changed on the chain.
		/// 
		/// Note: The 32 bit integers weights should represent 1.0 as the max u64.
		/// However, the function normalizes all integers to u64_max anyway. This means that if the sum of all
		/// elements is larger or smaller than the amount of elements * u64_max, all elements
		/// will be corrected for this deviation. 
		/// 
		/// # Args:
		/// 	* `origin`: (<T as frame_system::Trait>Origin):
		/// 		- The caller, a hotkey who wishes to set their weights.
		/// 
		/// 	* `uids` (Vec<u64>):
		/// 		- The edge endpoint for the weight, i.e. j for w_ij.
		///
		/// 	* 'weights' (Vec<u64>):
		/// 		- The u64 integer encoded weights. Interpreted as rational
		/// 		values in the range [0,1]. They must sum to in32::MAX.
		///
		/// # Emits:
		/// 	* WeightsSet;
		/// 		- On successfully setting the weights on chain.
		///
		/// # Raises:
		/// 	* 'WeightVecNotEqualSize':
		/// 		- If the passed weights and uids have unequal size.
		///
		/// 	* 'WeightSumToLarge':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'InsufficientBalance':
		/// 		- When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
		#[weight = (0, DispatchClass::Normal, Pays::No)]
		pub fn set_weights(origin, dests: Vec<u64>, weights: Vec<u32>) -> dispatch::DispatchResult {
			Self::do_set_weights(origin, dests, weights)
		}

		/// --- Adds stake to a neuron account. The call is made from the
		/// coldkey account linked in the neurons's NeuronMetadata.
		/// Only the associated coldkey is allowed to make staking and
		/// unstaking requests. This protects the neuron against
		/// attacks on its hotkey running in production code.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Trait>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The hotkey account to add stake to.
		///
		/// 	* 'ammount_staked' (u64):
		/// 		- The ammount to transfer from the balances account of the cold key
		/// 		into the staking account of the hotkey.
		///
		/// # Emits:
		/// 	* 'StakeAdded':
		/// 		- On the successful staking of funds.
		///
		/// # Raises:
		/// 	* 'NotActive':
		/// 		- If the hotkey account is not active (has not subscribed)
		///
		/// 	* 'NonAssociatedColdKey':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'InsufficientBalance':
		/// 		- When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
		#[weight = (0, DispatchClass::Normal, Pays::Yes)]
		pub fn add_stake(origin, hotkey: T::AccountId, ammount_staked: u64) -> dispatch::DispatchResult {
			Self::do_add_stake(origin, hotkey, ammount_staked)
		}

		/// ---- Remove stake from the staking account. The call must be made
		/// from the coldkey account attached to the neuron metadata. Only this key
		/// has permission to make staking and unstaking requests.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Trait>Origin):
		/// 		- The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	* 'hotkey' (T::AccountId):
		/// 		- The hotkey account to withdraw stake from.
		///
		/// 	* 'ammount_unstaked' (u64):
		/// 		- The ammount to transfer from the staking account into the balance
		/// 		of the coldkey.
		///
		/// # Emits:
		/// 	* 'StakeRemoved':
		/// 		- On successful withdrawl.
		///
		/// # Raises:
		/// 	* 'NonAssociatedColdKey':
		/// 		- When the calling coldkey is not associated with the hotkey account.
		///
		/// 	* 'NotEnoughStaketoWithdraw':
		/// 		- When the amount to unstake exceeds the quantity staked in the
		/// 		associated hotkey staking account.
		///
		#[weight = (0, DispatchClass::Normal, Pays::Yes)]
		pub fn remove_stake(origin, hotkey: T::AccountId, ammount_unstaked: u64) -> dispatch::DispatchResult {
			Self::do_remove_stake(origin, hotkey, ammount_unstaked)
		}

		/// ---- Subscribes or updates info for caller with the given metadata. If the caller
		/// already exists in the active set, the metadata is updated but the cold key remains unchanged.
		/// If the caller does not exist they make a link between this hotkey account
		/// and the passed coldkey account. Only the cold key has permission to make add_stake/remove_stake calls.
		///
		/// # Args:
		/// 	* 'origin': (<T as frame_system::Trait>Origin):
		/// 		- The caller, a hotkey associated with the subscribing neuron.
		///
		/// 	* 'ip' (u128):
		/// 		- The u64 encoded IP address of type 6 or 4.
		///
		/// 	* 'port' (u16):
		/// 		- The port number where this neuron receives RPC requests.
		///
		/// 	* 'ip_type' (u8):
		/// 		- The ip type one of (4,6).
		/// 
		/// 	* 'modality' (u8):
		/// 		- The neuron modality type.
		///
		/// 	* 'coldkey' (T::AccountId):
		/// 		- The associated coldkey to be attached to the account.
		///
		/// # Emits:
		/// 	* 'NeuronAdded':
		/// 		- On subscription of a new neuron to the active set.
		///
		/// 	* 'NeuronUpdated':
		/// 		- On subscription of new metadata attached to the calling hotkey.
		#[weight = (0, DispatchClass::Normal, Pays::No)]
		pub fn subscribe(origin, ip: u128, port: u16, ip_type: u8, modality: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {
			Self::do_subscribe(origin, ip, port, ip_type, modality, coldkey)
		}

		/// ---- Finalizes the block by updating pallet state. This function is not publicly available
		/// as an RPC but is called at the end of block execution. Subtensor's on_finalize updates
		/// the pending emission for each active peer and performs additional clean up.
		///
		/// # Args:
		/// 	* 'n': (T::BlockNumber):
		/// 		- The number of the block we are finalizing.
		///
		fn on_finalize(n: T::BlockNumber) {
			// Self::do_finalize(n);
		}

		/// ---- Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
		///
		/// # Args:
		/// 	* 'n': (T::BlockNumber):
		/// 		- The number of the block we are initializing.
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::update_pending_emissions()
		}
	}
}


// ---- Subtensor helper functions.
impl<T: Trait> Module<T> {
    // --- Returns Option if the u64 converts to a balance
    // use .unwarp if the result returns .some().
    pub fn u64_to_balance(input: u64) -> Option<<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance>
    {
        input.try_into().ok()
    }

    // --- Returns true if the account-id has an active
    // account on chain.
    pub fn add_hotkey_to_active_set(hotkey_id: &T::AccountId, uid: u64) {
        Active::<T>::insert(&hotkey_id, uid);
    }

    // --- Returns true if the account-id has an active
    // account on chain.
    pub fn is_hotkey_active(hotkey_id: &T::AccountId) -> bool {
        return Active::<T>::contains_key(&hotkey_id);
    }

    // --- Returns false if the account-id has an active
    // account on chain.
    pub fn is_not_active(hotkey_id: &T::AccountId) -> bool {
        return !Self::is_hotkey_active(hotkey_id);
    }

    // --- Returns true if the uid is active, i.e. there
    // is a staking, last_emit, and neuron account associated
    // with this uid.
    pub fn is_uid_active(uid: u64) -> bool {
        return Neurons::<T>::contains_key(uid);
    }

    // --- Returns hotkey associated with the hotkey account.
    // This should be called in conjunction with is_hotkey_active
    // to ensure this function does not throw an error.
    pub fn get_uid_for_hotkey(hotkey_id: &T::AccountId) -> u64 {
        return Active::<T>::get(&hotkey_id);
    }

    // --- Returns the neuron associated with the passed uid.
    // The function makes a single mapping from uid -> neuron.
    pub fn get_neuron_for_uid(uid: u64) -> NeuronMetadataOf<T> {
        return Neurons::<T>::get(uid);
    }

    // --- Returns the neuron associated with the passed hotkey.
    // The function makes a double mapping from hotkey -> uid -> neuron.
    pub fn get_neuron_for_hotkey(hotkey_id: &T::AccountId) -> NeuronMetadataOf<T> {
        let uid = Self::get_uid_for_hotkey(hotkey_id);
        return Self::get_neuron_for_uid(uid);
    }

    // --- Returns the next available network uid.
    // uids increment up to u64:MAX, this allows the chain to
    // have 18,446,744,073,709,551,615 peers before an overflow.
    pub fn get_neuron_count() -> u64 {
        let uid = NextUID::get();
        uid
    }

    // --- Returns the next available network uid.
    // uids increment up to u64:MAX, this allows the chain to
    // have 18,446,744,073,709,551,615 peers before an overflow.
    pub fn get_next_uid() -> u64 {
        let uid = NextUID::get();
        assert!(uid < u64::MAX);  // The system should fail if this is ever reached.
        NextUID::put(uid + 1);
        debug::info!("Incrementing the next uid by 1, now {:?} ", NextUID::get());
        uid
    }


    pub fn calculate_transaction_fee(len: u64) -> u64 {
        return len * 100;
    }

    pub fn can_pay_transaction_fee_from_coldkey_account(balance: <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance, amount: u64, transaction_fee: u64) -> bool
    {
        return balance - Self::u64_to_balance(amount).unwrap() > Self::u64_to_balance(transaction_fee).unwrap();
    }
}

/************************************************************
	CallType definition
************************************************************/

#[derive(Debug, PartialEq)]
pub enum CallType {
    SetWeights,
    AddStake,
    RemoveStake,
    Subscribe,
    Other,
}

impl Default for CallType {
    fn default() -> Self {
        CallType::Other
    }
}


type TransactionFee = u64;

/************************************************************
	ChargeTransactionPayment definition
************************************************************/

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ChargeTransactionPayment<T: Trait + Send + Sync>(pub PhantomData<T>);

impl<T: Trait + Send + Sync> ChargeTransactionPayment<T> where
    T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
    <T as frame_system::Trait>::Call: dispatch::IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn can_pay_set_weights(who: &T::AccountId) -> Result<TransactionFee, TransactionValidityError> {
        let transaction_fee = Module::<T>::get_self_emission_for_caller(who);
        Ok(transaction_fee)
    }

    pub fn can_pay_add_stake(who: &T::AccountId, len: u64) -> Result<TransactionFee, TransactionValidityError> {
        let transaction_fee = Module::<T>::calculate_transaction_fee(len as u64);
        let transaction_fee_as_balance = Module::<T>::u64_to_balance(transaction_fee);

        if Module::<T>::can_remove_balance_from_coldkey_account(&who, transaction_fee_as_balance.unwrap()) {
            Ok(transaction_fee)
        } else {
            Err(InvalidTransaction::Payment.into())
        }
    }

    pub fn can_pay_remove_stake(who: &T::AccountId, hotkey_id: &T::AccountId, len: u64) -> Result<TransactionFee, TransactionValidityError> {
        let neuron = Module::<T>::get_neuron_for_hotkey(&hotkey_id);
        let transaction_fee = Module::<T>::calculate_transaction_fee(len as u64);
        let transaction_fee_as_balance = Module::<T>::u64_to_balance(transaction_fee).unwrap();

        if Module::<T>::can_remove_balance_from_coldkey_account(&who, transaction_fee_as_balance) ||
            Module::<T>::has_enough_stake(&neuron, transaction_fee) {
            Ok(transaction_fee)
        } else {
            Err(InvalidTransaction::Payment.into())
        }
    }

    pub fn can_pay_subscribe() -> Result<TransactionFee, TransactionValidityError> {
        Ok(0)
    }

    pub fn can_pay_other(info: &DispatchInfoOf<T::Call>, who: &T::AccountId, len: u64) -> Result<TransactionFee, TransactionValidityError> {
        let transaction_fee = Module::<T>::calculate_transaction_fee(len as u64);

        if info.pays_fee == Pays::No {
            return Ok(transaction_fee);
        }

        let transaction_fee_as_balance = Module::<T>::u64_to_balance(transaction_fee);
        if Module::<T>::can_remove_balance_from_coldkey_account(&who, transaction_fee_as_balance.unwrap()) {
            Ok(transaction_fee)
        } else {
            Err(InvalidTransaction::Payment.into())
        }
    }

    pub fn get_priority_set_weights(transaction_fee: u64, len: u64) -> u64 {
        // Sanity check
        if len == 0 {
            return 0;
        }
        return transaction_fee / len;
    }

    pub fn get_priority_vanilla() -> u64 {
        // Just return a rediculously high priority. This means that all extrinsics exept
        // the set_weights function will have a priority over the set_weights calls.
        // This should probably be refined in the future.
        return u64::max_value();
    }
}


impl<T: Trait + Send + Sync> sp_std::fmt::Debug for ChargeTransactionPayment<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ChargeTransactionPayment")
    }
}

impl<T: Trait + Send + Sync> SignedExtension for ChargeTransactionPayment<T>
    where
        T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
        <T as frame_system::Trait>::Call: dispatch::IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";

    type AccountId = T::AccountId;
    type Call = T::Call;
    //<T as frame_system::Trait>::Call;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);
    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> { Ok(()) }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        match call.is_sub_type() {
            Some(Call::set_weights(..)) => {
                let transaction_fee = Self::can_pay_set_weights(who)?;
                Ok(ValidTransaction {
                    priority: Self::get_priority_set_weights(transaction_fee, len as u64),
                    longevity: 1,
                    ..Default::default()
                })
            }
            Some(Call::add_stake(..)) => {
                let _transaction_fee = Self::can_pay_add_stake(who, len as u64)?;
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::remove_stake(hotkey_id, ..)) => {
                let _transaction_fee = Self::can_pay_remove_stake(who, hotkey_id, len as u64)?;
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            Some(Call::subscribe(..)) => {
                let _transaction_fee = Self::can_pay_subscribe()?;
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
            _ => {
                let _transaction_fee = Self::can_pay_other(info, who, len as u64)?;
                Ok(ValidTransaction {
                    priority: Self::get_priority_vanilla(),
                    ..Default::default()
                })
            }
        }
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        match call.is_sub_type() {
            Some(Call::set_weights(..)) => {
                // To pay for the set_weights operation, the self_weight of a neuron is used for payment
                // This can be >= 0, however the lower the self weight, the lower the priority in the block
                // and may result the transaction is not put into a block
                let transaction_fee = Self::can_pay_set_weights(who)?;
                Ok((CallType::SetWeights, transaction_fee, who.clone())) // 0 indicates that post_dispatch should use the self-weight to pay for the transaction
            }
            Some(Call::add_stake(..)) => {
                // The transaction fee for the add_stake function is paid from the coldkey balance
                // let transaction_fee = Module::<T>::calculate_transaction_fee(len as u64);
                // let transaction_fee_as_balance = Module::<T>::u64_to_balance( transaction_fee );
                let transaction_fee = Self::can_pay_add_stake(who, len as u64)?;
                Ok((CallType::AddStake, transaction_fee, who.clone()))
            }
            Some(Call::remove_stake(hotkey_id, ..)) => {
                // The tranaction fee for the remove_stake call is paid from the coldkey balance
                // after the transaction completes. For this, a check is done on both the stake
                // as well as the coldkey balance to see if one of both is sufficient to pay
                // for the transaction

                let transaction_fee = Self::can_pay_remove_stake(who, hotkey_id, len as u64)?;
                Ok((CallType::RemoveStake, transaction_fee, who.clone()))
            }
            Some(Call::subscribe(..)) => {
                let transaction_fee = Self::can_pay_subscribe()?;
                Ok((CallType::Subscribe, transaction_fee, who.clone()))
            }
            _ => {
                let transaction_fee = Self::can_pay_other(info, who, len as u64)?;
                Ok((CallType::Other, transaction_fee, who.clone()))
            }
        }
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        let call_type = pre.0;
        let transaction_fee = pre.1;
        let coldkey_id = pre.2;
        let transaction_fee_as_balance = Module::<T>::u64_to_balance(transaction_fee).unwrap();

        match result {
            Ok(_) => {
                match call_type {
                    CallType::SetWeights => {
                        Module::<T>::deposit_self_emission_into_adam(transaction_fee);
                        Ok(Default::default())
                    }
                    CallType::AddStake => {
                        Module::<T>::remove_balance_from_coldkey_account(&coldkey_id, transaction_fee_as_balance);
                        Module::<T>::add_stake_to_neuron_hotkey_account(0, transaction_fee); // uid 0 == Adam
                        Ok(Default::default())
                    }
                    CallType::RemoveStake => {
                        Module::<T>::remove_balance_from_coldkey_account(&coldkey_id, transaction_fee_as_balance);
                        Module::<T>::add_stake_to_neuron_hotkey_account(0, transaction_fee); // uid 0 == Adam
                        Ok(Default::default())
                    }
                    CallType::Subscribe => {
                        Ok(Default::default())
                    }
                    _ => {
                        // Default behaviour for calls not otherwise specified
                        match info.pays_fee {
                            Pays::No => Ok(Default::default()),
                            Pays::Yes => {
                                Module::<T>::remove_balance_from_coldkey_account(&coldkey_id, transaction_fee_as_balance);
                                Module::<T>::add_stake_to_neuron_hotkey_account(0, transaction_fee); // uid 0 == Adam
                                Ok(Default::default())
                            }
                        }
                    }
                }
            }
            Err(_) => Ok(Default::default())
        }
    }
}


/************************************************************
	Transaction payments
************************************************************/
//
type BalanceOf<T> =
<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

// type NegativeImbalanceOf<T> =
// 	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
//
//
// /// Require the transactor pay for themselves and maybe include a tip to gain additional priority
// /// in the queue.
// #[derive(Encode, Decode, Clone, Eq, PartialEq)]
// pub struct ChargeTransactionPayment<T: Trait + Send + Sync>(#[codec(compact)] BalanceOf<T>);
//
// impl<T: Trait + Send + Sync> ChargeTransactionPayment<T> where
// 	T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
// 	BalanceOf<T>: Send + Sync + FixedPointOperand,
// {
// 	pub fn new() -> Self {
// 		Self(Default::default())
// 	}
// }
//
// impl<T: Trait + Send + Sync> sp_std::fmt::Debug for ChargeTransactionPayment<T> {
// 	#[cfg(feature = "std")]
// 	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
// 		write!(f, "ChargeTransactionPayment<{:?}>", self.0)
// 	}
// 	#[cfg(not(feature = "std"))]
// 	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
// 		Ok(())
// 	}
// }
//
// impl<T: Trait + Send + Sync> SignedExtension for ChargeTransactionPayment<T> where
// 	BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
// 	T::Call: Dispatchable<Info=DispatchInfo, PostInfo=PostDispatchInfo>,
// 	<T as frame_system::Trait>::Call: dispatch::IsSubType<Call<T>>,
// {
// 	const IDENTIFIER: &'static str = "ChargeTransactionPayment";
// 	type AccountId = T::AccountId;
// 	type Call = T::Call;
// 	type AdditionalSigned = ();
// 	type Pre = (BalanceOf<T>, Self::AccountId, Option<NegativeImbalanceOf<T>>, BalanceOf<T>);
// 	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> { Ok(()) }
//
// 	fn validate(
// 		&self,
// 		_who: &Self::AccountId,
// 		_call: &Self::Call,
// 		_info: &DispatchInfoOf<Self::Call>,
// 		_len: usize,
// 	) -> TransactionValidity {
// 		// let (fee, _) = self.withdraw_fee(who, info, len)?;
// 		// Ok(ValidTransaction {
// 		// 	priority: Self::get_priority(len, info, fee),
// 		// 	..Default::default()
// 		// })
// 		Ok(Default::default())
// 	}
//
// 	fn pre_dispatch(
// 		self,
// 		_who: &Self::AccountId,
// 		call: &Self::Call,
// 		_info: &DispatchInfoOf<Self::Call>,
// 		_len: usize
// 	) -> Result<Self::Pre, TransactionValidityError> {
// 		match call.is_sub_type() {
// 			Some(Call::set_weights(..)) => {
// 				// The payment of set_weight extrinsics is handled by the FeeFromSelfEmission signed extension.
// 				Ok(Default::default())
// 			}
// 			_ => Err(InvalidTransaction::Payment.into())
// 		}
//
//
// 		// println!("SHOULD NOT COME HERE");
//
// 		// let (fee, imbalance) = self.withdraw_fee(who, info, len)?;
// 		// Ok((self.0, who.clone(), imbalance, fee))
// 		// Ok(Default::default())
// 	}
//
// 	fn post_dispatch(
// 		_pre: Self::Pre,
// 		_info: &DispatchInfoOf<Self::Call>,
// 		_post_info: &PostDispatchInfoOf<Self::Call>,
// 		_len: usize,
// 		_result: &DispatchResult,
// 	) -> Result<(), TransactionValidityError> {
// 		// let (tip, who, imbalance, fee) = pre;
// 		// if let Some(payed) = imbalance {
// 		// 	let actual_fee = Module::<T>::compute_actual_fee(
// 		// 		len as u32,
// 		// 		info,
// 		// 		post_info,
// 		// 		tip,
// 		// 	);
// 		// 	let refund = fee.saturating_sub(actual_fee);
// 		// 	let actual_payment = match T::Currency::deposit_into_existing(&who, refund) {
// 		// 		Ok(refund_imbalance) => {
// 		// 			// The refund cannot be larger than the up front payed max weight.
// 		// 			// `PostDispatchInfo::calc_unspent` guards against such a case.
// 		// 			match payed.offset(refund_imbalance) {
// 		// 				Ok(actual_payment) => actual_payment,
// 		// 				Err(_) => return Err(InvalidTransaction::Payment.into()),
// 		// 			}
// 		// 		}
// 		// 		// We do not recreate the account using the refund. The up front payment
// 		// 		// is gone in that case.
// 		// 		Err(_) => payed,
// 		// 	};
// 		// 	let imbalances = actual_payment.split(tip);
// 		// 	T::OnTransactionPayment::on_unbalanceds(Some(imbalances.0).into_iter()
// 		// 		.chain(Some(imbalances.1)));
// 		// }
// 		Ok(())
// 	}
// }


