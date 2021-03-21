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

pub use self::exclusive_gateway::ExclusiveGateway;
pub use self::gate::Gate;
pub use self::generator::Generator;
pub use self::load_balancer::LoadBalancer;
pub use self::model::{AsModel, Model};
pub use self::parallel_gateway::ParallelGateway;
pub use self::processor::Processor;
pub use self::stochastic_gate::StochasticGate;
pub use self::storage::Storage;

#[derive(Debug, Clone)]
pub struct ModelMessage {
    pub port_name: String,
    pub message: String,
}
