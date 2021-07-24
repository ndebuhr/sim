use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

/// The storage model stores a value, and responds with it upon request.
/// Values are stored and value requests are handled instantantaneously.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    put: String,
    get: String,
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Put,
    Get,
    Records,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    stored: String,
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    job: Option<String>,
    records: Vec<Job>,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            job: None,
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Passive,
    JobFetch,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub operation: Operation,
    pub content: Option<String>,
    pub time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    Get,
    Put,
}

impl Storage {
    pub fn new(
        put_port: String,
        get_port: String,
        stored_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                put: put_port,
                get: get_port,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                stored: stored_port,
                records: default_records_port_name(),
            },
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if message_port == self.ports_in.put {
            ArrivalPort::Put
        } else if message_port == self.ports_in.get {
            ArrivalPort::Get
        } else if message_port == self.ports_in.records {
            ArrivalPort::Records
        } else {
            ArrivalPort::Unknown
        }
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

    fn get_stored_value(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::JobFetch;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn save_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.job = Some(incoming_message.content.clone());
        self.state.records.push(Job {
            operation: Operation::Put,
            content: Some(incoming_message.content.clone()),
            time: services.global_time(),
        });
        Ok(())
    }

    fn hold_job(&mut self, incoming_message: &ModelMessage) -> Result<(), SimulationError> {
        self.state.job = Some(incoming_message.content.clone());
        Ok(())
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn release_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        match &self.state.job {
            Some(job) => Ok(vec![ModelMessage {
                port_name: self.ports_out.stored.clone(),
                content: job.clone(),
            }]),
            None => Ok(Vec::new()),
        }
    }

    fn save_and_release_job(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        self.state.records.push(Job {
            operation: Operation::Get,
            content: self.state.job.clone(),
            time: services.global_time(),
        });
        match &self.state.job {
            Some(job) => Ok(vec![ModelMessage {
                port_name: self.ports_out.stored.clone(),
                content: job.clone(),
            }]),
            None => Ok(Vec::new()),
        }
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }
}

impl DevsModel for Storage {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.arrival_port(&incoming_message.port_name),
            self.store_records,
        ) {
            (ArrivalPort::Records, true) => self.request_records(incoming_message, services),
            (ArrivalPort::Records, false) => self.ignore_request(incoming_message, services),
            (ArrivalPort::Put, true) => self.save_job(incoming_message, services),
            (ArrivalPort::Put, false) => self.hold_job(incoming_message),
            (ArrivalPort::Get, _) => self.get_stored_value(),
            (ArrivalPort::Unknown, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (&self.state.phase, self.store_records) {
            (Phase::RecordsFetch, _) => self.release_records(),
            (Phase::Passive, _) => self.passivate(),
            (Phase::JobFetch, true) => self.save_and_release_job(services),
            (Phase::JobFetch, false) => self.release_job(),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Storage {
    fn status(&self) -> String {
        match &self.state.job {
            Some(stored) => format!["Storing {}", stored],
            None => String::from("Empty"),
        }
    }
}

impl ReportableModel for Storage {}
