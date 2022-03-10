use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::input_modeling::BooleanRandomVariable;
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

/// The stochastic gate blocks (drops) or passes jobs, based on a specified
/// Bernoulli distribution. If the Bernoulli random variate is a 0, the job
/// will be dropped. If the Bernoulli random variate is a 1, the job will be
/// passed.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct StochasticGate {
    pass_distribution: BooleanRandomVariable,
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
enum ArrivalPort {
    Job,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    until_next_event: f64,
    jobs: Vec<Job>,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        State {
            until_next_event: INFINITY,
            jobs: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub content: String,
    pub pass: bool,
}

#[cfg_attr(feature = "simx", event_rules)]
impl StochasticGate {
    pub fn new(
        pass_distribution: BooleanRandomVariable,
        job_in_port: String,
        job_out_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            pass_distribution,
            ports_in: PortsIn { job: job_in_port },
            ports_out: PortsOut { job: job_out_port },
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if message_port == self.ports_in.job {
            ArrivalPort::Job
        } else {
            ArrivalPort::Unknown
        }
    }

    fn receive_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            pass: self
                .pass_distribution
                .random_variate(services.uniform_rng())?,
        });
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
        Ok(())
    }

    fn passivate(&mut self) -> Vec<ModelMessage> {
        self.state.until_next_event = INFINITY;
        Vec::new()
    }

    fn pass_job(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.until_next_event = 0.0;
        let job = self.state.jobs.remove(0);
        self.record(
            services.global_time(),
            String::from("Pass"),
            job.content.clone(),
        );
        vec![ModelMessage {
            content: job.content,
            port_name: self.ports_out.job.clone(),
        }]
    }

    fn block_job(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.until_next_event = 0.0;
        let job = self.state.jobs.remove(0);
        self.record(services.global_time(), String::from("Block"), job.content);
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
impl DevsModel for StochasticGate {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match self.arrival_port(&incoming_message.port_name) {
            ArrivalPort::Job => self.receive_job(incoming_message, services),
            ArrivalPort::Unknown => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match self.state.jobs.get(0) {
            None => Ok(self.passivate()),
            Some(Job { pass: true, .. }) => Ok(self.pass_job(services)),
            Some(Job { pass: false, .. }) => Ok(self.block_job(services)),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for StochasticGate {
    fn status(&self) -> String {
        String::from("Gating")
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for StochasticGate {}
