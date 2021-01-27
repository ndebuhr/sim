//! The simulator module provides the mechanics to orchestrate the models and
//! connectors via discrete event simulation.  The specific formalism for
//! simulation execution is the Discrete Event System Specification.  User
//! interaction is also captured in this module - simulation stepping and
//! input injection.
//!
//! Two constructors are provided for creating a `Simulation`, `post_json`
//! and `post_yaml`.  If the default `Simulation::default()` is used to
//! create the simulation, models and connectors should be added with the
//! `put_json` or `put_yaml` method.
//!
//! Most simulation analysis will involve the collection, transformation,
//! and analysis of messages.  Messages can be retrieved after each
//! simulation step with the `get_messages_yaml` and `get_messages_json`
//! methods.  The Step N and Step Until methods enable the execution of many
//! simulation steps with a single method call.  Since message history isn't
//! stored in the simulation struct, `step_n` and `step_until` collect the
//! messages during the simulation steps, and provide them as a returned
//! value.  The returned value format depends on the method call:
//! `step_n_json`, `step_n_yaml`, `step_until_json`, or `step_until_yaml`.

use std::f64::INFINITY;

use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::input_modeling::uniform_rng::UniformRNG;
use crate::models::model::Model;
use crate::models::ModelMessage;
use crate::utils;

mod test_simulations;

/// The `Simulation` struct is the core of sim, and includes everything
/// needed to run a simulation - models, connectors, and a random number
/// generator.  State information, specifically global time and active
/// messages are additionally retained in the struct.
#[wasm_bindgen]
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    models: Vec<Box<dyn Model>>,
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
struct Connector {
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

#[wasm_bindgen]
impl Simulation {
    /// Like `post_yaml`, this constructor method creates a simulation from
    /// a supplied configuration (models and connectors).
    pub fn post_json(models: &str, connectors: &str) -> Simulation {
        utils::set_panic_hook();
        Simulation {
            models: serde_json::from_str(models).unwrap(),
            connectors: serde_json::from_str(connectors).unwrap(),
            ..Simulation::default()
        }
    }

    /// Like `put_yaml`, this method sets the models and connectors of an
    /// existing simulation.
    pub fn put_json(&mut self, models: &str, connectors: &str) {
        self.models = serde_json::from_str(models).unwrap();
        self.connectors = serde_json::from_str(connectors).unwrap();
    }

    /// This method provides the simulation state in a "pretty" serialized
    /// JSON format - with line breaks and indentations.  The simulation
    /// details not included in this serialization are internal
    /// implementation details.  If it is not included in the serialization,
    /// the data is not required to specify the simulation state, and it has
    /// been excluded deliberately.
    pub fn get_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Like `post_json`, this constructor method creates a simulation from
    /// a supplied configuration (models and connectors).
    pub fn post_yaml(models: &str, connectors: &str) -> Simulation {
        utils::set_panic_hook();
        Simulation {
            models: serde_yaml::from_str(models).unwrap(),
            connectors: serde_yaml::from_str(connectors).unwrap(),
            ..Simulation::default()
        }
    }

    /// Like `put_json`, this method sets the models and connectors of an
    /// existing simulation.
    pub fn put_yaml(&mut self, models: &str, connectors: &str) {
        self.models = serde_yaml::from_str(models).unwrap();
        self.connectors = serde_yaml::from_str(connectors).unwrap();
    }

    /// This method provides the simulation state in a yaml serialized JSON
    /// format - with line breaks and indentations.  The simulation details
    /// not included in this serialization are internal implementation
    /// details.  If it is not included in the serialization, the data is not
    /// required to specify the simulation state, and it has been excluded
    /// deliberately.
    pub fn get_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    /// The get_messages implementation underpinning `get_messages_js`,
    /// `get_messages_json`, and `get_messages_yaml`.  This cannot be made
    /// public due to
    /// https://github.com/rustwasm/wasm-bindgen/issues/111
    fn get_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    /// Simulation steps generate messages, which are then consumed on
    /// subsequent simulation steps.  These messages between models in a
    /// simulation drive much of the discovery, analysis, and design.  This
    /// accessor method provides the list of active messages, at the current
    /// point of time in the simulation.  Message history is not retained, so
    /// simulation products and projects should collect messages as needed
    /// throughout the simulation execution.
    pub fn get_messages_js(&self) -> Array {
        // Workaround for https://github.com/rustwasm/wasm-bindgen/issues/111
        self.get_messages().into_iter().map(JsValue::from).collect()
    }

