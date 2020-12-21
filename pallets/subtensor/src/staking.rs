use super::*;

impl<T: Trait> Module<T> {
    pub fn do_add_stake(origin: T::Origin, hotkey: T::AccountId, ammount_staked: u64) -> dispatch::DispatchResult
    {
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called add_stake with caller {:?}, hotkey {:?} and ammount_staked {:?}", caller, hotkey, ammount_staked);

        // ---- We query the Neuron set for the neuron data stored under
        // the passed hotkey and retrieve it as a NeuronMetadata struct.
        ensure!(Neurons::<T>::contains_key(&hotkey), Error::<T>::NotActive);
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&hotkey);
        debug::info!("Got metadata for hotkey {:?}", hotkey);

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(neuron.coldkey == caller, Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey. Neurons must call an emit before they change 
        // their stake or else can cheat the system by adding stake just before
        // and emission to maximize their inflation.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_from_uid( neuron.uid );

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let staked_currency = Self::u64_to_balance( ammount_staked );
        ensure!(staked_currency.is_some(), Error::<T>::CouldNotConvertToBalance);

        let staked_currency = staked_currency.unwrap();
        let new_potential_balance = T::Currency::free_balance(&caller) - staked_currency;
        let can_withdraw = T::Currency::ensure_can_withdraw(&caller, staked_currency, WithdrawReasons::except(WithdrawReason::Tip), new_potential_balance).is_ok();

       
        // ---- If we can withdraw the requested funds, we withdraw from the  
        // coldkey account and deposit the funds into the staking account of 
        // the associated hotkey-neuron.
        if can_withdraw {

            // ---- We perform the withdrawl from the coldkey account before
            // addding stake into the hotkey neuron's staking account.
            let _ = T::Currency::withdraw(&caller, staked_currency, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
            debug::info!("Withdrew {:?} from coldkey: {:?}", staked_currency, caller);

            // --- We update the hotkey's staking account with the new funds.
            let hotkey_stake: u64 = Stake::get( neuron.uid );
            Stake::insert( neuron.uid, hotkey_stake + ammount_staked);
            debug::info!("Added new stake: {:?} to hotkey {:?}", ammount_staked, hotkey);

            // --- We update the total staking pool with the new funds.
            let total_stake: u64 = TotalStake::get();
            TotalStake::put(total_stake + ammount_staked);
            debug::info!("Added {:?} to total stake, now {:?}", ammount_staked, TotalStake::get());

            // ---- Emit the staking event.
            Self::deposit_event(RawEvent::StakeAdded(hotkey, ammount_staked));

        } else {

            debug::info!("Could not withdraw {:?} from coldkey {:?}", staked_currency, caller);
        }

        // --- ok and return.
        debug::info!("--- Done add_stake.");
        Ok(())
    }


    pub fn do_remove_stake(origin: T::Origin, hotkey: T::AccountId, ammount_unstaked: u64) -> dispatch::DispatchResult {
        
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called remove_stake with {:?}, hotkey {:?} and ammount {:?}", caller, hotkey, ammount_unstaked);

        // ---- We query the Neuron set for the NeuronMetadata stored under
        // the passed hotkey.
        ensure!(Neurons::<T>::contains_key(&hotkey), Error::<T>::NotActive);
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&hotkey);
        debug::info!("Got metadata for hotkey.");

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(neuron.coldkey == caller, Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey.
        // Neurons must call an emit before they remove
        // stake or they may be able to cheat their peers of inflation.
        Self::emit_from_uid( neuron.uid );

        // ---- We check that the hotkey has enough stake to withdraw
        // and then withdraw from the account.
        let hotkey_stake: u64 = Stake::get( neuron.uid );
        ensure!(hotkey_stake >= ammount_unstaked, Error::<T>::NotEnoughStaketoWithdraw);
        Stake::insert( neuron.uid, hotkey_stake - ammount_unstaked );
        debug::info!("Withdraw: {:?} from hotkey staking account for new ammount {:?} staked", ammount_unstaked, hotkey_stake - ammount_unstaked);
    
        // --- We perform the withdrawl by converting the stake to a u64 balance
        // and deposit the balance into the coldkey account. If the coldkey account
        // does not exist it is created.
        let ammount_unstaked_as_currency = Self::u64_to_balance( ammount_unstaked );
        ensure!(ammount_unstaked_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);
        let _ = T::Currency::deposit_creating(&caller, ammount_unstaked_as_currency.unwrap());
        debug::info!("Deposit {:?} into coldkey balance ", ammount_unstaked_as_currency.unwrap());

        // --- We update the total staking pool with the removed funds.
        let total_stake: u64 = TotalStake::get();
        TotalStake::put(total_stake - ammount_unstaked);
        debug::info!("Remove {:?} from total stake, now {:?} ", ammount_unstaked, TotalStake::get());

        // ---- Emit the unstaking event.
        Self::deposit_event(RawEvent::StakeRemoved(hotkey, ammount_unstaked));
        debug::info!("--- Done remove_stake.");
        
        // --- Done and ok.
        Ok(())
    }
}