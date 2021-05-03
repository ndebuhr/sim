//! The input modeling module provides a foundation for configurable model
//! behaviors, whether that is deterministic or stochastic.  The module
//! includes a set of random variable distributions for use in atomic models,
//! a system around "thinning" for non-stationary model behaviors, and a
//! structure around random number generation.

pub mod random_variable;
pub mod thinning;
pub mod uniform_rng;

pub use thinning::Thinning;
pub use uniform_rng::UniformRNG;
