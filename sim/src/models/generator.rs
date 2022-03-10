use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::input_modeling::ContinuousRandomVariable;
use crate::input_modeling::Thinning;
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

#[cfg(feature = "simx")]
use simx::event_rules;

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
struct PortsIn {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    until_job: f64,
    last_job: usize,
    records: Vec<ModelRecord>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Initializing,
            until_next_event: 0.0,
            until_job: 0.0,
            last_job: 0,
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Phase {
    Initializing,
    Generating,
}

#[cfg_attr(feature = "simx", event_rules)]
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
            ports_in: PortsIn {},
            ports_out: PortsOut { job: job_port },
            store_records,
            state: State::default(),
        }
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
        self.state.last_job += 1;
        self.record(
            services.global_time(),
            String::from("Generation"),
            format!["{} {}", self.ports_out.job, self.state.last_job],
        );
        Ok(vec![ModelMessage {
            port_name: self.ports_out.job.clone(),
            content: format!["{} {}", self.ports_out.job, self.state.last_job],
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
        self.record(
            services.global_time(),
            String::from("Initialization"),
            String::from(""),
        );
        Ok(Vec::new())
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
impl DevsModel for Generator {
    fn events_ext(
        &mut self,
        _incoming_message: &ModelMessage,
        _services: &mut Services,
    ) -> Result<(), SimulationError> {
        Ok(())
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match &self.state.phase {
            Phase::Generating => self.release_job(services),
            Phase::Initializing => self.initialize_generation(services),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Generator {
    fn status(&self) -> String {
        format!["Generating {}s", self.ports_out.job]
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Generator {}
