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
    pub fn do_emit(origin: T::Origin) -> dispatch::DispatchResult {
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called emit with caller {:?}", hotkey_id);

        // ---- We query the Neuron set for the neuron data stored under
        // the passed hotkey and retrieve it as a NeuronMetadata struct.
        ensure!(Self::is_active(&hotkey_id), Error::<T>::NotActive);
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey_id);

        // ---- We call emit for this neuron.
        Self::emit_for_neuron(&neuron);

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
    pub fn emit_for_neuron(neuron: &NeuronMetadataOf<T>) -> u64 {
        // --- We init the Runtimelogger for WASM debugging
        RuntimeLogger::init();
        debug::info!("--- Calling emit, neuron_uid: {:?}", neuron.uid);

        // --- We calculate the total emission available to the caller.
        // the block reward is positive and non-zero, so are the stake_fraction and elapsed blocks.
        // this ensures the total_emission is positive non-zero. To begin the block reward is (0.5 * 10^12).
        // let callers_emission_u64_f64 = stake_fraction_u64_f64 * block_reward * elapsed_blocks;
        let emission_for_neuron = Self::calculate_emission_for_neuron(&neuron);

        // --- We get the callers weights. The total emission will be distributed
        // according to these weights. The weight_vals sum to u32::max. ie. They have been normalized
        // to u32:max. This normalization takes places when weights are set. See fn set_weights
        let (weight_uids, weight_vals) = Self::get_weights_for_neuron(neuron);

        // --- Before the the inflation can be emitted to the stake account of the destination neurons
        // we perform some sanity checks. This means:
        // - The emission for the neuron calling this function must be greater than zero
        // - The vectors containing the account ids and values of the destination neurons must be
        // non zero. If either of these requirements are not met, emission can not take place.
        if !Self::can_emission_proceed(&emission_for_neuron, &weight_uids, &weight_vals) {
            return 0;
        }

        // --- We iterate through the weights and distribute the caller's emission to
        // neurons on a weighted basis. The emission becomes new stake in their
        // staking account.
        let mut total_new_stake: u64 = 0; // Total stake added across all emissions.
        for (i, dest_uid) in weight_uids.iter().enumerate() {
            // --- We get the weight from neuron i to neuron j, where neuron i is the calling neuron
            // and j is the destination neuron.
            // The weights are normalized and sum to u32::max. (See fn set_weights)
            // This means we have to normalize the weights with respect to one.
            let w_ij = normalize(weight_vals[i]);
            debug::info!("Emitting to {:?} | weight: {:?}", dest_uid, w_ij);

            // ---The stake increment is calculated by multiplying the emission for the calling neuron, as
            // as calculated above, and the weight which is now a value between 0 and 1. The stake
            // increment is thus a proportion of the total emission the source neuron gets to emit.
            let stake_increment = Self::calulate_stake_increment(emission_for_neuron, w_ij);
            Self::add_stake_to_neuron_hotkey_account(*dest_uid, stake_increment);

            // --- We increase the total stake emitted.
            total_new_stake += stake_increment;
        }

        // --- Finally, we update the last emission by the caller.
        Self::update_last_emit_for_neuron(&neuron);

        // --- Return ok.
        debug::info!("--- Done emit");
        return total_new_stake;
    }

    fn can_emission_proceed(emission : &U64F64, weight_uids : &Vec<u64>, weight_vals : &Vec<u32>) -> bool {
        if *emission == U64F64::from_num(0) {return false;}
        if weight_uids.is_empty() { return false }
        if weight_vals.is_empty() { return false }
        return true;
    }

    fn calulate_stake_increment(emission : U64F64, weight : U64F64) -> u64 {
        let increment = emission * weight;
        return increment.to_num::<u64>()
    }

    fn calculate_emission_for_neuron(neuron : &NeuronMetadataOf<T>) -> U64F64 {
        // --- We get the current block reward and the number of blocks that have been
        // generated since the last time the user emitted the total proportions of his
        // stake to his peers.
        // This is needed to determine the proportion of inflation allocated to
        // the caller. Note also, that the block reward is a decreasing function
        // callers want to call emit before the block inflation decreases.

        let block_reward = Self::current_block_reward();
        let elapsed_blocks = Self::elapsed_blocks_for_neuron(&neuron);

        // --- We get the fraction of total stake held by the caller.
        // This should only be zero if the caller has zero stake. Otherwise
        // it returns a floating point (a.k.a, bits in the F64 part.)
        let stake_fraction = Self::calculate_stake_fraction_for_neuron(&neuron);

        let emission  = block_reward * stake_fraction * elapsed_blocks;

        return emission;
    }


    fn elapsed_blocks_for_neuron(neuron : &NeuronMetadataOf<T>) -> U64F64{
		// --- We compute the delta time (in blocks) since the user made an 
		// emission.
        let current_block = system::Module::<T>::block_number();
        let last_emit: T::BlockNumber = LastEmit::<T>::get(neuron.uid);

        let elapsed_blocks = current_block - last_emit;
        let elapsed_blocks_u64: usize = TryInto::try_into(elapsed_blocks).ok().expect("blockchain will not exceed 2^64 blocks; qed");
        let elapsed_blocks_u64_f64 = U64F64::from_num(elapsed_blocks_u64);

        debug::debug!("current block {:?}", current_block);
        debug::debug!("last emit block for uid: {:?} : {:?}", neuron.uid, last_emit);
        debug::debug!("elapsed_blocks_u64: {:?}", elapsed_blocks_u64);
        debug::debug!("elapsed_blocks_u64_f64: {:?}", elapsed_blocks_u64_f64);

        elapsed_blocks_u64_f64
    }


    pub fn update_last_emit_for_neuron(neuron: &NeuronMetadataOf<T>) {
        // The last emit determines the last time this peer made an incentive
        // mechanism emit call. Since he is just subscribed with zero stake,
        // this moment is considered his first emit.
        let current_block: T::BlockNumber = system::Module::<T>::block_number();
        debug::info!("The new last emit for this caller is: {:?} ", current_block);

        // ---- We initilize the neuron maps with nill weights,
        // the subscription gift and the current block as last emit.
        LastEmit::<T>::insert(neuron.uid, current_block);
    }

    pub fn remove_last_emit_info_for_neuron(neuron: &NeuronMetadataOf<T>) {
        LastEmit::<T>::remove(neuron.uid);
    }
}

fn normalize(value: u32) -> U64F64 {
    let converted_value = U64F64::from_num(value);
    let normalized_value = converted_value / U64F64::from_num(u32::MAX);
    return normalized_value;
}