use std::any::Any;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::{Model, Type};
use super::ModelMessage;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The load balancer routes jobs to a set of possible process paths, using a
/// round robin strategy. There is no stochastic behavior in this model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadBalancer {
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
    job: String,
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
    jobs: Vec<String>,
    next_port_out: usize,
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
            next_port_out: 0,
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
    last_job: Option<(String, String, f64)>, // Port, message, time
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics { last_job: None }
    }
}

impl LoadBalancer {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for LoadBalancer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn get_type(&self) -> Type {
        Type::LoadBalancer
    }

    fn status(&self) -> String {
        format!["Listening for {}s", self.ports_in.job]
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        self.state.jobs.push(incoming_message.message);
        self.state.event_list.push(ScheduledEvent {
            time: 0.0,
            event: Event::SendJob,
        });
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
                Event::SendJob => {
                    // Possible metrics updates
                    if self.need_snapshot_metrics() {
                        self.snapshot.last_job = Some((
                            self.ports_out.flow_paths[self.state.next_port_out].clone(),
                            self.state.jobs[0].clone(),
                            self.state.global_time,
                        ));
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    // State changes
                    outgoing_messages.push(ModelMessage {
                        port_name: self.ports_out.flow_paths[self.state.next_port_out].clone(),
                        message: self.state.jobs.remove(0),
                    });
                    self.state.next_port_out =
                        (self.state.next_port_out + 1) % self.ports_out.flow_paths.len();
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
