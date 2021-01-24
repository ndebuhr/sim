//! The output analysis module provides standard statistical analysis tools
//! for analyzing simulation outputs.  Independent, identically-distributed
//! (IID) samples are analyzed with the `IndependentSample`.  Time series
//! (including those with initialization bias and autocorrelation) can be
//! analyzed with `TerminatingSimulationOutput` or `SteadyStateOutput`.

use std::f64::INFINITY;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub mod t_scores;
use super::utils;

/// The confidence interval provides an upper and lower estimate on a given
/// output, whether that output is an independent, identically-distributed
/// sample or time series data.
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    lower: f64,
    upper: f64,
}

#[wasm_bindgen]
impl ConfidenceInterval {
    pub fn lower(&self) -> f64 {
        self.lower
    }

    pub fn upper(&self) -> f64 {
        self.upper
    }

    pub fn half_width(&self) -> f64 {
        (self.upper - self.lower) / 2.0
    }
}

/// The independent sample is for independent, identically-distributed (IID)
/// samples, or where treating the data as an IID sample is determined to be
/// reasonable.  Typically, this will be non-time series data - no
/// autocorrelation.  There are no additional requirements on the data beyond
/// being IID.  For example, there are no normality assumptions.  The
/// `TerminatingSimulationOutput` or `SteadyStateOutput` structs are
/// available for non-IID output analysis.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IndependentSample {
    points: Vec<f64>,
    mean: f64,
    variance: f64,
}

#[wasm_bindgen]
impl IndependentSample {
    /// This constructor method creates an `IndependentSample` from a set of
    /// f64 points.
    pub fn post(points: Vec<f64>) -> IndependentSample {
        let mean = utils::sample_mean(&points);
        let variance = utils::sample_variance(&points, &mean);
        IndependentSample {
            points,
            mean,
            variance,
        }
    }

    /// Calculate the confidence interval of the mean, base on the provided
    /// value of alpha.
    pub fn confidence_interval_mean(&self, alpha: f64) -> ConfidenceInterval {
        if self.points.len() == 1 {
            return ConfidenceInterval {
                lower: self.mean,
                upper: self.mean,
            };
        }
        ConfidenceInterval {
            lower: self.mean
                - t_scores::t_score(alpha, self.points.len() - 1) * self.variance.sqrt()
                    / (self.points.len() as f64).sqrt(),
            upper: self.mean
                + t_scores::t_score(alpha, self.points.len() - 1) * self.variance.sqrt()
                    / (self.points.len() as f64).sqrt(),
        }
    }

    /// Return the sample mean.
    pub fn point_estimate_mean(&self) -> f64 {
        self.mean
    }

    /// Return the sample variance.
    pub fn variance(&self) -> f64 {
        self.variance
    }
}

/// Terminating simulations are useful when the initial and final conditions
/// of a simulation are known, and set deliberately to match real world
/// conditions.  For example, a simulation spanning a 9:00 to 17:00 work day
/// might use the terminating simulation approach to simulation experiments
/// and analysis.  These initial and final conditions are known and of
/// interest.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TerminatingSimulationOutput {
    time_series_replications: Vec<Vec<f64>>,
    replication_means: Vec<f64>,
    replications_mean: Option<f64>,
    replications_variance: Option<f64>,
}

#[wasm_bindgen]
impl TerminatingSimulationOutput {
    /// This `TerminatingSimulationOutput` constructor method creates a new
    /// terminating simulation output, with no data.  The `put_time_series`
    /// method is then used to load each simulation replication output (time
    /// series).
    pub fn new() -> TerminatingSimulationOutput {
        TerminatingSimulationOutput {
            ..Default::default()
        }
    }

    /// This method loads a single simulation replication output into the
    /// `TerminatingSimulationOutput` object.  Typically, simulation analysis
    /// will require many replications, and thus many `put_time_series`
    /// calls.
    pub fn put_time_series(&mut self, time_series: Vec<f64>) {
        self.time_series_replications.push(time_series);
    }
}

/// Steady-state simulations are useful when the initial conditions and/or
/// final conditions of a simulation are not well-known or not of interest.
/// Steady-state simulation is interested in the long-run behavior of the
/// system, where initial condition effects are negligible.  Steady-state
/// simulation analysis is primarily concerned with initialization bias (bias
/// caused by setting initial conditions of the simulation) and
/// auto-correlation (the tendency of a data point in a time series to show
/// correlation with the latest, previous values in that time series).  When
/// the interest is a steady-state simulation output, standard simulation
/// design suggests the use of only a single simulation replication.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SteadyStateOutput {
    time_series: Vec<f64>,
    /// Points are removed from the beginning of the sample for initialization
    /// bias reduction.
    deletion_point: Option<usize>,
    /// Batching is used to combat autocorrelation in the time series.
    batch_size: Option<usize>,
    batch_count: Option<usize>,
    batch_means: Vec<f64>,
    batches_mean: Option<f64>,
    batches_variance: Option<f64>,
}

#[wasm_bindgen]
impl SteadyStateOutput {
    /// This `SteadyStateOutput` constructor method takes the simulation
    /// output time series, as a f64 vector.
    pub fn post(time_series: Vec<f64>) -> SteadyStateOutput {
        SteadyStateOutput {
            time_series,
            ..Default::default()
        }
    }

