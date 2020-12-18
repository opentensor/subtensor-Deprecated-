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


		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn set_weights(origin,
				dests: Vec<T::AccountId>,
				values: Vec<u32>) -> dispatch::DispatchResult {

		 	let neuron = ensure_signed(origin)?;

		 	ensure!(values.len() == dests.len(), Error::<T>::WeightVecNotEqualSize);

			let normalized_values = normalize(values);

			WeightVals::<T>::(&neuron, &normalized_values);
			WeightKeys::<T>::insert(&neuron, &dests);

			// Emit and return
			Self::deposit_event(RawEvent::WeightsSet(neuron));
			Ok(())
		}



		/// Emission. Called by an active Neuron. Distributes inflation neighbors and to themselves.
		/// The amount emitted = (blocks_since_last_emit) * (inflation_per_block) * (this_neurons_stake) / (total_stake)
		/// Neurons are incentivized to call this function often as to maximize inflation in the graph.
		/// Along with emission, the neuron is given the opportunity to set their weights with this function call.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn emit(origin, 
			dests: Vec<T::AccountId>, 
			values: Vec<u32>) -> dispatch::DispatchResult {
			RuntimeLogger::init();
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let calling_neuron = ensure_signed(origin)?;
			debug::info!("Emit sent by: {:?}", calling_neuron);

			// Check that the Neuron exists in the Neuron set.
			// Check that the Neuron has stake to emit.
			// Check that the Neuron has weights set. etc.
			ensure!(Neurons::<T>::contains_key(&calling_neuron), Error::<T>::NotActive);
			ensure!(Stake::<T>::contains_key(&calling_neuron), Error::<T>::NotActive);
			ensure!(LastEmit::<T>::contains_key(&calling_neuron), Error::<T>::NotActive);
			ensure!(WeightKeys::<T>::contains_key(&calling_neuron), Error::<T>::NotActive);
			ensure!(WeightVals::<T>::contains_key(&calling_neuron), Error::<T>::NotActive);

			// Get the last emission block.
			// Get the current block.
			// Get the block reward at this current block.
			// Set the current block as last emit.
			let last_block: T::BlockNumber = LastEmit::<T>::get(&calling_neuron);
			let current_block = system::Module::<T>::block_number();
			let block_reward = Self::block_reward(&current_block);
			LastEmit::<T>::insert(&calling_neuron, current_block);
			debug::info!("Last emit block: {:?}", last_block);
			debug::info!("Current block: {:?}", current_block);
			debug::info!("Block reward: {:?}", block_reward);

			// Get the number of elapsed blocks since last emit.
			// Convert to u32f32.
			let elapsed_blocks = current_block - last_block;
			let elapsed_blocks_u32: usize = TryInto::try_into(elapsed_blocks)
			.ok()
			.expect("blockchain will not exceed 2^32 blocks; qed");
			let elapsed_blocks_u32_f32 = U32F32::from_num(elapsed_blocks_u32);
			debug::info!("elapsed_blocks_u32: {:?}", elapsed_blocks_u32);
			debug::info!("elapsed_blocks_u32_f32: {:?}", elapsed_blocks_u32_f32);

			// Get local and total stake.
			// Convert to u32f32.
			// Calculate stake fraction.
			let total_stake: u32  = TotalStake::get();
			let total_stake_u32_f32 = U32F32::from_num(total_stake);
			let local_stake: u32 = Stake::<T>::get(&calling_neuron);
			let local_stake_u32_f32 = U32F32::from_num(local_stake);

			let mut stake_fraction_u32_f32 = U32F32::from_num(1);
			if total_stake_u32_f32 > 0 {
				stake_fraction_u32_f32 = local_stake_u32_f32 / total_stake_u32_f32;
			}

			debug::info!("total_stake_u32_f32 {:?}", total_stake_u32_f32);
			debug::info!("local_stake_u32_f32 {:?}", local_stake_u32_f32);
			debug::info!("stake_fraction_u32_f32 {:?}", stake_fraction_u32_f32);

			// Calculate total emission at this Neuron based on times since last emit
			// stake fraction and block reward.
			let total_emission_u32_f32 = stake_fraction_u32_f32 * block_reward * elapsed_blocks_u32_f32;
			let total_emission_u32 = total_emission_u32_f32.to_num::<u32>();
			debug::info!("total_emission_u32_f32 {:?} = {:?}*{:?}*{:?}", total_emission_u32_f32, stake_fraction_u32_f32, block_reward, elapsed_blocks_u32_f32);
			ensure!(total_emission_u32_f32 > U32F32::from_num(0), Error::<T>::NothingToEmit);

			// Get current weights and vals from storage.
			// Get the weight sum for normalization.
			let w_keys: Vec<T::AccountId> = WeightKeys::<T>::get(&calling_neuron);
			let w_vals: Vec<u32> = WeightVals::<T>::get(&calling_neuron);
			let mut w_sum = U32F32::from_num(0);
			let u32_max = U32F32::from_num(u32::MAX);
			for x in w_vals.iter() {
				// Overflow not possible since we check weight prior to adding to it
				let x_u32_f32 = U32F32::from_num(*x);
				if u32_max - x_u32_f32 <= w_sum {
					w_sum = w_sum + x_u32_f32;
				}
			}
		
			// Iterate through weight matrix and distribute emission to 
			// neurons on a weighted basis. 
			for (i, dest_key) in w_keys.iter().enumerate() {

				// Get emission to Neuron j from Neuron i.
				let wij_u32_f32 = U32F32::from_num(w_vals[i]);
				let wij_norm_u32_f32 = wij_u32_f32 / w_sum;
				let emission_u32_f32 = total_emission_u32_f32 * wij_norm_u32_f32;
				debug::info!("emit to {:?}", dest_key);
				debug::info!("wij {:?}", wij_norm_u32_f32);
				debug::info!("emission_u32_f32 {:?}", emission_u32_f32);

				// Determine stake ammount for Neuron j.
				let prev_stake: u32 = Stake::<T>::get(&dest_key);
				let prev_stake_u32_f32 = U32F32::from_num(prev_stake);
				let new_stake_u32_f32 = prev_stake_u32_f32 + emission_u32_f32;
				let new_stake_u32: u32 = new_stake_u32_f32.to_num::<u32>();
				debug::info!("prev_stake_u32_f32 {:?}", prev_stake_u32_f32);
				debug::info!("new_stake_u32_f32 {:?} = {:?} + {:?}", new_stake_u32_f32, prev_stake_u32_f32, emission_u32_f32);
				debug::info!("new_stake_u32 {:?}", new_stake_u32);

				// Update stake in storage.
				// Update total stake in storage.
				Stake::<T>::insert(&dest_key, new_stake_u32);
				let total_stake: u32  = TotalStake::get();
				TotalStake::put(total_stake + new_stake_u32); // TODO (const): check overflow.
				debug::info!("sink new stake.");
				
				let withdraw_amount = Self::u32_to_balance(new_stake_u32);
				let _ = T::Currency::withdraw(&calling_neuron, withdraw_amount, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
				debug::info!("Balance is {:?}", T::Currency::total_balance(&calling_neuron));
			}

			// if !values.is_empty() && !dests.is_empty() {
			// 	Self::set_weights(&calling_neuron, dests, values)?;
			// }
			
			Self::deposit_event(RawEvent::Emission(calling_neuron, total_emission_u32));
			// Return.
			Ok(())
		}

		// --- Adds stake to a neuron account. The call is made from the
		// coldkey account linked in the neurons's NeuronMetadata.
		// Only the associated coldkey is allowed to make staking and
		// unstaking requests. This protects the neuron against
		// attacks on its hotkey running in production code.
		/// Args:
		/// 	origin: (<T as frame_system::Trait>Origin):
		/// 		The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	hotkey (T::AccountId):
		/// 		The hotkey account to add stake to.
		///
		/// 	ammount_staked (u32):
		/// 		The ammount to transfer from the balances account of the cold key
		/// 		into the staking account of the hotkey.
		///
		/// Emits:
		/// 	StakeAdded:
		/// 		On the successful staking of funds.
		///
		/// Raises
		/// 	NotActive:
		/// 		If the hotkey account is not active (has not subscribed)
		///
		/// 	NonAssociatedColdKey:
		/// 		When the calling coldkey is not associated with the hotkey account.
		///
		/// 	InsufficientBalance:
		/// 		When the amount to stake exceeds the amount of balance in the
		/// 		associated colkey account.
		///
		#[weight = (0, DispatchClass::Operational, Pays::No)] // TODO(const): should be a normal transaction fee.
		fn add_stake(origin, hotkey: T::AccountId, ammount_staked: u32) -> dispatch::DispatchResult {

			// ---- We check the transaction is signed by the caller
			// and retrieve the T::AccountId pubkey information.
			let caller = ensure_signed(origin)?;
			debug::info!("--- Called add_stake with caller {:?}, hotkey {:?} and ammount_staked {:?}", caller, hotkey, ammount_staked);

			// ---- We query the Neuron set for the neuron data stored under
			// the passed hotkey and retrieve it as a NeuronMetadata struct.
			ensure!(Neurons::<T>::contains_key(&hotkey), Error::<T>::NotActive);
			let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&hotkey);
			debug::info!("Got metadata for hotkey {:?}", hotkey);

			// ---- We check that the NeuronMetadata is linked to the calling
			// cold key, otherwise throw a NonAssociatedColdKey error.
			ensure!(neuron.coldkey == caller, Error::<T>::NonAssociatedColdKey);

			// ---- We check that the calling coldkey contains enough funds to
			// create the staking transaction.
			let staked_currency = Self::u32_to_balance( ammount_staked );
			let new_potential_balance = T::Currency::free_balance(&caller) - staked_currency;
			let can_withdraw = T::Currency::ensure_can_withdraw(&caller, staked_currency, WithdrawReasons::except(WithdrawReason::Tip), new_potential_balance).is_ok();

			// ---- If we can withdraw the requested funds, we withdraw from the
			// coldkey account and deposit the funds into the staking account of
			// the associated hotkey-neuron.
			if can_withdraw {

				// ---- We perform the withdrawl from the coldkey account before
				// addding stake into the hotkey neuron's staking account.
				let _ = T::Currency::withdraw(&caller, staked_currency, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
				debug::info!("Withdrew {:?} from coldkey: {:?}", staked_currency, caller);

				// --- We update the hotkey's staking account with the new funds.
				let hotkey_stake: u32 = Stake::<T>::get(&hotkey);
				Stake::<T>::insert(&hotkey, hotkey_stake + ammount_staked);
				debug::info!("Added new stake: {:?} to hotkey {:?}", ammount_staked, hotkey);

				// --- We update the total staking pool with the new funds.
				let total_stake: u32 = TotalStake::get();
				TotalStake::put(total_stake + ammount_staked);
				debug::info!("Added {:?} to total stake, now {:?}", ammount_staked, TotalStake::get());

				// ---- Emit the staking event.
				Self::deposit_event(RawEvent::StakeAdded(hotkey, ammount_staked));

			} else {

				debug::info!("Could not withdraw {:?} from coldkey {:?}", staked_currency, caller);
			}

			// --- ok and return.
			debug::info!("--- Done add_stake.");
			Ok(())
		}

		// ---- Remove stake from the staking account. The call must be made
		// from the coldkey account attached to the neuron metadata. Only this key
		// has permission to make staking and unstaking requests.
		/// Args:
		/// 	origin: (<T as frame_system::Trait>Origin):
		/// 		The caller, a coldkey signature associated with the hotkey account.
		///
		/// 	hotkey (T::AccountId):
		/// 		The hotkey account to withdraw stake from.
		///
		/// 	ammount_unstaked (u32):
		/// 		The ammount to transfer from the staking account into the balance
		/// 		of the coldkey.
		/// Emits:
		/// 	StakeRemoved:
		/// 		On successful withdrawl.
		///
		/// Raises
		/// 	NonAssociatedColdKey:
		/// 		When the calling coldkey is not associated with the hotkey account.
		///
		/// 	NotEnoughStaketoWithdraw:
		/// 		When the amount to unstake exceeds the quantity staked in the
		/// 		associated hotkey staking account.
		///
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn remove_stake(origin, hotkey: T::AccountId, ammount_unstaked: u32) -> dispatch::DispatchResult {

			// ---- We check the transaction is signed by the caller
			// and retrieve the T::AccountId pubkey information.
			let caller = ensure_signed(origin)?;
			debug::info!("--- Called remove_stake with {:?}, hotkey {:?} and ammount {:?}", caller, hotkey, ammount_unstaked);

			// ---- We query the Neuron set for the NeuronMetadata stored under
			// the passed hotkey.
			ensure!(Neurons::<T>::contains_key(&hotkey), Error::<T>::NotActive);
			let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&hotkey);
			debug::info!("Got metadata for hotkey.");

			// ---- We check that the NeuronMetadata is linked to the calling
			// cold key, otherwise throw a NonAssociatedColdKey error.
			ensure!(neuron.coldkey == caller, Error::<T>::NonAssociatedColdKey);

			// ---- We check that the hotkey has enough stake to withdraw
			// and then withdraw from the account.
			let hotkey_stake: u32 = Stake::<T>::get(&hotkey);
			ensure!(hotkey_stake >= ammount_unstaked, Error::<T>::NotEnoughStaketoWithdraw);
			Stake::<T>::insert(&hotkey, hotkey_stake - ammount_unstaked);
			debug::info!("Withdraw: {:?} from hotkey staking account for new ammount {:?} staked", ammount_unstaked, hotkey_stake - ammount_unstaked);

			// --- We perform the withdrawl by converting the stake to a u32 balance
			// and deposit the balance into the coldkey account. If the coldkey account
			// does not exist it is created.
			// TODO(const): change to u32
			let _ = T::Currency::deposit_creating(&caller, Self::u32_to_balance(ammount_unstaked));
			debug::info!("Deposit {:?} into coldkey balance ", Self::u32_to_balance(ammount_unstaked));

			// --- We update the total staking pool with the removed funds.
			let total_stake: u32 = TotalStake::get();
			TotalStake::put(total_stake - ammount_unstaked);
			debug::info!("Remove {:?} from total stake, now {:?} ", ammount_unstaked, TotalStake::get());

			// ---- Emit the unstaking event.
			Self::deposit_event(RawEvent::StakeRemoved(hotkey, ammount_unstaked));
			debug::info!("--- Done remove_stake.");

			// --- Done and ok.
			Ok(())
		}

		/// ---- Subscribes the caller to the Neuron set with given metadata. If the caller
		/// already exists in the active set, the metadata is updated but the cold key remains unchanged.
		/// If the caller does not exist they make a link between this hotkey account
		/// and the passed coldkey account. Only the cold key has permission to make add_stake/remove_stake calls.
		/// Args:
		/// 	origin: (<T as frame_system::Trait>Origin):
		/// 		The caller, a hotkey associated with the subscribing neuron.
		/// 	ip (u128):
		/// 		The u32 encoded IP address of type 6 or 4.
		/// 	port (u16):
		/// 		The port number where this neuron receives RPC requests.
		/// 	ip_type (u8):
		/// 		The ip type one of (4,6).
		/// 	coldkey (T::AccountId):
		/// 		The associated coldkey to be attached to the account.
		/// Emits:
		/// 	NeuronAdded:
		/// 		On subscription of a new neuron to the active set.
		///
		/// 	NeuronUpdated:
		/// 		On subscription of new metadata attached to the calling hotkey.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn subscribe(origin, ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {

			// --- We check the callers (hotkey) signature.
			let caller = ensure_signed(origin)?;
			debug::info!("--- Called subscribe with caller {:?}", caller);

			// ---- We check to see if the Neuron already exists.
			// We do not allow peers to re-subscribe with the same key.
			ensure!( !Neurons::<T>::contains_key(&caller), Error::<T>::AlreadyActive );

			// ---- If the neuron is not-already subscribed, we create a
			// new entry in the table with the new metadata.
			debug::info!("Insert new metadata with ip: {:?}, port: {:?}, ip_type: {:?}, coldkey: {:?}", ip, port, ip_type, coldkey);
			Neurons::<T>::insert( &caller,
				NeuronMetadataOf::<T> {
					ip: ip,
					port: port,
					ip_type: ip_type,
					coldkey: coldkey,
				}
			);

			// ---- We provide the subscriber with and initial subscription gift.
			// NOTE: THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
			let subscription_gift: u32 = 1000;
			debug::info!("Adding subscription gift to the stake {:?} ", subscription_gift);

			// --- We update the total staking pool with the subscription.
			let total_stake: u32 = TotalStake::get();
			TotalStake::put(total_stake + subscription_gift);
			debug::info!("Adding amount: {:?} to total stake, now: {:?}", subscription_gift, TotalStake::get());

			// The last emit determines the last time this peer made an incentive
			// mechanism emit call. Since he is just subscribed with zero stake,
			// this moment is considered his first emit.
			let current_block: T::BlockNumber = system::Module::<T>::block_number();
			debug::info!("The new last emit for this caller is: {:?} ", current_block);

			// ---- We initilize the neuron maps with nill weights,
			// the subscription gift and the current block as last emit.
			Stake::<T>::insert(&caller, subscription_gift);
			LastEmit::<T>::insert(&caller, current_block);
			WeightVals::<T>::insert(&caller, &Vec::new());
			WeightKeys::<T>::insert(&caller, &Vec::new());

			// ---- We increment the neuron count for the additional member.
			let neuron_count = NeuronCount::get();
			NeuronCount::put(neuron_count + 1);
			debug::info!("Increment the neuron count to: {:?} ", NeuronCount::get());

			// --- We deposit the neuron added event.
			Self::deposit_event(RawEvent::NeuronAdded(caller));
			debug::info!("--- Done subscribe");

			Ok(())
		}

		/// ---- Unsubscribes the caller from the active Neuron. The call triggers
		/// an emit call before unstaking the current stake balance into the coldkey account.
		/// Args:
		/// 	origin: (<T as frame_system::Trait>Origin):
		/// 		The caller, a hotkey associated with the subscribing neuron.
		/// Emits:
		/// 	NeuronRemoved:
		/// 		On subscription of a new neuron to the active set.
		///
		/// 	NeuronUpdated:
		/// 		On subscription of new metadata attached to the calling hotkey.
		///
		/// Raises:
		/// 	NotActive:
		/// 		Raised if the unsubscriber does not exist.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn unsubscribe(origin) -> dispatch::DispatchResult {

			// --- We check the signature of the calling account.
			let caller = ensure_signed(origin)?;
			debug::info!("--- Called unsubscribe with caller: {:?}", caller);

			// --- We check that the Neuron already exists in the active set.
			ensure!(Neurons::<T>::contains_key(&caller), Error::<T>::NotActive);
			let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&caller);
			debug::info!("Metadata retrieved with coldkey: {:?}", neuron.coldkey);

			// TODO(const): call the emit function.
			// emit(caller)

			// --- If there are funds staked, we unstake them and add them to the coldkey.
			let ammount_unstaked: u32 = Stake::<T>::get( &caller );
			debug::info!("Ammount staked on this account is: {:?}", ammount_unstaked);

			if ammount_unstaked > 0 {
				// --- We perform the withdrawl by converting the stake to a u32 balance
				// and deposit the balance into the coldkey account. If the coldkey account
				// does not exist it is created.
				// TODO(const): change to u32
				T::Currency::deposit_creating( &neuron.coldkey, Self::u32_to_balance( ammount_unstaked ) );
				debug::info!("Depositing: {:?} into coldkey account: {:?}", ammount_unstaked, neuron.coldkey);


				// --- We update the total staking pool with the removed funds.
				let total_stake: u32 = TotalStake::get();
				TotalStake::put(total_stake - ammount_unstaked);
				debug::info!("Removing amount: {:?} from total stake, now: {:?}", ammount_unstaked, TotalStake::get());
			}

			// --- We remove the neuron info from the various maps.
			Stake::<T>::remove( &caller );
			Neurons::<T>::remove( &caller );
			LastEmit::<T>::remove( &caller );
			WeightVals::<T>::remove( &caller );
			WeightKeys::<T>::remove( &caller );
			debug::info!("Hotkey account removed: {:?}", caller);

			// --- We decrement the neuron counter.
			let neuron_count = NeuronCount::get();
			NeuronCount::put(neuron_count - 1);
			debug::info!("New neuron count: {:?}", NeuronCount::get());

			// --- We emit the neuron removed event and return ok.
			Self::deposit_event(RawEvent::NeuronRemoved(caller));
			debug::info!("--- Done unsubscribe.");

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	// Returns the bitcoin block reward from the block step.
	fn block_reward(now: &<T as system::Trait>::BlockNumber) -> U32F32 {
		// Convert block number into u32.
		let elapsed_blocks_u32 = TryInto::try_into(*now)
			.ok()
			.expect("blockchain will not exceed 2^32 blocks; QED.");

		// Convert block number into u32f32 float.
		let elapsed_blocks_u32_f32 = U32F32::from_num(elapsed_blocks_u32);

		// Bitcoin block halving rate was 210,000 blocks at block every 10 minutes.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (210,000 blocks) * (10 min * 60 sec / 6 sec) =  (210,000) * (100) = (21,000,000 blocks)
		let block_halving = U32F32::from_num(21000000);
		let fractional_halvings = elapsed_blocks_u32_f32 / block_halving;
		let floored_halvings = fractional_halvings.floor().to_num::<u32>();
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("floored_halvings: {:?}", floored_halvings);

		// Bitcoin block reward started at 50 tokens per block.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (50 blocks) / (10 min * 60 sec / 6 sec) =  (50) / (100) = (0.5 tokens per block)
		let block_reward = U32F32::from_num(0.5);

		// NOTE: Underflow occurs in 21,000,000 * (16 + 4) blocks essentially never.
		let block_reward_shift = block_reward.overflowing_shr(floored_halvings).0;
		block_reward_shift
	}

	pub fn u32_to_balance(input: u32) -> <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance
	{
		input.into()
	}
}

