use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::input_modeling::ContinuousRandomVariable;
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Job,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    job: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    queue: Vec<String>,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            queue: Vec::new(),
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Active,
    Passive,
}

#[cfg_attr(feature = "simx", event_rules)]
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
            ports_in: PortsIn { job: job_port },
            ports_out: PortsOut {
                job: processed_job_port,
            },
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

    fn add_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.queue.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn activate(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.queue.push(incoming_message.content.clone());
        self.state.phase = Phase::Active;
        self.state.until_next_event = self.service_time.random_variate(services.uniform_rng())?;
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
        self.record(
            services.global_time(),
            String::from("Processing Start"),
            incoming_message.content.clone(),
        );
        Ok(())
    }

    fn ignore_job(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.record(
            services.global_time(),
            String::from("Drop"),
            incoming_message.content.clone(),
        );
    }

    fn process_next(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Active;
        self.state.until_next_event = self.service_time.random_variate(services.uniform_rng())?;
        self.record(
            services.global_time(),
            String::from("Processing Start"),
            self.state.queue[0].clone(),
        );
        Ok(Vec::new())
    }

    fn release_job(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        let job = self.state.queue.remove(0);
        self.state.phase = Phase::Passive;
        self.state.until_next_event = 0.0;
        self.record(
            services.global_time(),
            String::from("Departure"),
            job.clone(),
        );
        vec![ModelMessage {
            content: job,
            port_name: self.ports_out.job.clone(),
        }]
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
impl DevsModel for Processor {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            self.arrival_port(&incoming_message.port_name),
            self.state.queue.is_empty(),
            self.state.queue.len() == self.queue_capacity,
        ) {
            (ArrivalPort::Job, true, true) => Err(SimulationError::InvalidModelState),
            (ArrivalPort::Job, false, true) => Ok(self.ignore_job(incoming_message, services)),
            (ArrivalPort::Job, true, false) => self.activate(incoming_message, services),
            (ArrivalPort::Job, false, false) => Ok(self.add_job(incoming_message, services)),
            (ArrivalPort::Unknown, _, _) => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (&self.state.phase, self.state.queue.is_empty()) {
            (Phase::Passive, true) => Ok(self.passivate()),
            (Phase::Passive, false) => self.process_next(services),
            (Phase::Active, _) => Ok(self.release_job(services)),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Processor {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Active => String::from("Processing"),
            Phase::Passive => String::from("Passive"),
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Processor {}
