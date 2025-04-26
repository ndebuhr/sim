use sim::utils::evaluate_polynomial;

#[test]
fn test_evaluate_polynomial_base() {
    let coefficients = vec![1.0];
    let x: f64 = 1.0;
    let actual_y = evaluate_polynomial(&coefficients, x).ok().unwrap();
    let expected_y = 1.0 * x.powf(0.0);
    assert_eq!(actual_y, expected_y);
}

#[test]
fn test_evaluate_polynomial_two() {
    // coefficients ordered as specified in comments.
    let coefficients = vec![1.0, 0.3];
    let x: f64 = 1.0;
    let actual_y = evaluate_polynomial(&coefficients, x).ok().unwrap();
    let expected_y = (1.0 * x.powf(1.0)) + (0.3 * x.powf(0.0));
    assert_eq!(actual_y, expected_y);
}

#[test]
fn test_evaluate_polynomial_sem() {
    // coefficients ordered as specified in comments.
    let coefficients = vec![2.0, -3.0, 1.0, -2.0, 3.0];
    let x: f64 = 2.0;
    let actual_y = evaluate_polynomial(&coefficients, x).ok().unwrap();
    let expected_y = (2.0 * x.powf(4.0))
        + (-3.0 * x.powf(3.0))
        + (1.0 * x.powf(2.0))
        + (-2.0 * x.powf(1.0))
        + (3.0 * x.powf(0.0));
    assert_eq!(actual_y, expected_y);
}