    /// The steady-state output analysis in `set_to_fixed_budget` analyzes
    /// the time series to determine the appropriate initialization data
    /// deletion and batching strategies.  Initialization data deletion and
    /// batching reduce concerns around initialization bias and
    /// autocorrelation, respectively.  After this method determines the
    /// strategy/configuration, the `calculate_batch_statistics` then
    /// executes the processing.
    fn set_to_fixed_budget(&mut self) {
        let mut s = 0.0;
        let mut q = 0.0;
        let mut d = self.time_series.len() - 2;
        let mut mser = vec![0.0; self.time_series.len() - 1];
        loop {
            s += self.time_series[d + 1];
            q += self.time_series[d + 1].powi(2);
            mser[d] = (q - s.powi(2) / ((self.time_series.len() - d) as f64))
                / ((self.time_series.len() - d).pow(2) as f64);
            if d == 0 {
                // Find the minimum MSER in the first half of the time series
                let min_mser = (0..(self.time_series.len() - 1) / 2)
                    .fold(INFINITY, |min_mser, mser_index| {
                        f64::min(min_mser, mser[mser_index])
                    });
                // Use that point for deletion determination
                self.deletion_point = mser
                    .iter()
                    .position(|mser_value| utils::equivalent_f64(*mser_value, min_mser));
                break;
            } else {
                d -= 1;
            }
        }
        // Schmeiser [1982] found that, for a fixed total sample size, there
        // is little benefit from dividing it into more than k = 30 batches,
        // even if we could do so and still retain independence between the
        // batch means.
        self.batch_count = Some(f64::min(
            ((self.time_series.len() - self.deletion_point.unwrap()) as f64).sqrt(),
            30.0,
        ) as usize);
        self.batch_size = Some(
            (self.time_series.len() - self.deletion_point.unwrap()) / self.batch_count.unwrap(),
        );
        // if data are left over, eliminate from the beginning
        self.deletion_point =
            Some(self.time_series.len() - self.batch_count.unwrap() * self.batch_size.unwrap());
    }

    /// After the `set_to_fixed_budget` method analyzes the time series to
    /// determine the appropriate initialization data deletion and batching
    /// configuration, this method uses that configuration for calculation
    /// and processing.  This method stores the batch statistics in the
    /// `SteadyStateOutput` struct, for later use in retrieving point and
    /// confidence interval estimates.
    fn calculate_batch_statistics(&mut self) {
        if self.batch_count.is_none() {
            self.set_to_fixed_budget();
        }
        self.batch_means = (0..self.batch_count.unwrap())
            .map(|batch_index| {
                let batch_start_index =
                    self.deletion_point.unwrap() + self.batch_size.unwrap() * batch_index;
                let batch_end_index =
                    self.deletion_point.unwrap() + self.batch_size.unwrap() * (batch_index + 1);
                let points: Vec<f64> = (batch_start_index..batch_end_index)
                    .map(|index| self.time_series[index])
                    .collect();
                utils::sample_mean(&points)
            })
            .collect();
        self.batches_mean = Some(utils::sample_mean(&self.batch_means));
        self.batches_variance = Some(utils::sample_variance(
            &self.batch_means,
            &self.batches_mean.unwrap(),
        ));
    }

    /// The method provides a confidence interval on the mean, for the
    /// simuation output.  If not already processed, the raw data will first
    /// use standard approaches for initialization bias reduction and
    /// autocorrelation management.
    pub fn confidence_interval_mean(&mut self, alpha: f64) -> ConfidenceInterval {
        if self.batches_mean.is_none() {
            self.calculate_batch_statistics();
        }
        if self.batch_count.unwrap() == 1 {
            return ConfidenceInterval {
                lower: self.batches_mean.unwrap(),
                upper: self.batches_mean.unwrap(),
            };
        }
        ConfidenceInterval {
            lower: self.batches_mean.unwrap()
                - t_scores::t_score(alpha, self.batch_count.unwrap() - 1)
                    * self.batches_variance.unwrap().sqrt()
                    / (self.batch_count.unwrap() as f64).sqrt(),
            upper: self.batches_mean.unwrap()
                + t_scores::t_score(alpha, self.batch_count.unwrap() - 1)
                    * self.batches_variance.unwrap().sqrt()
                    / (self.batch_count.unwrap() as f64).sqrt(),
        }
    }

    /// The method provides a point estimate on the mean, for the simulation
    /// output.  If not already processed, the raw data will first use
    /// standard approaches for initialization bias reduction and
    /// autocorrelation management.
    pub fn point_estimate_mean(&mut self) -> f64 {
        if self.batches_mean.is_none() {
            self.calculate_batch_statistics();
        }
        self.batches_mean.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epsilon() -> f64 {
        1.0e-12
    }

    #[test]
    fn confidence_interval_mean() {
        let sample = IndependentSample::post(vec![
            1.02, 0.73, 3.20, 0.23, 1.76, 0.47, 1.89, 1.45, 0.44, 0.23,
        ]);
        let confidence_interval = sample.confidence_interval_mean(0.1);
        assert!((confidence_interval.lower - 0.7492630635369267).abs() < epsilon());
        assert!((confidence_interval.upper - 1.534736936463073).abs() < epsilon());
    }
}
