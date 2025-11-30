//! The simulator module provides the mechanics to orchestrate the models and
//! connectors via discrete event simulation.  The specific formalism for
//! simulation execution is the Discrete Event System Specification.  User
//! interaction is also captured in this module - simulation stepping and
//! input injection.
//!
//! `Simulation` and `WebSimulation` are used for Rust- and npm-based
//! projects, respectively.  The `Simulation` methods use the associated
//! struct types directly, while the `WebSimulation` provides an interface
//! with better JS/WASM compatibility.
//!
//! Most simulation analysis will involve the collection, transformation,
//! and analysis of messages.  The `step`, `step_n`, and `step_until` methods
//! return the messages generated during the execution of the simulation
//! step(s), for use in message analysis.

use std::cell::RefCell;
use std::rc::Rc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::input_modeling::dyn_rng;
use crate::input_modeling::dynamic_rng::SimulationRng;
use crate::models::{DevsModel, Model, ModelMessage, ModelRecord, Reportable};
use crate::utils::errors::SimulationError;
use crate::utils::set_panic_hook;
use crate::simulator::time::{SDuration, STime};

pub mod coupling;
pub mod services;
pub mod web;
pub mod time;

pub use self::coupling::{Connector, Message};
pub use self::services::Services;
pub use self::web::Simulation as WebSimulation;

///Type representing a counted reference to a type.
pub type IMType<T> = Rc<RefCell<T>>;

///turn something into a counted reference.
pub fn imtype<T>(x:T) -> IMType<T> {
    Rc::new(RefCell::new(x))
}

///Type representing a counted reference to vec of types
pub type IMTypeVec<T> = IMType<Vec<T>>;

pub fn imtypevec<T>(x:Vec<T>) -> IMTypeVec<T> {
    Rc::new(RefCell::new(x))
}


/// The `Simulation` struct is the core of sim, and includes everything
/// needed to run a simulation - models, connectors, and a random number
/// generator.  State information, specifically global time and active
/// messages are additionally retained in the struct.
#[derive(Clone, Default)]
pub struct Simulation {
    models: IMTypeVec<Model>,
    connectors: Vec<Connector>,
    messages: Vec<Message>,
    services: Services,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationBase {
    models: Vec<Model>,
    connectors: Vec<Connector>,
    messages: Vec<Message>,
    services: Services
}

/// Take a simulation and render it in a json format.
impl Serialize for Simulation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let sb = SimulationBase { models: self.models.borrow().clone(),
            connectors: self.connectors.clone(),
            messages: self.messages.clone(),
            services: self.services.clone() };
        sb.serialize(serializer)
    }
}


/// Take a json representation of the Simulation and construct a
impl<'de> Deserialize<'de> for Simulation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let sb = SimulationBase::deserialize(deserializer)?;
        Ok(Simulation {models: imtypevec(sb.models.clone()),
        connectors: sb.connectors,
        messages: sb.messages,
        services: sb.services})
    }
}


impl Simulation {
    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post(models: Vec<Model>, connectors: Vec<Connector>) -> Self {
        set_panic_hook();
        Self {
            models: imtypevec(models),
            connectors,
            ..Self::default()
        }
    }

    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post_with_rng(
        models: Vec<Model>,
        connectors: Vec<Connector>,
        global_rng: impl SimulationRng + 'static,
    ) -> Self {
        set_panic_hook();
        Self {
            models: imtypevec(models),
            connectors,
            services: Services {
                global_rng: dyn_rng(global_rng),
                global_time: STime::NOW,
            },
            ..Self::default()
        }
    }

    pub fn set_rng(&mut self, rng: impl SimulationRng + 'static) {
        self.services.global_rng = dyn_rng(rng)
    }

    /// This method sets the models and connectors of an existing simulation.
    pub fn put(&mut self, models: Vec<Model>, connectors: Vec<Connector>) {
        self.models = imtypevec(models);
        self.connectors = connectors;
    }

    /// Simulation steps generate messages, which are then consumed on
    /// subsequent simulation steps.  These messages between models in a
    /// simulation drive much of the discovery, analysis, and design.  This
    /// accessor method provides the list of active messages, at the current
    /// point of time in the simulation.  Message history is not retained, so
    /// simulation products and projects should collect messages as needed
    /// throughout the simulation execution.
    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    /// An accessor method for the simulation global time.
    pub fn get_global_time(&self) -> STime {
        self.services.global_time()
    }

    /// This method provides a mechanism for getting the status of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the current status string for that model.
    pub fn get_status(&self, model_id: &str) -> Result<String, SimulationError> {
        Ok(self
            .models
            .borrow()
            .iter()
            .find(|model| model.id() == model_id)
            .ok_or(SimulationError::ModelNotFound)?
            .status())
    }

    /// This method provides a mechanism for getting the records of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the records for that model.
    pub fn get_records(&self, model_id: &str) -> Result<Vec<ModelRecord>,SimulationError>
    {
        Ok(self
            .models
            .borrow()
            .iter()
            .find(|model| model.id() == model_id)
            .ok_or(SimulationError::ModelNotFound)?
            .records().to_vec())
    }

