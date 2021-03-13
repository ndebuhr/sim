//! The simulator module provides the mechanics to orchestrate the models and
//! connectors via discrete event simulation.  The specific formalism for
//! simulation execution is the Discrete Event System Specification.  User
//! interaction is also captured in this module - simulation stepping and
//! input injection.
//!
//! `Simulation` and `WebSimulation` are used for Rust- and npm-based
//! projects, respectively.  The `Simulation` methods use the associated
//! struct types directly, while the `WebSimulation` provides an interface
//! with better JS/WASM compatibility.
//!
//! Most simulation analysis will involve the collection, transformation,
//! and analysis of messages.  The `step`, `step_n`, and `step_until` methods
//! return the messages generated during the execution of the simulation
//! step(s), for use in message analysis.

use std::f64::INFINITY;

use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::input_modeling::uniform_rng::UniformRNG;
use crate::models::model::{AsModel, Model};
use crate::models::ModelMessage;
use crate::utils;
use crate::utils::error::SimulationError;

mod test_simulations;

/// The `Simulation` struct is the core of sim, and includes everything
/// needed to run a simulation - models, connectors, and a random number
/// generator.  State information, specifically global time and active
/// messages are additionally retained in the struct.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    models: Vec<Model>,
    connectors: Vec<Connector>,
    messages: Vec<Message>,
    global_time: f64,
    #[serde(skip_serializing)]
    uniform_rng: UniformRNG,
}

/// Connectors are configured to connect models through their ports.  During
/// simulation, models exchange messages (as per the Discrete Event System
/// Specification) via these connectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connector {
    id: String,
    #[serde(rename = "sourceID")]
    source_id: String,
    #[serde(rename = "targetID")]
    target_id: String,
    source_port: String,
    target_port: String,
}

/// Messages are the mechanism of information exchange for models in a
/// a simulation.  The message must contain origin information (source model
/// ID and source model port), destination information (target model ID and
/// target model port), and the text/content of the message.
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    source_id: String,
    source_port: String,
    target_id: String,
    target_port: String,
    time: f64,
    message: String,
}

impl Simulation {
    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post(models: Vec<Model>, connectors: Vec<Connector>) -> Self {
        utils::set_panic_hook();
        Self {
            models,
            connectors,
            ..Self::default()
        }
    }

    /// This method sets the models and connectors of an existing simulation.
    pub fn put(&mut self, models: Vec<Model>, connectors: Vec<Connector>) {
        self.models = models;
        self.connectors = connectors;
    }

    /// Simulation steps generate messages, which are then consumed on
    /// subsequent simulation steps.  These messages between models in a
    /// simulation drive much of the discovery, analysis, and design.  This
    /// accessor method provides the list of active messages, at the current
    /// point of time in the simulation.  Message history is not retained, so
    /// simulation products and projects should collect messages as needed
    /// throughout the simulation execution.
    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    /// An accessor method for the simulation global time.
    pub fn get_global_time(&self) -> f64 {
        self.global_time
    }

    /// This method provides a mechanism for getting the status of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the current status string for that model.
    pub fn status(&self, model_id: &str) -> Result<String, SimulationError> {
        Ok(self
            .models
            .iter()
            .find(|model| model.id() == model_id)
            .ok_or_else(|| SimulationError::ModelNotFound)?
            .status())
    }

    /// To enable simulation replications, the reset method resets the state
    /// of the simulation, except for the random number generator.
    /// Recreating a simulation from scratch for additional replications
    /// does not work, due to the random number generator seeding.
    pub fn reset(&mut self) {
        self.reset_messages();
        self.reset_global_time();
    }

    /// Clear the active messages in a simulation.
    pub fn reset_messages(&mut self) {
        self.messages = Vec::new();
    }

    /// Reset the simulation global time to 0.0.
    pub fn reset_global_time(&mut self) {
        self.global_time = 0.0;
    }

    /// This method provides a convenient foundation for operating on the
    /// full set of models in the simulation.
    pub fn models(&mut self) -> Vec<&mut Model> {
        self.models.iter_mut().collect()
    }

