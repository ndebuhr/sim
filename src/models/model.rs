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

/// An enum encompassing all the available types of models. Each variant
/// holds a concrete type that implements AsModel.
#[enum_dispatch(AsModel)]
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Model {
    ExclusiveGateway,
    Gate,
    Generator,
    LoadBalancer,
    ParallelGateway,
    Processor,
    StochasticGate,
    Storage,
}

/// The `AsModel` trait defines everything required for a model to operate
/// within the discrete event simulation.  These requirements are based
/// largely on the Discrete Event System Specification (DEVS), but with a
/// small amount of plumbing (`as_any` and `id`) and a dedicated status
/// reporting method `status`.
#[enum_dispatch]
pub trait AsModel {
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
