use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::input_modeling::IndexRandomVariable;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

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
    jobs: Vec<Job>,    // port, message, time
    records: Vec<Job>, // port, message, time
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
    Respond, // Responding to a records request
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Job {
    arrival_port: String,
    departure_port: String,
    content: String,
    time: f64,
}

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
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                flow_paths: flow_paths_out,
                records: default_records_port_name(),
            },
            port_weights,
            store_records,
            state: Default::default(),
        }
    }

    fn pass_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::Pass;
        self.state.until_next_event = 0.0;
        let departure_port_index = self.port_weights.random_variate(services.uniform_rng())?;
        self.state.jobs.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.flow_paths[departure_port_index].clone(),
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
        let departure_port_index = self.port_weights.random_variate(services.uniform_rng())?;
        self.state.jobs.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.flow_paths[departure_port_index].clone(),
            content: incoming_message.content.clone(),
            time: services.global_time(),
        });
        self.state.records.push(Job {
            arrival_port: incoming_message.port_name.clone(),
            departure_port: self.ports_out.flow_paths[departure_port_index].clone(),
            content: incoming_message.content.clone(),
            time: services.global_time(),
        });
        Ok(())
    }

    fn records_request(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::Respond;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn ignore_request(&mut self) -> Result<(), SimulationError> {
        Ok(())
    }

    fn send_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn send_jobs(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
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

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }
}

impl AsModel for ExclusiveGateway {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Passive => String::from("Passive"),
            Phase::Pass => format!["Passing {}", self.state.jobs[0].content],
            Phase::Respond => String::from("Fetching records"),
        }
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.ports_in
                .flow_paths
                .contains(&incoming_message.port_name),
            self.store_records,
        ) {
            (true, true) => self.store_job(incoming_message, services),
            (true, false) => self.pass_job(incoming_message, services),
            (false, true) => self.records_request(),
            (false, false) => self.ignore_request(),
        }
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match &self.state.phase {
            Phase::Passive => self.passivate(),
            Phase::Pass => self.send_jobs(),
            Phase::Respond => self.send_records(),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}
