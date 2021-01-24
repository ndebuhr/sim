//! The utilies module provides general capabilities, that may span the
//! input modeling, models, output analysis, and simulator modules.  The
//! utilities are centered around debugging/traceability and common
//! arithmetic.

/// This function calculates the sample mean from a set of points - a simple
/// arithmetic mean.
pub fn sample_mean(points: &[f64]) -> f64 {
    points.iter().sum::<f64>() / (points.len() as f64)
}

/// This function calculates sample variance, given a set of points and the
/// sample mean.
pub fn sample_variance(points: &[f64], mean: &f64) -> f64 {
    points
        .iter()
        .fold(0.0, |acc, point| acc + (point - mean).powi(2))
        / (points.len() as f64)
}

pub fn equivalent_f64(a: f64, b: f64) -> bool {
    a - b == 0.0
}

/// The function evaluates a polynomial at a single value, with coefficients
/// defined as a slice, from the highest polynomial order to the zero order.
/// Horner's method is used for this polynomial evaluation
pub fn evaluate_polynomial(coefficients: &[f64], x: f64) -> f64 {
    let highest_order_polynomial_coeff = coefficients.first().unwrap();
    coefficients[0..coefficients.len() - 1]
        .iter()
        .fold(*highest_order_polynomial_coeff, |acc, coefficient| {
            coefficient + x * acc
        })
}

/// When the `console_error_panic_hook` feature is enabled, we can call the
/// `set_panic_hook` function at least once during initialization, and then
/// we will get better error messages if our code ever panics.
///
/// For more details see
/// https://github.com/rustwasm/console_error_panic_hook#readme
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
