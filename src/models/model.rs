use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeMap;
use serde_json::Value;
use serde::de::{self, Unexpected};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::ModelMessage;
use crate::input_modeling::UniformRNG;
use crate::utils::error::SimulationError;

/// `Model` wraps `model_type` and provides common ID functionality (a struct
/// field and associated accessor method).  The simulator requires all models
/// to have an ID.
pub struct Model {
    id: String,
    inner: Rc<RefCell<dyn AsModel>>,
}

impl Model {
    pub fn new(id: String, inner: Rc<RefCell<dyn AsModel>>) -> Self {
        Self { id, inner }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        // Fix self.inner cloning
        Model {
            id: self.id.clone(),
            inner: self.inner.clone()
        }
    }
}

impl Serialize for Model {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let extra_fields: serde_json::Value = self.inner.borrow().serialize();
        let mut model = serializer.serialize_map(None)?;
        model.serialize_entry("id", &self.id)?;
        model.serialize_entry("type", self.inner.borrow().get_type())?;
        if let serde_json::Value::Object(map) = extra_fields {
            for (key, value) in map.iter() {
                model.serialize_entry(&key, &value)?;
            }
        }
        model.end()
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let model_extra = super::ModelExtra::deserialize(deserializer)?;
        println!("New Model {:?}", model_extra);
        const VARIANTS: &'static [&'static str] = &[
            &"Generator", &"ExclusiveGateway", &"Processor", &"Storage"
        ];
        match &model_extra.model_type[..] {
            "Generator" => {
                if let Ok(generator) = serde_json::from_value::<super::Generator>(model_extra.extra) {
                    Ok(Model::new(
                        model_extra.id,
                        Rc::new(RefCell::new(generator))
                    ))
                } else {
                    Err(de::Error::invalid_value(Unexpected::Other("Generator"), &"Generator"))
                }
            },
            "ExclusiveGateway" => {
                if let Ok(exclusive_gateway) = serde_json::from_value::<super::ExclusiveGateway>(model_extra.extra) {
                    Ok(Model::new(
                        model_extra.id,
                        Rc::new(RefCell::new(exclusive_gateway))
                    ))
                } else {
                    Err(de::Error::invalid_value(Unexpected::Other("ExclusiveGateway"), &"ExclusiveGateway"))
                }
            },
            "Processor" => {
                if let Ok(processor) = serde_json::from_value::<super::Processor>(model_extra.extra) {
                    Ok(Model::new(
                        model_extra.id,
                        Rc::new(RefCell::new(processor))
                    ))
                } else {
                    Err(de::Error::invalid_value(Unexpected::Other("Processor"), &"Processor"))
                }
            },
            "Storage" => {
                if let Ok(storage) = serde_json::from_value::<super::Storage>(model_extra.extra) {
                    Ok(Model::new(
                        model_extra.id,
                        Rc::new(RefCell::new(storage))
                    ))
                } else {
                    Err(de::Error::invalid_value(Unexpected::Other("Storage"), &"Storage"))
                }
            },
            other => {
                Err(de::Error::unknown_variant(other, VARIANTS))
            }
        }
    }
}

impl AsModel for Model {
    fn status(&self) -> String {
        self.inner.borrow().status()
    }

    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.borrow_mut().events_ext(uniform_rng, incoming_message)
    }

    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.inner.borrow_mut().events_int(uniform_rng)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.inner.borrow_mut().time_advance(time_delta)
    }

    fn until_next_event(&self) -> f64 {
        self.inner.borrow_mut().until_next_event()
    }
}

/// The `AsModel` trait defines everything required for a model to operate
/// within the discrete event simulation.  The simulator formalism (Discrete
/// Event System Specification) requires `events_ext`, `events_int`,
/// `time_advance`, and `until_next_event`.  The additional `status` is for
/// facilitation of simulation reasoning, reporting, and debugging.
// #[enum_dispatch]
pub trait AsModel {
    fn get_type(&self) -> &'static str {
        ""
    }
    fn serialize(&self) -> serde_json::Value {
        serde_json::Value::Null
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
