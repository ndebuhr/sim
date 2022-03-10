use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

/// The batching process begins when the batcher receives a job.  It will
/// then accept additional jobs, adding them to a batch with the first job,
/// until a max batching time or max batch size is reached - whichever comes
/// first.  If the simultaneous arrival of multiple jobs causes the max batch
/// size to be exceeded, then the excess jobs will spillover into the next
/// batching period.  In this case of excess jobs, the next batching period
/// begins immediately after the release of the preceding batch.  If there
/// are no excess jobs, the batcher will become passive, and wait for a job
/// arrival to initiate the batching process.  
#[derive(Debug, Clone, Deserialize, Serialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Batcher {
    ports_in: PortsIn,
    ports_out: PortsOut,
    max_batch_time: f64,
    max_batch_size: usize,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsIn {
    job: String,
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
    jobs: Vec<String>,
    records: Vec<ModelRecord>,
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
    Passive,  // Doing nothing
    Batching, // Building a batch
    Release,  // Releasing a batch
}

#[cfg_attr(feature = "simx", event_rules)]
impl Batcher {
    pub fn new(
        job_in_port: String,
        job_out_port: String,
        max_batch_time: f64,
        max_batch_size: usize,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn { job: job_in_port },
            ports_out: PortsOut { job: job_out_port },
            max_batch_time,
            max_batch_size,
            store_records,
            state: State::default(),
        }
    }

    fn add_to_batch(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Batching;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn start_batch(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Batching;
        self.state.until_next_event = self.max_batch_time;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn fill_batch(&mut self, incoming_message: &ModelMessage, services: &mut Services) {
        self.state.phase = Phase::Release;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(incoming_message.content.clone());
        self.record(
            services.global_time(),
            String::from("Arrival"),
            incoming_message.content.clone(),
        );
    }

    fn release_full_queue(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.phase = Phase::Passive;
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

    fn release_partial_queue(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.phase = Phase::Batching;
        self.state.until_next_event = self.max_batch_time;
        (0..self.max_batch_size)
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

    fn release_multiple(&mut self, services: &mut Services) -> Vec<ModelMessage> {
        self.state.phase = Phase::Release;
        self.state.until_next_event = 0.0;
        (0..self.max_batch_size)
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
impl DevsModel for Batcher {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match (
            &self.state.phase,
            self.state.jobs.len() + 1 < self.max_batch_size,
        ) {
            (Phase::Batching, true) => Ok(self.add_to_batch(incoming_message, services)),
            (Phase::Passive, true) => Ok(self.start_batch(incoming_message, services)),
            (Phase::Release, true) => Err(SimulationError::InvalidModelState),
            (_, false) => Ok(self.fill_batch(incoming_message, services)),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (
            self.state.jobs.len() <= self.max_batch_size,
            self.state.jobs.len() >= 2 * self.max_batch_size,
        ) {
            (true, false) => Ok(self.release_full_queue(services)),
            (false, true) => Ok(self.release_multiple(services)),
            (false, false) => Ok(self.release_partial_queue(services)),
            (true, true) => Err(SimulationError::InvalidModelState),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Batcher {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Passive => String::from("Passive"),
            Phase::Batching => String::from("Creating batch"),
            Phase::Release => String::from("Releasing batch"),
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Batcher {}
