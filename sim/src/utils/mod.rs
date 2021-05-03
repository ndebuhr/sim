//! The utilies module provides general capabilities, that may span the
//! input modeling, models, output analysis, and simulator modules.  The
//! utilities are centered around debugging/traceability and common
//! arithmetic.

pub mod error;

use error::SimulationError;

/// The function evaluates a polynomial at a single value, with coefficients
/// defined as a slice, from the highest polynomial order to the zero order.
/// Horner's method is used for this polynomial evaluation
pub fn evaluate_polynomial(coefficients: &[f64], x: f64) -> Result<f64, SimulationError> {
    let highest_order_polynomial_coeff = coefficients
        .first()
        .ok_or_else(|| SimulationError::EmptyPolynomial)?;
    Ok(coefficients[0..coefficients.len() - 1]
        .iter()
        .fold(*highest_order_polynomial_coeff, |acc, coefficient| {
            coefficient + x * acc
        }))
}

/// When the `console_error_panic_hook` feature is enabled, we can call the
/// `set_panic_hook` function at least once during initialization, and then
/// we will get better error messages if our code ever panics.
///
/// For more details see
/// <https://github.com/rustwasm/console_error_panic_hook#readme>
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Integer square root calculation, using the Babylonian square-root
/// algorithm.
pub fn usize_sqrt(n: usize) -> usize {
    let mut x = n;
    let mut y = 1;
    while x > y {
        x = (x + y) / 2;
        y = n / x;
    }
    x
}

/// Populate a snapshot metrics port specification with either a None (if the
/// snapshot metrics are not required) or the default Some("snapshot") (if
/// snapshot metrics are required)
pub fn populate_snapshot_port(snapshot_metrics: bool) -> Option<String> {
    if snapshot_metrics {
        Some(String::from("snapshot"))
    } else {
        None
    }
}

/// Populate a history metrics port specification with either a None (if the
/// history metrics are not required) or the default Some("history") (if
/// history metrics are required)
pub fn populate_history_port(history_metrics: bool) -> Option<String> {
    if history_metrics {
        Some(String::from("history"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_usize_sqrt() {
        assert![1 == usize_sqrt(1)];
        assert![1 == usize_sqrt(3)];
        assert![2 == usize_sqrt(4)];
        assert![2 == usize_sqrt(8)];
        assert![3 == usize_sqrt(9)];
        assert![3 == usize_sqrt(15)];
    }
}
