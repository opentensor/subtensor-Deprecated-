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
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey);

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::neuron_belongs_to_coldkey(&neuron, &coldkey), Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey. Neurons must call an emit before they change 
        // their stake or else can cheat the system by adding stake just before
        // and emission to maximize their inflation.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_for_neuron( &neuron );

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let stake_as_balance = Self::u64_to_balance( stake_to_be_added );
        ensure!(stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance);
        // let stake_as_balance = stake_as_balance.unwrap();

        ensure!(Self::coldkey_has_enough_balance(&coldkey, &stake_as_balance), Error::<T>::NotEnoughBalanceToStake);
        Self::remove_stake_from_coldkey_account(&coldkey, &stake_as_balance);
        Self::add_stake_to_neuron_hotkey_account(neuron.uid, stake_to_be_added);
        Self::increase_total_stake(stake_to_be_added);

        // ---- Emit the staking event.
        Self::deposit_event(RawEvent::StakeAdded(hotkey, stake_to_be_added));


        // --- ok and return.
        debug::info!("--- Done add_stake.");
        Ok(())
    }

    /**
    * This function removes stake from a hotkey account and puts into a coldkey account.
    * This function should be called through an extrinsic signed with the coldkeypair's private
    * key. It takes a hotkey account id and an ammount as parameters.
    *
    * Generally, this function works as follows
    * 1) A Check is performed to see if the hotkey is active (ie, the node using the key is subscribed)
    * 2) The neuron metadata associated with the hotkey is retrieved, and is checked if it is subscribed with the supplied cold key
    * 3) If these checks pass, inflation is emitted to the nodes' peers
    * 4) If the account has enough stake, the requested amount it transferred to the coldkey account
    * 5) The total amount of stake is reduced after transfer is complete
    *
    * It throws the following errors if there is something wrong
    * - NotActive : The suplied hotkey is not in use. This ususally means a node that uses this key has not subscribed yet, or has unsubscribed
    * - NonAssociatedColdKey : The supplied hotkey account id is not subscribed using the supplied cold key
    * - NotEnoughStaketoWithdraw : The ammount of stake available in the hotkey account is lower than the requested amount
    * - CouldNotConvertToBalance : A conversion error occured while converting stake from u64 to Balance
    */
    pub fn do_remove_stake(origin: T::Origin, hotkey: T::AccountId, stake_to_be_removed: u64) -> dispatch::DispatchResult {
        
        // ---- We check the transaction is signed by the caller
        // and retrieve the T::AccountId pubkey information.
        let coldkey = ensure_signed(origin)?;
        debug::info!("--- Called remove_stake with {:?}, hotkey {:?} and ammount {:?}", coldkey, hotkey, stake_to_be_removed);

        // ---- We query the Neuron set for the NeuronMetadata stored under
        // the passed hotkey.
        ensure!(Self::is_active(&hotkey), Error::<T>::NotActive);
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey);

        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::neuron_belongs_to_coldkey(&neuron, &coldkey), Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey.
        // Neurons must call an emit before they remove
        // stake or they may be able to cheat their peers of inflation.
        Self::emit_for_neuron( &neuron );

        // ---- We check that the hotkey has enough stake to withdraw
        // and then withdraw from the account.
        // let hotkey_stake: u64 = Stake::get( neuron.uid );
        ensure!(Self::has_enough_stake(&neuron, stake_to_be_removed), Error::<T>::NotEnoughStaketoWithdraw);

        let stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);
        ensure!(stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);

        // --- We perform the withdrawl by converting the stake to a u64 balance
        // and deposit the balance into the coldkey account. If the coldkey account
        // does not exist it is created.
        Self::add_stake_to_coldkey_account(&coldkey, &stake_to_be_added_as_currency);
        Self::remove_stake_from_neuron_hotkey_account(neuron.uid, stake_to_be_removed);
        Self::reduce_total_stake(stake_to_be_removed);

        // ---- Emit the unstaking event.
        Self::deposit_event(RawEvent::StakeRemoved(hotkey, stake_to_be_removed));
        debug::info!("--- Done remove_stake.");
        
        // --- Done and ok.
        Ok(())
    }




    /********************************
    --==[[  Helper functions   ]]==--
    *********************************/




    /**
    * Increases the amount of stake of the entire stake pool by the supplied amount
    */
    pub fn increase_total_stake(increment: u64) {
        // --- We update the total staking pool with the new funds.
        let total_stake: u64 = TotalStake::get();
        TotalStake::put(total_stake + increment);
        debug::info!("Added {:?} to total stake, now {:?}", increment, TotalStake::get());
    }

    /**
    * Reduces the amount of stake of the entire stake pool by the supplied amount
    */
    pub fn reduce_total_stake(decrement: u64) {
        // --- We update the total staking pool with the removed funds.
        let total_stake: u64 = TotalStake::get();
        TotalStake::put(total_stake - decrement);
        debug::info!("Remove {:?} from total stake, now {:?} ", decrement, TotalStake::get());
    }

    /**
    * Increases the amount of stake in a neuron's hotkey account by the amount provided
    * The uid parameter identifies the neuron holding the hotkey account
    */
    pub fn add_stake_to_neuron_hotkey_account(uid: u64, amount: u64) {
        let prev_stake: u64 = Stake::get(uid);
        let new_stake = prev_stake + amount;
        Stake::insert(uid, new_stake);
        debug::info!("Added new stake: | uid: {:?} | prev stake: {:?} | increment: {:?} | new stake: {:?}|", uid, prev_stake, amount, new_stake);
    }

    /**
    * Decreases the amount of stake in a neuron's hotkey account by the amount provided
    * The uid parameter identifies the neuron holding the hotkey account.
    * When using this function, it is important to also increase another account by the same value,
    * as otherwise value gets lost.
    */
    fn remove_stake_from_neuron_hotkey_account(uid: u64, amount: u64) {
        let hotkey_stake: u64 = Stake::get(uid);
        Stake::insert(uid, hotkey_stake - amount);
        debug::info!("Withdraw: {:?} from hotkey staking account for new ammount {:?} staked", amount, hotkey_stake - amount);
    }

    /**
    * This removes a completed entry from the stake map. The stake map is a map between hotkey account -> amount of stake
    * This function is used remove the hotkey account's stake entry from the map when unsubscribing
    * Care needs to be taken to transfer ALL stake to a different account, lest value gets lost
    */
    pub fn remove_all_stake_from_neuron_hotkey_account(uid: u64) {
        Stake::remove(uid);
    }

    pub fn add_stake_to_coldkey_account(coldkey: &T::AccountId, amount: &Option<<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance>) {
        //@todo (Parallax, 28-12-2020) implement error handling
        let _ = T::Currency::deposit_creating(&coldkey, amount.unwrap());
        debug::info!("Deposit {:?} into coldkey balance ", amount.unwrap());
    }

    fn remove_stake_from_coldkey_account(coldkey: &T::AccountId, amount: &Option<<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance>) {
        // @Todo (parallax, 28-12-2020) Do error handling on this call
        let _ = T::Currency::withdraw(&coldkey, amount.unwrap(), WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive);
        debug::info!("Withdrew {:?} from coldkey: {:?}", amount.unwrap(), coldkey);
    }

    fn neuron_belongs_to_coldkey(neuron : &NeuronMetadataOf<T>, coldkey : &T::AccountId) -> bool {
        return neuron.coldkey == *coldkey
    }

    fn coldkey_has_enough_balance(coldkey: &T::AccountId, amount: &Option<<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance>) -> bool {
        let new_potential_balance = T::Currency::free_balance(&coldkey) - amount.unwrap();
        let can_withdraw = T::Currency::ensure_can_withdraw(&coldkey, amount.unwrap(), WithdrawReasons::except(WithdrawReason::Tip), new_potential_balance).is_ok();
        can_withdraw
    }

    fn has_enough_stake(neuron : &NeuronMetadataOf<T>, amount : u64) -> bool{
        let hotkey_stake: u64 = Stake::get( neuron.uid );
        return hotkey_stake >= amount;
    }


    pub fn calculate_stake_fraction_for_neuron(neuron : &NeuronMetadataOf<T>) -> U64F64 {
        let total_stake = U64F64::from_num(TotalStake::get());
        let neuron_stake = U64F64::from_num(Stake::get(neuron.uid));

        debug::info!("total stake {:?}", total_stake);
        debug::info!("neuron stake (uid: {:?}) :  {:?}", neuron.uid, neuron_stake);

        // Total stake is 0, this should virtually never happen, but is still here because it could
        if total_stake == U64F64::from_num(0) {
            return U64F64::from_num(0);
        }

        // Neuron stake is zero. This means there will be nothing to emit
        if neuron_stake ==U64F64::from_num(0) {
            return U64F64::from_num(0);
        }

        let stake_fraction = neuron_stake / total_stake;

        debug::info!("Stake fraction for neuron (uid: {:?}) : {:?}", neuron.uid, stake_fraction);

        return stake_fraction;
    }
}

