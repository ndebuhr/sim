use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    next_port_out: usize,
    jobs: Vec<String>,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            next_port_out: 0,
            jobs: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Passive,
    LoadBalancing,
}

#[cfg_attr(feature = "simx", event_rules)]
impl LoadBalancer {
    pub fn new(job_port: String, flow_path_ports: Vec<String>, store_records: bool) -> Self {
        Self {
            ports_in: PortsIn { job: job_port },
            ports_out: PortsOut {
                flow_paths: flow_path_ports,
            },
            store_records,
            state: State::default(),
        }
    }

    fn pass_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::LoadBalancing;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn passivate(&mut self) -> Vec<ModelMessage> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Vec::new()
    }

    fn send_job(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.until_next_event = 0.0;
        self.state.next_port_out = (self.state.next_port_out + 1) % self.ports_out.flow_paths.len();
        self.record(
            services.global_time(),
            String::from("Departure"),
            format![
                "{} on {}",
                self.state.jobs[0].clone(),
                self.ports_out.flow_paths[self.state.next_port_out].clone()
            ],
        );
        vec![ModelMessage {
            port_name: self.ports_out.flow_paths[self.state.next_port_out].clone(),
            content: self.state.jobs.remove(0),
        }]
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
impl DevsModel for LoadBalancer {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        Ok(self.pass_job(incoming_message, services))
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match self.state.jobs.len() {
            0 => Ok(self.passivate()),
            _ => Ok(self.send_job(services)),
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

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for LoadBalancer {}
