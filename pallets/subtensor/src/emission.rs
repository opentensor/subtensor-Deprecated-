use super::*;

impl<T: Trait> Module<T> {
/// TODO(const & paralax): The self emission can be used to fund the transaction, this allows us to remove the need
	/// for transactions costs.
	/// Emits inflation from the calling neuron to neighbors and themselves. Returns the total amount of emitted stake.
	/// The inflation available to this caller is given by (blocks_since_last_emit) * (inflation_per_block) * (this_neurons_stake) / (total_stake).
	/// Neurons are incentivized to make calls to this function often as-to maximize inflation in the graph.
	///
	/// # Args:
	///  	* `caller` (&T::AccountId):
	/// 		- The associated calling neuron key. Not a signature
	/// 		just the associated public key. Checking the permissions is left to
	/// 		the calling function.
	///
	/// # Returns
	/// 	* emission (u32):
	/// 		- The total amount emitted to the caller.
	///
	pub fn emit( caller: &T::AccountId ) -> u32 {
		// --- We init the Runtimelogger for WASM debugging
		RuntimeLogger::init();
		debug::info!("--- Calling emit, caller: {:?}", caller);

		// --- We dont check that the caller exists in the Neuron set with corresponding
		// mapped values. These are initialized on subscription. This should never
		// occur unless the calling function does not check the callers subscription first.
		// ensure!(Neurons::<T>::contains_key(&caller), Error::<T>::NotActive);
		// ensure!(Stake::<T>::contains_key(&caller), Error::<T>::NotActive);
		// ensure!(LastEmit::<T>::contains_key(&caller), Error::<T>::NotActive);
		// ensure!(WeightKeys::<T>::contains_key(&caller), Error::<T>::NotActive);
		// ensure!(WeightVals::<T>::contains_key(&caller), Error::<T>::NotActive);

		// --- We get the current block reward and the last time the user emitted.
		// This is needed to determine the proportion of inflation allocated to
		// the caller. Note also, that the block reward is a decreasing function
		// callers want to call emit before the block inflation decreases.
		let last_emit: T::BlockNumber = LastEmit::<T>::get(&caller);
		let current_block = system::Module::<T>::block_number();
		let block_reward = Self::block_reward(&current_block);
		debug::info!("Last emit block: {:?}", last_emit);
		debug::info!("Current block: {:?}", current_block);
		debug::info!("Block reward: {:?}", block_reward);

		// --- We get the number of blocks since the last emit and
		// convert types into U32F32. The floating precision enables
		// the following calculations.
		let elapsed_blocks = current_block - last_emit;
		let elapsed_blocks_u32: usize = TryInto::try_into(elapsed_blocks).ok().expect("blockchain will not exceed 2^32 blocks; qed");
		let elapsed_blocks_u32_f32 = U32F32::from_num(elapsed_blocks_u32);
		debug::info!("elapsed_blocks_u32: {:?}", elapsed_blocks_u32);
		debug::info!("elapsed_blocks_u32_f32: {:?}", elapsed_blocks_u32_f32);
		if elapsed_blocks_u32_f32 == U32F32::from_num(0) {
			// No blocks have passed, nothing to emit. Return without error.
			return 0;
		}

		// --- We get the callers stake and the total stake ammounts
		// converting them to U32F32 for the following calculations.
		let total_stake: u32  = TotalStake::get();
		let total_stake_u32_f32 = U32F32::from_num(total_stake);
		let caller_stake: u32 = Stake::<T>::get(&caller);
		let caller_stake_u32_f32 = U32F32::from_num(caller_stake);
		debug::info!("total_stake_u32_f32 {:?}", total_stake_u32_f32);
		debug::info!("caller_stake_u32_f32 {:?}", caller_stake_u32_f32);
		if total_stake_u32_f32 == U32F32::from_num(0) {
			// total stake is zero, nothing to emit. Return without error.
			return 0;
		}

		// --- We get the fraction of total stake held by the caller.
		// This should only be zero if the caller has zero stake. Otherwise
		// it returns a floating point (a.k.a, bits in the F32 part.)
		let stake_fraction_u32_f32 = caller_stake_u32_f32 / total_stake_u32_f32;
		debug::info!("stake_fraction_u32_f32 {:?}", stake_fraction_u32_f32);
		if stake_fraction_u32_f32 == U32F32::from_num(0) {
			// stake fraction is zero, nothing to emit. Return without error.
			return 0;
		}

		// --- We calculate the total emission available to the caller.
		// the block reward is positive and non-zero, so is the stake_fraction and elapsed blocks.
		// this ensures the total_emission is positive non-zero. To begin the block reward is (0.5 * 10^12).
		let callers_emission_u32_f32 = stake_fraction_u32_f32 * block_reward * elapsed_blocks_u32_f32;
		debug::info!("callers_emission_u32_f32: {:?} = {:?} * {:?} * {:?}", callers_emission_u32_f32, stake_fraction_u32_f32, block_reward, elapsed_blocks_u32_f32);
		if callers_emission_u32_f32 == U32F32::from_num(0) {
			// callers emission is zero, nothing to emit. Return without error.
			return 0;
		}

		// --- We get the callers weights. The total emission will be distributed
		// according to these weights. Previous checks in fn set_weights ensure
		// that the weight_vals sum to u32::max / are nomalized to 1.
		let weight_vals: Vec<u32> = WeightVals::<T>::get( &caller );
		let weight_keys: Vec<T::AccountId> = WeightKeys::<T>::get( &caller );
		if weight_keys.is_empty() || weight_vals.is_empty() {
			// callers has no weights, nothing to emit. Return without error.
			return 0;
		}

		// --- We iterate through the weights and distribute the caller's emission to
		// neurons on a weighted basis. The emission is added as new stake to their
		// staking account and the total emission is increased.
		let mut total_new_stake_u32: u32 = 0; // Total stake added across all emissions.
		for (i, dest_key) in weight_keys.iter().enumerate() {

			// --- We get the weight from neuron i to neuron j.
			// The weights are normalized and sum to u32::max.
			// This weight value as floating point value in the
			// range [0, 1] is thus given by w_ij_u32 / u32::max
			let wij_u32_f32 = U32F32::from_num( weight_vals[i] );
			let wij_norm_u32_f32 = wij_u32_f32 / U32F32::from_num( u32::MAX );
			debug::info!("Emitting to {:?}", dest_key);
			debug::info!("wij {:?}", wij_norm_u32_f32);

			// --- We get the emission from neuron i to neuron j.
			// The multiplication of the weight \in [0, 1]
			// by the total_emission gives us the emission proportion.
			let emission_u32_f32 = callers_emission_u32_f32 * wij_norm_u32_f32;
			debug::info!("emission_u32_f32 {:?}", emission_u32_f32);

			// --- We increase the staking account by this new emission
			// value by first converting both to u32f32 floats. The floating
			// point emission is dropped in the conversion back to u32.
			let prev_stake: u32 = Stake::<T>::get(&dest_key);
			let prev_stake_u32_f32 = U32F32::from_num(prev_stake);
			let new_stake_u32_f32 = prev_stake_u32_f32 + emission_u32_f32;
			let new_stake_u32: u32 = new_stake_u32_f32.to_num::<u32>();
			Stake::<T>::insert(&dest_key, new_stake_u32);
			debug::info!("prev_stake_u32_f32 {:?}", prev_stake_u32_f32);
			debug::info!("new_stake_u32_f32 {:?} = {:?} + {:?}", new_stake_u32_f32, prev_stake_u32_f32, emission_u32_f32);
			debug::info!("new_stake_u32 {:?}", new_stake_u32);

			// --- We increase the total stake emitted. For later addition to
			// the total staking pool.
			total_new_stake_u32 = total_new_stake_u32 + new_stake_u32
		}

		// --- We add the total amount of stake emitted to the staking pool.
		// Note: This value may not perfectly match total_emission_u32_f32 after rounding.
		let total_stake: u32  = TotalStake::get();
		TotalStake::put(total_stake + total_new_stake_u32);
		debug::info!("Adding new total stake {:?}, now {:?}", total_stake, TotalStake::get());

		// --- Finally, we update the last emission by the caller.
		LastEmit::<T>::insert(&caller, current_block);
		debug::info!("The old last emit: {:?} the new last emit: {:?}", last_emit, current_block);

		// --- Return ok.
		debug::info!("--- Done emit");
		return total_new_stake_u32;
	}

}