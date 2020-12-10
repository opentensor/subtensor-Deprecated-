#![cfg_attr(not(feature = "std"), no_std)]

// Frame imports.
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, debug,
	traits::{Currency, WithdrawReasons, WithdrawReason, ExistenceRequirement},
};
use frame_support::weights::{DispatchClass, Pays};
use codec::{Decode, Encode};
use frame_system::{self as system, ensure_signed};
use substrate_fixed::types::U32F32;
use sp_std::convert::TryInto;
use sp_std::{
	prelude::*
};

use frame_support::debug::RuntimeLogger;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// Currency type that will be used to place deposits on neurons
	type Currency: Currency<Self::AccountId>;
}

#[derive(Encode, Decode, Default)]
pub struct NeuronMetadata {
	ip: u128,
	port: u16,
	ip_type: u8,
}

// The pallet's runtime storage items.
decl_storage! {
	trait Store for Module<T: Trait> as SubtensorModule {
	
		// Weight values: Map from account to vector of weights.
		pub WeightKeys: map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;
		pub WeightVals: map hasher(blake2_128_concat) T::AccountId => Vec<u32>;

		// Stake Values: Map from account to u32 stake ammount.
		pub Stake get(fn stake): map hasher(blake2_128_concat) T::AccountId => u32;

		// Last Emit Block: Last emission block.
		pub LastEmit get(fn block): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;
		
		// Active Neuron set: Active neurons in graph.
		pub Neurons get(fn neuron): map hasher(blake2_128_concat) T::AccountId => NeuronMetadata;

		// Active Neuron count.
		NeuronCount: u32;
		
		// Total amount staked.
        TotalStake: u32;
	}
}

// Subtensor events.
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		// Sent when a Neuron updates their weights on chain.
		WeightsSet(AccountId),

		// Sent when a Neuron is added to the active set.
		NeuronAdded(AccountId),

		// Sent when a Neuron is removed from the active set.
		NeuronRemoved(AccountId),

		// Sent when a Neuron updates their stake.
		StakeAdded(AccountId, u32),

		// Sent when there is emission from a Neuron.
		Emission(AccountId, u32),
		
	}
);

// Subtensor Errors.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Cannot join as a member because you are already a member
		AlreadyActive,
		/// Cannot perform staking or emission unless the Neuron is already subscribed.
		NotActive,
		// Neuron calling emit has no emission.
		NothingToEmit,
		// Neuron updating weights caused overflow.
		WeightVecNotEqualSize,
		// Neuron setting weights are too large. Cause u32 overlfow.
		WeightSumToLarge,
	}
}

