use std::collections::HashMap;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::error::SimulationError;

use sim_derive::SerializableModel;

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
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    collections: HashMap<String, usize>,
    records: Vec<Record>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Processing,
            until_next_event: 0.0,
            collections: HashMap::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Processing,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    content: String,
    time: f64,
}

impl ParallelGateway {
    pub fn new(
        flow_paths_in: Vec<String>,
        flow_paths_out: Vec<String>,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                flow_paths: flow_paths_in,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                flow_paths: flow_paths_out,
                records: default_records_port_name(),
            },
            store_records,
            state: Default::default(),
        }
    }

    fn full_collection(&self) -> Option<(&String, &usize)> {
        self.state
            .collections
            .iter()
            .find(|(_, count)| **count == self.ports_in.flow_paths.len())
    }

    fn increment_collection(
        &mut self,
        incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        *self
            .state
            .collections
            .entry(incoming_message.content.clone())
            .or_insert(0) += 1;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn request_records(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::RecordsFetch;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn ignore_request(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        Ok(())
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Processing;
        self.state.until_next_event = 0.0;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn send_and_save_job(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.until_next_event = 0.0;
        let completed_collection = self
            .full_collection()
            .ok_or(SimulationError::InvalidModelState)?
            .0
            .to_string();
        self.state.collections.remove(&completed_collection);
        self.state.records.push(Record {
            content: completed_collection.clone(),
            time: services.global_time(),
        });
        Ok(self
            .ports_out
            .flow_paths
            .iter()
            .fold(Vec::new(), |mut messages, flow_path| {
                messages.push(ModelMessage {
                    port_name: flow_path.clone(),
                    content: completed_collection.clone(),
                });
                messages
            }))
    }

    fn send_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
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
            .iter()
            .fold(Vec::new(), |mut messages, flow_path| {
                messages.push(ModelMessage {
                    port_name: flow_path.clone(),
                    content: completed_collection.clone(),
                });
                messages
            }))
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Processing;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }
}

impl AsModel for ParallelGateway {
    fn status(&self) -> String {
        String::from("Active")
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if incoming_message.port_name == self.ports_in.records && self.store_records {
            self.request_records(incoming_message, services)?;
        } else if incoming_message.port_name == self.ports_in.records && !self.store_records {
            self.ignore_request(incoming_message, services)?;
        } else if incoming_message.port_name != self.ports_in.records {
            self.increment_collection(incoming_message, services)?;
        } else {
            return Err(SimulationError::InvalidModelState);
        }
        Ok(Vec::new())
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if self.state.phase == Phase::RecordsFetch {
            self.release_records()
        } else if self.state.phase == Phase::Processing
            && self.store_records
            && self.full_collection().is_some()
        {
            self.send_and_save_job(services)
        } else if self.state.phase == Phase::Processing
            && !self.store_records
            && self.full_collection().is_some()
        {
            self.send_job()
        } else if self.state.phase == Phase::Processing && self.full_collection().is_none() {
            self.passivate()
        } else {
            Err(SimulationError::InvalidModelState)
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}
