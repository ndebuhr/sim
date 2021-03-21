use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::Model;
use super::ModelMessage;
use crate::input_modeling::random_variable::IndexRandomVariable;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The exclusive gateway splits a process flow into a set of possible paths.
/// The process will only follow one of the possible paths. Path selection is
/// determined by Weighted Index distribution random variates, so this atomic
/// model exhibits stochastic behavior. The exclusive gateway is a BPMN
/// concept.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusiveGateway {
    id: String,
    ports_in: PortsIn,
    ports_out: PortsOut,
    port_weights: IndexRandomVariable,
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
    last_job: Option<(String, String, f64)>, // port, message, time
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics { last_job: None }
    }
}

impl ExclusiveGateway {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for ExclusiveGateway {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn status(&self) -> String {
        String::from("Active")
    }

    fn events_ext(
        &mut self,
        uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let port_number = self.port_weights.random_variate(uniform_rng)?;
        // Possible metrics updates
        if self.need_snapshot_metrics() {
            self.snapshot.last_job = Some((
                self.ports_out.flow_paths[port_number].clone(),
                incoming_message.message.clone(),
                self.state.global_time,
            ));
        }
        if self.need_historical_metrics() {
            self.history.push(self.snapshot.clone());
        }
        // State changes
        outgoing_messages.push(ModelMessage {
            port_name: self.ports_out.flow_paths[port_number].clone(),
            message: incoming_message.message,
        });
        Ok(outgoing_messages)
    }

    fn events_int(
        &mut self,
        _uniform_rng: &mut UniformRNG,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
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
