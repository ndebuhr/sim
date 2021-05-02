use crate::models::ModelType;
use crate::simulator::Services;
use js_sys::Array;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use wasm_bindgen::prelude::*;

use crate::utils::set_panic_hook;

use super::Simulation;

/// The `WebSimulation` provides JS/WASM-compatible interfaces to the core
/// `Simulation` struct.  For additional insight on these methods, refer to
/// the associated `Simulation` methods.  Errors are unwrapped, instead of
/// returned, in the `WebSimulation` methods.
#[wasm_bindgen]
#[derive(SmartDefault, Serialize, Deserialize)]
pub struct WebSimulation {
    #[default(_code = "Simulation::default()")]
    simulation: Simulation<ModelType>,
}

#[wasm_bindgen]
impl WebSimulation {
    /// A JS/WASM interface for `Simulation.post`, which uses JSON
    /// representations of the simulation models and connectors.
    pub fn post_json(models: &str, connectors: &str) -> Self {
        set_panic_hook();
        Self {
            simulation: Simulation {
                models: serde_json::from_str(models).unwrap(),
                connectors: serde_json::from_str(connectors).unwrap(),
                messages: vec![],
                services: Services::default(),
            },
        }
    }

    /// A JS/WASM interface for `Simulation.put`, which uses JSON
    /// representations of the simulation models and connectors.
    pub fn put_json(&mut self, models: &str, connectors: &str) {
        self.simulation.models = serde_json::from_str(models).unwrap();
        self.simulation.connectors = serde_json::from_str(connectors).unwrap();
    }

    /// Get a JSON representation of the full `Simulation` configuration.
    pub fn get_json(&self) -> String {
        serde_json::to_string_pretty(&self.simulation).unwrap()
    }

    /// A JS/WASM interface for `Simulation.post`, which uses YAML
    /// representations of the simulation models and connectors.
    pub fn post_yaml(models: &str, connectors: &str) -> WebSimulation {
        set_panic_hook();
        Self {
            simulation: Simulation {
                models: serde_yaml::from_str(models).unwrap(),
                connectors: serde_yaml::from_str(connectors).unwrap(),
                messages: vec![],
                services: Services::default(),
            },
        }
    }

    /// A JS/WASM interface for `Simulation.put`, which uses YAML
    /// representations of the simulation models and connectors.
    pub fn put_yaml(&mut self, models: &str, connectors: &str) {
        self.simulation.models = serde_yaml::from_str(models).unwrap();
        self.simulation.connectors = serde_yaml::from_str(connectors).unwrap();
    }

    /// Get a YAML representation of the full `Simulation` configuration.
    pub fn get_yaml(&self) -> String {
        serde_yaml::to_string(&self.simulation).unwrap()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a JavaScript Array.
    pub fn get_messages_js(&self) -> Array {
        // Workaround for https://github.com/rustwasm/wasm-bindgen/issues/111
        self.simulation
            .get_messages()
            .clone()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a JSON string.
    pub fn get_messages_json(&self) -> String {
        serde_json::to_string(&self.simulation.get_messages()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.get_messages`, which converts the
    /// messages to a YAML string.
    pub fn get_messages_yaml(&self) -> String {
        serde_yaml::to_string(&self.simulation.get_messages()).unwrap()
    }

    /// An interface to `Simulation.get_global_time`.
    pub fn get_global_time(&self) -> f64 {
        self.simulation.get_global_time()
    }

    /// An interface to `Simulation.status`.
    pub fn status(&self, model_id: &str) -> String {
        self.simulation.status(model_id).unwrap()
    }

    /// An interface to `Simulation.reset`.
    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    /// An interface to `Simulation.reset_messages`.
    pub fn reset_messages(&mut self) {
        self.simulation.reset_messages();
    }

    /// An interface to `Simulation.reset_global_time`
    pub fn reset_global_time(&mut self) {
        self.simulation.reset_global_time();
    }

    /// A JS/WASM interface for `Simulation.inject_input`, which uses a JSON
    /// representation of the injected messages.
    pub fn inject_input_json(&mut self, message: &str) {
        self.simulation
            .inject_input(serde_json::from_str(message).unwrap());
    }

    /// A JS/WASM interface for `Simulation.inject_input`, which uses a YAML
    /// representation of the injected messages.
    pub fn inject_input_yaml(&mut self, message: &str) {
        self.simulation
            .inject_input(serde_yaml::from_str(message).unwrap());
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_js(&mut self) -> Array {
        self.simulation
            .step()
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a JSON string.
    pub fn step_json(&mut self) -> String {
        serde_json::to_string(&self.simulation.step().unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step`, which converts the
    /// returned messages to a YAML string.
    pub fn step_yaml(&mut self) -> String {
        serde_yaml::to_string(&self.simulation.step().unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_until_js(&mut self, until: f64) -> Array {
        self.simulation
            .step_until(until)
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a JSON string.
    pub fn step_until_json(&mut self, until: f64) -> String {
        serde_json::to_string(&self.simulation.step_until(until).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_until`, which converts the
    /// returned messages to a YAML string.
    pub fn step_until_yaml(&mut self, until: f64) -> String {
        serde_yaml::to_string(&self.simulation.step_until(until).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a JavaScript Array.
    pub fn step_n_js(&mut self, n: usize) -> Array {
        self.simulation
            .step_n(n)
            .unwrap()
            .into_iter()
            .map(JsValue::from)
            .collect()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a JSON string.
    pub fn step_n_json(&mut self, n: usize) -> String {
        serde_json::to_string(&self.simulation.step_n(n).unwrap()).unwrap()
    }

    /// A JS/WASM interface for `Simulation.step_n`, which converts the
    /// returned messages to a YAML string.
    pub fn step_n_yaml(&mut self, n: usize) -> String {
        serde_yaml::to_string(&self.simulation.step_n(n).unwrap()).unwrap()
    }
}
