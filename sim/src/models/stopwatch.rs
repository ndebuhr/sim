use std::f64::{INFINITY, NEG_INFINITY};
use std::iter::once;

use serde::{Deserialize, Serialize};

use super::model_trait::{AsModel, SerializableModel};
use super::ModelMessage;
use crate::simulator::Services;
use crate::utils::default_records_port_name;
use crate::utils::error::SimulationError;

use sim_derive::SerializableModel;

#[derive(Debug, Clone, Serialize, Deserialize, SerializableModel)]
#[serde(rename_all = "camelCase")]
pub struct Stopwatch {
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    metric: Metric,
    #[serde(default)]
    store_records: bool,
    #[serde(default)]
    state: State,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    start: String,
    stop: String,
    metric: String,
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
pub enum Metric {
    Minimum,
    Maximum,
}

impl Default for Metric {
    fn default() -> Self {
        Metric::Minimum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    phase: Phase,
    until_next_event: f64,
    jobs: Vec<Job>,
    records: Vec<Record>,
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
    JobFetch,
    RecordsFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    name: String,
    start: Option<f64>,
    stop: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    timestamp: f64,
    name: String,
    start: Option<f64>,
    stop: Option<f64>,
}

impl Stopwatch {
    pub fn new(
        start_port: String,
        stop_port: String,
        metric_port: String,
        job_port: String,
        metric: Metric,
        store_records: bool,
    ) -> Self {
        Self {
            ports_in: PortsIn {
                start: start_port,
                stop: stop_port,
                metric: metric_port,
                records: default_records_port_name(),
            },
            ports_out: PortsOut {
                job: job_port,
                records: default_records_port_name(),
            },
            metric,
            store_records,
            state: Default::default(),
        }
    }

    fn matching_or_new_job(&mut self, incoming_message: &ModelMessage) -> &mut Job {
        if self
            .state
            .jobs
            .iter()
            .find(|job| job.name == incoming_message.content)
            .is_none()
        {
            self.state.jobs.push(Job {
                name: incoming_message.content.clone(),
                start: None,
                stop: None,
            });
        }
        self.state
            .jobs
            .iter_mut()
            .find(|job| job.name == incoming_message.content)
            .unwrap()
    }

    fn some_duration(&self, job: &Job) -> Option<(String, f64)> {
        match (job.start, job.stop) {
            (Some(start), Some(stop)) => Some((job.name.to_string(), stop - start)),
            _ => None,
        }
    }

    fn minimum_duration_job(&self) -> Option<String> {
        self.state
            .jobs
            .iter()
            .filter_map(|job| self.some_duration(job))
            .fold((None, INFINITY), |minimum, (job_name, job_duration)| {
                if job_duration < minimum.1 {
                    (Some(job_name), job_duration)
                } else {
                    minimum
                }
            })
            .0
    }

    fn maximum_duration_job(&self) -> Option<String> {
        self.state
            .jobs
            .iter()
            .filter_map(|job| self.some_duration(job))
            .fold((None, NEG_INFINITY), |maximum, (job_name, job_duration)| {
                if job_duration > maximum.1 {
                    (Some(job_name), job_duration)
                } else {
                    maximum
                }
            })
            .0
    }

    fn calculate_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        if incoming_message.port_name == self.ports_in.start {
            self.matching_or_new_job(incoming_message).start = Some(services.global_time())
        } else if incoming_message.port_name == self.ports_in.stop {
            self.matching_or_new_job(incoming_message).stop = Some(services.global_time())
        } else {
            return Err(SimulationError::InvalidModelState);
        }
        Ok(())
    }

    fn calculate_and_save_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        if incoming_message.port_name == self.ports_in.start {
            self.matching_or_new_job(incoming_message).start = Some(services.global_time())
        } else if incoming_message.port_name == self.ports_in.stop {
            self.matching_or_new_job(incoming_message).stop = Some(services.global_time())
        } else {
            return Err(SimulationError::InvalidModelState);
        }
        let job = self.matching_or_new_job(incoming_message).clone();
        self.state.records.push(Record {
            timestamp: services.global_time(),
            name: job.name,
            start: job.start,
            stop: job.stop,
        });
        Ok(())
    }

    fn get_job(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::JobFetch;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn get_records(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::RecordsFetch;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn release_records(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(vec![ModelMessage {
            port_name: self.ports_out.records.clone(),
            content: serde_json::to_string(&self.state.records).unwrap(),
        }])
    }

    fn release_job(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(match &self.metric {
            Metric::Minimum => once(self.minimum_duration_job())
                .flatten()
                .map(|job| ModelMessage {
                    content: job,
                    port_name: self.ports_out.job.clone(),
                })
                .collect(),
            Metric::Maximum => once(self.maximum_duration_job())
                .flatten()
                .map(|job| ModelMessage {
                    content: job,
                    port_name: self.ports_out.job.clone(),
                })
                .collect(),
        })
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }
}

impl AsModel for Stopwatch {
    fn status(&self) -> String {
        let durations: Vec<f64> = self
            .state
            .jobs
            .iter()
            .filter_map(|job| self.some_duration(job))
            .map(|(_, duration)| duration)
            .collect();
        format![
            "Average {:.3}",
            durations.iter().sum::<f64>() / durations.len() as f64
        ]
    }

    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        if (self.ports_in.start == incoming_message.port_name
            || self.ports_in.stop == incoming_message.port_name)
            && self.store_records
        {
            self.calculate_and_save_job(incoming_message, services)
        } else if (self.ports_in.start == incoming_message.port_name
            || self.ports_in.stop == incoming_message.port_name)
            && !self.store_records
        {
            self.calculate_job(incoming_message, services)
        } else if incoming_message.port_name == self.ports_in.metric {
            self.get_job()
        } else if incoming_message.port_name == self.ports_in.records {
            self.get_records()
        } else {
            Err(SimulationError::InvalidModelState)
        }
    }

    fn events_int(
        &mut self,
        _services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        if self.state.phase == Phase::RecordsFetch {
            self.release_records()
        } else if self.state.phase == Phase::JobFetch {
            self.release_job()
        } else if self.state.phase == Phase::Passive {
            self.passivate()
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
