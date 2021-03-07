use std::any::Any;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::{Model, Type};
use super::ModelMessage;
use crate::input_modeling::random_variable::ContinuousRandomVariable;
use crate::input_modeling::thinning::Thinning;
use crate::input_modeling::uniform_rng::UniformRNG;
use crate::utils::error::SimulationError;

/// The generator produces jobs based on a configured interarrival
/// distribution. A normalized thinning function is used to enable
/// non-stationary job generation. For non-stochastic generation of jobs, a
/// random variable distribution with a single point can be used - in which
/// case, the time between job generation is constant. This model will
/// produce jobs through perpetuity, and the generator does not receive
/// messages or otherwise change behavior throughout a simulation (except
/// through the thinning function).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    id: String,
    // Time between job generations
    message_interdeparture_time: ContinuousRandomVariable,
    // Thinning for non-stationarity
    #[serde(default)]
    thinning: Option<Thinning>,
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
    until_message_interdeparture: f64,
    job_counter: usize,
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
            until_message_interdeparture: INFINITY,
            job_counter: 0,
            global_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    Run,
    BeginGeneration,
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
    last_generation: Option<(String, f64)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            last_generation: None,
        }
    }
}

impl Generator {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for Generator {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn get_type(&self) -> Type {
        Type::Generator
    }

    fn status(&self) -> String {
        format!["Generating {}s", self.ports_out.job]
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        _incoming_message: ModelMessage,
    ) -> Result<Vec<ModelMessage>, SimulationError> {
        Ok(Vec::new())
    }

    fn events_int(
        &mut self,
        uniform_rng: &mut UniformRNG,
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
            .map(
                |scheduled_event| -> Result<Vec<ModelMessage>, SimulationError> {
                    match scheduled_event.event {
                        Event::Run => {
                            self.state.event_list.push(ScheduledEvent {
                                time: 0.0,
                                event: Event::BeginGeneration,
                            });
                        }
                        Event::BeginGeneration => {
                            self.state.until_message_interdeparture = self
                                .message_interdeparture_time
                                .random_variate(uniform_rng)?;
                            self.state.event_list.push(ScheduledEvent {
                                time: self.state.until_message_interdeparture,
                                event: Event::BeginGeneration,
                            });
                            if let Some(thinning) = self.thinning.clone() {
                                let thinning_threshold =
                                    thinning.evaluate(self.state.global_time)?;
                                let uniform_rn = uniform_rng.rn();
                                if uniform_rn < thinning_threshold {
                                    self.state.event_list.push(ScheduledEvent {
                                        time: self.state.until_message_interdeparture,
                                        event: Event::SendJob,
                                    });
                                }
                            } else {
                                self.state.event_list.push(ScheduledEvent {
                                    time: self.state.until_message_interdeparture,
                                    event: Event::SendJob,
                                });
                            }
                        }
                        Event::SendJob => {
                            self.state.job_counter += 1;
                            let generated = format![
                                "{job_type} {job_id}",
                                job_type = self.ports_out.job,
                                job_id = self.state.job_counter
                            ];
                            outgoing_messages.push(ModelMessage {
                                port_name: self.ports_out.job.clone(),
                                message: generated.clone(),
                            });
                            // Possible metrics updates
                            if self.need_snapshot_metrics() {
                                self.snapshot.last_generation =
                                    Some((generated, self.state.global_time));
                            }
                            if self.need_historical_metrics() {
                                self.history.push(self.snapshot.clone());
                            }
                        }
                    }
                    Ok(Vec::new())
                },
            )
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
