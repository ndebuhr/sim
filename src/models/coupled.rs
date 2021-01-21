use std::any::Any;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::Model;
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::simulator::{Connector, Message};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Coupled {
    id: String,
    ports_in: PortsIn,
    ports_out: PortsOut,
    models: Vec<Box<dyn Model>>,
    connectors: Vec<Connector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    input: Vec<String>,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    output: Vec<String>,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    messages: Vec<Message>,
    global_time: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            messages: Vec::new(),
            global_time: 0.0,
        }
    }
}

impl Coupled {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
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
}

impl Model for Coupled {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn status(&self) -> String {
        String::from("Active")
    }

    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Vec<ModelMessage> {
        let mut incoming_messages: Vec<Message> = Vec::new();
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        // Get atomic model message targets
        let target_ids = self.get_message_target_ids(
            self.id(),  // Outgoing message source model ID
            incoming_message.port_name.clone(),  // Outgoing message source model port
        );
        let target_ports = self.get_message_target_ports(
            self.id(),  // Outgoing message source model ID
            incoming_message.port_name.clone(),  // Outgoing message source model port
        );
        // Build full message objects for incoming atomic model messages
        target_ids.iter().zip(target_ports.iter()).for_each(
            |(target_id, target_port)| {
                incoming_messages.push(Message {
                    source_id: self.models[model_index].id(),
                    source_port: outgoing_message.port_name.clone(),
                    target_id: target_id.clone(),
                    target_port: target_port.clone(),
                    time: self.global_time,
                    message: outgoing_message.message.clone(),
                });
            }
        );
        // Create vector of incoming atomic model messages
        let model_messages: Vec<ModelMessage> = incoming_messages
            .iter()
            .filter(|message| message.target_id == self.models[model_index].id())
            .map(|message| ModelMessage {
                port_name: message.target_port.clone(),
                message: message.message.clone(),
            })
            .collect();
        // Run the atomic model external events
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
                            messages.push(Message {
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
        // Residual messages for atomic model interactions next step
        self.state.messages = messages.iter()
            .filter(|message| {
                message.
            })
        // Output outgoing messages from the coupled parent
        messages.
    }

    fn events_int(&mut self, _uniform_rng: &mut UniformRNG) -> Vec<ModelMessage> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let events = self.state.event_list.clone();
        self.state.event_list = self
            .state
            .event_list
            .iter()
            .filter(|scheduled_event| scheduled_event.time != 0.0)
            .cloned()
            .collect();
        events
            .iter()
            .filter(|scheduled_event| scheduled_event.time == 0.0)
            .for_each(|scheduled_event| match scheduled_event.event {
                Event::Run => {}
                Event::DropJob => {
                    // Possible metrics updates
                    if self.need_snapshot_metrics() {
                        self.snapshot.block_count += 1;
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // Execution
                    self.state.jobs.remove(0);
                }
                Event::SendJob => {
                    // Possible metrics updates
                    if self.need_snapshot_metrics() {
                        self.snapshot.pass_count += 1;
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // Execution
                    outgoing_messages.push(ModelMessage {
                        port_name: self.ports_out.job.clone(),
                        message: self.state.jobs.remove(0),
                    });
                }
            });
        outgoing_messages
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state
            .event_list
            .iter_mut()
            .for_each(|scheduled_event| {
                scheduled_event.time -= time_delta;
            });
        self.state.global_time += time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state
            .event_list
            .iter()
            .fold(INFINITY, |until_next_event, event| {
                f64::min(until_next_event, event.time)
            })
    }
}
