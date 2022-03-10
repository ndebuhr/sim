use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{Model, ModelMessage, ModelRecord};

use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

#[derive(Clone, Deserialize, Serialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Coupled {
    ports_in: PortsIn,
    ports_out: PortsOut,
    components: Vec<Model>,
    external_input_couplings: Vec<ExternalInputCoupling>,
    external_output_couplings: Vec<ExternalOutputCoupling>,
    internal_couplings: Vec<InternalCoupling>,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsIn {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInputCoupling {
    #[serde(rename = "targetID")]
    pub target_id: String,
    pub source_port: String,
    pub target_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalOutputCoupling {
    #[serde(rename = "sourceID")]
    pub source_id: String,
    pub source_port: String,
    pub target_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalCoupling {
    #[serde(rename = "sourceID")]
    pub source_id: String,
    #[serde(rename = "targetID")]
    pub target_id: String,
    pub source_port: String,
    pub target_port: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    parked_messages: Vec<ParkedMessage>,
    records: Vec<ModelRecord>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParkedMessage {
    component_id: String,
    port: String,
    content: String,
}

#[cfg_attr(feature = "simx", event_rules)]
impl Coupled {
    pub fn new(
        ports_in: Vec<String>,
        ports_out: Vec<String>,
        components: Vec<Model>,
        external_input_couplings: Vec<ExternalInputCoupling>,
        external_output_couplings: Vec<ExternalOutputCoupling>,
        internal_couplings: Vec<InternalCoupling>,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                flow_paths: ports_in,
            },
            ports_out: PortsOut {
                flow_paths: ports_out,
            },
            components,
            external_input_couplings,
            external_output_couplings,
            internal_couplings,
            state: State::default(),
        }
    }

    fn park_incoming_messages(
        &self,
        incoming_message: &ModelMessage,
    ) -> Option<Vec<ParkedMessage>> {
        let parked_messages: Vec<ParkedMessage> = self
            .external_input_couplings
            .iter()
            .filter_map(|coupling| {
                if coupling.source_port == incoming_message.port_name {
                    Some(ParkedMessage {
                        component_id: coupling.target_id.to_string(),
                        port: coupling.target_port.to_string(),
                        content: incoming_message.content.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if parked_messages.is_empty() {
            None
        } else {
            Some(parked_messages)
        }
    }

    fn external_output_targets(&self, source_id: &str, source_port: &str) -> Vec<String> {
        // Vec<target_port>

        self.external_output_couplings
            .iter()
            .filter_map(|coupling| {
                if coupling.source_id == source_id && coupling.source_port == source_port {
                    Some(coupling.target_port.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn internal_targets(&self, source_id: &str, source_port: &str) -> Vec<(String, String)> {
        // Vec<(target_id, target_port)>

        self.internal_couplings
            .iter()
            .filter_map(|coupling| {
                if coupling.source_id == source_id && coupling.source_port == source_port {
                    Some((
                        coupling.target_id.to_string(),
                        coupling.target_port.to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    fn distribute_events_ext(
        &mut self,
        parked_messages: &[ParkedMessage],
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        parked_messages.iter().try_for_each(|parked_message| {
            self.components
                .iter_mut()
                .find(|component| component.id() == parked_message.component_id)
                .unwrap()
                .events_ext(
                    &ModelMessage {
                        port_name: parked_message.port.to_string(),
                        content: parked_message.content.to_string(),
                    },
                    services,
                )
        })
    }

    fn distribute_events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        // Find the (internal message) events_ext relevant models (parked message id == component id)
        let ext_transitioning_component_triggers: Vec<(usize, String, String)> = (0..self
            .components
            .len())
            .flat_map(|component_index| -> Vec<(usize, String, String)> {
                self.state
                    .parked_messages
                    .iter()
                    .filter_map(|parked_message| {
                        if parked_message.component_id == self.components[component_index].id() {
                            Some((
                                component_index,
                                parked_message.port.to_string(),
                                parked_message.content.to_string(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();
        ext_transitioning_component_triggers
            .iter()
            .map(
                |(component_index, message_port, message_content)| -> Result<(), SimulationError> {
                    self.components[*component_index].events_ext(
                        &ModelMessage {
                            port_name: message_port.to_string(),
                            content: message_content.to_string(),
                        },
                        services,
                    )
                },
            )
            .collect::<Result<Vec<()>, SimulationError>>()?;
        self.state.parked_messages = Vec::new();
        // Find the events_int relevant models (until_next_event == 0.0)
        // Run events_int for each model, and compile the internal and external messages
        // Store the internal messages in the Coupled model struct, and output the external messages
        let int_transitioning_component_indexes: Vec<usize> = (0..self.components.len())
            .filter(|component_index| self.components[*component_index].until_next_event() == 0.0)
            .collect();
        Ok(int_transitioning_component_indexes
            .iter()
            .flat_map(
                |component_index| -> Result<Vec<ModelMessage>, SimulationError> {
                    Ok(self.components[*component_index]
                        .events_int(services)?
                        .iter()
                        .flat_map(|outgoing_message| -> Vec<ModelMessage> {
                            // For internal messages (those transmitted on internal couplings), store the messages
                            // as Parked Messages, to be ingested by the target components on the next simulation step
                            self.internal_targets(
                                self.components[*component_index].id(),
                                &outgoing_message.port_name,
                            )
                            .iter()
                            .for_each(|(target_id, target_port)| {
                                self.state.parked_messages.push(ParkedMessage {
                                    component_id: target_id.to_string(),
                                    port: target_port.to_string(),
                                    content: outgoing_message.content.clone(),
                                });
                            });
                            // For external messages (those transmitted on external output couplings), prepare the
                            // output as standard events_int output
                            self.external_output_targets(
                                self.components[*component_index].id(),
                                &outgoing_message.port_name,
                            )
                            .iter()
                            .map(|target_port| ModelMessage {
                                port_name: target_port.to_string(),
                                content: outgoing_message.content.clone(),
                            })
                            .collect()
                        })
                        .collect())
                },
            )
            .flatten()
            .collect())
    }
}

#[cfg_attr(feature = "simx", event_rules)]
impl DevsModel for Coupled {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match self.park_incoming_messages(incoming_message) {
            None => Ok(()),
            Some(parked_messages) => self.distribute_events_ext(&parked_messages, services),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.distribute_events_int(services)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.components.iter_mut().for_each(|component| {
            component.time_advance(time_delta);
        });
    }

    fn until_next_event(&self) -> f64 {
        self.components.iter().fold(INFINITY, |min, component| {
            f64::min(min, component.until_next_event())
        })
    }
}

impl Reportable for Coupled {
    fn status(&self) -> String {
        if self.state.parked_messages.is_empty() {
            format!["Processing {} messages", self.state.parked_messages.len()]
        } else {
            String::from("Processing no messages")
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Coupled {}
