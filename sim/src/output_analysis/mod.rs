//! The output analysis module provides standard statistical analysis tools
//! for analyzing simulation outputs.  Independent, identically-distributed
//! (IID) samples are analyzed with the `IndependentSample`.  Time series
//! (including those with initialization bias and autocorrelation) can be
//! analyzed with `TerminatingSimulationOutput` or `SteadyStateOutput`.

use std::f64::INFINITY;

use num_traits::{Float, NumAssign};
use serde::{Deserialize, Serialize};

pub mod t_scores;
use crate::utils::errors::SimulationError;
use crate::utils::usize_sqrt;

fn sum<T: Float>(points: &[T]) -> T
where
    f64: Into<T>,
{
    points.iter().fold(0.0.into(), |sum, point| sum + *point)
}

/// This function calculates the sample mean from a set of points - a simple
/// arithmetic mean.
fn sample_mean<T: Float>(points: &[T]) -> Result<T, SimulationError>
where
    f64: Into<T>,
{
    Ok(sum(points) / usize_to_float(points.len())?)
}

/// This function calculates sample variance, given a set of points and the
/// sample mean.
fn sample_variance<T: Float>(points: &[T], mean: &T) -> Result<T, SimulationError>
where
    f64: Into<T>,
{
    Ok(points
        .iter()
        .fold(0.0.into(), |acc, point| acc + (*point - *mean).powi(2))
        / usize_to_float(points.len())?)
}

/// This function converts a usize to a Float, with an associated
/// `SimulationError` returned for failed conversions
fn usize_to_float<T: Float>(unconv: usize) -> Result<T, SimulationError> {
    T::from(unconv).ok_or(SimulationError::FloatConvError)
}

/// The confidence interval provides an upper and lower estimate on a given
/// output, whether that output is an independent, identically-distributed
/// sample or time series data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval<T: Float> {
    lower: T,
    upper: T,
}

impl<T: Float> ConfidenceInterval<T>
where
    f64: Into<T>,
{
    pub fn lower(&self) -> T {
        self.lower
    }

    pub fn upper(&self) -> T {
        self.upper
    }

    pub fn half_width(&self) -> T {
        (self.upper - self.lower) / 2.0.into()
    }
}

/// The independent sample is for independent, identically-distributed (IID)
/// samples, or where treating the data as an IID sample is determined to be
/// reasonable.  Typically, this will be non-time series data - no
/// autocorrelation.  There are no additional requirements on the data beyond
/// being IID.  For example, there are no normality assumptions.  The
/// `TerminatingSimulationOutput` or `SteadyStateOutput` structs are
/// available for non-IID output analysis.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IndependentSample<T> {
    points: Vec<T>,
    mean: T,
    variance: T,
}

impl<T: Float> IndependentSample<T>
where
    f64: Into<T>,
{
    /// This constructor method creates an `IndependentSample` from a vector
    /// of floating point values.
    pub fn post(points: Vec<T>) -> Result<IndependentSample<T>, SimulationError> {
        let mean = sample_mean(&points)?;
        let variance = sample_variance(&points, &mean)?;
        Ok(IndependentSample {
            points,
            mean,
            variance,
        })
    }

    /// Calculate the confidence interval of the mean, base on the provided
    /// value of alpha.
    pub fn confidence_interval_mean(
        &self,
        alpha: T,
    ) -> Result<ConfidenceInterval<T>, SimulationError> {
        if self.points.len() == 1 {
            return Ok(ConfidenceInterval {
                lower: self.mean,
                upper: self.mean,
            });
        }
        let points_len: T = usize_to_float(self.points.len())?;
        Ok(ConfidenceInterval {
            lower: self.mean
                - t_scores::t_score(alpha, self.points.len() - 1) * self.variance.sqrt()
                    / points_len.sqrt(),
            upper: self.mean
                + t_scores::t_score(alpha, self.points.len() - 1) * self.variance.sqrt()
                    / points_len.sqrt(),
        })
    }

    /// Return the sample mean.
    pub fn point_estimate_mean(&self) -> T {
        self.mean
    }

    /// Return the sample variance.
    pub fn variance(&self) -> T {
        self.variance
    }
}

