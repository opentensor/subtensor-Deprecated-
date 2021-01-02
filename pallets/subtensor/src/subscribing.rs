use super::*;

impl<T: Trait> Module<T> {
    pub fn do_subscribe(origin: T::Origin, ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {

        // --- We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called subscribe with caller {:?}", hotkey_id);

        // --- We check to see if the Neuron already exists.
        // We do not allow peers to re-subscribe with the same key.
        ensure!( Self::is_not_active(&hotkey_id), Error::<T>::AlreadyActive );
        ensure!(is_valid_ip_type(ip_type), Error::<T>::InvalidIpType);
        ensure!(is_valid_ip_address(ip_type, ip), Error::<T>::InvalidIpAddress);

        // --- We get the next available subscription uid.
        // uids increment by one up u64:MAX, this allows the chain to 
        // have over 18,446,744,073,709,551,615 peers before and overflow
        // one per ipv6 address without an memory overflow. 
        let uid = Self::get_next_uid();

        // ---- If the neuron is not-already subscribed, we create a 
        // new entry in the table with the new metadata.
        let neuron = Self::add_neuron_to_metagraph(ip, port, ip_type, coldkey, &hotkey_id, uid);

        // ---- We provide the subscriber with and initial subscription gift.
        // NOTE: THIS IS FOR TESTING, NEEDS TO BE REMOVED FROM PRODUCTION
        // Self::add_subscription_gift(&neuron, 1000000000);

        // Create hotkey account where the neuron can receive stake
        Self::create_hotkey_account(neuron.uid);
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
        ensure!(Self::is_hotkey_active(&hotkey_id), Error::<T>::NotActive);
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
            Self::add_stake_to_coldkey_account(&neuron.coldkey, stake_to_be_added_as_currency.unwrap());
        }

        // --- We remove the neuron-info from the various maps.
        Self::remove_all_stake_from_neuron_hotkey_account(neuron.uid);
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


    pub fn increase_neuron_count() {
        let neuron_count = ActiveCount::get();
        assert!(!neuron_count < u64::MAX); // We can't increase beyon 2^64. If ever. But still
        ActiveCount::put(neuron_count + 1);
        debug::info!("Increment the neuron count to: {:?} ", ActiveCount::get());
    }

    pub fn decrease_neuron_count() {
        // --- We decrement the neuron counter.
        let neuron_count = ActiveCount::get();
        assert!(neuron_count > 0); // We can't reduce beyond zero. This would be a serious problem
        ActiveCount::put(neuron_count - 1);
        debug::info!("New neuron count: {:?}", ActiveCount::get());
    }

    pub fn get_neuron_count() -> u64 {
        return ActiveCount::get();
    }

    pub fn set_neuron_count(count : u64) {
        ActiveCount::set(count);
    }



    fn init_weight_matrix_for_neuron(neuron: &NeuronMetadataOf<T>) {
        // ---- We fill subscribing nodes initially with the self-weight = [1]
        let weights = vec![u32::max_value()]; // w_ii = 1
        let uids = vec![neuron.uid]; // Self edgr

        Self::set_new_weights(neuron, &uids, &weights);
    }

    // fn add_subscription_gift(neuron: &NeuronMetadataOf<T>, amount: u64) {
    //     debug::info!("Adding subscription gift to the stake {:?} ", amount);
    //
    //     Self::add_stake_to_neuron_hotkey_account(neuron.uid, amount);
    //     Self::update_last_emit_for_neuron(&neuron);
    // }

    fn add_neuron_to_metagraph(ip: u128, port: u16, ip_type: u8, coldkey: T::AccountId, hotkey_id: &T::AccountId, uid: u64) -> NeuronMetadataOf<T> {
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

fn is_valid_ip_type(ip_type : u8) -> bool {
    let allowed_values: Vec<u8> = vec![4,6];
    return allowed_values.contains(&ip_type);
}


// @todo (Parallax 2-1-2021) : Implement exclusion of private IP ranges
fn is_valid_ip_address(ip_type : u8, addr : u128) -> bool {
    if !is_valid_ip_type(ip_type) {
        return false;
    }

    if addr == 0 {
        return false;
    }

    if ip_type == 4 {
        if addr == 0 { return false; }
        if addr >= u32::MAX as u128 { return false; }
        if addr == 0x7f000001 { return false; } // Localhost
    }

    if ip_type == 6 {
        if addr == 0x0 {return false }
        if addr == u128::MAX { return false; }
        if addr == 1 { return false; } // IPv6 localhost
    }

    return true;
}

#[cfg(test)]
mod test {
    use crate::subscribing::{is_valid_ip_type, is_valid_ip_address};
    use std::net::{Ipv6Addr, Ipv4Addr};


    // Generates an ipv6 address based on 8 ipv6 words and returns it as u128
    pub fn ipv6(a: u16, b : u16, c : u16, d : u16, e : u16 ,f: u16, g: u16,h :u16) -> u128 {
        return Ipv6Addr::new(a,b,c,d,e,f,g,h).into();
    }

    // Generate an ipv4 address based on 4 bytes and returns the corresponding u128, so it can be fed
    // to the module::subscribe() function
    pub fn ipv4(a: u8 ,b: u8,c : u8,d : u8) -> u128 {
        let ipv4 : Ipv4Addr =  Ipv4Addr::new(a, b, c, d);
        let integer : u32 = ipv4.into();
        return u128::from(integer);
    }

    #[test]
    fn test_is_valid_ip_type_ok_ipv4() {
        assert_eq!(is_valid_ip_type(4), true);
    }

    #[test]
    fn test_is_valid_ip_type_ok_ipv6() {
        assert_eq!(is_valid_ip_type(6), true);
    }

    #[test]
    fn test_is_valid_ip_type_nok() {
        assert_eq!(is_valid_ip_type(10), false);
    }
    
    #[test]
    fn test_is_valid_ip_address_ipv4() {
        assert_eq!(is_valid_ip_address(4,ipv4(8,8,8,8)), true);
    }

    #[test]
    fn test_is_valid_ip_address_ipv6() {
        assert_eq!(is_valid_ip_address(6,ipv6(1,2,3,4,5,6,7,8)), true);
        assert_eq!(is_valid_ip_address(6,ipv6(1,2,3,4,5,6,7,8)), true);
    }

    #[test]
    fn test_is_invalid_ipv4_address() {
        assert_eq!(is_valid_ip_address(4, ipv4(0,0,0,0)), false);
        assert_eq!(is_valid_ip_address(4, ipv4(255,255,255,255)), false);
        assert_eq!(is_valid_ip_address(4, ipv4(127,0,0,1)), false);
        assert_eq!(is_valid_ip_address(4,ipv6(0xffff,2,3,4,5,6,7,8)), false);
    }

    #[test]
    fn test_is_invalid_ipv6_addres() {
        assert_eq!(is_valid_ip_address(6, ipv6(0,0,0,0,0,0,0,0)), false);
        assert_eq!(is_valid_ip_address(4, ipv6(0xffff,0xffff,0xffff,0xffff,0xffff,0xffff,0xffff,0xffff)), false);
    }



}
