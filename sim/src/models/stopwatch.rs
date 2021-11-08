use std::f64::{INFINITY, NEG_INFINITY};
use std::iter::once;

use serde::{Deserialize, Serialize};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::SimulationError;

use sim_derive::SerializableModel;

/// The stopwatch calculates durations by matching messages on the start and
/// stop ports.  For example, a "job 1" message arrives at the start port at
/// time 0.1, and then a "job 1" message arrives at the stop port at time
/// 1.3.  The duration for job 1 will be saved as 1.2.  The status reporting
/// provides the average duration across all jobs.  The maximum or minimum
/// duration job is also accessible through the metric and job ports.
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ArrivalPort {
    Start,
    Stop,
    Metric,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
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
    Passive,
    JobFetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
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
            },
            ports_out: PortsOut { job: job_port },
            metric,
            store_records,
            state: State::default(),
        }
    }

    fn arrival_port(&self, message_port: &str) -> ArrivalPort {
        if message_port == self.ports_in.start {
            ArrivalPort::Start
        } else if message_port == self.ports_in.stop {
            ArrivalPort::Stop
        } else if message_port == self.ports_in.metric {
            ArrivalPort::Metric
        } else {
            ArrivalPort::Unknown
        }
    }

    fn matching_or_new_job(&mut self, incoming_message: &ModelMessage) -> &mut Job {
        if !self
            .state
            .jobs
            .iter()
            .any(|job| job.name == incoming_message.content)
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

    fn start_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.record(
            services.global_time(),
            String::from("Start"),
            incoming_message.content.clone(),
        );
        self.matching_or_new_job(incoming_message).start = Some(services.global_time());
        Ok(())
    }

    fn stop_job(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        self.record(
            services.global_time(),
            String::from("Stop"),
            incoming_message.content.clone(),
        );
        self.matching_or_new_job(incoming_message).stop = Some(services.global_time());
        Ok(())
    }

    fn get_job(&mut self) -> Result<(), SimulationError> {
        self.state.phase = Phase::JobFetch;
        self.state.until_next_event = 0.0;
        Ok(())
    }

    fn release_minimum(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        self.record(
            services.global_time(),
            String::from("Minimum Fetch"),
            self.minimum_duration_job()
                .unwrap_or_else(|| "None".to_string()),
        );
        Ok(once(self.minimum_duration_job())
            .flatten()
            .map(|job| ModelMessage {
                content: job,
                port_name: self.ports_out.job.clone(),
            })
            .collect())
    }

    fn release_maximum(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        self.record(
            services.global_time(),
            String::from("Maximum Fetch"),
            self.maximum_duration_job()
                .unwrap_or_else(|| "None".to_string()),
        );
        Ok(once(self.maximum_duration_job())
            .flatten()
            .map(|job| ModelMessage {
                content: job,
                port_name: self.ports_out.job.clone(),
            })
            .collect())
    }

    fn passivate(&mut self) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.phase = Phase::Passive;
        self.state.until_next_event = INFINITY;
        Ok(Vec::new())
    }

    fn record(&mut self, time: f64, action: String, subject: String) {
        if self.store_records {
            self.state.records.push(ModelRecord {
                time,
                action,
                subject,
            })
        }
    }
}

impl DevsModel for Stopwatch {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> Result<(), SimulationError> {
        match self.arrival_port(&incoming_message.port_name) {
            ArrivalPort::Start => self.start_job(incoming_message, services),
            ArrivalPort::Stop => self.stop_job(incoming_message, services),
            ArrivalPort::Metric => self.get_job(),
            ArrivalPort::Unknown => Err(SimulationError::InvalidMessage),
        }
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        match (&self.state.phase, &self.metric) {
            (Phase::JobFetch, Metric::Minimum) => self.release_minimum(services),
            (Phase::JobFetch, Metric::Maximum) => self.release_maximum(services),
            (Phase::Passive, _) => self.passivate(),
        }
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state.until_next_event -= time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state.until_next_event
    }
}

impl Reportable for Stopwatch {
    fn status(&self) -> String {
        if self.state.jobs.is_empty() {
            String::from("Measuring durations")
        } else {
            let durations: Vec<f64> = self
                .state
                .jobs
                .iter()
                .filter_map(|job| {
                    self.some_duration(job)
                        .map(|duration_record| duration_record.1)
                })
                .collect();
            format![
                "Average {:.3}",
                durations.iter().sum::<f64>() / durations.len() as f64
            ]
        }
    }

    fn records(&self) -> &Vec<ModelRecord> {
        &self.state.records
    }
}

impl ReportableModel for Stopwatch {}
