//! Random variables underpin both stochastic and deterministic model
//! behaviors, in that deterministic operation is simply a random variable
//! with a single value of probability 1.  Common distributions, with their
//! common parameterizations, are wrapped in enums `Continuous`, `Boolean`,
//! `Discrete`, and `Index`.

use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};
// Continuous distributions
use rand_distr::{Beta, Exp, Gamma, LogNormal, Normal, Triangular, Uniform, Weibull};
// Discrete distributions
use rand_distr::{Bernoulli, Geometric, Poisson, WeightedIndex};

use super::UniformRNG;
use crate::utils::errors::SimulationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Continuous {
    Beta { alpha: f64, beta: f64 },
    Exp { lambda: f64 },
    Gamma { shape: f64, scale: f64 },
    LogNormal { mu: f64, sigma: f64 },
    Normal { mean: f64, std_dev: f64 },
    Triangular { min: f64, max: f64, mode: f64 },
    Uniform { min: f64, max: f64 },
    Weibull { shape: f64, scale: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Boolean {
    Bernoulli { p: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Discrete {
    Geometric {
        p: f64,
    },
    Poisson {
        lambda: f64,
    },
    /// Range is inclusive of min, exclusive of max: [min, max)
    Uniform {
        min: u64,
        max: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Index {
    /// Range is inclusive of min, exclusive of max: [min, max)
    Uniform {
        min: usize,
        max: usize,
    },
    WeightedIndex {
        weights: Vec<u64>,
    },
}

impl Continuous {
    /// The generation of random variates drives stochastic behaviors during
    /// simulation execution.  This function requires the random number
    /// generator of the simulation, and produces a f64 random variate.
    pub fn random_variate(&mut self, uniform_rng: &mut UniformRNG) -> Result<f64, SimulationError> {
        match self {
            Continuous::Beta { alpha, beta } => {
                Ok(Beta::new(*alpha, *beta)?.sample(uniform_rng.rng()))
            }
            Continuous::Exp { lambda } => Ok(Exp::new(*lambda)?.sample(uniform_rng.rng())),
            Continuous::Gamma { shape, scale } => {
                Ok(Gamma::new(*shape, *scale)?.sample(uniform_rng.rng()))
            }
            Continuous::LogNormal { mu, sigma } => {
                Ok(LogNormal::new(*mu, *sigma)?.sample(uniform_rng.rng()))
            }
            Continuous::Normal { mean, std_dev } => {
                Ok(Normal::new(*mean, *std_dev)?.sample(uniform_rng.rng()))
            }
            Continuous::Triangular { min, max, mode } => {
                Ok(Triangular::new(*min, *max, *mode)?.sample(uniform_rng.rng()))
            }
            Continuous::Uniform { min, max } => {
                Ok(Uniform::new(*min, *max).sample(uniform_rng.rng()))
            }
            Continuous::Weibull { shape, scale } => {
                Ok(Weibull::new(*shape, *scale)?.sample(uniform_rng.rng()))
            }
        }
    }
}

impl Boolean {
    /// The generation of random variates drives stochastic behaviors during
    /// simulation execution.  This function requires the random number
    /// generator of the simulation, and produces a boolean random variate.
    pub fn random_variate(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<bool, SimulationError> {
        match self {
            Boolean::Bernoulli { p } => Ok(Bernoulli::new(*p)?.sample(uniform_rng.rng())),
        }
    }
}

impl Discrete {
    /// The generation of random variates drives stochastic behaviors during
    /// simulation execution.  This function requires the random number
    /// generator of the simulation, and produces a u64 random variate.
    pub fn random_variate(&mut self, uniform_rng: &mut UniformRNG) -> Result<u64, SimulationError> {
        match self {
            Discrete::Geometric { p } => Ok(Geometric::new(*p)?.sample(uniform_rng.rng())),
            Discrete::Poisson { lambda } => {
                Ok(Poisson::new(*lambda)?.sample(uniform_rng.rng()) as u64)
            }
            Discrete::Uniform { min, max } => {
                Ok(Uniform::new(*min, *max).sample(uniform_rng.rng()))
            }
        }
    }
}

impl Index {
    /// The generation of random variates drives stochastic behaviors during
    /// simulation execution.  This function requires the random number
    /// generator of the simulation, and produces a usize random variate.
    pub fn random_variate(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<usize, SimulationError> {
        match self {
            Index::Uniform { min, max } => Ok(Uniform::new(*min, *max).sample(uniform_rng.rng())),
            Index::WeightedIndex { weights } => {
                Ok(WeightedIndex::new(weights.clone())?.sample(uniform_rng.rng()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum RandomVariable {
        Continuous(Continuous),
        Discrete(Discrete),
    }

    enum ChiSquareTest {
        Continuous {
            variable: Continuous,
            bin_mapping_fn: fn(f64) -> usize,
        },
        Boolean {
            variable: Boolean,
            bin_mapping_fn: fn(bool) -> usize,
        },
        Discrete {
            variable: Discrete,
            bin_mapping_fn: fn(u64) -> usize,
        },
        Index {
            variable: Index,
            bin_mapping_fn: fn(usize) -> usize,
        },
    }

    fn empirical_mean(random_variable: &mut RandomVariable, sample_size: usize) -> f64 {
        let mut uniform_rng = UniformRNG::default();
        (0..sample_size)
            .map(|_| match random_variable {
                RandomVariable::Continuous(variable) => {
                    variable.random_variate(&mut uniform_rng).unwrap()
                }
                RandomVariable::Discrete(variable) => {
                    variable.random_variate(&mut uniform_rng).unwrap() as f64
                }
            })
            .sum::<f64>()
            / (sample_size as f64)
    }

    fn chi_square(test: &mut ChiSquareTest, expected_counts: &[usize]) -> f64 {
        let mut class_counts = vec![0; expected_counts.len()];
        let mut uniform_rng = UniformRNG::default();
        let sample_size = expected_counts.iter().sum();
        (0..sample_size).for_each(|_| {
            let index = match test {
                ChiSquareTest::Continuous {
                    variable,
                    bin_mapping_fn,
                } => bin_mapping_fn(variable.random_variate(&mut uniform_rng).unwrap()),
                ChiSquareTest::Boolean {
                    variable,
                    bin_mapping_fn,
                } => bin_mapping_fn(variable.random_variate(&mut uniform_rng).unwrap()),
                ChiSquareTest::Discrete {
                    variable,
                    bin_mapping_fn,
                } => bin_mapping_fn(variable.random_variate(&mut uniform_rng).unwrap()),
                ChiSquareTest::Index {
                    variable,
                    bin_mapping_fn,
                } => bin_mapping_fn(variable.random_variate(&mut uniform_rng).unwrap()),
            };
            class_counts[index] += 1
        });
        class_counts.iter().zip(expected_counts.iter()).fold(
            0.0,
            |acc, (class_count, expected_count)| {
                let f_class_count = *class_count as f64;
                let f_expected_count = *expected_count as f64;
                acc + (f_class_count - f_expected_count).powi(2) / f_expected_count
            },
        )
    }

    #[test]
    fn beta_samples_match_expectation() {
        let variable = Continuous::Beta {
            alpha: 7.0,
            beta: 11.0,
        };
        let mean = empirical_mean(&mut RandomVariable::Continuous(variable), 10000);
        let expected = 7.0 / (7.0 + 11.0);
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn exponential_samples_match_expectation() {
        let variable = Continuous::Exp { lambda: 7.0 };
        let mean = empirical_mean(&mut RandomVariable::Continuous(variable), 10000);
        let expected = 1.0 / 7.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn gamma_samples_match_expectation() {
        let variable = Continuous::Gamma {
            shape: 7.0,
            scale: 11.0,
        };
        let mean = empirical_mean(&mut RandomVariable::Continuous(variable), 10000);
        let expected = 77.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn lognormal_samples_match_expectation() {
        let variable = Continuous::LogNormal {
            mu: 11.0,
            sigma: 1.0,
        };
        let mean = empirical_mean(&mut RandomVariable::Continuous(variable), 10000);
        let expected = (11.0f64 + 1.0f64.powi(2) / 2.0f64).exp();
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn normal_samples_chi_square() {
        fn bins_mapping(variate: f64) -> usize {
            let mean = 11.0;
            let std_dev = 3.0;
            if variate < mean - 3.0 * std_dev {
                0
            } else if variate < mean - 2.0 * std_dev {
                1
            } else if variate < mean - std_dev {
                2
            } else if variate < mean {
                3
            } else if variate < mean + std_dev {
                4
            } else if variate < mean + 2.0 * std_dev {
                5
            } else if variate < mean + 3.0 * std_dev {
                6
            } else {
                7
            }
        }
        let variable = Continuous::Normal {
            mean: 11.0,
            std_dev: 3.0,
        };
        // 8 classes (a.k.a. bins)
        // On each side: within 1 sigma, 1 sigma to 2 sigma, 2 sigma to 3 sigma, 3+ sigma
        let expected_counts: [usize; 8] = [20, 210, 1360, 3410, 3410, 1360, 210, 20];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Continuous {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=7 degrees of freedom, the chi square critical
        // value for this scenario is 18.475
        let chi_square_critical = 18.475;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn triangular_samples_chi_square() {
        fn bins_mapping(variate: f64) -> usize {
            ((variate - 5.0) / 5.0) as usize
        }
        let variable = Continuous::Triangular {
            min: 5.0,
            max: 25.0,
            mode: 15.0,
        };
        // 4 classes/bins - each of width 5
        let expected_counts: [usize; 4] = [125, 375, 375, 125];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Continuous {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=3 degrees of freedom, the chi square critical
        // value for this scenario is 134.642
        let chi_square_critical = 11.345;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn continuous_uniform_samples_chi_square() {
        fn bins_mapping(variate: f64) -> usize {
            let min = 7.0;
            let max = 11.0;
            ((variate - min) * (max - 1.0)) as usize
        }
        let variable = Continuous::Uniform {
            min: 7.0,
            max: 11.0,
        };
        // Constant bin counts, due to uniformity of distribution
        let expected_counts: [usize; 40] = [250; 40];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Continuous {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=39 degrees of freedom, the chi square critical
        // value for this scenario is 62.428
        let chi_square_critical = 62.428;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn weibull_samples_match_expectation() {
        let variable = Continuous::Weibull {
            shape: 7.0,
            scale: 0.5,
        };
        let mean = empirical_mean(&mut RandomVariable::Continuous(variable), 10000);
        let expected = 14.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn bernoulli_samples_chi_square() {
        fn bins_mapping(variate: bool) -> usize {
            variate as usize
        }
        let variable = Boolean::Bernoulli { p: 0.3 };
        // Failures (false == 0) is 70% of trials and success (true == 1) is 30% of trials
        let expected_counts: [usize; 2] = [7000, 3000];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Boolean {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=1 degrees of freedom, the chi square critical
        // value for this scenario is 6.635
        let chi_square_critical = 6.635;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn geometric_samples_match_expectation() {
        let variable = Discrete::Geometric { p: 0.2 };
        let mean = empirical_mean(&mut RandomVariable::Discrete(variable), 10000);
        let expected = (1.0 - 0.2) / 0.2;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn poisson_samples_match_expectation() {
        let variable = Discrete::Poisson { lambda: 7.0 };
        let mean = empirical_mean(&mut RandomVariable::Discrete(variable), 10000);
        let expected = 7.0;
        assert!((mean - expected).abs() / expected < 0.025);
    }

    #[test]
    fn discrete_uniform_samples_chi_square() {
        fn bins_mapping(variate: u64) -> usize {
            let min = 7;
            (variate - min) as usize
        }
        let variable = Discrete::Uniform { min: 7, max: 11 };
        // Constant bin counts, due to uniformity of distribution
        let expected_counts: [usize; 4] = [2500; 4];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Discrete {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=4 degrees of freedom, the chi square critical
        // value for this scenario is 13.277
        let chi_square_critical = 13.277;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn weighted_index_samples_chi_square() {
        fn bins_mapping(variate: usize) -> usize {
            variate
        }
        let variable = Index::WeightedIndex {
            weights: vec![1, 2, 3, 4],
        };
        // The expected bin counts scale linearly with the weights
        let expected_counts: [usize; 4] = [1000, 2000, 3000, 4000];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Index {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=3 degrees of freedom, the chi square critical
        // value for this scenario is 11.345
        let chi_square_critical = 11.345;
        assert![chi_square_actual < chi_square_critical];
    }

    #[test]
    fn index_uniform_samples_chi_square() {
        fn bins_mapping(variate: usize) -> usize {
            let min = 7;
            variate - min
        }
        let variable = Index::Uniform { min: 7, max: 11 };
        // Constant bin counts, due to uniformity of distribution
        let expected_counts: [usize; 4] = [2500; 4];
        let chi_square_actual = chi_square(
            &mut ChiSquareTest::Index {
                variable: variable,
                bin_mapping_fn: bins_mapping,
            },
            &expected_counts,
        );
        // At a significance level of 0.01, and with n-1=4 degrees of freedom, the chi square critical
        // value for this scenario is 13.277
        let chi_square_critical = 13.277;
        assert![chi_square_actual < chi_square_critical];
    }
}
