use super::*;

impl<T: Trait> Module<T> {
    pub fn do_set_weights(origin: T::Origin, uids: Vec<u64>, values: Vec<u32>) -> dispatch::DispatchResult
    {
        // ---- We check the caller signature
        let hotkey_id = ensure_signed(origin)?;
        debug::info!("--- Called set_weights key {:?}, with uids: {:?} and weights: {:?}", hotkey_id, uids, values);

        // ---- We check to see that the calling neuron is in the active set.
        ensure!(Self::is_active(&hotkey_id), Error::<T>::NotActive);
        let neuron = Self::get_neuron_metadata_for_hotkey(&hotkey_id);
        debug::info!("Got metadata with uid {:?}", neuron.uid);

        // --- We check that the length of these two lists are equal.
        debug::info!("uids.len= {:?}, dests.len= {:?}", uids.len(), values.len());
        ensure!(uids_match_values(&uids, &values), Error::<T>::WeightVecNotEqualSize);

        // ---- We call an inflation emit before setting the weights
        // to ensure that the caller is pays for his previously set weights.
        // TODO(const): can we pay for this transaction through inflation.
        Self::emit_from_uid(neuron.uid);
        debug::info!("finished emit");

        let normalized_values = normalize(values);
        debug::info!("normalized values {:?}", normalized_values);

        // --- We update the weights under the uid map.
        Self::set_new_weights(neuron, &uids, &normalized_values);
        debug::info!("values set.");

        // ---- Emit the staking event.
        Self::deposit_event(RawEvent::WeightsSet(hotkey_id));

        // --- Emit the event and return ok.
        Ok(())
    }


    fn set_new_weights(neuron: NeuronMetadataOf<T>, uids: &Vec<u64>, values: &Vec<u32>) {
        WeightVals::insert(neuron.uid, &values);
        WeightUids::insert(neuron.uid, &uids);
    }
}

fn uids_match_values(uids: &Vec<u64>, values: &Vec<u32>) -> bool {
    return uids.len() == values.len();
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
    use crate::weights::normalize;

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
}
