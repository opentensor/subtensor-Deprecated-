// use pallet_subtensor::{Error};
// use frame_support::{assert_ok};
// use frame_system::Trait;
mod mock;
use mock::*;

#[test]
fn test_block_reward_for_blocks_0_and_1() {
	new_test_ext().execute_with(|| {
        assert_eq!(SubtensorModule::get_reward_for_current_block(), 500000000);
        run_to_block(1);
        assert_eq!(SubtensorModule::get_reward_for_current_block(), 500000000);
	});
}

#[test]
fn test_block_reward_at_halvings() {
	new_test_ext().execute_with(|| {
        // Halving 1
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(20999999 as u64)), 500000000);
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(21000000 as u64)), 250000000);

        // Halving 2
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(41999999 as u64)), 250000000);
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(42000000 as u64)), 125000000);

        // Halving 3
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(62999999 as u64)), 125000000);
        assert_eq!(SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(63000000 as u64)), 62500000);

        // Halving i from 1..10
        let halving_time: u64 = 21000000;
        let block_reward: u64 = 500000000;
        for i in 1..10 {
            let iu32 = i as u32;
            let reward_as_computed: u64 = block_reward.overflowing_shr( iu32 ).0;
            let reward: u64 = SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from( halving_time * i)).to_num::<u64>();
            assert_eq!(reward, reward_as_computed);
        }
	});
}

#[test]
fn test_roughly_21000000() {
	new_test_ext().execute_with(|| {
        let halving_time_usize: u64 = 21000000;
        let halving_time: u64 = 21000000;
        let mut sum_reward: u64 = 0;
        for i in 0..100 {
            let reward_during_epoch: u64 = SubtensorModule::block_reward_for_blocknbr(&BlockNumber::from(halving_time_usize * i)).to_num::<u64>();
            let total_during_epoch: u64 = reward_during_epoch * halving_time;
            sum_reward += total_during_epoch;
        }
        let almost_21000000 = 20999999727000000;
        assert_eq!(sum_reward, almost_21000000);

	});
}