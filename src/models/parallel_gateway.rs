use std::collections::HashMap;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::Model;
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The parallel gateway splits a job across multiple processing paths. The
/// job is duplicated across every one of the processing paths. In addition
/// to splitting the process, a second parallel gateway can be used to join
/// the split paths. The parallel gateway is a BPMN concept.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelGateway {
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
#[serde(rename_all = "camelCase")]
struct PortsIn {
    flow_paths: Vec<String>,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    flow_paths: Vec<String>,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    event_list: Vec<ScheduledEvent>,
    collections: HashMap<String, usize>,
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
            collections: HashMap::new(),
            global_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    Run,
    SendJob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledEvent {
    time: f64,
    event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    last_arrival: Option<(String, f64)>,
    last_departure: Option<(String, f64)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            last_arrival: None,
            last_departure: None,
        }
    }
}

impl ParallelGateway {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for ParallelGateway {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn status(&self) -> String {
        String::from("Active")
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        // Possible metrics updates
        if self.need_snapshot_metrics() {
            self.snapshot.last_arrival =
                Some((incoming_message.message.clone(), self.state.global_time));
        }
        if self.need_historical_metrics() {
            self.history.push(self.snapshot.clone());
        }
        // State changes
        let matching_collection = self
            .state
            .collections
            .entry(incoming_message.message)
            .or_insert(0);
        *matching_collection += 1;
        if *matching_collection == self.ports_in.flow_paths.len() {
            self.state.event_list.push(ScheduledEvent {
                time: 0.0,
                event: Event::SendJob,
            })
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
            .map(|scheduled_event| {
                match scheduled_event.event {
                    Event::Run => {}
                    Event::SendJob => {
                        let completed_collection = self
                            .state
                            .collections
                            .iter()
                            .find(|(_, count)| **count == self.ports_in.flow_paths.len())
                            .ok_or_else(|| SimulationError::InvalidModelState)?
                            .0
                            .to_string();
                        self.ports_out.flow_paths.iter().for_each(|port_out| {
                            outgoing_messages.push(ModelMessage {
                                port_name: String::from(port_out),
                                message: completed_collection.clone(),
                            });
                        });
                        self.state.collections.remove(&completed_collection);
                        // Possible metrics updates
                        if self.need_snapshot_metrics() {
                            self.snapshot.last_departure =
                                Some((completed_collection, self.state.global_time));
                        }
                        if self.need_historical_metrics() {
                            self.history.push(self.snapshot.clone());
                        }
                    }
                }
                Ok(Vec::new())
            })
            .find(|result| result.is_err())
            .unwrap_or(Ok(outgoing_messages))
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
