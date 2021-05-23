use super::*;


impl<T: Trait> Module<T> {
    /***********************************************************
     * do_compute_connectivity() - computes connectivity between all pairs
     ***********************************************************/
    pub fn do_compute_connectivity()
    {
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed(origin)?;
        debug::info!("--- Called add_stake with coldkey id {:?}, hotkey {:?} and ammount_staked {:?}", coldkey, hotkey, stake_to_be_added);

        // Check if the hotkey is active
        ensure!(Self::is_hotkey_active(&hotkey), Error::<T>::NotActive);
        let neuron = Self::get_neuron_for_hotkey(&hotkey);

        // Check if uid is active
        ensure!(Self::is_uid_active(neuron.uid), Error::<T>::NotActive);

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::neuron_belongs_to_coldkey(&neuron, &coldkey), Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey. Neurons must call an emit before they change 
        // their stake or else can cheat the system by adding stake just before
        // and emission to maximize their inflation.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_for_neuron(&neuron);

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let stake_as_balance = Self::u64_to_balance(stake_to_be_added);
        ensure!(stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance);

        ensure!(Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()), Error::<T>::NotEnoughBalanceToStake);
        ensure!(Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true, Error::<T>::BalanceWithdrawalError);
        Self::add_stake_to_neuron_hotkey_account(neuron.uid, stake_to_be_added);

        // ---- Emit the staking event.
        Self::deposit_event(RawEvent::StakeAdded(hotkey, stake_to_be_added));

        // --- ok and return.
        Ok(())
    }
}