    /// To enable simulation replications, the reset method resets the state
    /// of the simulation, except for the random number generator.
    /// Recreating a simulation from scratch for additional replications
    /// does not work, due to the random number generator seeding.
    pub fn reset(&mut self) {
        self.reset_messages();
        self.reset_global_time();
    }

    /// Clear the active messages in a simulation.
    pub fn reset_messages(&mut self) {
        self.messages = Vec::new();
    }

    /// Reset the simulation global time to 0.0.
    pub fn reset_global_time(&mut self) {
        self.services.set_global_time(STime::NOW);
    }

    /// Input injection creates a message during simulation execution,
    /// without needing to create that message through the standard
    /// simulation constructs.  This enables live simulation interaction,
    /// disruption, and manipulation - all through the standard simulation
    /// message system.
    pub fn inject_input(&mut self, message: Message) {
        self.messages.push(message);
    }


    // Search through all the connectors and find any that match the source model id and source port.
    // There may be more than one.
    pub fn find_target_connectors_matching_source(&self, id: &str, port: &str) -> Vec<(String, String)> {
        self.connectors.iter()
            .filter_map(|connector| {
                if connector.source() == (&id.to_string(), &port.to_string()) {
                    Some((connector.target_id().to_string(), connector.target_port().to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    ///Given a model and an outgoing internal message, find the next model and next port for the internal message.
    pub fn next_model_messages(&self, source_model_id: &str, message: &ModelMessage) -> Vec<Message> {
        self.find_target_connectors_matching_source(source_model_id, message.port_name.as_str())
            .into_iter()
            .map(|(target_model, target_port)| {
                Message::new(source_model_id.to_string(), message.port_name.clone(), target_model, target_port, self.services.global_time(), message.content.clone())
            }).collect()
    }

    pub fn process_external_messages(&mut self) -> Result<(), SimulationError> {
        for m in self.models.borrow_mut().iter_mut() {
            let internal_messages: Vec<ModelMessage> = self.messages.iter()
                .filter_map(|message| {
                    if message.target_id() == m.id() {
                        Some(ModelMessage::new(message.target_port(), message.content()))
                    } else {
                        None
                    }
                }).collect();
            for message in internal_messages {
                m.events_ext(&message.clone(), &mut self.services)?;
            }
        }
        Ok(())
    }

    pub fn advance_time(&mut self) -> SDuration {
        let until_next_event: SDuration = if self.messages.is_empty() {
            self.models.borrow().iter().fold(SDuration::INFINITY, |min, model| {
                SDuration::min(min, model.until_next_event())
            })
        } else {
            SDuration::NOW
        };

        self.models.borrow_mut().iter_mut().for_each(|model| {
            model.time_advance(until_next_event);
        });
        self.services.advance_global_time(until_next_event);
        until_next_event
    }

    pub fn process_internal_events(&self) -> Result<Vec<Message>, SimulationError> {
        let results: Result<Vec<Vec<Message>>, SimulationError> = self.models
            .borrow_mut()
            .iter_mut()
            .filter(|model| model.until_next_event() == SDuration::NOW) //Only need to work on things that are now due
            .map(|source_model| {
                //Given the model, trigger any internal events.  Internal events my produce outbound messages or may error.
                let emitted_internal_messages = source_model.events_int(&self.services)?;

                let new_messages = emitted_internal_messages
                    .iter()
                    .map(|emitted_internal_message|
                        self.next_model_messages(source_model.id(), &emitted_internal_message))
                    .flatten()
                    .collect::<Vec<Message>>();

                Ok(new_messages)
            })
            .collect();

        let new_messages = results?.into_iter().flatten().collect();
        Ok(new_messages)
    }


    /// The simulation step is foundational for a discrete event simulation.
    /// This method executes a single discrete event simulation step,
    /// including internal state transitions, external state transitions,
    /// message orchestration, global time accounting, and step messages
    /// output.
    pub fn step(&mut self) -> Result<Vec<Message>, SimulationError> {
        self.process_external_messages()?;
        self.advance_time();
        let new_messages = self.process_internal_events()?;
        self.messages = new_messages.clone();
        Ok(new_messages)
    }

    /// This method executes simulation `step` calls, until a global time
    /// has been exceeded.  At which point, the messages from all the
    /// simulation steps are returned.
    pub fn step_until(&mut self, until: STime) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
        loop {
            self.step()?;
            if self.services.global_time() < until {
                message_records.extend(self.get_messages().clone());
            } else {
                break;
            }
        }
        Ok(message_records)
    }

    /// This method executes the specified number of simulation steps, `n`.
    /// Upon execution of the n steps, the messages from all the steps are
    /// returned.
    pub fn step_n(&mut self, n: usize) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
        (0..n)
            .map(|_| -> Result<Vec<Message>, SimulationError> {
                self.step()?;
                message_records.extend(self.messages.clone());
                Ok(Vec::new())
            })
            .find(Result::is_err)
            .unwrap_or(Ok(message_records))
    }
}
