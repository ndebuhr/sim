use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use super::ExclusiveGateway;
use super::Gate;
use super::Generator;
use super::LoadBalancer;
use super::ModelMessage;
use super::ParallelGateway;
use super::Processor;
use super::StochasticGate;
use super::Storage;
use crate::input_modeling::UniformRNG;
use crate::utils::error::SimulationError;

/// `Model` wraps `ModelType` and provides common ID functionality (a struct
/// field and associated accessor method).  The simulator requires all models
/// to have an ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    id: String,
    #[serde(flatten)]
    inner: ModelType,
}

impl Model {
    pub fn new(id: String, inner: ModelType) -> Self {
        Self { id, inner }
    }

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

/// `ModelType` is an enum encompassing all the available model types. Each
/// variant holds a concrete type that implements AsModel.
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
/// within the discrete event simulation.  The simulator formalism (Discrete
/// Event System Specification) requires `events_ext`, `events_int`,
/// `time_advance`, and `until_next_event`.  The additional `status` is for
/// facilitation of simulation reasoning, reporting, and debugging.
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
