use serde::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

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
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ModelVisitor;
        impl<'de> Visitor<'de> for ModelVisitor {
            type Value = Model;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Model")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Model, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut model_type = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "type" => {
                            if model_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            model_type = Some(map.next_value()?);
                        }
                        field => {
                            println!("Unparsed field: {}", field);
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let _model_type = model_type.ok_or_else(|| de::Error::missing_field("type"))?;
                Ok(Model::new(id, Rc::new(RefCell::new(super::storage::Storage::new(
                    String::from("store"),
                    String::from("read"),
                    String::from("stored"),
                    false,
                    false,
                )))))
            }
        }
        const FIELDS: &'static [&'static str] = &["id", "type"];
        deserializer.deserialize_struct("Model", FIELDS, ModelVisitor)
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
