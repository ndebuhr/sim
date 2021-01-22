use std::f64::INFINITY;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::input_modeling::uniform_rng::UniformRNG;
use crate::models::model::Model;
use crate::models::ModelMessage;
use crate::utils;

mod test_simulations;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    source_id: String,
    source_port: String,
    target_id: String,
    target_port: String,
    time: f64,
    message: String,
}

#[wasm_bindgen]
impl Simulation {
    pub fn post_json(models: String, connectors: String) -> Simulation {
        utils::set_panic_hook();
        Simulation {
            models: serde_json::from_str(&models).unwrap(),
            connectors: serde_json::from_str(&connectors).unwrap(),
            ..Default::default()
        }
    }

    pub fn put_json(&mut self, models: String, connectors: String) {
        self.models = serde_json::from_str(&models).unwrap();
        self.connectors = serde_json::from_str(&connectors).unwrap();
    }

    pub fn get_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn post_yaml(models: String, connectors: String) -> Simulation {
        utils::set_panic_hook();
        Simulation {
            models: serde_yaml::from_str(&models).unwrap(),
            connectors: serde_yaml::from_str(&connectors).unwrap(),
            ..Default::default()
        }
    }

    pub fn put_yaml(&mut self, models: String, connectors: String) {
        self.models = serde_yaml::from_str(&models).unwrap();
        self.connectors = serde_yaml::from_str(&connectors).unwrap();
    }

    pub fn get_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn reset(&mut self) {
        // Resets to enable simulation replications without uniform RNG reset
        self.reset_messages();
        self.reset_global_time();
    }

    pub fn reset_messages(&mut self) {
        self.messages = Vec::new();
    }

    pub fn reset_global_time(&mut self) {
        self.global_time = 0.0;
    }

    fn models(&mut self) -> Vec<&mut Box<dyn Model>> {
        self.models.iter_mut().collect()
    }

    pub fn messages(&self) -> String {
        serde_json::to_string(&self.messages).unwrap()
    }

    pub fn global_time(&self) -> f64 {
        self.global_time
    }

    pub fn status(&self, id: String) -> String {
        self.models
            .iter()
            .find(|model| model.id() == id)
            .unwrap()
            .status()
    }

    fn get_message_target_ids(&self, source_id: String, source_port: String) -> Vec<String> {
        self.connectors
            .iter()
            .filter(|connector| {
                connector.source_id == source_id && connector.source_port == source_port
            })
            .map(|connector| connector.target_id.clone())
            .collect()
    }

    fn get_message_target_ports(&self, source_id: String, source_port: String) -> Vec<String> {
        self.connectors
            .iter()
            .filter(|connector| {
                connector.source_id == source_id && connector.source_port == source_port
            })
            .map(|connector| connector.target_port.clone())
            .collect()
    }

    pub fn inject_input(&mut self, message: String) {
        self.messages.push(serde_json::from_str(&message).unwrap());
    }

    pub fn step(&mut self) {
        let messages = self.messages.clone();
        let mut next_messages: Vec<Message> = Vec::new();
        // Process external events and gather associated messages
        if !messages.is_empty() {
            (0..self.models.len()).for_each(|model_index| {
                let model_messages: Vec<ModelMessage> = messages
                    .iter()
                    .filter(|message| message.target_id == self.models[model_index].id())
                    .map(|message| ModelMessage {
                        port_name: message.target_port.clone(),
                        message: message.message.clone(),
                    })
                    .collect();
                model_messages.iter().for_each(|model_message| {
                    self.models[model_index]
                        .events_ext(&mut self.uniform_rng, model_message.clone())
                        .iter()
                        .for_each(|outgoing_message| {
                            let target_ids = self.get_message_target_ids(
                                self.models[model_index].id(),      // Outgoing message source model ID
                                outgoing_message.port_name.clone(), // Outgoing message source model port
                            );
                            let target_ports = self.get_message_target_ports(
                                self.models[model_index].id(),      // Outgoing message source model ID
                                outgoing_message.port_name.clone(), // Outgoing message source model port
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
                        self.models[model_index].id(),      // Outgoing message source model ID
                        outgoing_message.port_name.clone(), // Outgoing message source model port
                    );
                    let target_ports = self.get_message_target_ports(
                        self.models[model_index].id(),      // Outgoing message source model ID
                        outgoing_message.port_name.clone(), // Outgoing message source model port
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

    pub fn step_until(&mut self, until: f64) -> String {
        let mut message_records: Vec<Message> = Vec::new();
        loop {
            self.step();
            if self.global_time < until {
                let messages_set: Vec<Message> = serde_json::from_str(&self.messages()).unwrap();
                message_records.extend(messages_set);
            } else {
                break;
            }
        }
        serde_json::to_string(&message_records).unwrap()
    }

    pub fn step_n(&mut self, n: usize) -> String {
        let mut message_records: Vec<Message> = Vec::new();
        (0..n).for_each(|_| {
            self.step();
            let messages_set: Vec<Message> = serde_json::from_str(&self.messages()).unwrap();
            message_records.extend(messages_set);
        });
        serde_json::to_string(&message_records).unwrap()
    }
}
