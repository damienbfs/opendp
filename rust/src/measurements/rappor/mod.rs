use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{BitVector, BitVectorDomain};
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;
use crate::traits::{samplers::sample_bernoulli_float, InfDiv, InfLn, InfMul, InfSub};

use bitvec::prelude::{bitvec, Lsb0};

/// Make a Measurement that implements Basic RAPPOR
///
/// # Citations
/// * [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal Response](https://arxiv.org/abs/1407.6981)
///
/// # Arguments
/// * `f` - Per-bit flipping probability. Must be in $(0, 1]$.
/// * `m` - number of ones set in each boolean vector (1 if one-hot encoding, more if using a bloom filter)
///
/// eps = 2mln((2-f)/f)
pub fn make_rappor(
    input_domain: BitVectorDomain,
    input_metric: DiscreteDistance,
    f: f64,
    constant_time: bool,
) -> Fallible<Measurement<BitVectorDomain, BitVector, DiscreteDistance, MaxDivergence<f64>>> {
    let m = match input_domain.max_weight {
        Some(m) => m,
        None => {
            return fallible!(
                MakeMeasurement,
                "RAPPOR requires a maximum number of set bits!"
            )
        }
    };

    if f <= 0.0 || f > 1.0 {
        return fallible!(MakeMeasurement, "f must be in (0, 1]");
    };

    // priv = 2mln((2-f)/f)
    let epsilon = (2.0f64)
        .inf_sub(&f)?
        .inf_div(&f)?
        .inf_ln()?
        .inf_mul(&2.0)?
        .inf_mul(&f64::from(m))?;
    let f_2 = f.inf_div(&2.0)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &BitVector| {
            let n = arg.len();
            let noise_vector = (1..n)
                .into_iter()
                .map(|_| sample_bernoulli_float(f_2, constant_time))
                .collect::<Fallible<BitVector>>()?;
            // I wanted to avoid cloning here but the closure makes it necessary
            // Shouldn't use much memory anyway given bit-vecs
            Ok(arg.clone() ^ noise_vector) // xor on bit vectors
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |&d_in: &u32| {
            if d_in == 0 {
                return Ok(0.0);
            }
            if d_in > 1 {
                return fallible!(FailedFunction, "d_in must be 0 or 1.");
            }
            Ok(epsilon)
        }),
    )
}

pub fn debias_basic_rappor(answers: Vec<BitVector>, f: f64) -> Fallible<Vec<f64>> {
    if answers.len() == 0 {
        return fallible!(FailedFunction, "No answers provided");
    }
    if f <= 0.0 || f > 1.0 {
        return fallible!(FailedFunction, "f must be in (0, 1]");
    }

    let n = answers.len() as f64;
    let k = answers[0].len();
    let mut counts = vec![0.0; k];

    if answers.iter().any(|a| a.len() != k) {
        return fallible!(FailedFunction, "Answers have inconsistent lengths");
    }

    answers.into_iter().for_each(|answer| {
        counts.iter_mut().zip(answer).for_each(|(c, a)| {
            if a {
                *c += 1.0;
            }
        });
    });

    Ok(counts
        .into_iter()
        .map(|y_i| (y_i - ((f / 2.0) * n)) / (1.0 - f))
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_rappor() -> Fallible<()> {
        let rappor = make_rappor(
            BitVectorDomain::new().with_max_weight(1),
            DiscreteDistance::default(),
            0.5,
            false,
        )?;
        rappor.invoke(&bitvec![usize, Lsb0;
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ])?;
        assert_eq!(rappor.map(&1)?, 2.1972245773362196);
        Ok(())
    }
    #[test]
    fn test_debias_rappor() -> Fallible<()> {
        let f = 0.1;
        let mut answer = vec![0.0; 10];
        answer[0] = 1.0;

        let answers = vec![
            bitvec![usize, Lsb0; 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; 
            10
        ];

        let high = 10.555555555555555;
        let low = -0.5555555555555556;

        let expected_dist = vec![high, low, low, low, low, low, low, low, low, low];
        assert_eq!(debias_basic_rappor(answers, f)?, expected_dist);

        Ok(())
    }
}
