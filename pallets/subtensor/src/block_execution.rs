use super::*;
use frame_support::debug::RuntimeLogger;

impl<T: Trait> Module<T> {
    /// Called on the finalization of a substrate block. (the order of on_finalize calls is determined in the runtime)
    ///
    /// # Args:
	/// 	* 'n': (T::BlockNumber):
	/// 		- The number of the block we are finalizing.
    pub fn do_finalize(n: T::BlockNumber) {
        Self::update_pending_emission();

    }

    /// Called on the initialization of this pallet. (the order of on_finalize calls is determined in the runtime)
    ///
    /// # Args:
	/// 	* 'n': (T::BlockNumber):
	/// 		- The number of the block we are finalizing.
    pub fn do_initialize(n: T::BlockNumber) -> u64 {
        Self::update_pending_emission()
    }

    /// Sets the pending emission for all active peers based on a single block transition.
    pub fn update_pending_emission() -> u64 {
        let mut weight = 0;
        let block_reward = Self::current_block_reward();
        let total_stake = TotalStake::get();
        for (uid, stake) in <Stake as IterableStorageMap<u64, u64>>::iter() {
            let stake_fraction = Self::calculate_stake_fraction_with_total_stake( total_stake, stake );
            let new_emission = block_reward * stake_fraction;
            let new_emission_u64 = new_emission.to_num::<u64>();
            PendingEmission::mutate(uid, |el| *el += new_emission_u64);
            weight += 1;
        }
        weight
    }

    // Calculates the stake fraction for a passed stake and total stake item.
    pub fn calculate_stake_fraction_with_total_stake( total_stake: u64, stake: u64 ) -> U64F64 {
        let total_stake = U64F64::from_num(TotalStake::get());
        let neuron_stake = U64F64::from_num(stake);
        // Total stake is 0, this should virtually never happen, but is still here because it could
        if total_stake == U64F64::from_num(0) {
            return U64F64::from_num(0);
        }
        // Neuron stake is zero. This means there will be nothing to emit
        if neuron_stake ==U64F64::from_num(0) {
            return U64F64::from_num(0);
        }
        let stake_fraction = neuron_stake / total_stake;
        return stake_fraction;
    }
}
