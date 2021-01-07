use super::*;
use frame_support::debug::RuntimeLogger;

impl<T: Trait> Module<T> {
    /// Called on the finalization of a substrate block. (the order of on_finalize calls is determined in the runtime)
    ///
    /// # Args:
	/// 	* 'n': (T::BlockNumber):
	/// 		- The number of the block we are finalizing.
    pub fn do_finalize(n: T::BlockNumber) {
        // Self::update_pending_emission();
        // does nothing.
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
        let total_stake = U64F64::from_num( TotalStake::get() );
        if total_stake == U64F64::from_num(0) {
            return 0
        }
        for (uid, stake) in <Stake as IterableStorageMap<u64, u64>>::iter() {
            if stake == 0 { continue; }
            let stake = U64F64::from_num(stake);
            let stake_fraction = stake / total_stake;
            let new_emission = block_reward * stake_fraction;
            let new_emission_u64 = new_emission.to_num::<u64>();
            PendingEmission::mutate(uid, |el| *el += new_emission_u64);
            weight += 1;
        }
        weight
    }
}
