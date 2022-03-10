use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

/// The gate model passes or blocks jobs, when it is in the open or closed
/// state, respectively. The gate can be opened and closed throughout the
/// course of a simulation. This model contains no stochastic behavior - job
/// passing/blocking is based purely on the state of the model at that time
/// in the simulation. A blocked job is a dropped job - it is not stored,
/// queued, or redirected.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Gate {
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    job: String,
    activation: String,
    deactivation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Job,
    Activation,
    Deactivation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    jobs: Vec<String>,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Open,
            until_next_event: INFINITY,
            jobs: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Open,
    Closed,
    Pass,
}

#[cfg_attr(feature = "simx", event_rules)]
impl Gate {
    pub fn new(
        job_in_port: String,
        activation_port: String,
        deactivation_port: String,
        job_out_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                job: job_in_port,
                activation: activation_port,
                deactivation: deactivation_port,
            },
            ports_out: PortsOut { job: job_out_port },
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if message_port == self.ports_in.job {
            ArrivalPort::Job
        } else if message_port == self.ports_in.activation {
            ArrivalPort::Activation
        } else if message_port == self.ports_in.deactivation {
            ArrivalPort::Deactivation
        } else {
            ArrivalPort::Unknown
        }
    }

    fn activate(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Open;
        self.state.until_next_event = INFINITY;
        self.record(
            services.global_time(),
            String::from("Activation"),
            incoming_message.content.clone(),
        );
    }

    fn deactivate(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Closed;
        self.state.until_next_event = INFINITY;
        self.record(
            services.global_time(),
            String::from("Deactivation"),
            incoming_message.content.clone(),
        );
    }

    fn pass_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Pass;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn drop_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn send_jobs(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.phase = Phase::Open;
        self.state.until_next_event = INFINITY;
        (0..self.state.jobs.len())
            .map(|_| {
                self.record(
                    services.global_time(),
                    String::from("Departure"),
                    self.state.jobs[0].clone(),
                );
                ModelMessage {
                    port_name: self.ports_out.job.clone(),
                    content: self.state.jobs.remove(0),
                }
            })
            .collect()
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
impl DevsModel for Gate {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.arrival_port(&incoming_message.port_name),
            self.state.phase == Phase::Closed,
        ) {
            (ArrivalPort::Activation, _) => Ok(self.activate(incoming_message, services)),
            (ArrivalPort::Deactivation, _) => Ok(self.deactivate(incoming_message, services)),
            (ArrivalPort::Job, false) => Ok(self.pass_job(incoming_message, services)),
            (ArrivalPort::Job, true) => Ok(self.drop_job(incoming_message, services)),
            (ArrivalPort::Unknown, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        Ok(self.send_jobs(services))
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Gate {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Open => String::from("Open"),
            Phase::Closed => String::from("Closed"),
            Phase::Pass => format!["Passing {}", self.state.jobs[0]],
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Gate {}
