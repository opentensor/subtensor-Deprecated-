use super::*;

impl<T: Trait> Module<T> {
    pub fn do_subscribe(origin : T::Origin, ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {
        
        // --- We check the callers (hotkey) signature.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called subscribe with caller {:?}", caller);

        // --- We check to see if the Neuron already exists.
        // We do not allow peers to re-subscribe with the same key.
        ensure!( !Neurons::<T>::contains_key(&caller), Error::<T>::AlreadyActive );

        // --- We get the next available subscription uid.
        // uids increment by one up u64:MAX, this allows the chain to 
        // have over 18,446,744,073,709,551,615 peers before and overflow
        // one per ipv6 address without an memory overflow. 
        let uid: u64 = NextUID::get();
        NextUID::put(uid + 1);
        debug::info!("Incrementing the next uid by 1, now {:?} ", NextUID::get());

        // ---- If the neuron is not-already subscribed, we create a 
        // new entry in the table with the new metadata.
        debug::info!("Insert new metadata with ip: {:?}, port: {:?}, ip_type: {:?}, uid: {:?}, coldkey: {:?}", ip, port, ip_type, uid, coldkey);
        Neurons::<T>::insert( &caller,
            NeuronMetadataOf::<T> {
                ip: ip,
                port: port,
                ip_type: ip_type,
                uid: uid,
                coldkey: coldkey,
            }
        );

        // ---- We provide the subscriber with and initial subscription gift.
        // NOTE: THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
        let subscription_gift: u64 = 1000000000;
        debug::info!("Adding subscription gift to the stake {:?} ", subscription_gift);

        // --- We update the total staking pool with the subscription.
        let total_stake: u64 = TotalStake::get();
        TotalStake::put(total_stake + subscription_gift);
        debug::info!("Adding amount: {:?} to total stake, now: {:?}", subscription_gift, TotalStake::get());

        // The last emit determines the last time this peer made an incentive 
        // mechanism emit call. Since he is just subscribed with zero stake,
        // this moment is considered his first emit.
        let current_block: T::BlockNumber = system::Module::<T>::block_number();
        debug::info!("The new last emit for this caller is: {:?} ", current_block);

        // ---- We initilize the neuron maps with nill weights, 
        // the subscription gift and the current block as last emit.
        LastEmit::<T>::insert(uid, current_block);
        Stake::insert(uid, subscription_gift);

        // ---- We fill subscribing nodes initially with the self-weight = [1]
        let mut _weights = Vec::new();
        let mut _uids = Vec::new();
        _weights.push(u32::max_value()); // w_ii = 1
        _uids.push(uid); // Self edge
        WeightVals::insert(uid, _weights);
        WeightUids::insert(uid, _uids);

        // ---- We increment the active count for the additional member.
        let neuron_count = ActiveCount::get();
        ActiveCount::put(neuron_count + 1);
        debug::info!("Increment the neuron count to: {:?} ", ActiveCount::get());

        // --- We deposit the neuron added event.
        Self::deposit_event(RawEvent::NeuronAdded(caller));
        debug::info!("--- Done subscribe");

        Ok(())
    }

    pub fn do_unsubscribe(origin : T::Origin) -> dispatch::DispatchResult
    {
        // --- We check the signature of the calling account.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called unsubscribe with caller: {:?}", caller);

        // --- We check that the Neuron already exists in the active set.
        ensure!(Neurons::<T>::contains_key( &caller ), Error::<T>::NotActive);
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get( &caller );
        debug::info!("Metadata retrieved with coldkey: {:?}", neuron.coldkey);

        // --- We call the emit function. Neurons must call an emit before
        // they leave the incentive mechanim or else they can cheat their peers
        // of promised inflation.
        Self::emit_from_uid( neuron.uid );

        // --- If there are funds staked, we unstake them and add them to the coldkey.
        let ammount_unstaked: u64 = Stake::get( neuron.uid );
        debug::info!("Ammount staked on this account is: {:?}", ammount_unstaked);

        if ammount_unstaked > 0 {
            // --- We perform the withdrawl by converting the stake to a u64 balance
            // and deposit the balance into the coldkey account. If the coldkey account
            // does not exist it is created.
            let ammount_unstaked_as_currency = Self::u64_to_balance( ammount_unstaked );
            ensure!(ammount_unstaked_as_currency.is_some(), Error::<T>::CouldNotConvertToBalance);
            let ammount_unstaked_as_currency = ammount_unstaked_as_currency.unwrap();
            T::Currency::deposit_creating( &neuron.coldkey, ammount_unstaked_as_currency);
            debug::info!("Depositing: {:?} into coldkey account: {:?}", ammount_unstaked, neuron.coldkey);


            // --- We update the total staking pool with the removed funds.
            let total_stake: u64 = TotalStake::get();
            TotalStake::put(total_stake - ammount_unstaked);
            debug::info!("Removing amount: {:?} from total stake, now: {:?}", ammount_unstaked, TotalStake::get());
        }

        // --- We remove the neuron-info from the various maps.
        Neurons::<T>::remove( &caller );
        Stake::remove( neuron.uid );
        LastEmit::<T>::remove( neuron.uid );
        WeightVals::remove( neuron.uid );
        WeightUids::remove( neuron.uid );
        debug::info!("Hotkey account removed: {:?}", caller);

        // --- We decrement the neuron counter.
        let neuron_count = ActiveCount::get();
        ActiveCount::put(neuron_count - 1);
        debug::info!("New neuron count: {:?}", ActiveCount::get());

        // --- We emit the neuron removed event and return ok.
        Self::deposit_event(RawEvent::NeuronRemoved(caller));
        debug::info!("--- Done unsubscribe.");

        Ok(())
		
    }
}