fn normalize(mut weights: Vec<u32>) -> Vec<u32> {
	let sum : u64  = weights.iter().map(|x| *x as u64).sum();

	if sum == 0 {
		return weights;
	}

	weights.iter_mut().for_each(|x| {
		*x = (*x as u64 * u32::max_value() as u64 / sum) as u32;
	});

	return weights;
}

#[cfg(test)]
mod tests {
	use crate::{normalize};

	#[test]
	fn normalize_sum_smaller_than_one() {
		let values : Vec<u32> = vec![u32::max_value() / 10, u32::max_value() / 10];
		assert_eq!(normalize(values), vec![u32::max_value() / 2, u32::max_value() / 2]);
	}

	#[test]
	fn normalize_sum_greater_than_one() {
		let values : Vec<u32> = vec![u32::max_value() / 7, u32::max_value() / 7];
		assert_eq!(normalize(values), vec![u32::max_value() / 2, u32::max_value() / 2]);
	}

	#[test]
	fn normalize_sum_zero() {
		let weights: Vec<u32> = vec![0,0];
		assert_eq!(normalize(weights), vec![0,0]);
	}

	#[test]
	fn normalize_values_maxed() {
		let weights: Vec<u32> = vec![u32::max_value(),u32::max_value()];
		assert_eq!(normalize(weights), vec![u32::max_value() / 2,u32::max_value() / 2]);
	}
}