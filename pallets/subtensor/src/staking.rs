use super::*;

impl<T: Trait> Module<T> {

    /***********************************************************
     * do_add_stake() - main function called from parent module
     ***********************************************************/
    pub fn do_add_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_added: u64) -> dispatch::DispatchResult
    {
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed(origin)?;
        debug::info!("--- Called add_stake with coldkey id {:?}, hotkey {:?} and ammount_staked {:?}", coldkey, hotkey, stake_to_be_added);

        // Check if the hotkey is active
        ensure!(Self::is_active(&hotkey), Error::<T>::NotActive);

        let neuron = Self::get_neuron_for_hotkey(&hotkey);

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::does_neuron_belong_to_coldkey(&neuron, &coldkey), Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey. Neurons must call an emit before they change 
        // their stake or else can cheat the system by adding stake just before
        // and emission to maximize their inflation.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_from_uid( neuron.uid );

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let stake_as_balance = Self::u64_to_balance( stake_to_be_added );
        ensure!(stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance);
        let stake_as_balance = stake_as_balance.unwrap();

        ensure!(Self::coldkey_has_enough_balance(&coldkey, &stake_as_balance), Error::<T>::NotEnoughBalanceToStake);
        Self::withdraw_stake_from_coldkey_account(&coldkey, &stake_as_balance);
        Self::add_stake_to_neuron_hotkey_account(neuron, stake_to_be_added);
        Self::increase_total_stake(stake_to_be_added);

        // ---- Emit the staking event.
        Self::deposit_event(RawEvent::StakeAdded(hotkey, stake_to_be_added));


        // --- ok and return.
        debug::info!("--- Done add_stake.");
        Ok(())
    }

    fn coldkey_has_enough_balance(coldkey: &T::AccountId, staked_currency: &<T::Currency as Currency<<T as system::Trait>::AccountId>>::Balance) -> bool {
        let new_potential_balance = T::Currency::free_balance(&coldkey) - *staked_currency;
        let can_withdraw = T::Currency::ensure_can_withdraw(&coldkey, *staked_currency, WithdrawReasons::except(WithdrawReason::Tip), new_potential_balance).is_ok();
        can_withdraw
    }

    fn increase_total_stake(increment: u64) {
        // --- We update the total staking pool with the new funds.
        let total_stake: u64 = TotalStake::get();
        TotalStake::put(total_stake + increment);
        debug::info!("Added {:?} to total stake, now {:?}", increment, TotalStake::get());
    }

    fn add_stake_to_neuron_hotkey_account(neuron: NeuronMetadataOf<T>, amount: u64) {
        let hotkey_stake: u64 = Stake::get(neuron.uid);
        Stake::insert(neuron.uid, hotkey_stake + amount);
        debug::info!("Added new stake: {:?} to uid {:?}", amount, neuron.uid);
    }

    fn withdraw_stake_from_coldkey_account(coldkey: &T::AccountId, amount: &<T::Currency as Currency<T::AccountId>>::Balance) {
        // @Todo (parallax, 28-12-2020) Do error handling on this call
        let _ = T::Currency::withdraw(&coldkey, *amount, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
        debug::info!("Withdrew {:?} from coldkey: {:?}", *amount, coldkey);
    }

    // ---- We query the Neuron set for the neuron data stored under
    // the passed hotkey and retrieve it as a NeuronMetadata struct.
    fn get_neuron_for_hotkey(hotkey: &T::AccountId) -> NeuronMetadataOf<T> {
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&hotkey);
        debug::info!("Got neuron metadata for hotkey {:?}", hotkey);
        neuron
    }

    fn is_active(hotkey: &T::AccountId) -> bool {
        return Neurons::<T>::contains_key(&hotkey);
    }

    fn does_neuron_belong_to_coldkey(neuron : &NeuronMetadataOf<T>, coldkey : &T::AccountId) -> bool {
        return neuron.coldkey == *coldkey
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