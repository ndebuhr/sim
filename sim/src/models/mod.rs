//! The models module provides a set of prebuilt atomic models, for easy
//! reuse in simulation products and projects.  Additionally, this module
//! specifies the requirements of any additional custom models, via the
//! `Model` trait.

use serde::{Deserialize, Serialize};

pub mod batcher;
pub mod coupled;
pub mod exclusive_gateway;
pub mod gate;
pub mod generator;
pub mod load_balancer;
pub mod model;
pub mod parallel_gateway;
pub mod processor;
pub mod stochastic_gate;
pub mod stopwatch;
pub mod storage;

pub mod model_factory;
pub mod model_repr;
pub mod model_trait;

pub use self::batcher::Batcher;
pub use self::coupled::{Coupled, ExternalInputCoupling, ExternalOutputCoupling, InternalCoupling};
pub use self::exclusive_gateway::ExclusiveGateway;
pub use self::gate::Gate;
pub use self::generator::Generator;
pub use self::load_balancer::LoadBalancer;
pub use self::model::Model;
pub use self::model_trait::{DevsModel, Reportable, ReportableModel};
pub use self::parallel_gateway::ParallelGateway;
pub use self::processor::Processor;
pub use self::stochastic_gate::StochasticGate;
pub use self::stopwatch::Stopwatch;
pub use self::storage::Storage;

pub use self::model_repr::ModelRepr;

#[derive(Debug, Clone)]
pub struct ModelMessage {
    pub port_name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub time: f64,
    pub action: String,
    pub subject: String,
}
