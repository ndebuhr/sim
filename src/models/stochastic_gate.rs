use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::AsModel;
use super::ModelMessage;
use crate::input_modeling::random_variable::BooleanRandomVariable;
use crate::simulator::Services;
use crate::utils::error::SimulationError;
use crate::utils::{populate_history_port, populate_snapshot_port};

/// The stochastic gate blocks (drops) or passes jobs, based on a specified
/// Bernoulli distribution. If the Bernoulli random variate is a 0, the job
/// will be dropped. If the Bernoulli random variate is a 1, the job will be
/// passed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StochasticGate {
    pass_distribution: BooleanRandomVariable,
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    state: State,
    #[serde(default)]
    snapshot: Metrics,
    #[serde(default)]
    history: Vec<Metrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsIn {
    job: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    job: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    event_list: Vec<ScheduledEvent>,
    jobs: Vec<String>,
    phase: Phase,
}

impl Default for State {
    fn default() -> Self {
        let initalization_event = ScheduledEvent {
            time: 0.0,
            event: Event::Run,
        };
        State {
            event_list: vec![initalization_event],
            jobs: Vec::new(),
            phase: Phase::Open,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledEvent {
    time: f64,
    event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    Run,
    DropJob,
    SendJob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Phase {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    last_pass: Option<(String, f64)>,
    last_block: Option<(String, f64)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            last_pass: None,
            last_block: None,
        }
    }
}

impl StochasticGate {
    pub fn new(
        pass_distribution: BooleanRandomVariable,
        job_in_port: String,
        job_out_port: String,
        snapshot_metrics: bool,
        history_metrics: bool,
    ) -> Self {
        Self {
            pass_distribution,
            ports_in: PortsIn {
                job: job_in_port,
                snapshot: populate_snapshot_port(snapshot_metrics),
                history: populate_history_port(history_metrics),
            },
            ports_out: PortsOut {
                job: job_out_port,
                snapshot: populate_snapshot_port(snapshot_metrics),
                history: populate_history_port(history_metrics),
            },
            state: Default::default(),
            snapshot: Default::default(),
            history: Default::default(),
        }
    }

    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl AsModel for StochasticGate {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Open => String::from("Pass"),
            Phase::Closed => String::from("Block"),
        }
    }

    fn events_ext(
        &mut self,
        incoming_message: ModelMessage,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let incoming_port: &str = &incoming_message.port_name;
        match &self.ports_in {
            PortsIn { job, .. } if job == incoming_port => {
                // Execution
                if self
                    .pass_distribution
                    .random_variate(services.uniform_rng())?
                {
                    self.state.event_list.push(ScheduledEvent {
                        time: 0.0,
                        event: Event::SendJob,
                    })
                } else {
                    self.state.event_list.push(ScheduledEvent {
                        time: 0.0,
                        event: Event::DropJob,
                    })
                }
                self.state.jobs.push(incoming_message.content);
            }
            _ => return Err(SimulationError::PortNotFound),
        }
        Ok(Vec::new())
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let events = self.state.event_list.clone();
        self.state.event_list = self
            .state
            .event_list
            .iter()
            .filter(|scheduled_event| scheduled_event.time != 0.0)
            .cloned()
            .collect();
        events
            .iter()
            .filter(|scheduled_event| scheduled_event.time == 0.0)
            .for_each(|scheduled_event| match scheduled_event.event {
                Event::Run => {}
                Event::DropJob => {
                    // Possible metrics updates
                    if self.need_snapshot_metrics() {
                        self.snapshot.last_block =
                            Some((self.state.jobs[0].clone(), services.global_time()));
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // Execution
                    self.state.jobs.remove(0);
                }
                Event::SendJob => {
                    // Possible metrics updates
                    if self.need_snapshot_metrics() {
                        self.snapshot.last_pass =
                            Some((self.state.jobs[0].clone(), services.global_time()));
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // Execution
                    outgoing_messages.push(ModelMessage {
                        port_name: self.ports_out.job.clone(),
                        content: self.state.jobs.remove(0),
                    });
                }
            });
        Ok(outgoing_messages)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state
            .event_list
            .iter_mut()
            .for_each(|scheduled_event| {
                scheduled_event.time -= time_delta;
            });
    }

    fn until_next_event(&self) -> f64 {
        self.state
            .event_list
            .iter()
            .fold(INFINITY, |until_next_event, event| {
                f64::min(until_next_event, event.time)
            })
    }
}
