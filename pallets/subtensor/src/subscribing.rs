use super::*;

impl<T: Trait> Module<T> {
    
    pub fn do_subscribe(origin: T::Origin, ip: u128, port: u16, ip_type: u8, modality: u8, coldkey: T::AccountId) -> dispatch::DispatchResult {

        // --- We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called subscribe with caller {:?}", hotkey_id);

        // --- We make validy checks on the passed data.
        ensure!(is_valid_ip_type(ip_type), Error::<T>::InvalidIpType);
        ensure!(is_valid_ip_address(ip_type, ip), Error::<T>::InvalidIpAddress);

        // --- We switch here between an update and a subscribe.
        if !Self::is_hotkey_active(&hotkey_id) {
            // --- We get the next available subscription uid.
            let uid = Self::get_next_uid();

            // -- We add this hotkey to the active set.
            Self::add_hotkey_to_active_set(&hotkey_id, uid);

            // ---- If the neuron is not-already subscribed, we create a 
            // new entry in the table with the new metadata.
            let neuron = Self::add_neuron_to_metagraph(ip, port, ip_type, modality, coldkey, hotkey_id.clone(), uid);

            // -- We initialize table values for this peer.
            Self::create_hotkey_account(neuron.uid);
            Self::update_last_emit_for_neuron(&neuron);
            Self::init_weight_matrix_for_neuron(&neuron);

            // --- We deposit the neuron added event.
            Self::deposit_event(RawEvent::NeuronAdded(uid)); 

        } else {
            // --- We get the uid associated with this hotkey account.
            let uid = Self::get_uid_for_hotkey(&hotkey_id);

            // --- If the neuron is already subscribed, we allow an update to their
            // modality and ip.
            let neuron = Self::update_neuron_in_metagraph(uid, ip, port, ip_type, modality);

            // --- We update their last emit
            Self::update_last_emit_for_neuron(&neuron);

            // --- We deposit the neuron updated event
            Self::deposit_event(RawEvent::NeuronUpdated(uid));
        }
        Ok(())
    }

    /********************************
     --==[[  Helper functions   ]]==--
    *********************************/

    pub fn add_neuron_to_metagraph(ip: u128, port: u16, ip_type: u8, modality: u8, coldkey: T::AccountId, hotkey: T::AccountId, uid: u64) -> NeuronMetadataOf<T> {
        // Before calling this function, a check should be made to see if
        // the account_id is already used. If this is omitted, this assert breaks.
        assert_eq!(Self::is_uid_active(uid), false);
        debug::info!("Insert new metadata with ip: {:?}, port: {:?}, ip_type: {:?}, uid: {:?}, modality: {:?}, hotkey {:?}, coldkey: {:?}", ip, port, ip_type, uid, modality, hotkey, coldkey);
        let metadata = NeuronMetadataOf::<T> {
            ip: ip,
            port: port,
            ip_type: ip_type,
            uid: uid,
            modality: modality,
            hotkey: hotkey,
            coldkey: coldkey,
        };
        Neurons::<T>::insert(uid, &metadata);
        return metadata;
    }

    pub fn update_neuron_in_metagraph(uid: u64, ip: u128, port: u16, ip_type: u8, modality: u8) -> NeuronMetadataOf<T> {
        // Before calling this function, a check should be made to see if
        // the account_id is already used. If this is omitted, this assert breaks.
        assert_eq!(Self::is_uid_active(uid), true);
        debug::info!("Updated neuron metadata ip: {:?}, port: {:?}, ip_type: {:?}, uid: {:?}, modality: {:?}", ip, port, ip_type, uid, modality);
        let old_metadata = Self::get_neuron_for_uid(uid);
        let new_metadata = NeuronMetadataOf::<T> {
            ip: ip,
            port: port,
            ip_type: ip_type,
            modality: modality,
            uid: old_metadata.uid,
            hotkey: old_metadata.hotkey,
            coldkey: old_metadata.coldkey,
        };

        Neurons::<T>::insert(uid, &new_metadata);
        return new_metadata;
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
