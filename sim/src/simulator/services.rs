use serde::{Deserialize, Serialize};

use crate::input_modeling::dynamic_rng::{default_rng, DynRng};
use crate::simulator::time::{SDuration, STime};

/// The simulator provides a uniform random number generator and simulation
/// clock to models during the execution of a simulation
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Services {
    #[serde(skip, default = "default_rng")]
    pub(crate) global_rng: DynRng,
    pub(crate) global_time: STime,
}

impl Services {
    pub fn advance_global_time(&mut self, duration: SDuration) {
        self.set_global_time(self.global_time() + duration);    }
}

impl Default for Services {
    fn default() -> Self {
        Self {
            global_rng: default_rng(),
            global_time: STime::NOW,
        }
    }
}

impl Services {
    pub fn global_rng(&self) -> DynRng {
        self.global_rng.clone()
    }

    pub fn global_time(&self) -> STime {
        self.global_time
    }

    pub fn set_global_time(&mut self, time: STime) {
        self.global_time = time;
    }
}
