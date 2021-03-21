use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::exclusive_gateway::ExclusiveGateway;
use super::gate::Gate;
use super::generator::Generator;
use super::load_balancer::LoadBalancer;
use super::parallel_gateway::ParallelGateway;
use super::processor::Processor;
use super::stochastic_gate::StochasticGate;
use super::storage::Storage;
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The type of the Model.
#[enum_dispatch(Model)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Type {
    ExclusiveGateway(ExclusiveGateway),
    Gate            (Gate),
    Generator       (Generator),
    LoadBalancer    (LoadBalancer),
    ParallelGateway (ParallelGateway),
    Processor       (Processor),
    StochasticGate  (StochasticGate),
    Storage         (Storage),
}

/// The `Model` trait defines everything required for a model to operate
/// within the discrete event simulation.  These requirements are based
/// largely on the Discrete Event System Specification (DEVS), but with a
/// small amount of plumbing (`as_any` and `id`) and a dedicated status
/// reporting method `status`.
#[enum_dispatch]
pub trait Model {
    fn id(&self) -> String;
    fn status(&self) -> String;
    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError>;
    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError>;
    fn time_advance(&mut self, time_delta: f64);
    fn until_next_event(&self) -> f64;
}

