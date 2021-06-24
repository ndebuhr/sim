use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::input_modeling::random_variable::ContinuousRandomVariable;
use crate::input_modeling::Thinning;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::error::SimulationError;

use sim_derive::SerializableModel;

/// The generator produces jobs based on a configured interarrival
/// distribution. A normalized thinning function is used to enable
/// non-stationary job generation. For non-stochastic generation of jobs, a
/// random variable distribution with a single point can be used - in which
/// case, the time between job generation is constant. This model will
/// produce jobs through perpetuity, and the generator does not receive
/// messages or otherwise change behavior throughout a simulation (except
/// through the thinning function).
#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    // Time between job generations
    message_interdeparture_time: ContinuousRandomVariable,
    // Thinning for non-stationarity
    #[serde(default)]
    thinning: Option<Thinning>,
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
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
    until_job: f64,
    last_job: Job,
    records: Vec<Job>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Initializing,
            until_next_event: 0.0,
            until_job: 0.0,
            last_job: Job {
                index: 0,
                content: String::from("job 0"),
                time: 0.0,
            },
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Initializing,
    RecordsFetch,
    Generating,
    Saved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Job {
    index: usize,
    content: String,
    time: f64,
}

impl Generator {
    pub fn new(
        message_interdeparture_time: ContinuousRandomVariable,
        thinning: Option<Thinning>,
        job_port: String,
        store_records: bool,
    ) -> Self {
        Self {
            message_interdeparture_time,
            thinning,
            ports_in: PortsIn {
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                job: job_port,
                records: default_records_port_name(),
            },
            store_records,
            state: Default::default(),
        }
    }

    fn request_records(
        &mut self,
        _incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.state.phase = Phase::RecordsFetch;
        self.state.until_next_event = 0.0;
        self.state.until_job -= services.global_time() - self.state.last_job.time;
        Ok(())
    }

    fn ignore_request(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        Ok(())
    }

    fn save_job(&mut self, services: &mut Services) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Saved;
        self.state.until_next_event = 0.0;
        self.state.records.push(Job {
            index: self.state.last_job.index + 1,
            content: format!["{} {}", self.ports_out.job, self.state.last_job.index + 1],
            time: services.global_time(),
        });
        Ok(Vec::new())
    }

    fn release_job(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let interdeparture = self
            .message_interdeparture_time
            .random_variate(services.uniform_rng())?;
        self.state.phase = Phase::Generating;
        self.state.until_next_event = interdeparture;
        self.state.until_job = interdeparture;
        self.state.last_job = Job {
            index: self.state.last_job.index + 1,
            content: format!["{} {}", self.ports_out.job, self.state.last_job.index + 1],
            time: services.global_time(),
        };
        Ok(vec![ModelMessage {
            port_name: self.ports_out.job.clone(),
            content: self.state.last_job.content.clone(),
        }])
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Generating;
        self.state.until_next_event = self.state.until_job;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn initialize_generation(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let interdeparture = self
            .message_interdeparture_time
            .random_variate(services.uniform_rng())?;
        self.state.phase = Phase::Generating;
        self.state.until_next_event = interdeparture;
        self.state.until_job = interdeparture;
        Ok(Vec::new())
    }
}

impl AsModel for Generator {
    fn status(&self) -> String {
        format!["Generating {}s", self.ports_out.job]
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        if self.store_records {
            self.request_records(incoming_message, services)
        } else if !self.store_records {
            self.ignore_request(incoming_message, services)
        } else {
            Err(SimulationError::InvalidModelState)
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if self.state.phase == Phase::Generating && self.store_records {
            self.save_job(services)
        } else if (self.state.phase == Phase::Generating && !self.store_records)
            || self.state.phase == Phase::Saved
        {
            self.release_job(services)
        } else if self.state.phase == Phase::RecordsFetch {
            self.release_records()
        } else if self.state.phase == Phase::Initializing {
            self.initialize_generation(services)
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
