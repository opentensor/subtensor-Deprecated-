use super::*;

impl<T: Trait> Module<T> {
	/// Returns the bitcoin inflation rate at passed block number. We use a mapping between the bitcoin and substrate blocks.
	/// Substrate blocks mint 100x faster and so the halving time and inflation rate need to be correspondingly changed.
	/// Each block produces 0.5 x 10^12 tokens, or semantically, 0.5 full coins every 6 seconds. Likewise, the halving
	/// occurs every 21 million blocks. Like bitcoin, this ensures there can only ever be 21 million full tokens 
	/// created. In our case this ammount is furthre limited since inflation is only triggered by calling peers.
	/// The inflation is therefore not continuous, and we lose out when peers fail to emit with their stake, or
	/// fail to emit before a halving.
	/// 
	/// # Args:
	///  	* `now` (&T::BlockNumber):
	/// 		- The block number we wish to know the inflation rate at.
	/// 
	/// # Returns
	/// 	* block_reward (U64F64):
	/// 		- The number of tokens to emit at this block as a fixed point.
	/// 	
	pub fn block_reward(blocknr: &<T as system::Trait>::BlockNumber) -> U64F64 {

		// --- We convert the block number to u64 and then to a fixed point.
		let elapsed_blocks_u64 = TryInto::try_into(*blocknr).ok().expect("blockchain will not exceed 2^64 blocks; QED.");
		let elapsed_blocks_u64_f64 = U64F64::from_num(elapsed_blocks_u64);

		// --- We get the initial block reward.
		// Bitcoin block reward started at 50 tokens per block and the average substrate block time is 6 seconds. 
		// Therefore the equivalent halving is (50 blocks) / (10 min * 60 sec / 6 sec) = (50) / (100) = (0.5 tokens per block) or 0.5 * 10^9 = 500000000
		let block_reward = U64F64::from_num(500000000);

		// --- We calculate the number of halvings since the chain was initialized
		// Bitcoin inflation halves every 210,000 blocks which mint every 10 minutes.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (210,000 blocks) * (10 min * 60 sec / 6 sec) =  (210,000) * (100) = (21,000,000 blocks)
		let block_halving = U64F64::from_num(21000000);
        let fractional_halvings = elapsed_blocks_u64_f64 / block_halving;
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("fractional_halvings: {:?}", fractional_halvings);

		// --- We shift the block reward for each halving to get the actual reward at this block.
		// NOTE: Underflow occurs in 21,000,000 * 64 blocks, essentially never QED.        
        let block_reward_shift = block_reward.overflowing_shr(fractional_halvings.to_num::<u32>()).0;

		// --- We return the result.
		block_reward_shift
	}

	pub fn current_block_reward() -> U64F64{
		let current_block = system::Module::<T>::block_number();
		let block_reward =  Self::block_reward(&current_block);

		debug::info!("Current block reward ({:?}): {:?}", current_block, block_reward);
		return block_reward;
	}
}