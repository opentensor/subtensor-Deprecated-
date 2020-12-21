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
	/// 	* block_reward (U32F32):
	/// 		- The number of tokens to emit at this block as a fixed point.
	/// 	
	pub fn block_reward(now: &<T as system::Trait>::BlockNumber) -> U32F32 {

		// --- We convert the block number to u32 and then to a fixed point.
		let elapsed_blocks_u32 = TryInto::try_into(*now).ok().expect("blockchain will not exceed 2^32 blocks; QED.");
		let elapsed_blocks_u32_f32 = U32F32::from_num(elapsed_blocks_u32);

		// --- We get the initial block reward.
		// TODO(const): should be 0.5 x 10 ^ 12 not 0.5
		// Bitcoin block reward started at 50 tokens per block and the average substrate block time is 6 seconds. 
		// Therefore the equivalent halving is (50 blocks) / (10 min * 60 sec / 6 sec) = (50) / (100) = (0.5 tokens per block)
		let block_reward = U32F32::from_num(0.5);

		// --- We calculate the number of halvings since the chain was initialized
		// Bitcoin inflation halves every 210,000 blocks which mint every 10 minutes.
		// The average substrate block time is 6 seconds.
		// The equivalent halving is (210,000 blocks) * (10 min * 60 sec / 6 sec) =  (210,000) * (100) = (21,000,000 blocks)
		let block_halving = U32F32::from_num(21000000);
		let fractional_halvings = elapsed_blocks_u32_f32 / block_halving;
		let floored_halvings = fractional_halvings.floor().to_num::<u32>();
		debug::info!("block_halving: {:?}", block_halving);
		debug::info!("floored_halvings: {:?}", floored_halvings);

		// --- We shift the block reward for each halving to get the actual reward at this block.
		// NOTE: Underflow occurs in 21,000,000 * (16 + 4) blocks, essentially never.
		let block_reward_shift = block_reward.overflowing_shr(floored_halvings).0;

		// --- We return the result.
		block_reward_shift
	}
}