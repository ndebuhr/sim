use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

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
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Passive,
            until_next_event: INFINITY,
            jobs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Passive,  // Doing nothing
    Batching, // Building a batch
    Release,  // Releasing a batch
}

impl Batcher {
    pub fn new(
        job_in_port: String,
        job_out_port: String,
        max_batch_time: f64,
        max_batch_size: usize,
    ) -> Self {
        Self {
            ports_in: PortsIn { job: job_in_port },
            ports_out: PortsOut { job: job_out_port },
            max_batch_time,
            max_batch_size,
            state: Default::default(),
        }
    }

    fn add_to_batch(&mut self, incoming_message: &ModelMessage) -> Result<(), SimulationError> {
        self.state.phase = Phase::Batching;
        self.state.jobs.push(incoming_message.content.clone());
        Ok(())
    }

    fn start_batch(&mut self, incoming_message: &ModelMessage) -> Result<(), SimulationError> {
        self.state.phase = Phase::Batching;
        self.state.until_next_event = self.max_batch_time;
        self.state.jobs.push(incoming_message.content.clone());
        Ok(())
    }

    fn fill_batch(&mut self, incoming_message: &ModelMessage) -> Result<(), SimulationError> {
        self.state.phase = Phase::Release;
        self.state.until_next_event = 0.0;
        self.state.jobs.push(incoming_message.content.clone());
        Ok(())
    }

    fn release_full_queue(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok((0..self.state.jobs.len())
            .map(|_| ModelMessage {
                port_name: self.ports_out.job.clone(),
                content: self.state.jobs.remove(0),
            })
            .collect())
    }

    fn release_partial_queue(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Batching;
        self.state.until_next_event = self.max_batch_time;
        Ok((0..self.max_batch_size)
            .map(|_| ModelMessage {
                port_name: self.ports_out.job.clone(),
                content: self.state.jobs.remove(0),
            })
            .collect())
    }
}

impl AsModel for Batcher {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Passive => String::from("Passive"),
            Phase::Batching => String::from("Creating batch"),
            Phase::Release => String::from("Releasing batch"),
        }
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        if self.state.phase == Phase::Batching && self.state.jobs.len() + 1 < self.max_batch_size {
            self.add_to_batch(incoming_message)
        } else if self.state.phase == Phase::Passive
            && self.state.jobs.len() + 1 < self.max_batch_size
        {
            self.start_batch(incoming_message)
        } else if self.state.jobs.len() + 1 >= self.max_batch_size {
            self.fill_batch(incoming_message)
        } else {
            Err(SimulationError::InvalidModelState)
        }
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if self.state.jobs.len() <= self.max_batch_size {
            self.release_full_queue()
        } else if self.state.jobs.len() <= self.max_batch_size {
            self.release_partial_queue()
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
