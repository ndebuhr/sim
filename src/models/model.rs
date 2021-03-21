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

/// The overall "wrapper" around a model, complete with the model's ID.
/// This is what you probably want to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    id: String,
    #[serde(flatten)]
    inner: ModelType,
}

impl Model {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl AsModel for Model {
    fn status(&self) -> String {
        self.inner.status()
    }

    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_ext(uniform_rng, incoming_message)
    }

    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_int(uniform_rng)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.inner.time_advance(time_delta)
    }

    fn until_next_event(&self) -> f64 {
        self.inner.until_next_event()
    }
}

/// An enum encompassing all the available types of models. Each variant
/// holds a concrete type that implements AsModel.
#[enum_dispatch(AsModel)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ModelType {
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
