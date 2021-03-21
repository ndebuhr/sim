use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::AsModel;
use super::ModelMessage;
use crate::input_modeling::UniformRNG;
use crate::utils::error::SimulationError;

/// The gate model passes or blocks jobs, when it is in the open or closed
/// state, respectively. The gate can be opened and closed throughout the
/// course of a simulation. This model contains no stochastic behavior - job
/// passing/blocking is based purely on the state of the model at that time
/// in the simulation. A blocked job is a dropped job - it is not stored,
/// queued, or redirected.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gate {
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
    activation: String,
    deactivation: String,
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
    #[serde(default)]
    global_time: f64,
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
            global_time: 0.0,
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
    last_received: Option<(String, f64)>,
    last_activation: Option<f64>,
    last_deactivation: Option<f64>,
    pass_count: usize,
    block_count: usize,
    is_open: Option<bool>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            last_received: None,
            last_activation: None,
            last_deactivation: None,
            pass_count: 0,
            block_count: 0,
            is_open: None,
        }
    }
}

impl Gate {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl AsModel for Gate {
    fn status(&self) -> String {
        match self.state.phase {
            Phase::Open => String::from("Listening"),
            Phase::Closed => String::from("Blocked"),
        }
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let incoming_port: &str = &incoming_message.port_name;
        match &self.ports_in {
            PortsIn { activation, .. } if activation == incoming_port => {
                // Possible metrics updates
                if self.need_snapshot_metrics() {
                    self.snapshot.last_activation = Some(self.state.global_time);
                    self.snapshot.is_open = Some(true);
                }
                if self.need_historical_metrics() {
                    self.history.push(self.snapshot.clone());
                }
                // Execution
                self.state.phase = Phase::Open;
            }
            PortsIn { deactivation, .. } if deactivation == incoming_port => {
                // Possible metrics updates
                if self.need_snapshot_metrics() {
                    self.snapshot.last_deactivation = Some(self.state.global_time);
                    self.snapshot.is_open = Some(false);
                }
                if self.need_historical_metrics() {
                    self.history.push(self.snapshot.clone());
                }
                // Execution
                self.state.phase = Phase::Closed;
            }
            PortsIn { job, .. } if job == incoming_port => {
                // Possible metrics updates
                if self.need_snapshot_metrics() {
                    self.snapshot.last_received =
                        Some((incoming_message.message.clone(), self.state.global_time));
                }
                if self.need_historical_metrics() {
                    self.history.push(self.snapshot.clone());
                }
                // Execution
                self.state.jobs.push(incoming_message.message);
                match self.state.phase {
                    Phase::Closed => self.state.event_list.push(ScheduledEvent {
                        time: 0.0,
                        event: Event::DropJob,
                    }),
                    Phase::Open => self.state.event_list.push(ScheduledEvent {
                        time: 0.0,
                        event: Event::SendJob,
                    }),
                }
            }
            _ => return Err(SimulationError::PortNotFound),
        }
        Ok(Vec::new())
    }

    fn events_int(
        &mut self,
        _uniform_rng: &mut UniformRNG,
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
                        self.snapshot.block_count += 1;
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
                        self.snapshot.pass_count += 1;
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // Execution
                    outgoing_messages.push(ModelMessage {
                        port_name: self.ports_out.job.clone(),
                        message: self.state.jobs.remove(0),
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
        self.state.global_time += time_delta;
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
