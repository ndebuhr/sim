use serde::{Deserialize, Serialize};

use crate::input_modeling::UniformRNG;

/// The simulator provides a uniform random number generator and simulation
/// clock to models during the execution of a simulation
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Services {
    #[serde(skip_serializing)]
    uniform_rng: UniformRNG,
    global_time: f64,
}

impl Services {
    pub fn uniform_rng(&mut self) -> &mut UniformRNG {
        &mut self.uniform_rng
    }

    pub fn global_time(&self) -> f64 {
        self.global_time
    }

    pub fn set_global_time(&mut self, time: f64) {
        self.global_time = time;
    }
}
