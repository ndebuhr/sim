use serde_json;
use sim::utils::evaluate_polynomial;
#[test]
fn test_evaluate_polynomial_base() {
    let coefficients = vec![1.0];
    let y = evaluate_polynomial(&coefficients, 1.0);
    assert!(y.is_ok());
    assert_eq!(y.unwrap(), 1.0);
}

#[test]
fn test_evaluate_polynomial_two() {
    let coefficients = vec![1.0, 0.3];
    let y = evaluate_polynomial(&coefficients, 1.0);
    assert!(y.is_ok());
    assert_eq!(y.unwrap(), 0.3);
}

#[test]
fn test_evaluate_polynomial_sem() {
    // let coefficients = vec![2.0, -3.0, 1.0, -2.0, 3.0];
    let coefficients = vec![3.0, -2.0, 1.0, -3.0, 2.0];
    let x:f64 = 2.0;
    let actual_y = evaluate_polynomial(&coefficients, 2.0).ok().unwrap();
    let expected_y = (2.0 * x.powf(4.0))
        + (-3.0 * x.powf(3.0))
        + (1.0 * x.powf(2.0))
        + (-2.0 * x.powf(1.0))
        + (3.0 * x.powf(0.0));
    assert_eq!(actual_y, expected_y);
}
