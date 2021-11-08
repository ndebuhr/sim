use serde::{Deserialize, Serialize};

use crate::utils::errors::SimulationError;
use crate::utils::evaluate_polynomial;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ThinningFunction {
    // Coefficients, from the highest order coefficient to the zero order coefficient
    Polynomial { coefficients: Vec<f64> },
}

/// Thinning provides a means for non-stationary stochastic model behaviors.
/// By providing a normalized thinning function (with the maximum value over
/// the support being =1), model behavior will change based on the current
/// global time.  While thinning is a widely generalizable strategy for
/// non-stationary stochastic behaviors, it is very inefficient for models
/// where there is "heavy thinning" during large portions of the simulation
/// execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thinning {
    // Normalized thinning function with max(fn) = 1 over the support
    function: ThinningFunction,
}

impl Thinning {
    pub fn evaluate(self, point: f64) -> Result<f64, SimulationError> {
        match &self.function {
            ThinningFunction::Polynomial { coefficients } => {
                evaluate_polynomial(coefficients, point)
            }
        }
    }
}