    /// This method is like `get_messages_js`, but returns the messages as JSON.
    pub fn get_messages_json(&self) -> String {
        serde_json::to_string(&self.get_messages()).unwrap()
    }

    /// This method is like `get_messages_js`, but returns the messages as YAML.
    pub fn get_messages_yaml(&self) -> String {
        serde_yaml::to_string(&self.get_messages()).unwrap()
    }

    /// An accessor method for the simulation global time.
    pub fn get_global_time(&self) -> f64 {
        self.global_time
    }

    /// This method provides a mechanism for getting the status of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the current status string for that model.
    pub fn status(&self, model_id: &str) -> String {
        self.models
            .iter()
            .find(|model| model.id() == model_id)
            .unwrap()
            .status()
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
    fn models(&mut self) -> Vec<&mut Box<dyn Model>> {
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

    /// This method is like `inject_input`, but the message is provided in
    /// JSON format.
    pub fn inject_input_json(&mut self, message: &str) {
        self.inject_input(serde_json::from_str(message).unwrap());
    }

    /// This method is like `inject_input`, but the message is provided in
    /// YAML format.
    pub fn inject_input_yaml(&mut self, message: &str) {
        self.inject_input(serde_yaml::from_str(message).unwrap());
    }

    /// The simulation step is foundational for a discrete event simulation.
    /// This method executes a single discrete event simulation step,
    /// including internal state transitions, external state transitions,
    /// message orchestration, and global time accounting.
    pub fn step(&mut self) {
        let messages = self.messages.clone();
        let mut next_messages: Vec<Message> = Vec::new();
        // Process external events and gather associated messages
        if !messages.is_empty() {
            (0..self.models.len()).for_each(|model_index| {
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
                model_messages.iter().for_each(|model_message| {
                    self.models[model_index]
                        .events_ext(&mut self.uniform_rng, model_message.clone())
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
                });
            });
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
        (0..self.models.len()).for_each(|model_index| {
            self.models[model_index]
                .events_int(&mut self.uniform_rng)
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
        });
        self.messages = next_messages;
    }

    /// The step_until implementation underpinning `step_until_js`,
    /// `step_until_json`, and `step_until_yaml`.  This cannot be made public
    /// due to
    /// https://github.com/rustwasm/wasm-bindgen/issues/111
    fn step_until(&mut self, until: f64) -> Vec<Message> {
        let mut message_records: Vec<Message> = Vec::new();
        loop {
            self.step();
            if self.global_time < until {
                message_records.extend(self.get_messages());
            } else {
                break;
            }
        }
        message_records
    }

    /// This method executes simulation `step` calls, until a global time
    /// has been exceeded.  At which point, the messages from all the
    /// simulation steps are returned.
    pub fn step_until_js(&mut self, until: f64) -> Array {
        self.step_until(until)
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// This method is like `step_until_js`, but returns the messages as
    /// JSON.
    pub fn step_until_json(&mut self, until: f64) -> String {
        serde_json::to_string(&self.step_until(until)).unwrap()
    }

    /// This method is like `step_until_js`, but returns the messages as
    /// YAML.
    pub fn step_until_yaml(&mut self, until: f64) -> String {
        serde_yaml::to_string(&self.step_until(until)).unwrap()
    }

    /// The step_n implementation underpinning `step_n_js`, `step_n_json`,
    /// and `step_n_yaml`.  This cannot be made public due to
    /// https://github.com/rustwasm/wasm-bindgen/issues/111
    fn step_n(&mut self, n: usize) -> Vec<Message> {
        let mut message_records: Vec<Message> = Vec::new();
        (0..n).for_each(|_| {
            self.step();
            message_records.extend(self.messages.clone());
        });
        message_records
    }

    /// This method executes the specified number of simulation steps, `n`.
    /// Upon execution of the n steps, the messages from all the steps are
    /// returned.
    pub fn step_n_js(&mut self, n: usize) -> Array {
        self.step_n(n).into_iter().map(JsValue::from).collect()
    }

    /// This method is like `step_n_js`, but returns the messages as JSON.
    pub fn step_n_json(&mut self, n: usize) -> String {
        serde_json::to_string(&self.step_n(n)).unwrap()
    }

    /// This method is like `step_n_js`, but returns the messages as YAML.
    pub fn step_n_yaml(&mut self, n: usize) -> String {
        serde_yaml::to_string(&self.step_n(n)).unwrap()
    }
}
