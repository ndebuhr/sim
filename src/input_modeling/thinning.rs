use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ThinningFunction {
    // Coefficients, from the highest order coefficient to the zero order coefficient
    Polynomial { coefficients: Vec<f64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thinning {
    // Normalized thinning function with max(fn) = 1 over the support
    function: ThinningFunction,
}

impl Thinning {
    pub fn evaluate(self, point: f64) -> f64 {
        match &self.function {
            ThinningFunction::Polynomial { coefficients } => {
                utils::evaluate_polynomial(&coefficients, point)
            }
        }
    }
}
