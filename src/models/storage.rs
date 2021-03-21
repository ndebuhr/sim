use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::Model;
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The storage model stores a value, and responds with it upon request.
/// Values are stored and value requests are handled instantantaneously.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    id: String,
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
    store: String,
    read: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortsOut {
    stored: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    event_list: Vec<ScheduledEvent>,
    job: Option<String>,
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
            job: None,
            global_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledEvent {
    time: f64,
    event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    last_store: Option<(String, f64)>,
    last_read: Option<(String, f64)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            last_store: None,
            last_read: None,
        }
    }
}

impl Storage {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for Storage {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn status(&self) -> String {
        match &self.state.job {
            Some(stored) => format!["Storing {}", stored],
            None => String::from("Empty"),
        }
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let incoming_port: &str = &incoming_message.port_name;
        match &self.ports_in {
            PortsIn { store, .. } if store == incoming_port => {
                // Possible metrics updates
                if self.need_snapshot_metrics() {
                    self.snapshot.last_store =
                        Some((incoming_message.message.clone(), self.state.global_time));
                }
                if self.need_historical_metrics() {
                    self.history.push(self.snapshot.clone());
                }
                // State changes
                self.state.job = Some(incoming_message.message);
            }
            PortsIn { read, .. } if read == incoming_port => {
                // Deliberately not unwrapping here
                // Read requests could come before writes
                match &self.state.job {
                    Some(job) => {
                        // Possible metrics updates
                        if self.need_snapshot_metrics() {
                            self.snapshot.last_read =
                                Some((String::from(job), self.state.global_time));
                        }
                        if self.need_historical_metrics() {
                            self.history.push(self.snapshot.clone());
                        }
                        // State changes
                        outgoing_messages.push(ModelMessage {
                            port_name: self.ports_out.stored.clone(),
                            message: String::from(job),
                        });
                    }
                    None => {
                        // Possible metrics updates
                        if self.need_snapshot_metrics() {
                            self.snapshot.last_read =
                                Some((String::from(""), self.state.global_time));
                        }
                        if self.need_historical_metrics() {
                            self.history.push(self.snapshot.clone());
                        }
                        // State changes
                        outgoing_messages.push(ModelMessage {
                            port_name: self.ports_out.stored.clone(),
                            message: String::from(""),
                        });
                    }
                }
            }
            _ => return Err(SimulationError::PortNotFound),
        }
        Ok(outgoing_messages)
    }

    fn events_int(
        &mut self,
        _uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        // Currently, there is no events_int behavior except the initialization
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
            });
        Ok(Vec::new())
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
