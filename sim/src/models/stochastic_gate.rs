use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::input_modeling::random_variable::BooleanRandomVariable;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::error::SimulationError;

use sim_derive::SerializableModel;

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
    #[serde(default = "default_records_port_name")]
    records: String,
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
    Passive,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub content: String,
    pub time: f64,
    pub pass: bool,
}

impl StochasticGate {
    pub fn new(
        pass_distribution: BooleanRandomVariable,
        job_in_port: String,
        job_out_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            pass_distribution,
            ports_in: PortsIn {
                job: job_in_port,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                job: job_out_port,
                records: default_records_port_name(),
            },
            store_records,
            state: Default::default(),
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

    fn accept_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.until_next_event = 0.0;
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            pass: self
                .pass_distribution
                .random_variate(services.uniform_rng())?,
        });
        Ok(())
    }

    fn save_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.until_next_event = 0.0;
        let pass = self
            .pass_distribution
            .random_variate(services.uniform_rng())?;
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            pass,
        });
        self.state.jobs.push(Job {
            content: incoming_message.content.clone(),
            time: services.global_time(),
            pass,
        });
        Ok(())
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }

    fn release_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        Ok(vec![ModelMessage {
            content: self.state.jobs.remove(0).content,
            port_name: self.ports_out.job.clone(),
        }])
    }

    fn block_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        self.state.jobs.remove(0);
        Ok(Vec::new())
    }
}

impl AsModel for StochasticGate {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Passive => String::from("Gating"),
            Phase::RecordsFetch => String::from("Fetching Records"),
        }
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
        } else if incoming_message.port_name == self.ports_in.job && self.store_records {
            self.save_job(incoming_message, services)?;
        } else if incoming_message.port_name == self.ports_in.job && !self.store_records {
            self.accept_job(incoming_message, services)?;
        } else {
            return Err(SimulationError::InvalidModelState);
        }
        Ok(Vec::new())
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if self.state.phase == Phase::RecordsFetch {
            self.release_records()
        } else if self.state.phase == Phase::Passive && self.state.jobs.is_empty() {
            self.passivate()
        } else if self.state.phase == Phase::Passive
            && !self.state.jobs.is_empty()
            && self.state.jobs[0].pass
        {
            self.release_job()
        } else if self.state.phase == Phase::Passive
            && !self.state.jobs.is_empty()
            && !self.state.jobs[0].pass
        {
            self.block_job()
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
