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
        Self::emit_for_neuron( &neuron );

        // ---- We check that the calling coldkey contains enough funds to
        // create the staking transaction.
        let stake_as_balance = Self::u64_to_balance( stake_to_be_added );
        ensure!(stake_as_balance.is_some(), Error::<T>::CouldNotConvertToBalance);

        ensure!(Self::coldkey_has_enough_balance(&coldkey, stake_as_balance.unwrap()), Error::<T>::NotEnoughBalanceToStake);
        ensure!(Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true, Error::<T>::BalanceWithdrawalError);
        Self::add_stake_to_neuron_hotkey_account(neuron.uid, stake_to_be_added);

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
        ensure!(Self::is_hotkey_active(&hotkey), Error::<T>::NotActive);
        let neuron = Self::get_neuron_for_hotkey(&hotkey);

        // Check if uid is active
        ensure!(Self::is_uid_active(neuron.uid), Error::<T>::NotActive);


        // ---- We check that the NeuronMetadata is linked to the calling
        // cold key, otherwise throw a NonAssociatedColdKey error.
        ensure!(Self::neuron_belongs_to_coldkey(&neuron, &coldkey), Error::<T>::NonAssociatedColdKey);

        // --- We call the emit function for the associated hotkey.
        // Neurons must call an emit before they remove
        // stake or they may be able to cheat their peers of inflation.
        Self::emit_for_neuron( &neuron );

        // ---- We check that the hotkey has enough stake to withdraw
        // and then withdraw from the account.
        ensure!(Self::has_enough_stake(&neuron, stake_to_be_removed), Error::<T>::NotEnoughStaketoWithdraw);
        let stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);
        ensure!(stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);

        // --- We perform the withdrawl by converting the stake to a u64 balance
        // and deposit the balance into the coldkey account. If the coldkey account
        // does not exist it is created.
        Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());
        Self::remove_stake_from_neuron_hotkey_account(neuron.uid, stake_to_be_removed);

        // ---- Emit the unstaking event.
        Self::deposit_event(RawEvent::StakeRemoved(hotkey, stake_to_be_removed));
        debug::info!("--- Done remove_stake.");
        
        // --- Done and ok.
        Ok(())
    }




    /********************************
    --==[[  Helper functions   ]]==--
    *********************************/

    pub fn get_stake_of_neuron_hotkey_account_by_uid(uid : u64) -> u64 {
        return Stake::get(uid);
    }

    pub fn get_total_stake() -> u64 {
        return TotalStake::get();
    }

    /**
    * Increases the amount of stake of the entire stake pool by the supplied amount
    */
    pub fn increase_total_stake(increment: u64) {
        // --- We update the total staking pool with the new funds.
        let total_stake: u64 = TotalStake::get();

        // Sanity check
        assert!(increment <= u64::MAX - total_stake);

        TotalStake::put(total_stake + increment);
        debug::info!("Added {:?} to total stake, now {:?}", increment, TotalStake::get());
    }

    /**
    * Reduces the amount of stake of the entire stake pool by the supplied amount
    */
    pub fn decrease_total_stake(decrement: u64) {
        // --- We update the total staking pool with the removed funds.
        let total_stake: u64 = TotalStake::get();

        // Sanity check so that total stake does not underflow past 0
        assert!(decrement <= total_stake);

        TotalStake::put(total_stake - decrement);
        debug::info!("Remove {:?} from total stake, now {:?} ", decrement, TotalStake::get());
    }

    /**
    * Increases the amount of stake in a neuron's hotkey account by the amount provided
    * The uid parameter identifies the neuron holding the hotkey account
    *
    * Calling function should make sure the uid exists within the system
    */
    pub fn add_stake_to_neuron_hotkey_account(uid: u64, amount: u64) {
        assert!(Self::is_uid_active(uid));

        let prev_stake: u64 = Stake::get(uid);

        // This should never happen. If a user has this ridiculous amount of stake,
        // we need to come up with a better solution
        assert!(u64::MAX - amount > prev_stake);

        let new_stake = prev_stake + amount;

        Stake::insert(uid, new_stake);
        debug::info!("Added new stake: | uid: {:?} | prev stake: {:?} | increment: {:?} | new stake: {:?}|", uid, prev_stake, amount, new_stake);

        Self::increase_total_stake(amount);
    }

    /**
    * Decreases the amount of stake in a neuron's hotkey account by the amount provided
    * The uid parameter identifies the neuron holding the hotkey account.
    * When using this function, it is important to also increase another account by the same value,
    * as otherwise value gets lost.
    *
    * A check if there is enough stake in the hotkey account should have been performed
    * before this function is called. If not, the node will crap out.
    *
    * Furthermore, a check to see if the uid is active before this method is called is also required
    */
    pub fn remove_stake_from_neuron_hotkey_account(uid: u64, amount: u64) {
        assert!(Self::is_uid_active(uid));

        let hotkey_stake: u64 = Stake::get(uid);

        // By this point, there should be enough stake in the hotkey account for this to work.
        assert!(hotkey_stake >= amount);

        Stake::insert(uid, hotkey_stake - amount);
        debug::info!("Withdraw: {:?} from hotkey staking account for new ammount {:?} staked", amount, hotkey_stake - amount);

        Self::decrease_total_stake(amount);
    }

    /**
    * This adds stake (balance) to a cold key account. It takes the account id of the coldkey account and a Balance as parameters.
    * The Balance parameter is a from u64 converted number. This is needed for T::Currency to work.
    * Make sure stake is removed from another account before calling this method, otherwise you'll end up with double the value
    */
    pub fn add_balance_to_coldkey_account(coldkey: &T::AccountId, amount: <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance) {
        T::Currency::deposit_creating(&coldkey, amount); // Infallibe
        debug::info!("Deposited {:?} into coldkey balance ", amount);
    }

    /**
    * This removes stake from the hotkey. This should be used together with the function to store the stake
    * in the hot key account.
    * The internal mechanics can fail. When this happens, this function returns false, otherwise true
    * The output of this function MUST be checked before writing the amount to the hotkey account
    *
    */
    pub fn remove_balance_from_coldkey_account(coldkey: &T::AccountId, amount: <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance) -> bool {
        return match T::Currency::withdraw(&coldkey, amount, WithdrawReasons::except(WithdrawReason::Tip), ExistenceRequirement::KeepAlive) {
            Ok(_result) => {
                debug::info!("Withdrew {:?} from coldkey: {:?}", amount, coldkey);
                true
            },
            Err(error) => {
                debug::info!("Could NOT withdraw {:?} from coldkey: {:?} for reason {:?}", amount, coldkey, error);
                false
            }
        };
    }

    /**
    * Checks if the neuron as specified in the neuron parameter has subscribed with the cold key
    * as specified in the coldkey parameter. See fn subscribe() for more info.
    */
    fn neuron_belongs_to_coldkey(neuron : &NeuronMetadataOf<T>, coldkey : &T::AccountId) -> bool {
        return neuron.coldkey == *coldkey
    }

    /**
    * Checks if the coldkey account has enough balance to be able to withdraw the specified amount.
    */
    fn coldkey_has_enough_balance(coldkey: &T::AccountId, amount: <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // @todo (Parallax, 02-01-2021) // Split this function up in two
        let new_potential_balance =  current_balance - amount;
        let can_withdraw = T::Currency::ensure_can_withdraw(&coldkey, amount, WithdrawReasons::except(WithdrawReason::Tip), new_potential_balance).is_ok();
        can_withdraw
    }

    /**
    * Returns the current balance in the cold key account
    */
    pub fn get_coldkey_balance(coldkey: &T::AccountId) -> <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance {
        return T::Currency::free_balance(&coldkey);
    }

    /**
    * Checks if the hotkey account of the specified account has enough stake to be able to withdraw
    * the requested amount.
    */
    fn has_enough_stake(neuron : &NeuronMetadataOf<T>, amount : u64) -> bool {
        let hotkey_stake: u64 = Stake::get( neuron.uid );
        return hotkey_stake >= amount;
    }

    /**
    * Creates a hotkey account to which stake can be added.
    * This needs to be done on subsribed, as its presence is used for
    * uid validity checking
    */
    pub fn create_hotkey_account(uid : u64) {
        Stake::insert(uid, 0);
    }

    /**
    * Returns true if there is an entry for uid in the Stake map,
    * false otherwise
    */
    pub fn has_hotkey_account(uid : &u64) -> bool {
        return Stake::contains_key(*uid);
    }

    /**
    * This calculates the fraction of the total amount of stake the specfied neuron owns.
    * This function is part of the algorithm that calculates the emission of this neurons
    * to its peers. See fn calculate_emission_for_neuron()
    *
    * This function returns 0 if the total amount of stake is 0, or the amount of stake the
    * neuron has is 0.
    *
    * Otherwise, it returns the result of neuron_stake / total stake
    */
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

