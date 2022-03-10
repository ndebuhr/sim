use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Put,
    Get,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    stored: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    job: Option<String>,
    records: Vec<ModelRecord>,
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
}

#[cfg_attr(feature = "simx", event_rules)]
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
            },
            ports_out: PortsOut {
                stored: stored_port,
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
        } else {
            ArrivalPort::Unknown
        }
    }

    fn get_job(&mut self) {
        self.state.phase = Phase::JobFetch;
        self.state.until_next_event = 0.0;
    }

    fn hold_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.job = Some(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn release_job(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        self.record(
            services.global_time(),
            String::from("Departure"),
            self.state.job.clone().unwrap_or_else(|| "None".to_string()),
        );
        match &self.state.job {
            Some(job) => vec![ModelMessage {
                port_name: self.ports_out.stored.clone(),
                content: job.clone(),
            }],
            None => Vec::new(),
        }
    }

    fn passivate(&mut self) -> Vec<ModelMessage> {
        self.state.phase = Phase::Passive;
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
impl DevsModel for Storage {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match self.arrival_port(&incoming_message.port_name) {
            ArrivalPort::Put => Ok(self.hold_job(incoming_message, services)),
            ArrivalPort::Get => Ok(self.get_job()),
            ArrivalPort::Unknown => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match &self.state.phase {
            Phase::Passive => Ok(self.passivate()),
            Phase::JobFetch => Ok(self.release_job(services)),
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

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Storage {}
