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
    // Parallax (14-1-2021) This is not used. Keeping it in here for now. Remove when not used
    // down the line.
    // pub fn do_emit(origin: T::Origin) -> dispatch::DispatchResult {
    //     // ---- We check the transaction is signed by the caller
    //     // and retrieve the T::AccountId pubkey information.
    //     let hotkey_id = ensure_signed(origin)?;
    //     debug::info!("--- Called emit with caller {:?}", hotkey_id);
    //
    //     // ---- We query the Neuron set for the neuron data stored under
    //     // the passed hotkey and retrieve it as a NeuronMetadata struct.
    //     ensure!(Self::is_hotkey_active(&hotkey_id), Error::<T>::NotActive);
    //     let neuron = Self::get_neuron_for_hotkey(&hotkey_id);
    //
    //     // ---- We call emit for this neuron.
    //     Self::emit_for_neuron(&neuron);
    //
    //     // ---- Done.
    //     Ok(())
    // }

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
        let pending_emission_for_neuron = Self::get_pending_emission_for_neuron(neuron.uid);

        // --- We get the callers weights. The total emission will be distributed
        // according to these weights. The weight_vals sum to u32::max. ie. They have been normalized
        // to u32:max. This normalization takes places when weights are set. See fn set_weights
        let (weight_uids,  weight_vals) = Self::get_weights_for_neuron(neuron);

        // --- Before the the inflation can be emitted to the stake account of the destination neurons
        // we perform some sanity checks. This means:
        // - The emission for the neuron calling this function must be greater than zero
        // - The vectors containing the account ids and values of the destination neurons must be
        // non zero. If either of these requirements are not met, emission can not take place.
        if !Self::can_emission_proceed(&pending_emission_for_neuron, &weight_uids, &weight_vals) {
            return 0;
        }

        // --- Emission will be distributed but we drain the emission before this occurs 
        // just in case the following steps bork and allows the user to emit the same tokens
        // multiple times.
        Self::reset_pending_emission_for_neuron(neuron.uid);

        // --- We iterate through the weights and distribute the caller's emission to
        // neurons on a weighted basis. The emission becomes new stake in their
        // staking account.
        let mut total_new_stake: u64 = 0; // Total stake added across all emissions.
        for (i, dest_uid) in weight_uids.iter().enumerate() {

			// --- We check that the staking account exists. We do not emit to non-existing nodes.
			// This emission is burned if the user does not exist.
			let is_existent_neuron = Self::is_uid_active(*dest_uid);
			if !is_existent_neuron { continue; }

            // --- We get the weight from neuron i to neuron j, where neuron i is the calling neuron
            // and j is the destination neuron.
            // The weights are normalized and sum to u32::max. (See fn set_weights)
            // This means we have to normalize the weights with respect to one.
            let w_ij = normalize(weight_vals[i]);
            debug::info!("Emitting to {:?} | weight: {:?}", dest_uid, w_ij);

            // ---The stake increment is calculated by multiplying the emission for the calling neuron, as
            // as calculated above, and the weight which is now a value between 0 and 1. The stake
            // increment is thus a proportion of the total emission the source neuron gets to emit.
            let stake_increment = Self::calculate_stake_increment(pending_emission_for_neuron, w_ij);

            // --- We check if the weight is a self loop. In this case, the emission does not proceed
            // to deposit new funds. The self weight is purely used to pay for transactions fees.
            //if *dest_uid != neuron.uid {
            Self::add_stake_to_neuron_hotkey_account(*dest_uid, stake_increment);
            //}

            // --- We increase the total stake emitted.
            total_new_stake += stake_increment;
        }

        // --- Finally, we update the last emission by the caller.
        Self::update_last_emit_for_neuron(neuron.uid);

        // --- Return ok.
        debug::info!("--- Done emit");
        return total_new_stake;
    }

    pub fn get_self_emission_for_caller( caller: &T::AccountId) -> u64 {

        // --- We get the neuron associated with the calling hotkey account.
        let neuron = Self::get_neuron_for_hotkey( caller );

        // --- Non active uids have zero emission.
		let is_existent_neuron = Self::is_uid_active(neuron.uid);
		if !is_existent_neuron { return 0 }

        // --- How much total remaining emission is available for this neuron.
        let pending_emission_for_neuron = Self::get_pending_emission_for_neuron( neuron.uid );

        // --- We get the callers weights.
        let (weight_uids,  weight_vals) = Self::get_weights_for_neuron( &neuron );

        // - The emission for the neuron calling this function must be greater than zero
        // - The vectors containing the account ids and values of the destination neurons must be
        // non zero. If either of these requirements are not met, emission is zero.
        if !Self::can_emission_proceed(&pending_emission_for_neuron, &weight_uids, &weight_vals) {
            return 0;
        }

        // --- We iterate through the weights to find the self-weight. The self emission
        // is easily computed as pending_emission * normalize_self_weight.
        for (i, dest_uid) in weight_uids.iter().enumerate() {

            // Is this the self weight.
            if *dest_uid == neuron.uid {
                // Normalize the self weight.
                let w_ii = normalize(weight_vals[i]);

                // Compute the stake increment emission.dest_uid
                let stake_increment = Self::calculate_stake_increment(pending_emission_for_neuron, w_ii);

                return stake_increment
            }
        }
        // No self weight?
        return 0
    }

     /// Sets the pending emission for all active peers based on a single block transition.
     ///
     /// This function is called every time a new block is started. ie, before the processing of
     /// extrinsics happens. In essence, this function integrates the proportional block reward for
     /// each neuron (which at a later point, can be distributed among its peers per the weights vector),
     /// with respect to the block number.
     /// Every block, the proportional block reward is calculated per neuron, and put in the PendingEmission
     /// map. If a value already exists for the neuron, the new reward is added to the existing value.
     ///
     /// Then, when a neuron sets new weights, or when stake is added/removed, the emit_for_neuron function is
     /// called which distributes this pending emission among the peers in the weights vector.
     /// At this point, the PendingEmission is reset, and this cycle starts again.
    pub fn update_pending_emissions() -> u64 {
        let mut weight = 0;
        let block_reward = Self::current_block_reward();
        let total_stake = Self::get_total_stake();

        if total_stake == 0 {
            return weight;
        }

        for (uid, neuron_stake) in <Stake as IterableStorageMap<u64, u64>>::iter() {
            if neuron_stake == 0 { continue; }
            let stake_fraction = Self::calulate_stake_fraction(neuron_stake, total_stake);
            let new_emission = Self::calculate_new_emission(block_reward, stake_fraction);
            Self::update_pending_emission_for_neuron(uid, new_emission);
            weight += 1;
        }
        weight
    }

    /// This is a check to determine if an emission to a set of peers can proceed.
    /// The conditions are:
    /// 1) The emission > 0; Without emission, there is nothing to emit. (You don't say?)
    /// 2) The weights uid and values vector must be non empty
    pub fn can_emission_proceed(emission : &U64F64, weight_uids : &Vec<u64>, weight_vals : &Vec<u32>) -> bool {
        if *emission == U64F64::from_num(0) {return false;}
        if weight_uids.is_empty() { return false }
        if weight_vals.is_empty() { return false }
        return true;
    }

    /// This function calculates the amount with which existing stake of a neuron is increment4d
    /// It is calculated by taking the product of the emission for a neuron that can be
    /// devided among its peers by the weight to a peer
    /// emission : Total emission for neuron i
    /// weight: weight from neuron i to neuron j  0..1
    ///
    pub fn calculate_stake_increment(emission : U64F64, weight : U64F64) -> u64 {
        assert!(weight >= 0);
        assert!(weight <= 1);

        let increment = emission * weight;
        return increment.to_num::<u64>()
    }

    /// Returns the pending emission for a neuron in U64F64 format
    /// The default behaviour when a uid does not exist, is to return 0
    pub fn get_pending_emission_for_neuron(uid : u64) -> U64F64 {
        U64F64::from_num( PendingEmission::get(uid) )
    }

    /// Resets the pending emission for a neuron to zero
    pub fn reset_pending_emission_for_neuron(uid : u64 ) {
        PendingEmission::insert(uid, 0);
    } 


    pub fn update_last_emit_for_neuron(uid: u64) {
        // The last emit determines the last time this peer made an incentive
        // mechanism emit call. Since he is just subscribed with zero stake,
        // this moment is considered his first emit.
        let current_block: T::BlockNumber = system::Module::<T>::block_number();
        debug::info!("The new last emit for this caller is: {:?} ", current_block);

        // ---- We initialize the neuron maps with nill weights,
        // the subscription gift and the current block as last emit.
        LastEmit::<T>::insert(uid, current_block);
    }

    /// Returns the block number for the last block and emission for the specified neuron
    /// has occurred. Returns 0 when a uid does not exist
    pub fn get_last_emit_for_neuron(uid : u64) -> T::BlockNumber {
        return LastEmit::<T>::get(uid);
    }

    /// Calculates the total emission for a neuron that it can distribute among its peers
    /// per its weight matrix.
    /// block_reward : The block reward for the current block
    /// stake_fraction : The proportion of the stake a neuron has to the total stake
    ///
    /// Warning, the stake fraction should be number between 0 and 1 (inclusive)
    /// The calling function should check for this constraint.
    ///
    /// This constraint makes sure the u64 is not overflowed
    ///
    pub fn calculate_new_emission(block_reward : U64F64, stake_fraction : U64F64) -> u64{
        assert!(stake_fraction >=0);
        assert!(stake_fraction <=1);

        let new_emission = block_reward * stake_fraction;
        return new_emission.to_num::<u64>();
    }

    /// Persist the increase of the pending emission for the neuron to the database
    /// uid: the uid of the neuron for whom the pending emision is updated
    /// new_emission : The amount of emission that is added to the already existing amount
    ///
    pub fn update_pending_emission_for_neuron(uid: u64, new_emission : u64) {
        PendingEmission::mutate(uid, |el| *el += new_emission);
    }
}

fn normalize(value: u32) -> U64F64 {
    let converted_value = U64F64::from_num(value);
    let normalized_value = converted_value / U64F64::from_num(u32::MAX);
    return normalized_value;
}
