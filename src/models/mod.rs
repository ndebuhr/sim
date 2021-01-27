//! The models module provides a set of prebuilt atomic models, for easy
//! reuse in simulation products and projects.  Additionally, this module
//! specifies the requirements of any additional custom models, via the
//! `Model` trait.

pub mod exclusive_gateway;
pub mod gate;
pub mod generator;
pub mod load_balancer;
pub mod model;
pub mod parallel_gateway;
pub mod processor;
pub mod stochastic_gate;
pub mod storage;

#[derive(Debug, Clone)]
pub struct ModelMessage {
    pub port_name: String,
    pub message: String,
}
