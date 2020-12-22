use super::*;
use frame_support::debug::RuntimeLogger;

impl<T: Trait> Module<T> {


	/// Emits inflation from the calling neuron to neighbors and themselves. Returns the total amount of emitted stake.
	/// The inflation available to this caller is given by (blocks_since_last_emit) * (inflation_per_block) * (this_neurons_stake) / (total_stake).
	/// Neurons are incentivized to make calls to this function often as-to maximize inflation in the graph.
	/// 
	/// # Args:
	///  	* `origin` (T::Origin):
	/// 		- The transaction caller.
	/// 
	/// # Returns
	/// 	* emission (u64):
	/// 		- The total amount emitted to the caller.
	/// 	
	pub fn do_emit( origin: T::Origin ) -> dispatch::DispatchResult {
		// ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called emit with caller {:?}", caller);

        // ---- We query the Neuron set for the neuron data stored under
        // the passed hotkey and retrieve it as a NeuronMetadata struct.
        ensure!(Neurons::<T>::contains_key(&caller), Error::<T>::NotActive);
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&caller);
		debug::info!("Got metadata for hotkey {:?}", caller);

		// ---- We call emit for this neuron.
		Self::emit_from_uid( neuron.uid );

		// ---- Done.
		Ok(())		
	}

	/// Emits inflation from the neuron uid to neighbors and themselves. Returns the total amount of emitted stake.
	/// The inflation available to this caller is given by (blocks_since_last_emit) * (inflation_per_block) * (this_neurons_stake) / (total_stake).
	/// Neurons are incentivized to make calls to this function often as-to maximize inflation in the graph.
	/// 
	/// # Args:
	///  	* `neuron_uid` (u64):
	/// 		- The uid for the neuron we are emitting from. 
	/// 
	/// # Returns
	/// 	* emission (u64):
	/// 		- The total amount emitted to the caller.
	/// 	
	pub fn emit_from_uid( neuron_uid: u64 ) -> u64 {
		// --- We init the Runtimelogger for WASM debugging
		RuntimeLogger::init();
		debug::info!("--- Calling emit, neuron_uid: {:?}", neuron_uid);

		// --- We get the current block reward and the last time the user emitted.
		// This is needed to determine the proportion of inflation allocated to 
		// the caller. Note also, that the block reward is a decreasing function
		// callers want to call emit before the block inflation decreases.
		let last_emit: T::BlockNumber = LastEmit::<T>::get( neuron_uid );
		let current_block = system::Module::<T>::block_number();
		let block_reward = Self::block_reward(&current_block);
		debug::info!("Last emit block: {:?}", last_emit);
		debug::info!("Current block: {:?}", current_block);
		debug::info!("Block reward: {:?}", block_reward);

		// --- We get the number of blocks since the last emit and 
		// convert types into U64F64. The floating precision enables 
		// the following calculations. 
		let elapsed_blocks = current_block - last_emit;
		let elapsed_blocks_u64: usize = TryInto::try_into(elapsed_blocks).ok().expect("blockchain will not exceed 2^64 blocks; qed");
		let elapsed_blocks_u64_f64 = U64F64::from_num(elapsed_blocks_u64);
		debug::info!("elapsed_blocks_u64: {:?}", elapsed_blocks_u64);
		debug::info!("elapsed_blocks_u64_f64: {:?}", elapsed_blocks_u64_f64);
		if elapsed_blocks_u64_f64 == U64F64::from_num(0) {
			// No blocks have passed, nothing to emit. Return without error.
			return 0;
		}

		// --- We get the callers stake and the total stake ammounts
		// converting them to U64F64 for the following calculations.
		let total_stake: u64  = TotalStake::get();
		let total_stake_u64_f64 = U64F64::from_num(total_stake);
		let caller_stake: u64 = Stake::get( neuron_uid );
		let caller_stake_u64_f64 = U64F64::from_num(caller_stake);
		debug::info!("total_stake_u64_f64 {:?}", total_stake_u64_f64);
		debug::info!("caller_stake_u64_f64 {:?}", caller_stake_u64_f64);
		if total_stake_u64_f64 == U64F64::from_num(0) {
			// total stake is zero, nothing to emit. Return without error.
			return 0;	
		}

		// --- We get the fraction of total stake held by the caller.
		// This should only be zero if the caller has zero stake. Otherwise 
		// it returns a floating point (a.k.a, bits in the F64 part.)
		let stake_fraction_u64_f64 = caller_stake_u64_f64 / total_stake_u64_f64;
		debug::info!("stake_fraction_u64_f64 {:?}", stake_fraction_u64_f64);
		if stake_fraction_u64_f64 == U64F64::from_num(0) {
			// stake fraction is zero, nothing to emit. Return without error.
			return 0;
		}

		// --- We calculate the total emission available to the caller.
		// the block reward is positive and non-zero, so are the stake_fraction and elapsed blocks.
		// this ensures the total_emission is positive non-zero. To begin the block reward is (0.5 * 10^12).
		let callers_emission_u64_f64 = stake_fraction_u64_f64 * block_reward * elapsed_blocks_u64_f64;
		debug::info!("callers_emission_u64_f64: {:?} = {:?} * {:?} * {:?}", callers_emission_u64_f64, stake_fraction_u64_f64, block_reward, elapsed_blocks_u64_f64);
		if callers_emission_u64_f64 == U64F64::from_num(0) {
			// callers emission is zero, nothing to emit. Return without error.
			return 0;
		}

		// --- We get the callers weights. The total emission will be distributed
		// according to these weights. The weight_vals sum to u32::max. 
		let weight_vals: Vec<u32> = WeightVals::get( neuron_uid );
		let weight_uids: Vec<u64> = WeightUids::get( neuron_uid );
		if weight_uids.is_empty() || weight_vals.is_empty() {
			// callers has no weights, nothing to emit. Return without error.
			return 0;
		}

		// --- We iterate through the weights and distribute the caller's emission to 
		// neurons on a weighted basis. The emission becomes new stake in their 
		// staking account. 
		let mut total_new_stake_u64: u64 = 0; // Total stake added across all emissions.
		for (i, dest_uid) in weight_uids.iter().enumerate() {

			// --- We get the weight from neuron i to neuron j.
			// The weights are normalized and sum to u32::max. 
			let wij_u64_f64 = U64F64::from_num( weight_vals[i] );
			let wij_norm_u64_f64 = wij_u64_f64 / U64F64::from_num( u32::MAX );
			debug::info!("Emitting to {:?}", dest_uid);
			debug::info!("wij {:?}", wij_norm_u64_f64);

			// --- We get the emission from neuron i to neuron j.
			// The multiplication of the weight \in [0, 1]
			// by the total_emission gives us the emission proportion.
			let emission_u64_f64 = callers_emission_u64_f64 * wij_norm_u64_f64;
			debug::info!("emission_u64_f64 {:?}", emission_u64_f64);

			// --- We increase the staking account. The floating 
			// point emission is dropped in the conversion back to u64.
			let prev_stake: u64 = Stake::get( dest_uid );
			let prev_stake_u64_f64 = U64F64::from_num( prev_stake );
			let new_stake_u64_f64 = prev_stake_u64_f64 + emission_u64_f64;
			let new_stake_u64: u64 = new_stake_u64_f64.to_num::<u64>();
			Stake::insert(dest_uid, new_stake_u64);
			debug::info!("prev_stake_u64_f64 {:?}", prev_stake_u64_f64);
			debug::info!("new_stake_u64_f64 {:?} = {:?} + {:?}", new_stake_u64_f64, prev_stake_u64_f64, emission_u64_f64);
			debug::info!("new_stake_u64 {:?}", new_stake_u64);

			// --- We increase the total stake emitted. 
			total_new_stake_u64 = total_new_stake_u64 + emission_u64_f64.to_num::<u64>();
		}
		
		// --- We add the total amount of stake emitted to the staking pool.
		// Note: This value may not perfectly match total_emission_u64_f64 after rounding.
		let total_stake: u64  = TotalStake::get();
		TotalStake::put(total_stake + total_new_stake_u64); 
		debug::info!("Adding new total stake {:?} to old total {:?} now {:?}", total_new_stake_u64, total_stake, TotalStake::get());

		// --- Finally, we update the last emission by the caller.
		LastEmit::<T>::insert( neuron_uid, current_block );
		debug::info!("The old last emit: {:?} the new last emit: {:?}", last_emit, current_block);
	
		// --- Return ok.
		debug::info!("--- Done emit");
		return total_new_stake_u64;
	}
}