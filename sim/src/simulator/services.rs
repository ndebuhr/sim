use serde::{Deserialize, Serialize};

use crate::input_modeling::dynamic_rng::{default_rng, DynRng};

/// The simulator provides a uniform random number generator and simulation
/// clock to models during the execution of a simulation
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Services {
    #[serde(skip, default = "default_rng")]
    pub(crate) global_rng: DynRng,
    pub(crate) global_time: f64,
}

impl Default for Services {
    fn default() -> Self {
        Self {
            global_rng: default_rng(),
            global_time: 0.0,
        }
    }
}

impl Services {
    pub fn global_rng(&self) -> DynRng {
        self.global_rng.clone()
    }

    pub fn global_time(&self) -> f64 {
        self.global_time
    }

    pub fn set_global_time(&mut self, time: f64) {
        self.global_time = time;
    }
}
