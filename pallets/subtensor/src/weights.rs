use super::*;


impl<T: Trait> Module<T> {
    pub fn do_set_weights(origin: T::Origin, uids: Vec<u64>, values: Vec<u32>) -> dispatch::DispatchResult
    {
        // ---- We check the caller signature
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called set_weights key {:?}, with uids: {:?} and weights: {:?}", hotkey_id, uids, values);

        // ---- We check to see that the calling neuron is in the active set.
        ensure!(Self::is_hotkey_active(&hotkey_id), Error::<T>::NotActive);
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey_id);
        debug::info!("Got metadata with uid {:?}", neuron.uid);

        // --- We check that the length of these two lists are equal.
        debug::info!("uids.len= {:?}, dests.len= {:?}", uids.len(), values.len());
        ensure!(uids_match_values(&uids, &values), Error::<T>::WeightVecNotEqualSize);

        // --- We check if the uids vector does not contain duplicate ids
        ensure!(!has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

        // --- We check if the weight uids are valid
        ensure!(!Self::contains_invalid_uids(&uids), Error::<T>::InvalidUid);


        // ---- We call an inflation emit before setting the weights
        // to ensure that the caller is pays for his previously set weights.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_for_neuron(&neuron);
        debug::info!("finished emit");

        let normalized_values = normalize(values);
        debug::info!("normalized values {:?}", normalized_values);

        // --- We update the weights under the uid map.
        Self::set_new_weights(&neuron, &uids, &normalized_values);
        debug::info!("values set.");

        // ---- Emit the staking event.
        Self::deposit_event(RawEvent::WeightsSet(hotkey_id));

        // --- Emit the event and return ok.
        Ok(())
    }

     /********************************
     --==[[  Helper functions   ]]==--
    *********************************/

    /**
    * Sets the actual weights. This function takes two parameters: uids, values
    * that contain the weight for each uid.
    * This function assumes both vectors are of the same size, and is agnostic if the specifed
    * uid's exist or not.
    */
    pub fn set_new_weights(neuron: &NeuronMetadataOf<T>, uids: &Vec<u64>, values: &Vec<u32>) {
        WeightVals::insert(neuron.uid, &values);
        WeightUids::insert(neuron.uid, &uids);
    }


    pub fn remove_weight_matrix_for_neuron(neuron: &NeuronMetadataOf<T>) {
        WeightVals::remove(neuron.uid);
        WeightUids::remove(neuron.uid);
    }

    pub fn get_weights_for_neuron(neuron : &NeuronMetadataOf<T>) -> (Vec<u64>, Vec<u32>) {
        (WeightUids::get(neuron.uid), WeightVals::get(neuron.uid))
    }

    pub fn contains_invalid_uids(uids : &Vec<u64>) -> bool {
        for uid in uids {
            if !Self::is_uid_active(uid) {
                return true;
            }
        }

        return false;
    }
}

fn uids_match_values(uids: &Vec<u64>, values: &Vec<u32>) -> bool {
    return uids.len() == values.len();
}

/**
* This function tests if the uids half of the weight matrix contains duplicate uid's.
* If it does, an attacker could
*/
fn has_duplicate_uids(items: &Vec<u64>) -> bool {
    let mut parsed : Vec<u64> = Vec::new();
    for item in items {
        if parsed.contains(&item) { return true }
        parsed.push(item.clone());
    }

    return false;
}


fn normalize(mut weights: Vec<u32>) -> Vec<u32> {
    let sum: u64 = weights.iter().map(|x| *x as u64).sum();

    if sum == 0 {
        return weights;
    }

    weights.iter_mut().for_each(|x| {
        *x = (*x as u64 * u32::max_value() as u64 / sum) as u32;
    });

    return weights;
}


#[cfg(test)]
mod tests {
    use crate::weights::{normalize, has_duplicate_uids};

    #[test]
    fn normalize_sum_smaller_than_one() {
        let values: Vec<u32> = vec![u32::max_value() / 10, u32::max_value() / 10];
        assert_eq!(normalize(values), vec![u32::max_value() / 2, u32::max_value() / 2]);
    }

    #[test]
    fn normalize_sum_greater_than_one() {
        let values: Vec<u32> = vec![u32::max_value() / 7, u32::max_value() / 7];
        assert_eq!(normalize(values), vec![u32::max_value() / 2, u32::max_value() / 2]);
    }

    #[test]
    fn normalize_sum_zero() {
        let weights: Vec<u32> = vec![0, 0];
        assert_eq!(normalize(weights), vec![0, 0]);
    }

    #[test]
    fn normalize_values_maxed() {
        let weights: Vec<u32> = vec![u32::max_value(), u32::max_value()];
        assert_eq!(normalize(weights), vec![u32::max_value() / 2, u32::max_value() / 2]);
    }

    #[test]
    fn has_duplicate_elements_true() {
        let weights = vec![1,2,3,4,4,4,4];
        assert_eq!(has_duplicate_uids(&weights), true);
    }

    #[test]
    fn has_duplicate_elements_false() {
        let weights = vec![1,2,3,4,5];
        assert_eq!(has_duplicate_uids(&weights), false);
    }
}
