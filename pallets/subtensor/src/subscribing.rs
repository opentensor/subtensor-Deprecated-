use super::*;

impl<T: Trait> Module<T> {
    pub fn do_subscribe(origin: T::Origin, ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {

        // --- We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called subscribe with caller {:?}", hotkey_id);

        // --- We check to see if the Neuron already exists.
        // We do not allow peers to re-subscribe with the same key.
        ensure!( Self::is_not_active(&hotkey_id), Error::<T>::AlreadyActive );

        // --- We get the next available subscription uid.
        // uids increment by one up u64:MAX, this allows the chain to 
        // have over 18,446,744,073,709,551,615 peers before and overflow
        // one per ipv6 address without an memory overflow. 
        let uid = Self::get_next_uid();

        // ---- If the neuron is not-already subscribed, we create a 
        // new entry in the table with the new metadata.
        let neuron = Self::add_neuron_to_block_chain(ip, port, ip_type, coldkey, &hotkey_id, uid);

        // ---- We provide the subscriber with and initial subscription gift.
        // NOTE: THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
        Self::add_subscription_gift(&neuron, 1000000000);
        Self::init_weight_matrix_for_neuron(&neuron);

        // ---- We increment the active count for the additional member.
        Self::increase_neuron_count();

        // --- We deposit the neuron added event.
        Self::deposit_event(RawEvent::NeuronAdded(hotkey_id));
        debug::info!("--- Done subscribe");

        Ok(())
    }



    pub fn do_unsubscribe(origin: T::Origin) -> dispatch::DispatchResult
    {
        // --- We check the signature of the calling account.
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called unsubscribe with caller: {:?}", hotkey_id);

        // --- We check that the Neuron already exists in the active set.
        ensure!(Self::is_active(&hotkey_id), Error::<T>::NotActive);
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey_id);

        // --- We call the emit function. Neurons must call an emit before
        // they leave the incentive mechanim or else they can cheat their peers
        // of promised inflation.
        Self::emit_for_neuron(&neuron);

        // --- If there are funds staked, we unstake them and add them to the coldkey.
        let amount_staked: u64 = Stake::get(neuron.uid);
        debug::info!("Ammount staked on this account is: {:?}", amount_staked);

        if amount_staked > 0 {
            // --- We perform the withdrawl by converting the stake to a u64 balance
            // and deposit the balance into the coldkey account. If the coldkey account
            // does not exist it is created.
            let stake_to_be_added_as_currency = Self::u64_to_balance(amount_staked);
            ensure!(stake_to_be_added_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);
            Self::add_stake_to_coldkey_account(&neuron.coldkey, &stake_to_be_added_as_currency);

            // --- We update the total staking pool with the removed funds.
            Self::reduce_total_stake(amount_staked);
        }

        // --- We remove the neuron-info from the various maps.
        Self::remove_all_stake_from_neuron_hotkey_account(&neuron);
        Self::remove_last_emit_info_for_neuron(&neuron);
        Self::remove_weight_matrix_for_neuron(&neuron);
        Self::remove_neuron_metadata(&hotkey_id);
        Self::decrease_neuron_count();
        debug::info!("Hotkey account removed: {:?}", hotkey_id);

        // --- We emit the neuron removed event and return ok.
        Self::deposit_event(RawEvent::NeuronRemoved(hotkey_id));
        debug::info!("--- Done unsubscribe.");

        Ok(())
    }



    fn remove_neuron_metadata(hotkey_id: &T::AccountId) {
        Neurons::<T>::remove(&hotkey_id);
    }


    /********************************
     --==[[  Helper functions   ]]==--
    *********************************/


    fn increase_neuron_count() {
        let neuron_count = ActiveCount::get();
        ActiveCount::put(neuron_count + 1);
        debug::info!("Increment the neuron count to: {:?} ", ActiveCount::get());
    }

    fn decrease_neuron_count() {
        // --- We decrement the neuron counter.
        let neuron_count = ActiveCount::get();
        ActiveCount::put(neuron_count - 1);
        debug::info!("New neuron count: {:?}", ActiveCount::get());
    }



    fn init_weight_matrix_for_neuron(neuron: &NeuronMetadataOf<T>) {
        // ---- We fill subscribing nodes initially with the self-weight = [1]
        let weights = vec![u32::max_value()]; // w_ii = 1
        let uids = vec![neuron.uid]; // Self edgr

        Self::set_new_weights(neuron, &uids, &weights);
    }

    fn add_subscription_gift(neuron: &NeuronMetadataOf<T>, amount: u64) {
        debug::info!("Adding subscription gift to the stake {:?} ", amount);

        Self::add_stake_to_neuron_hotkey_account(neuron, amount);
        Self::increase_total_stake(amount);
        Self::update_last_emit_for_neuron(&neuron);
    }

    fn add_neuron_to_block_chain(ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId, hotkey_id: &T::AccountId, uid: u64) -> NeuronMetadataOf<T> {
        debug::info!("Insert new metadata with ip: {:?}, port: {:?}, ip_type: {:?}, uid: {:?}, coldkey: {:?}", ip, port, ip_type, uid, coldkey);

        let metadata = NeuronMetadataOf::<T> {
                                 ip: ip,
                                 port: port,
                                 ip_type: ip_type,
                                 uid: uid,
                                 coldkey: coldkey,
                             };

        Neurons::<T>::insert(&hotkey_id, &metadata);

        return metadata;
    }

    fn get_next_uid() -> u64 {
        let uid: u64 = NextUID::get();
        NextUID::put(uid + 1);
        debug::info!("Incrementing the next uid by 1, now {:?} ", NextUID::get());
        uid
    }
}