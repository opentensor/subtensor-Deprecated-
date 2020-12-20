use super::*;

impl<T: Trait> Module<T> {
    pub fn do_add_stake(origin: T::Origin, hotkey: T::AccountId, ammount_staked: u32) -> dispatch::DispatchResult
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
        Self::emit(&hotkey);

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let staked_currency = Self::u32_to_balance(ammount_staked);
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
            let hotkey_stake: u32 = Stake::<T>::get(&hotkey);
            Stake::<T>::insert(&hotkey, hotkey_stake + ammount_staked);
            debug::info!("Added new stake: {:?} to hotkey {:?}", ammount_staked, hotkey);

            // --- We update the total staking pool with the new funds.
            let total_stake: u32 = TotalStake::get();
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
}