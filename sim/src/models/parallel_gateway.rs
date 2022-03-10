use std::collections::HashMap;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

/// The parallel gateway splits a job across multiple processing paths. The
/// job is duplicated across every one of the processing paths. In addition
/// to splitting the process, a second parallel gateway can be used to join
/// the split paths. The parallel gateway is a BPMN concept.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct ParallelGateway {
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsIn {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    FlowPath,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    until_next_event: f64,
    collections: HashMap<String, usize>,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            until_next_event: INFINITY,
            collections: HashMap::new(),
            records: Vec::new(),
        }
    }
}

#[cfg_attr(feature = "simx", event_rules)]
impl ParallelGateway {
    pub fn new(
        flow_paths_in: Vec<String>,
        flow_paths_out: Vec<String>,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                flow_paths: flow_paths_in,
            },
            ports_out: PortsOut {
                flow_paths: flow_paths_out,
            },
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if self.ports_in.flow_paths.contains(&message_port.to_string()) {
            ArrivalPort::FlowPath
        } else {
            ArrivalPort::Unknown
        }
    }

    fn full_collection(&self) -> Option<(&String, &usize)> {
        self.state
            .collections
            .iter()
            .find(|(_, count)| **count == self.ports_in.flow_paths.len())
    }

    fn increment_collection(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        *self
            .state
            .collections
            .entry(incoming_message.content.clone())
            .or_insert(0) += 1;
        self.record(
            services.global_time(),
            String::from("Arrival"),
            format![
                "{} on {}",
                incoming_message.content.clone(),
                incoming_message.port_name.clone()
            ],
        );
        self.state.until_next_event = 0.0;
    }

    fn send_job(&mut self, services: &mut Services) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.until_next_event = 0.0;
        let completed_collection = self
            .full_collection()
            .ok_or(SimulationError::InvalidModelState)?
            .0
            .to_string();
        self.state.collections.remove(&completed_collection);
        Ok(self
            .ports_out
            .flow_paths
            .clone()
            .iter()
            .fold(Vec::new(), |mut messages, flow_path| {
                self.record(
                    services.global_time(),
                    String::from("Departure"),
                    format!["{} on {}", completed_collection.clone(), flow_path.clone()],
                );
                messages.push(ModelMessage {
                    port_name: flow_path.clone(),
                    content: completed_collection.clone(),
                });
                messages
            }))
    }

    fn passivate(&mut self) -> Vec<ModelMessage> {
        self.state.until_next_event = INFINITY;
        Vec::new()
    }

    fn record(&mut self, time: f64, action: String, subject: String) {
        if self.store_records {
            self.state.records.push(ModelRecord {
                time,
                action,
                subject,
            });
        }
    }
}

#[cfg_attr(feature = "simx", event_rules)]
impl DevsModel for ParallelGateway {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match self.arrival_port(&incoming_message.port_name) {
            ArrivalPort::FlowPath => Ok(self.increment_collection(incoming_message, services)),
            ArrivalPort::Unknown => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match self.full_collection() {
            Some(_) => self.send_job(services),
            None => Ok(self.passivate()),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for ParallelGateway {
    fn status(&self) -> String {
        String::from("Active")
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for ParallelGateway {}
