use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

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
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Job,
    Activation,
    Deactivation,
    Records,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    jobs: Vec<Job>,
    records: Vec<Job>,
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
    RespondWhileOpen,
    RespondWhileClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Job {
    arrival_port: String,
    departure_port: String,
    content: String,
    time: f64,
}

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
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                job: job_out_port,
                records: default_records_port_name(),
            },
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
        } else if message_port == self.ports_in.records {
            ArrivalPort::Records
        } else {
            ArrivalPort::Unknown
        }
    }

    fn activate(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::Open;
        self.state.until_next_event = INFINITY;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::Closed;
        self.state.until_next_event = INFINITY;
        Ok(())
    }

    fn pass_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::Pass;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.job.clone(),
            content: incoming_message.content.clone(),
            time: services.global_time(),
        });
        Ok(())
    }

    fn store_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::Pass;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.job.clone(),
            content: incoming_message.content.clone(),
            time: services.global_time(),
        });
        self.state.records.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.job.clone(),
            content: incoming_message.content.clone(),
            time: services.global_time(),
        });
        Ok(())
    }

    fn drop_job(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        // Do nothing
        Ok(())
    }

    fn records_request_while_open(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::RespondWhileOpen;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn records_request_while_closed(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::RespondWhileClosed;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn send_jobs(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Open;
        self.state.until_next_event = INFINITY;
        Ok((0..self.state.jobs.len())
            .map(|_| {
                let job = self.state.jobs.remove(0);
                ModelMessage {
                    port_name: job.departure_port,
                    content: job.content,
                }
            })
            .collect())
    }

    fn send_records_while_open(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Open;
        self.state.until_next_event = INFINITY;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn send_records_while_closed(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Closed;
        self.state.until_next_event = INFINITY;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }
}

impl AsModel for Gate {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Open => String::from("Open"),
            Phase::Closed => String::from("Closed"),
            Phase::Pass => format!["Passing {}", self.state.jobs[0].content],
            Phase::RespondWhileOpen | Phase::RespondWhileClosed => String::from("Fetching records"),
        }
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.arrival_port(&incoming_message.port_name),
            self.state.phase == Phase::Closed,
            self.store_records,
        ) {
            (ArrivalPort::Activation, _, _) => self.activate(),
            (ArrivalPort::Deactivation, _, _) => self.deactivate(),
            (ArrivalPort::Job, false, false) => self.pass_job(incoming_message, services),
            (ArrivalPort::Job, false, true) => self.store_job(incoming_message, services),
            (ArrivalPort::Job, true, _) => self.drop_job(incoming_message, services),
            (ArrivalPort::Records, false, _) => self.records_request_while_open(),
            (ArrivalPort::Records, true, _) => self.records_request_while_closed(),
            (ArrivalPort::Unknown, _, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match &self.state.phase {
            Phase::Open | Phase::Closed | Phase::Pass => self.send_jobs(),
            Phase::RespondWhileOpen => self.send_records_while_open(),
            Phase::RespondWhileClosed => self.send_records_while_closed(),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}