// Subtensor Dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;
		
		// Reservation Fee
		//CurrentBalance: BalanceOf<T> = T::CurrentBalance::get();

		/// Emission. Called by an active Neuron with stake in-order to distribute 
		/// tokens to weighted neurons and to himself. The amount emitted is dependent on
		/// the ammount of stake held at this Neuron and the time since last emission.
		/// neurons are encouraged to calle this function often as to maximize
		/// their inflation in the graph.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn emit(origin) -> dispatch::DispatchResult {
			RuntimeLogger::init();
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let calling_neuron = ensure_signed(origin)?;
			debug::info!("Emit sent by: {:?}", calling_neuron);


			// Return.
			Ok(())
		}

		// Staking: Adds stake to the stake account for calling Neuron.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn add_stake(origin, stake_amount: u32) -> dispatch::DispatchResult {
			
			// Check sig.
			let neuron = ensure_signed(origin)?;

			debug::info!("add_stake sent by: {:?}", neuron);
			debug::info!("stake_amount {:?}", stake_amount);

			// Check subscribed.
			ensure!(Neurons::<T>::contains_key(&neuron), Error::<T>::NotActive);

			// Update stake at Neuron.
			// TODO (const): transfer from balance pallet.
			Stake::<T>::insert(&neuron, stake_amount);

			// Remove stake_amount from neuron's balance to put it in the "stake"
			let stake_currency = Self::u32_to_balance(stake_amount);

			// Adding to stake amount, remove from currency account.
			let _ = T::Currency::withdraw(&neuron, stake_currency, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);

			debug::info!("New neuron balance is {:?}", T::Currency::total_balance(&neuron));

			// Update total staked storage iterm.
			let total_stake: u32  = TotalStake::get();
			TotalStake::put(total_stake + stake_amount); // TODO (const): check overflow.
			debug::info!("sink new stake.");


			// Remove stake amount from balance

			// Emit event and finish.
			Self::deposit_event(RawEvent::StakeAdded(neuron, stake_amount));
			Ok(())
		}

		// Subscribes the calling Neuron to the active set.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn subscribe(origin, ip: u128, port: u16, ip_type: u8) -> dispatch::DispatchResult {
			
			// Check sig.
			let new_neuron = ensure_signed(origin)?;
			debug::info!("new_neuron sent by: {:?}", new_neuron);

			// Check Neuron does not already exist.
			ensure!(!Neurons::<T>::contains_key(&new_neuron), Error::<T>::AlreadyActive);
	
			// Insert the new Neuron into the active set.
			Neurons::<T>::insert(&new_neuron, 
				NeuronMetadata {
					ip: ip,
					port: port,
					ip_type: ip_type,
				}
			);

			// Update Neuron count.
			let neuron_count = NeuronCount::get();
			NeuronCount::put(neuron_count + 1); // overflow check not necessary because of maximum
			debug::info!("neuron_count: {:?}", neuron_count + 1);

			// Add current block to last emit under Neuron account.
			let current_block: T::BlockNumber = system::Module::<T>::block_number();
			LastEmit::<T>::insert(&new_neuron, current_block);
			debug::info!("add last emit.");

			// Initizialize stake to some arbitrary number for testing purposes.
			// TODO (shibshib): Fix this so we're not just arbitrarily setting numbers.
			debug::info!("Minimum balance for an account is {:?}", T::Currency::minimum_balance());
			
			// Should withdraw from this account to stake the amount. 

			// Let's add the minimum balance required to test this out
			// THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
			// Provide each new subscriber with 1000 tokens
			let subscription_gift = 1000;
			Stake::<T>::insert(&new_neuron, 100);
			let new_stake = Self::u32_to_balance(Stake::<T>::get(&new_neuron));
			TotalStake::put(Stake::<T>::get(&new_neuron));
			debug::info!("set stake to {:?}.", new_stake);
			
			// Gift the new neuron the 1000 tokens, just as starting point. 
			let _ = T::Currency::deposit_into_existing(&new_neuron, Self::u32_to_balance(subscription_gift));
			debug::info!("Balance is now {:?}, withdrawing {:?} to stake.", T::Currency::free_balance(&new_neuron), new_stake);

			// Stake 100 of the total tokens it has
			let _ = T::Currency::withdraw(&new_neuron, new_stake, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
			debug::info!("Balance left: {:?}", T::Currency::free_balance(&new_neuron));
			// Init empty weights.
			WeightVals::<T>::insert(&new_neuron, &Vec::new());
			WeightKeys::<T>::insert(&new_neuron, &Vec::new());

			// Emit event.
			Self::deposit_event(RawEvent::NeuronAdded(new_neuron));
			Ok(())
		}
		

		// Removes Neuron from active set. 
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		fn unsubscribe(origin) -> dispatch::DispatchResult {

			// Check sig.
			let old_neuron = ensure_signed(origin)?;
			debug::info!("unsubscribe sent by: {:?}", old_neuron);

			// Check that the Neuron already exists.
			ensure!(Neurons::<T>::contains_key(&old_neuron), Error::<T>::NotActive);
		
			// Remove Neuron.
			Neurons::<T>::remove(&old_neuron);
			NeuronCount::mutate(|v| *v -= 1);
			debug::info!("remove from Neuron set and decrement count.");

			// Remove Last Emit.
			LastEmit::<T>::remove(&old_neuron);
			debug::info!("remove from last emit set.");

			// Remove Stake.
			Stake::<T>::remove(&old_neuron);
			debug::info!("remove stake");

			// Add back to balance
			let deposit_back = Self::u32_to_balance(TotalStake::get());
			T::Currency::deposit_into_existing(&old_neuron, deposit_back).ok();
			debug::info!("Balance is {:?}", T::Currency::total_balance(&old_neuron));

			// Remove Weights.
			WeightVals::<T>::remove(&old_neuron);
			WeightKeys::<T>::remove(&old_neuron);
			debug::info!("remove weights.");

			// Emit event.
			Self::deposit_event(RawEvent::NeuronRemoved(old_neuron));

			Ok(())
		}

		/// Set Weights: Sets weight vec for Neuron on chain.
		#[weight = (0, DispatchClass::Operational, Pays::No)]
		pub fn set_weights(origin, 
				dests: Vec<T::AccountId>, 
				values: Vec<u32>) -> dispatch::DispatchResult {
			
			// Check sig.
			let calling_neuron = ensure_signed(origin)?;
			debug::info!("set_weights sent by: {:?}", calling_neuron);
			debug::info!("dests: {:?}", dests);
			debug::info!("values: {:?}", values);

			// Ensure weights and vals have equal size.
			debug::info!("values.len= {:?}, dests.len= {:?}", values.len(), dests.len());
			ensure!(values.len() == dests.len(), Error::<T>::WeightVecNotEqualSize);

			// Check weights do not cause overflow.
			let mut weights_sum: u64 = 0;
			for wij in values.iter() {
				let wij_u64 = *wij as u64;
				weights_sum = weights_sum + wij_u64;
			}
			let u32_max = u32::MAX;
			let u32_max_u64 = u32_max as u64;
			ensure!(weights_sum <= u32_max_u64, Error::<T>::WeightSumToLarge);

			// Update weights.
			WeightVals::<T>::insert(&calling_neuron, &values);
			WeightKeys::<T>::insert(&calling_neuron, &dests);


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

			// POSSIBLE DIVISION BY ZERO HERE.
			// User may not have stake yet. So we accidentally divide by 0 and break.
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
			for x in w_vals.iter() {
				// Overflow no possible since weight sum has been previously checked.
				let x_u32_f32 = U32F32::from_num(*x);
				w_sum = w_sum + x_u32_f32;
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

			Self::deposit_event(RawEvent::Emission(calling_neuron, total_emission_u32));


			// Emit and return

			//Self::deposit_event(RawEvent::WeightsSet(neuron));

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
		// The equivalent halving would be 10 min * 60 sec / 6 sec =  100 * 210,000.
		// So our halving is every 21,000,000 blocks.
		let block_halving = U32F32::from_num(21000000);
		let fractional_halvings = elapsed_blocks_u32_f32 / block_halving;
		let floored_halvings = fractional_halvings.floor().to_num::<u32>();
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("floored_halvings: {:?}", floored_halvings);

		// Return the bitcoin block reward.
		let block_reward = U32F32::from_num(50);
		// NOTE: Underflow occurs in 21,000,000 * (16 + 4) blocks essentially never.
		let block_reward_shift = block_reward.overflowing_shr(floored_halvings).0;
		block_reward_shift
	}

	pub fn u32_to_balance(input: u32) -> <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance
	{
		input.into()
	}
	
}