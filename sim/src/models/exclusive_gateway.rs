use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::input_modeling::IndexRandomVariable;
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

/// The exclusive gateway splits a process flow into a set of possible paths.
/// The process will only follow one of the possible paths. Path selection is
/// determined by Weighted Index distribution random variates, so this atomic
/// model exhibits stochastic behavior. The exclusive gateway is a BPMN
/// concept.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct ExclusiveGateway {
    ports_in: PortsIn,
    ports_out: PortsOut,
    port_weights: IndexRandomVariable,
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
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    jobs: Vec<String>,         // port, message, time
    records: Vec<ModelRecord>, // port, message, time
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            jobs: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Passive, // Doing nothing
    Pass,    // Passing a job from input to output
}

#[cfg_attr(feature = "simx", event_rules)]
impl ExclusiveGateway {
    pub fn new(
        flow_paths_in: Vec<String>,
        flow_paths_out: Vec<String>,
        port_weights: IndexRandomVariable,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                flow_paths: flow_paths_in,
            },
            ports_out: PortsOut {
                flow_paths: flow_paths_out,
            },
            port_weights,
            store_records,
            state: State::default(),
        }
    }

    fn pass_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Pass;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            format![
                "{} on {}",
                incoming_message.content.clone(),
                incoming_message.port_name
            ],
        );
    }

    fn send_jobs(&mut self, services: &mut Services) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        let departure_port_index = self.port_weights.random_variate(services.uniform_rng())?;
        Ok((0..self.state.jobs.len())
            .map(|_| {
                self.record(
                    services.global_time(),
                    String::from("Departure"),
                    format![
                        "{} on {}",
                        self.state.jobs[0].clone(),
                        self.ports_out.flow_paths[departure_port_index].clone()
                    ],
                );
                ModelMessage {
                    port_name: self.ports_out.flow_paths[departure_port_index].clone(),
                    content: self.state.jobs.remove(0),
                }
            })
            .collect())
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
impl DevsModel for ExclusiveGateway {
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
        match &self.state.phase {
            Phase::Passive => Ok(self.passivate()),
            Phase::Pass => self.send_jobs(services),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for ExclusiveGateway {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Passive => String::from("Passive"),
            Phase::Pass => format!["Passing {}", self.state.jobs[0]],
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for ExclusiveGateway {}
