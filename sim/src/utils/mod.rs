//! The utilies module provides general capabilities, that may span the
//! input modeling, models, output analysis, and simulator modules.  The
//! utilities are centered around debugging/traceability and common
//! arithmetic.

pub mod errors;

use errors::SimulationError;

/// The function evaluates a polynomial at a single value, with coefficients
/// defined as a slice, from the highest polynomial order to the zero order.
/// Horner's method is used for this polynomial evaluation
pub fn evaluate_polynomial(coefficients: &[f64], x: f64) -> Result<f64, SimulationError> {
    //Problem.  The comment above describes the coefficient order from highest to zero order.  That
    // is different that what is specified in the horner fold algorithm.  So need to honor the commented requirement.
    // https://users.rust-lang.org/t/reversing-an-array/44975
    // hopefully this is a small list of coefficients so a copy is acceptable.
    let h_coeff: Vec<f64> = coefficients.iter().copied().rev().collect();
    Ok(horner_fold(&h_coeff, x))
}


/// Horner Algorithm for polynomial evaluation
/// It is expected that the coefficents are ordered from least significant to most significnat.
/// For example for the polynomial:
/// 2x^4 -3x^3 + x^2 -2x + 3 the coefficients would be presented as
/// vec![3.0, -2.0, 1.0, -3.0, 2.0]
///
/// https://rosettacode.org/wiki/Horner%27s_rule_for_polynomial_evaluation#Rust
fn horner_fold(coefficients: &[f64], x: f64) -> f64 {
    coefficients.iter().rev().fold(0.0, |acc, &a| acc * x + a)
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
