use rand::distributions::{Distribution, Uniform};
use rand_pcg::Pcg64Mcg;
use serde::{Deserialize, Serialize};

/// The random number generator used in simulations is a permuted
/// congruential generator with 128-bit state, internal multiplicative
/// congruential generator, and 64-bit output via "xorshift low (bits),
/// random rotation" output function.  This random number generator is
/// seeded and portable across platforms (e.g., WASM compilation targets).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformRNG {
    rng: Pcg64Mcg,
}

impl Default for UniformRNG {
    fn default() -> Self {
        UniformRNG {
            rng: Pcg64Mcg::new(42),
        }
    }
}

impl UniformRNG {
    pub fn rn(&mut self) -> f64 {
        // Random number in [0.0, 1.0)
        Uniform::new(0.0_f64, 1.0_f64).sample(&mut self.rng)
    }

    pub fn rng(&mut self) -> &mut Pcg64Mcg {
        &mut self.rng
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_chi_square_uniformity_test() {
        let mut uniform_rng = UniformRNG::default();
        // 100 "classes" (bins)
        let mut class_counts: [f64; 100] = [0.0; 100];
        (0..100000).for_each(|_| {
            let rn = uniform_rng.rn();
            class_counts[(rn * 100.0) as usize] += 1.0;
        });
        let expected_count = 1000.0; // 100000 points, 100 classes, 10000 points per class
        let chi_square = class_counts.iter().fold(0.0, |acc, class_count| {
            acc + (*class_count - expected_count).powi(2) / expected_count
        });
        // At a significance level of 0.01, and with n-1=99 degrees of freedom, the chi square critical
        // value for this scenario is 134.642
        let chi_square_critical = 134.642;
        assert![chi_square < chi_square_critical];
    }
}
