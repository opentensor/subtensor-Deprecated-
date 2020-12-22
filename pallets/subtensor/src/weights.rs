use super::*;

impl<T: Trait> Module<T> {
    pub fn do_set_weights(origin: T::Origin, keys: Vec<T::AccountId>, values: Vec<u32>) -> dispatch::DispatchResult
    {
        let neuron = ensure_signed(origin)?;
        ensure!(values.len() == keys.len(), Error::<T>::WeightVecNotEqualSize);

        let normalized_values = normalize(values);

        // Put the weights from this neurons to its peers onto the chain
        WeightKeys::<T>::insert(&neuron, &keys);
        WeightVals::<T>::insert(&neuron, &normalized_values);

        Self::deposit_event(RawEvent::WeightsSet(neuron));
        Ok(())
    }
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
