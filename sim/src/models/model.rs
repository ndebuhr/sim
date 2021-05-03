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
use crate::simulator::Services;
use crate::utils::error::SimulationError;

/// `Model` wraps `ModelType` and provides common ID functionality (a struct
/// field and associated accessor method).  The simulator requires all models
/// to have an ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model<M: AsModel> {
    id: String,
    #[serde(flatten)]
    inner: M,
}

impl<M: AsModel> Model<M> {
    pub fn new(id: String, inner: M) -> Self {
        Self { id, inner }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl<M: AsModel> AsModel for Model<M> {
    fn status(&self) -> String {
        self.inner.status()
    }

    fn events_ext(
        &mut self,
        incoming_message: ModelMessage,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_ext(incoming_message, services)
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.events_int(services)
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
        incoming_message: ModelMessage,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError>;
    fn events_int(&mut self, services: &mut Services)
        -> Result<Vec<ModelMessage>, SimulationError>;
    fn time_advance(&mut self, time_delta: f64);
    fn until_next_event(&self) -> f64;
}
