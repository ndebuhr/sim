use super::ModelMessage;
use crate::input_modeling::UniformRNG;
use crate::utils::error::SimulationError;

pub trait ModelClone {
    fn clone_box(&self) -> Box<dyn AsModel>;
}

impl<T> ModelClone for T
where
    T: 'static + AsModel + Clone,
{
    fn clone_box(&self) -> Box<dyn AsModel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AsModel> {
    fn clone(&self) -> Box<dyn AsModel> {
        self.clone_box()
    }
}

/// The `AsModel` trait defines everything required for a model to operate
/// within the discrete event simulation.  The simulator formalism (Discrete
/// Event System Specification) requires `events_ext`, `events_int`,
/// `time_advance`, and `until_next_event`.  The additional `status` is for
/// facilitation of simulation reasoning, reporting, and debugging.
// #[enum_dispatch]
pub trait AsModel: ModelClone {
    fn get_type(&self) -> &'static str {
        ""
    }
    fn serialize(&self) -> serde_yaml::Value {
        serde_yaml::Value::Null
    }
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
