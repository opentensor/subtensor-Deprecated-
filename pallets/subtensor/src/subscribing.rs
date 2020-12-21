use super::*;

impl<T: Trait> Module<T> {
    pub fn do_subscribe(origin : T::Origin, ip: u128, port: u16, ip_type:u8, coldkey : T::AccountId) -> dispatch::DispatchResult
    {
        // --- We check the callers (hotkey) signature.
        let caller = ensure_signed(origin)?;
        debug::info!("--- Called subscribe with caller {:?}", caller);

        // ---- We check to see if the Neuron already exists.
        // We do not allow peers to re-subscribe with the same key.
        ensure!( !Neurons::<T>::contains_key(&caller), Error::<T>::AlreadyActive );

        // ---- If the neuron is not-already subscribed, we create a
        // new entry in the table with the new metadata.
        debug::info!("Insert new metadata with ip: {:?}, port: {:?}, ip_type: {:?}, coldkey: {:?}", ip, port, ip_type, coldkey);
        Neurons::<T>::insert( &caller,
            NeuronMetadataOf::<T> {
                ip: ip,
                port: port,
                ip_type: ip_type,
                coldkey: coldkey,
            }
        );

        // ---- We provide the subscriber with and initial subscription gift.
        // NOTE: THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
        let subscription_gift: u32 = 1000;
        debug::info!("Adding subscription gift to the stake {:?} ", subscription_gift);

        // --- We update the total staking pool with the subscription.
        let total_stake: u32 = TotalStake::get();
        TotalStake::put(total_stake + subscription_gift);
        debug::info!("Adding amount: {:?} to total stake, now: {:?}", subscription_gift, TotalStake::get());

        // The last emit determines the last time this peer made an incentive
        // mechanism emit call. Since he is just subscribed with zero stake,
        // this moment is considered his first emit.
        let current_block: T::BlockNumber = system::Module::<T>::block_number();
        debug::info!("The new last emit for this caller is: {:?} ", current_block);

        // ---- We initilize the neuron maps with nill weights,
        // the subscription gift and the current block as last emit.
        Stake::<T>::insert(&caller, subscription_gift);
        LastEmit::<T>::insert(&caller, current_block);
        WeightVals::<T>::insert(&caller, &Vec::new());
        WeightKeys::<T>::insert(&caller, &Vec::new());

        // ---- We increment the neuron count for the additional member.
        let neuron_count = NeuronCount::get();
        NeuronCount::put(neuron_count + 1);
        debug::info!("Increment the neuron count to: {:?} ", NeuronCount::get());

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
        ensure!(Neurons::<T>::contains_key(&caller), Error::<T>::NotActive);
        let neuron: NeuronMetadataOf<T> = Neurons::<T>::get(&caller);
        debug::info!("Metadata retrieved with coldkey: {:?}", neuron.coldkey);

        // --- We call the emit function. Neurons must call an emit before
        // they leave the incentive mechanim or else they can cheat their peers
        // of promised inflation.
        Self::emit( &caller );

        // --- If there are funds staked, we unstake them and add them to the coldkey.
        let ammount_unstaked: u32 = Stake::<T>::get( &caller );
        debug::info!("Ammount staked on this account is: {:?}", ammount_unstaked);

        if ammount_unstaked > 0 {
            // --- We perform the withdrawl by converting the stake to a u32 balance
            // and deposit the balance into the coldkey account. If the coldkey account
            // does not exist it is created.
            // TODO(const): change to u32
            T::Currency::deposit_creating( &neuron.coldkey, Self::u32_to_balance( ammount_unstaked ) );
            debug::info!("Depositing: {:?} into coldkey account: {:?}", ammount_unstaked, neuron.coldkey);


            // --- We update the total staking pool with the removed funds.
            let total_stake: u32 = TotalStake::get();
            TotalStake::put(total_stake - ammount_unstaked);
            debug::info!("Removing amount: {:?} from total stake, now: {:?}", ammount_unstaked, TotalStake::get());
        }

        // --- We remove the neuron info from the various maps.
        Stake::<T>::remove( &caller );
        Neurons::<T>::remove( &caller );
        LastEmit::<T>::remove( &caller );
        WeightVals::<T>::remove( &caller );
        WeightKeys::<T>::remove( &caller );
        debug::info!("Hotkey account removed: {:?}", caller);

        // --- We decrement the neuron counter.
        let neuron_count = NeuronCount::get();
        NeuronCount::put(neuron_count - 1);
        debug::info!("New neuron count: {:?}", NeuronCount::get());

        // --- We emit the neuron removed event and return ok.
        Self::deposit_event(RawEvent::NeuronRemoved(caller));
        debug::info!("--- Done unsubscribe.");

        Ok(())
    }
}