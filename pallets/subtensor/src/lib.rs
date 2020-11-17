#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, debug};
use frame_support::weights::{DispatchClass, Pays};
use codec::{Decode, Encode};
use frame_system::{self as system, ensure_signed};
use substrate_fixed::types::U32F32;
use sp_arithmetic::Permill;
use sp_std::convert::TryInto;

use sp_std::{
	prelude::*
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

#[derive(Encode, Decode, Default)]
pub struct PeerMetadata {
	ip: u128,
	port: u16,
	ip_type: u8,
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as SubtensorModule {
	
		// Weight values.
		pub Weight_keys: map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;
		pub Weight_vals: map hasher(blake2_128_concat) T::AccountId => Vec<u32>;

		// Stake Values
		pub Stake get(fn stake): map hasher(blake2_128_concat) T::AccountId => u32;

		// Last Emit Block
		pub LastEmit get(fn block): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;
		
		// Active Peer set.
		pub Peers get(fn peer): map hasher(blake2_128_concat) T::AccountId => PeerMetadata;

		// Active peer count.
		PeerCount: u32;
		
		// Total ammount staked.
        TotalStake: u32;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		// Set Weight: [who A]
		WeightsSet(AccountId),

		PeerAdded(AccountId),

		PeerRemoved(AccountId),

		StakeAdded(AccountId, u32),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Cannot join as a member because you are already a member
		AlreadyPeer,
		/// Cannot give up membership because you are not currently a member
		NotPeer,
		// Peer calling emit has no emission.
		NothingToEmit,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn emit(origin) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let calling_peer = ensure_signed(origin)?;
			debug::info!("Emit sent by: {:?}", calling_peer);

			// Check that the peer exists in the peer set
			// Check that the peer has stake to emit.
			// Check that the peer has weights set.
			ensure!(Peers::<T>::contains_key(&calling_peer), Error::<T>::NotPeer);
			ensure!(Stake::<T>::contains_key(&calling_peer), Error::<T>::NotPeer);
			ensure!(LastEmit::<T>::contains_key(&calling_peer), Error::<T>::NotPeer);
			ensure!(Weight_keys::<T>::contains_key(&calling_peer), Error::<T>::NotPeer);
			ensure!(Weight_vals::<T>::contains_key(&calling_peer), Error::<T>::NotPeer);

			// Get the last emission block.
			// Get the current block.
			// Get the block reward from current block.
			// Set the current block as last emit.
			let last_block: T::BlockNumber = LastEmit::<T>::get(&calling_peer);
			let current_block = system::Module::<T>::block_number();
			let block_reward = Self::block_reward(&current_block);
			LastEmit::<T>::insert(&calling_peer, current_block);
			debug::info!("Last emit block: {:?}", last_block);
			debug::info!("Current block: {:?}", current_block);
			debug::info!("Block reward: {:?}", block_reward);

			// Get number of elapsed blocks since last emit.
			// Convert to u32f32.
			let elapsed_blocks = current_block - last_block;
			let elapsed_blocks_u32: usize = TryInto::try_into(elapsed_blocks)
			.ok()
			.expect("blockchain will not exceed 2^32 blocks; qed");
			let elapsed_blocks_U32F32 = U32F32::from_num(elapsed_blocks_u32);
			debug::info!("elapsed_blocks_u32: {:?}", elapsed_blocks_u32);
			debug::info!("elapsed_blocks_u32f32: {:?}", elapsed_blocks_U32F32);

			// Get local and total stake.
			// Convert to u32f32.
			// Calculate stake fraction.
			let total_stake: u32  = TotalStake::get();
			let total_stake_U32F32 = U32F32::from_num(total_stake);
			let local_stake: u32 = Stake::<T>::get(&calling_peer);
			let local_stake_U32F32 = U32F32::from_num(local_stake);
			let stake_fraction_u32f32 = local_stake_U32F32 / total_stake_U32F32;
			debug::info!("total_stake_U32F32 {:?}", total_stake_U32F32);
			debug::info!("local_stake_U32F32 {:?}", local_stake_U32F32);
			debug::info!("stake_fraction_u32f32 {:?}", stake_fraction_u32f32);

			// Calculate total emission at this peer.
			let total_emission_u32f32 = stake_fraction_u32f32 * block_reward * elapsed_blocks_U32F32;
			//let total_emission_u32f32 = total_emission_u32f32.floor();
			debug::info!("total_emission_u32f32 {:?} = {:?}*{:?}*{:?}", total_emission_u32f32, stake_fraction_u32f32, block_reward, elapsed_blocks_U32F32);
			ensure!(total_emission_u32f32 > U32F32::from_num(0), Error::<T>::NothingToEmit);

			// Pull weights and vals from weight set.
			let w_keys: Vec<T::AccountId> = Weight_keys::<T>::get(&calling_peer);
			let w_vals: Vec<u32> = Weight_vals::<T>::get(&calling_peer);

			// Get the weight sum for normalization.
			let mut w_sum = U32F32::from_num(0);
			for x in w_vals.iter() {
				// TODO(const) Could cause overflow.
				let x_U32F32 = U32F32::from_num(*x);
				w_sum = w_sum + x_U32F32;
			}
			for (i, dest_key) in w_keys.iter().enumerate() {

				// Get emission to peer i.
				let wij_U32F32 = U32F32::from_num(w_vals[i]);
				let wij_norm_U32F32 = wij_U32F32 / w_sum;
				let emission_U32F32 = total_emission_u32f32 * wij_norm_U32F32;

				debug::info!("emit to {:?}", dest_key);
				debug::info!("wij {:?}", wij_norm_U32F32);
				debug::info!("emission_U32F32 {:?}", emission_U32F32);

				// Get stake as a U32F32.
				let prev_stake: u32 = Stake::<T>::get(&dest_key);
				let prev_stake_U32F32 = U32F32::from_num(prev_stake);
				let new_stake_U32F32 = prev_stake_U32F32 + emission_U32F32;

				debug::info!("prev_stake_U32F32 {:?}", prev_stake_U32F32);
				debug::info!("new_stake_U32F32 {:?} = {:?} + {:?}", new_stake_U32F32, prev_stake_U32F32, emission_U32F32);

				// Convert to u32.
				let new_stake_u32: u32 = new_stake_U32F32.to_num::<u32>();
				debug::info!("new_stake_u32 {:?}", new_stake_u32);

				Stake::<T>::insert(&dest_key, new_stake_u32);
				let total_stake: u32  = TotalStake::get();
				TotalStake::put(total_stake + new_stake_u32); // TODO (const): check overflow.
				debug::info!("sink new stake.");
			}

			// Return.
			Ok(())
		}

		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn add_stake(origin, stake_amount: u32) -> dispatch::DispatchResult {
			let peer = ensure_signed(origin)?;
			debug::info!("add_stake sent by: {:?}", peer);
			debug::info!("stake_amount {:?}", stake_amount);

			let total_stake = TotalStake::get();
			debug::info!("total_stake: {:?}", total_stake + stake_amount);

			ensure!(Peers::<T>::contains_key(&peer), Error::<T>::NotPeer);

			Stake::<T>::insert(&peer, stake_amount);
			TotalStake::put(total_stake + stake_amount); // TODO (const): check overflow.

			Self::deposit_event(RawEvent::StakeAdded(peer, stake_amount));
			Ok(())
		}

		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn subscribe(origin, ip: u128, port: u16, ip_type: u8) -> dispatch::DispatchResult {
			let new_peer = ensure_signed(origin)?;
			debug::info!("new_peer sent by: {:?}", new_peer);

			let peer_count = PeerCount::get();
			debug::info!("peer_count: {:?}", peer_count);

			// We don't want to add duplicate peer, so we check whether the potential new
			// member is already present in the list. Because the peer is stored as a hash
			// map this check is constant time O(1)
			ensure!(!Peers::<T>::contains_key(&new_peer), Error::<T>::AlreadyPeer);
	
			// Insert the new member and emit the event
			Peers::<T>::insert(&new_peer, 
				PeerMetadata {
					ip: ip,
					port: port,
					ip_type: ip_type,
				}
			);
			PeerCount::put(peer_count + 1); // overflow check not necessary because of maximum
			debug::info!("add to peer set");

			// Add current block to last emit.
			let current_block: T::BlockNumber = system::Module::<T>::block_number();
			LastEmit::<T>::insert(&new_peer, current_block);
			debug::info!("add last emit.");

			// Init stake.
			Stake::<T>::insert(&new_peer, 0);
			debug::info!("set stake to zero.");

			Self::deposit_event(RawEvent::PeerAdded(new_peer));
			Ok(())
		}
		

		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn unsubscribe(origin) -> dispatch::DispatchResult {
			let old_peer = ensure_signed(origin)?;
			debug::info!("unsubscribe sent by: {:?}", old_peer);

			// Check that the peer already exists.
			ensure!(Peers::<T>::contains_key(&old_peer), Error::<T>::NotPeer);
		
			// Remove peer.
			Peers::<T>::remove(&old_peer);
			PeerCount::mutate(|v| *v -= 1);
			debug::info!("remove from peer set and decrement count.");

			// Remove Last Emit.
			LastEmit::<T>::remove(&old_peer);
			debug::info!("remove from last emit set.");

			// Remove Stake.
			Stake::<T>::remove(&old_peer);
			debug::info!("remove stake");

			// Remove Weights.
			Weight_vals::<T>::remove(&old_peer);
			Weight_keys::<T>::remove(&old_peer);
			debug::info!("remove weights.");

			Self::deposit_event(RawEvent::PeerRemoved(old_peer));
			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn set_weights(origin, 
				dests: Vec<T::AccountId>, 
				values: Vec<u32>) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let peer = ensure_signed(origin)?;
			debug::info!("set_weights sent by: {:?}", peer);
			debug::info!("dests: {:?}", dests);
			debug::info!("values: {:?}", values);

			// TODO check the length of these two arrays and ensure they are the same size.
			// The separation allows us to update the vals without updating the keys.
			Weight_vals::<T>::insert(&peer, &values);
			Weight_keys::<T>::insert(&peer, &dests);

			Self::deposit_event(RawEvent::WeightsSet(peer));

			// Return a successful DispatchResult
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
			.expect("blockchain will not exceed 2^32 blocks; qed");

		// Convert block number into u32f32 float.
		let elapsed_blocks_u32f32 = U32F32::from_num(elapsed_blocks_u32);

		// Bitcoin block halving rate was 210,000 blocks at block every 10 minutes.
		// The average substrate block time is 6 seconds.
		// The equivalent halving would be 10 min * 60 sec / 6 sec =  100 * 210,000.
		// So our halving is every 21,000,000 blocks.
		let block_halving = U32F32::from_num(21000000);
		let fractional_halvings = elapsed_blocks_u32f32 / block_halving;
		let floored_halvings = fractional_halvings.floor().to_num::<u32>();
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("floored_halvings: {:?}", floored_halvings);

		// Return the bitcoin block reward.
		let block_reward = U32F32::from_num(50);
		// TODO(const): catch underflow.
		let block_reward_shift = block_reward.overflowing_shr(floored_halvings).0;
		block_reward_shift
	}
}
