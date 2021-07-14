use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::input_modeling::ContinuousRandomVariable;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

/// The processor accepts jobs, processes them for a period of time, and then
/// outputs a processed job. The processor can have a configurable queue, of
/// size 0 to infinity, inclusive. The default queue size is infinite. The
/// queue allows collection of jobs as other jobs are processed. A FIFO
/// strategy is employed for the processing of incoming jobs. A random
/// variable distribution dictates the amount of time required to process a
/// job. For non-stochastic behavior, a random variable distribution with a
/// single point can be used - in which case, every job takes exactly the
/// specified amount of time to process.
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Processor {
    service_time: ContinuousRandomVariable,
    #[serde(default = "max_usize")]
    queue_capacity: usize,
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

fn max_usize() -> usize {
    usize::MAX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    job: String,
    #[serde(default = "default_records_port_name")]
    records: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    until_job_completion: f64,
    queue: Vec<Job>,
    records: Vec<Job>,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            until_job_completion: INFINITY,
            queue: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Active,
    Passive,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub content: String,
    pub arrival_time: f64,
    pub processing_start_time: f64,
    pub departure_time: f64,
}

impl Processor {
    pub fn new(
        service_time: ContinuousRandomVariable,
        queue_capacity: Option<usize>,
        job_port: String,
        processed_job_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            service_time,
            queue_capacity: queue_capacity.unwrap_or(usize::MAX),
            ports_in: PortsIn {
                job: job_port,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                job: processed_job_port,
                records: default_records_port_name(),
            },
            store_records,
            state: Default::default(),
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
        self.state.until_job_completion = self.state.until_next_event;
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

    fn add_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.queue.push(Job {
            content: incoming_message.content.clone(),
            arrival_time: services.global_time(),
            processing_start_time: INFINITY,
            departure_time: INFINITY,
        });
        Ok(())
    }

    fn start_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.queue.push(Job {
            content: incoming_message.content.clone(),
            arrival_time: services.global_time(),
            processing_start_time: INFINITY,
            departure_time: INFINITY,
        });
        self.state.phase = Phase::Passive;
        self.state.until_job_completion =
            self.service_time.random_variate(services.uniform_rng())?;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn ignore_job(&mut self) -> Result<(), SimulationError> {
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

    fn resume_processing(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Active;
        self.state.until_next_event = self.state.until_job_completion;
        self.state.queue[0].processing_start_time = f64::min(
            self.state.queue[0].processing_start_time,
            services.global_time(),
        );
        Ok(Vec::new())
    }

    fn save_and_release_job(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let mut job = self.state.queue.remove(0);
        job.departure_time = services.global_time();
        self.state.records.push(job.clone());
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        self.state.until_job_completion =
            self.service_time.random_variate(services.uniform_rng())?;
        Ok(vec![ModelMessage {
            content: job.content,
            port_name: self.ports_out.job.clone(),
        }])
    }

    fn release_job(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let job = self.state.queue.remove(0);
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        self.state.until_job_completion =
            self.service_time.random_variate(services.uniform_rng())?;
        Ok(vec![ModelMessage {
            content: job.content,
            port_name: self.ports_out.job.clone(),
        }])
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }
}

impl AsModel for Processor {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::RecordsFetch => String::from("Fetching Records"),
            Phase::Active => String::from("Processing"),
            Phase::Passive => String::from("Passive"),
        }
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.arrival_port(&incoming_message.port_name),
            &self.state.phase,
            self.store_records,
            self.state.queue.len() < self.queue_capacity,
        ) {
            (ArrivalPort::Records, _, true, _) => self.request_records(incoming_message, services),
            (ArrivalPort::Records, _, false, _) => self.ignore_request(incoming_message, services),
            (ArrivalPort::Job, Phase::Active, _, true) => self.add_job(incoming_message, services),
            (ArrivalPort::Job, Phase::Passive, _, true) => {
                self.start_job(incoming_message, services)
            }
            (ArrivalPort::Job, Phase::RecordsFetch, _, true) => {
                self.add_job(incoming_message, services)
            }
            (ArrivalPort::Job, _, _, false) => self.ignore_job(),
            (ArrivalPort::Unknown, _, _, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (
            &self.state.phase,
            self.state.queue.is_empty(),
            self.store_records,
        ) {
            (Phase::RecordsFetch, _, _) => self.release_records(),
            (Phase::Passive, true, _) => self.passivate(),
            (Phase::Passive, false, _) => self.resume_processing(services),
            (Phase::Active, _, true) => self.save_and_release_job(services),
            (Phase::Active, _, false) => self.release_job(services),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}
