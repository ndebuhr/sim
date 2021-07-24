use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

/// The load balancer routes jobs to a set of possible process paths, using a
/// round robin strategy. There is no stochastic behavior in this model.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct LoadBalancer {
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
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Job,
    Records,
    Unknown,
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
    next_port_out: usize,
    jobs: Vec<Job>,
    records: Vec<Job>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::LoadBalancing,
            until_next_event: 0.0,
            next_port_out: 0,
            jobs: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    LoadBalancing,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Job {
    content: String,
    time: f64,
    port_out: String,
}

impl LoadBalancer {
    pub fn new(job_port: String, flow_path_ports: Vec<String>, store_records: bool) -> Self {
        Self {
            ports_in: PortsIn {
                job: job_port,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                flow_paths: flow_path_ports,
                records: default_records_port_name(),
            },
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if message_port == self.ports_in.job {
            ArrivalPort::Job
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

    fn pass_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::LoadBalancing;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            port_out: self.ports_out.flow_paths[self.state.next_port_out].clone(),
        });
        self.state.next_port_out = (self.state.next_port_out + 1) % self.ports_out.flow_paths.len();
        Ok(())
    }

    fn save_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::LoadBalancing;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            port_out: self.ports_out.flow_paths[self.state.next_port_out].clone(),
        });
        self.state.records.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            port_out: self.ports_out.flow_paths[self.state.next_port_out].clone(),
        });
        self.state.next_port_out = (self.state.next_port_out + 1) % self.ports_out.flow_paths.len();
        Ok(())
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::LoadBalancing;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }

    fn send_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.until_next_event = 0.0;
        let job = self.state.jobs.remove(0);
        Ok(vec![ModelMessage {
            port_name: job.port_out,
            content: job.content,
        }])
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::LoadBalancing;
        self.state.until_next_event = 0.0;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }
}

impl DevsModel for LoadBalancer {
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
            (ArrivalPort::Job, true) => self.save_job(incoming_message, services),
            (ArrivalPort::Job, false) => self.pass_job(incoming_message, services),
            (ArrivalPort::Unknown, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (&self.state.phase, self.state.jobs.is_empty()) {
            (Phase::RecordsFetch, _) => self.release_records(),
            (Phase::LoadBalancing, true) => self.passivate(),
            (Phase::LoadBalancing, false) => self.send_job(),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for LoadBalancer {
    fn status(&self) -> String {
        format!["Listening for {}s", self.ports_in.job]
    }
}

impl ReportableModel for LoadBalancer {}