    /// This method constructs a list of target IDs for a given source model
    /// ID and port.  This message target information is derived from the
    /// connectors configuration.
    fn get_message_target_ids(&self, source_id: &str, source_port: &str) -> Vec<String> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                if connector.source_id == source_id && connector.source_port == source_port {
                    Some(connector.target_id.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// This method constructs a list of target ports for a given source model
    /// ID and port.  This message target information is derived from the
    /// connectors configuration.
    fn get_message_target_ports(&self, source_id: &str, source_port: &str) -> Vec<String> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                if connector.source_id == source_id && connector.source_port == source_port {
                    Some(connector.target_port.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Input injection creates a message during simulation execution,
    /// without needing to create that message through the standard
    /// simulation constructs.  This enables live simulation interaction,
    /// disruption, and manipulation - all through the standard simulation
    /// message system.
    pub fn inject_input(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// The simulation step is foundational for a discrete event simulation.
    /// This method executes a single discrete event simulation step,
    /// including internal state transitions, external state transitions,
    /// message orchestration, global time accounting, and step messages
    /// output.
    pub fn step(&mut self) -> Result<Vec<Message>, SimulationError> {
        let messages = self.messages.clone();
        let mut next_messages: Vec<Message> = Vec::new();
        // Process external events and gather associated messages
        if !messages.is_empty() {
            let errors: Result<(), SimulationError> = (0..self.models.len())
                .map(|model_index| -> Result<(), SimulationError> {
                    let model_messages: Vec<ModelMessage> = messages
                        .iter()
                        .filter_map(|message| {
                            if message.target_id == self.models[model_index].id() {
                                Some(ModelMessage {
                                    port_name: message.target_port.clone(),
                                    message: message.message.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    model_messages
                        .iter()
                        .map(|model_message| -> Result<(), SimulationError> {
                            self.models[model_index]
                                .events_ext(&mut self.uniform_rng, model_message.clone())?
                                .iter()
                                .for_each(|outgoing_message| {
                                    let target_ids = self.get_message_target_ids(
                                        &self.models[model_index].id(), // Outgoing message source model ID
                                        &outgoing_message.port_name, // Outgoing message source model port
                                    );
                                    let target_ports = self.get_message_target_ports(
                                        &self.models[model_index].id(), // Outgoing message source model ID
                                        &outgoing_message.port_name, // Outgoing message source model port
                                    );
                                    target_ids.iter().zip(target_ports.iter()).for_each(
                                        |(target_id, target_port)| {
                                            next_messages.push(Message {
                                                source_id: self.models[model_index].id(),
                                                source_port: outgoing_message.port_name.clone(),
                                                target_id: target_id.clone(),
                                                target_port: target_port.clone(),
                                                time: self.global_time,
                                                message: outgoing_message.message.clone(),
                                            });
                                        },
                                    );
                                });
                            Ok(())
                        })
                        .collect()
                })
                .collect();
            errors?;
        }
        // Process internal events and gather associated messages
        let until_next_event: f64;
        if self.messages.is_empty() {
            until_next_event = self.models().iter().fold(INFINITY, |min, model| {
                f64::min(min, model.until_next_event())
            });
        } else {
            until_next_event = 0.0;
        }
        self.models().iter_mut().for_each(|model| {
            model.time_advance(until_next_event);
        });
        self.global_time += until_next_event;
        let errors: Result<Vec<()>, SimulationError> = (0..self.models.len())
            .map(|model_index| -> Result<(), SimulationError> {
                self.models[model_index]
                    .events_int(&mut self.uniform_rng)?
                    .iter()
                    .for_each(|outgoing_message| {
                        let target_ids = self.get_message_target_ids(
                            &self.models[model_index].id(), // Outgoing message source model ID
                            &outgoing_message.port_name,    // Outgoing message source model port
                        );
                        let target_ports = self.get_message_target_ports(
                            &self.models[model_index].id(), // Outgoing message source model ID
                            &outgoing_message.port_name,    // Outgoing message source model port
                        );
                        target_ids.iter().zip(target_ports.iter()).for_each(
                            |(target_id, target_port)| {
                                next_messages.push(Message {
                                    source_id: self.models[model_index].id(),
                                    source_port: outgoing_message.port_name.clone(),
                                    target_id: target_id.clone(),
                                    target_port: target_port.clone(),
                                    time: self.global_time,
                                    message: outgoing_message.message.clone(),
                                });
                            },
                        );
                    });
                Ok(())
            })
            .collect();
        errors?;
        self.messages = next_messages;
        Ok(self.get_messages().to_vec())
    }

    /// This method executes simulation `step` calls, until a global time
    /// has been exceeded.  At which point, the messages from all the
    /// simulation steps are returned.
    pub fn step_until(&mut self, until: f64) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
        loop {
            self.step()?;
            if self.global_time < until {
                message_records.extend(self.get_messages().clone());
            } else {
                break;
            }
        }
        Ok(message_records)
    }

    /// This method executes the specified number of simulation steps, `n`.
    /// Upon execution of the n steps, the messages from all the steps are
    /// returned.
    fn step_n(&mut self, n: usize) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
        (0..n)
            .map(|_| -> Result<Vec<Message>, SimulationError> {
                self.step()?;
                message_records.extend(self.messages.clone());
                Ok(Vec::new())
            })
            .find(|result| result.is_err())
            .unwrap_or(Ok(message_records))
    }
}

/// The `WebSimulation` provides JS/WASM-compatible interfaces to the core
/// `Simulation` struct.  For additional insight on these methods, refer to
/// the associated `Simulation` methods.  Errors are unwrapped, instead of
/// returned, in the `WebSimulation` methods.
#[wasm_bindgen]
#[derive(Default, Serialize, Deserialize)]
pub struct WebSimulation {
    simulation: Simulation,
}

#[wasm_bindgen]
impl WebSimulation {
    /// A JS/WASM interface for `Simulation.post`, which uses JSON
    /// representations of the simulation models and connectors.
    pub fn post_json(models: &str, connectors: &str) -> Self {
        utils::set_panic_hook();
        Self {
            simulation: Simulation {
                models: serde_json::from_str(models).unwrap(),
                connectors: serde_json::from_str(connectors).unwrap(),
                ..Simulation::default()
            },
        }
    }

    /// A JS/WASM interface for `Simulation.put`, which uses JSON
    /// representations of the simulation models and connectors.
    pub fn put_json(&mut self, models: &str, connectors: &str) {
        self.simulation.models = serde_json::from_str(models).unwrap();
        self.simulation.connectors = serde_json::from_str(connectors).unwrap();
    }

    /// Get a JSON representation of the full `Simulation` configuration.
    pub fn get_json(&self) -> String {
        serde_json::to_string_pretty(&self.simulation).unwrap()
    }

    /// A JS/WASM interface for `Simulation.post`, which uses YAML
    /// representations of the simulation models and connectors.
    pub fn post_yaml(models: &str, connectors: &str) -> WebSimulation {
        utils::set_panic_hook();
        Self {
            simulation: Simulation {
                models: serde_yaml::from_str(models).unwrap(),
                connectors: serde_yaml::from_str(connectors).unwrap(),
                ..Simulation::default()
            },
        }
    }

    /// A JS/WASM interface for `Simulation.put`, which uses YAML
    /// representations of the simulation models and connectors.
    pub fn put_yaml(&mut self, models: &str, connectors: &str) {
        self.simulation.models = serde_yaml::from_str(models).unwrap();
        self.simulation.connectors = serde_yaml::from_str(connectors).unwrap();
    }

    /// Get a YAML representation of the full `Simulation` configuration.
    pub fn get_yaml(&self) -> String {
        serde_yaml::to_string(&self.simulation).unwrap()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a JavaScript Array.
    pub fn get_messages_js(&self) -> Array {
        // Workaround for https://github.com/rustwasm/wasm-bindgen/issues/111
        self.simulation
            .get_messages()
            .clone()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a JSON string.
    pub fn get_messages_json(&self) -> String {
        serde_json::to_string(&self.simulation.get_messages()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a YAML string.
    pub fn get_messages_yaml(&self) -> String {
        serde_yaml::to_string(&self.simulation.get_messages()).unwrap()
    }

    /// An interface to `Simulation.get_global_time`.
    pub fn get_global_time(&self) -> f64 {
        self.simulation.get_global_time()
    }

    /// An interface to `Simulation.status`.
    pub fn status(&self, model_id: &str) -> String {
        self.simulation.status(model_id).unwrap()
    }

    /// An interface to `Simulation.reset`.
    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    /// An interface to `Simulation.reset_messages`.
    pub fn reset_messages(&mut self) {
        self.simulation.reset_messages();
    }

    /// An interface to `Simulation.reset_global_time`
    pub fn reset_global_time(&mut self) {
        self.simulation.reset_global_time();
    }

    /// A JS/WASM interface for `Simulation.inject_input`, which uses a JSON
    /// representation of the injected messages.
    pub fn inject_input_json(&mut self, message: &str) {
        self.simulation
            .inject_input(serde_json::from_str(message).unwrap());
    }

    /// A JS/WASM interface for `Simulation.inject_input`, which uses a YAML
    /// representation of the injected messages.
    pub fn inject_input_yaml(&mut self, message: &str) {
        self.simulation
            .inject_input(serde_yaml::from_str(message).unwrap());
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_js(&mut self) -> Array {
        self.simulation
            .step()
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a JSON string.
    pub fn step_json(&mut self) -> String {
        serde_json::to_string(&self.simulation.step().unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a YAML string.
    pub fn step_yaml(&mut self) -> String {
        serde_yaml::to_string(&self.simulation.step().unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_until_js(&mut self, until: f64) -> Array {
        self.simulation
            .step_until(until)
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a JSON string.
    pub fn step_until_json(&mut self, until: f64) -> String {
        serde_json::to_string(&self.simulation.step_until(until).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a YAML string.
    pub fn step_until_yaml(&mut self, until: f64) -> String {
        serde_yaml::to_string(&self.simulation.step_until(until).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_n_js(&mut self, n: usize) -> Array {
        self.simulation
            .step_n(n)
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a JSON string.
    pub fn step_n_json(&mut self, n: usize) -> String {
        serde_json::to_string(&self.simulation.step_n(n).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a YAML string.
    pub fn step_n_yaml(&mut self, n: usize) -> String {
        serde_yaml::to_string(&self.simulation.step_n(n).unwrap()).unwrap()
    }
}
