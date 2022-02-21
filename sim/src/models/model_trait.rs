use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

pub trait ModelClone {
    fn clone_box(&self) -> Box<dyn ReportableModel>;
}

impl<T> ModelClone for T
where
    T: 'static + ReportableModel + Clone,
{
    fn clone_box(&self) -> Box<dyn ReportableModel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ReportableModel> {
    fn clone(&self) -> Box<dyn ReportableModel> {
        self.clone_box()
    }
}

pub trait SerializableModel {
    fn get_type(&self) -> &'static str {
        "Model"
    }
    fn serialize(&self) -> serde_yaml::Value {
        serde_yaml::Value::Null
    }
}

/// The `DevsModel` trait defines everything required for a model to operate
/// within the discrete event simulation.  The simulator formalism (Discrete
/// Event System Specification) requires `events_ext`, `events_int`,
/// `time_advance`, and `until_next_event`.
pub trait DevsModel: ModelClone + SerializableModel {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError>;
    fn events_int(&mut self, services: &mut Services)
        -> Result<Vec<ModelMessage>, SimulationError>;
    fn time_advance(&mut self, time_delta: f64);
    fn until_next_event(&self) -> f64;
    #[cfg(feature = "simx")]
    fn event_rules_scheduling(&self) -> &str;
    #[cfg(feature = "simx")]
    fn event_rules(&self) -> String;
}

/// The additional status and record-keeping methods of `Reportable` provide
/// improved simulation reasoning, reporting, and debugging, but do not
/// impact simulation execution or results.
pub trait Reportable {
    fn status(&self) -> String;
    fn records(&self) -> &Vec<ModelRecord>;
}

/// A `ReportableModel` has the required Discrete Event System Specification
/// methods of trait `DevsModel` and the status reporting and record keeping
/// mechanisms of trait `Reportable`.
pub trait ReportableModel: DevsModel + Reportable {}