/// Terminating simulations are useful when the initial and final conditions
/// of a simulation are known, and set deliberately to match real world
/// conditions.  For example, a simulation spanning a 9:00 to 17:00 work day
/// might use the terminating simulation approach to simulation experiments
/// and analysis.  These initial and final conditions are known and of
/// interest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TerminatingSimulationOutput<T> {
    time_series_replications: Vec<Vec<T>>,
    replication_means: Vec<T>,
    replications_mean: Option<T>,
    replications_variance: Option<T>,
}

impl<T: Float> TerminatingSimulationOutput<T> {
    /// This `TerminatingSimulationOutput` constructor method creates a new
    /// terminating simulation output, with a single replication.  The
    /// `put_time_series` method is then used to load additional simulation
    /// replications (time series).
    pub fn post(time_series: Vec<T>) -> TerminatingSimulationOutput<T> {
        TerminatingSimulationOutput {
            time_series_replications: vec![time_series],
            replication_means: Vec::new(),
            replications_mean: None,
            replications_variance: None,
        }
    }

    /// This method loads a single simulation replication output into the
    /// `TerminatingSimulationOutput` object.  Typically, simulation analysis
    /// will require many replications, and thus many `put_time_series`
    /// calls.
    pub fn put_time_series(&mut self, time_series: Vec<T>) {
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SteadyStateOutput<T> {
    time_series: Vec<T>,
    /// Points are removed from the beginning of the sample for initialization
    /// bias reduction.
    deletion_point: Option<usize>,
    /// Batching is used to combat autocorrelation in the time series.
    batch_size: Option<usize>,
    batch_count: Option<usize>,
    batch_means: Vec<T>,
    batches_mean: Option<T>,
    batches_variance: Option<T>,
}

impl<T: Float + NumAssign> SteadyStateOutput<T>
where
    f64: Into<T>,
{
    /// This `SteadyStateOutput` constructor method takes the simulation
    /// output time series, as a vector of floating point values.
    pub fn post(time_series: Vec<T>) -> SteadyStateOutput<T> {
        SteadyStateOutput {
            time_series,
            deletion_point: None,
            batch_size: None,
            batch_count: None,
            batch_means: Vec::new(),
            batches_mean: None,
            batches_variance: None,
        }
    }

    /// The steady-state output analysis in `set_to_fixed_budget` analyzes
    /// the time series to determine the appropriate initialization data
    /// deletion and batching strategies.  Initialization data deletion and
    /// batching reduce concerns around initialization bias and
    /// autocorrelation, respectively.  After this method determines the
    /// strategy/configuration, the `calculate_batch_statistics` then
    /// executes the processing.
    fn set_to_fixed_budget(&mut self) -> Result<(), SimulationError> {
        let mut s = 0.0.into();
        let mut q = 0.0.into();
        let mut d = self.time_series.len() - 2;
        let mut mser = vec![0.0.into(); self.time_series.len() - 1];
        let time_series_len: T = usize_to_float(self.time_series.len())?;
        loop {
            s += self.time_series[d + 1];
            q += self.time_series[d + 1].powi(2);
            mser[d] = q - s.powi(2) / (time_series_len - usize_to_float(d)?).powi(3);
            if d == 0 {
                // Find the minimum MSER in the first half of the time series
                let min_mser = (0..(self.time_series.len() - 1) / 2)
                    .fold(INFINITY.into(), |min_mser, mser_index| {
                        min_mser.min(mser[mser_index])
                    });
                // Use that point for deletion determination
                self.deletion_point = mser.iter().position(|mser_value| *mser_value == min_mser);
                break;
            }
            d -= 1;
        }
        // Schmeiser [1982] found that, for a fixed total sample size, there
        // is little benefit from dividing it into more than k = 30 batches,
        // even if we could do so and still retain independence between the
        // batch means.
        let deletion_point = self
            .deletion_point
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let batch_count = usize::min(usize_sqrt(self.time_series.len() - deletion_point), 30);
        self.batch_count = Some(batch_count);
        let batch_size = (self.time_series.len() - deletion_point) / batch_count;
        // if data are left over, eliminate from the beginning
        self.deletion_point = Some(self.time_series.len() - batch_count * batch_size);
        self.batch_size = Some(batch_size);
        Ok(())
    }

    /// After the `set_to_fixed_budget` method analyzes the time series to
    /// determine the appropriate initialization data deletion and batching
    /// configuration, this method uses that configuration for calculation
    /// and processing.  This method stores the batch statistics in the
    /// `SteadyStateOutput` struct, for later use in retrieving point and
    /// confidence interval estimates.
    fn calculate_batch_statistics(&mut self) -> Result<(), SimulationError> {
        if self.batch_count.is_none() {
            self.set_to_fixed_budget()?;
        }
        let deletion_point = self
            .deletion_point
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let batch_size = self
            .batch_size
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let batch_count = self
            .batch_count
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let batch_means: Result<Vec<T>, SimulationError> = (0..batch_count)
            .map(|batch_index| {
                let batch_start_index = deletion_point + batch_size * batch_index;
                let batch_end_index = deletion_point + batch_size * (batch_index + 1);
                let points: Vec<T> = (batch_start_index..batch_end_index)
                    .map(|index| self.time_series[index])
                    .collect();
                sample_mean(&points)
            })
            .collect();
        self.batch_means = batch_means?;
        let batches_mean = sample_mean(&self.batch_means)?;
        self.batches_variance = Some(sample_variance(&self.batch_means, &batches_mean)?);
        self.batches_mean = Some(batches_mean);
        Ok(())
    }

    /// The method provides a confidence interval on the mean, for the
    /// simuation output.  If not already processed, the raw data will first
    /// use standard approaches for initialization bias reduction and
    /// autocorrelation management.
    pub fn confidence_interval_mean(
        &mut self,
        alpha: T,
    ) -> Result<ConfidenceInterval<T>, SimulationError> {
        if self.batches_mean.is_none() {
            self.calculate_batch_statistics()?;
        }
        let batches_mean = self
            .batches_mean
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let batch_count = self
            .batch_count
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        let f_batch_count: T = usize_to_float(batch_count)?;
        let batches_variance = self
            .batches_variance
            .ok_or(SimulationError::PrerequisiteCalcError)?;
        if batch_count == 1 {
            return Ok(ConfidenceInterval {
                lower: batches_mean,
                upper: batches_mean,
            });
        }
        Ok(ConfidenceInterval {
            lower: batches_mean
                - t_scores::t_score(alpha, batch_count) * batches_variance.sqrt()
                    / f_batch_count.sqrt(),
            upper: batches_mean
                + t_scores::t_score(alpha, batch_count - 1) * batches_variance.sqrt()
                    / f_batch_count.sqrt(),
        })
    }

    /// The method provides a point estimate on the mean, for the simulation
    /// output.  If not already processed, the raw data will first use
    /// standard approaches for initialization bias reduction and
    /// autocorrelation management.
    pub fn point_estimate_mean(&mut self) -> Result<T, SimulationError> {
        if self.batches_mean.is_none() {
            self.calculate_batch_statistics()?;
        }
        self.batches_mean
            .ok_or(SimulationError::PrerequisiteCalcError)
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
        let confidence_interval = sample.unwrap().confidence_interval_mean(0.1).unwrap();
        assert!((confidence_interval.lower - 0.7492630635369267).abs() < epsilon());
        assert!((confidence_interval.upper - 1.534736936463073).abs() < epsilon());
    }
}
