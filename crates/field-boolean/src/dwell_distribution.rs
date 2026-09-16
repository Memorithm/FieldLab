//! Exact descriptive distributions for validated Boolean dwell runs.
//!
//! These histograms preserve the complete observed run-length distribution for
//! each Boolean value. They are observational only: no duration is interpreted
//! as stability, an attractor, a bifurcation, hysteresis, or a controller gate.

use crate::{summarize_predicate_dwell_runs, PredicateDwellRun, PredicateDwellSummaryError};

/// Versioned contract for exact Boolean dwell-run distributions.
pub const BOOLEAN_FIELD_DWELL_DISTRIBUTION_SCHEMA: &str = "fieldlab.boolean-dwell-distribution.v1";

/// Exact run-length histograms for one validated Boolean trajectory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateDwellDistribution {
    /// Number of observations represented by the source trajectory.
    pub observations: usize,
    /// Number of value changes between adjacent maximal runs.
    pub transitions: usize,
    /// `false_run_lengths[d]` is the number of maximal `false` runs of length `d`.
    pub false_run_lengths: Vec<usize>,
    /// `true_run_lengths[d]` is the number of maximal `true` runs of length `d`.
    pub true_run_lengths: Vec<usize>,
}

/// Failure while constructing an exact dwell distribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellDistributionError {
    /// The supplied public dwell-run values are not one canonical trajectory.
    InvalidRuns(PredicateDwellSummaryError),
    /// Histogram length or a bucket count overflowed `usize`.
    CountOverflow,
    /// Histogram allocation failed.
    AllocationFailed,
}

impl From<PredicateDwellSummaryError> for PredicateDwellDistributionError {
    fn from(value: PredicateDwellSummaryError) -> Self {
        Self::InvalidRuns(value)
    }
}

/// Validates one maximal dwell-run sequence and returns exact run-length
/// histograms for `false` and `true` runs.
///
/// Bucket zero is always present and always zero because canonical dwell runs
/// cannot be empty. Histogram truncation is lossless: both histograms have
/// length `max_observed_run + 1`, where `max_observed_run` is taken across both
/// values.
///
/// # Errors
///
/// Returns a structural validation error for malformed public runs, an overflow
/// error when exact counts cannot be represented, or an allocation error if the
/// bounded histogram cannot be allocated.
pub fn predicate_dwell_distribution(
    runs: &[PredicateDwellRun],
) -> Result<PredicateDwellDistribution, PredicateDwellDistributionError> {
    let summary = summarize_predicate_dwell_runs(runs)?;
    let max_run = summary.longest_false_run.max(summary.longest_true_run);
    let histogram_len = max_run
        .checked_add(1)
        .ok_or(PredicateDwellDistributionError::CountOverflow)?;

    let mut false_run_lengths = Vec::new();
    false_run_lengths
        .try_reserve_exact(histogram_len)
        .map_err(|_| PredicateDwellDistributionError::AllocationFailed)?;
    false_run_lengths.resize(histogram_len, 0usize);

    let mut true_run_lengths = Vec::new();
    true_run_lengths
        .try_reserve_exact(histogram_len)
        .map_err(|_| PredicateDwellDistributionError::AllocationFailed)?;
    true_run_lengths.resize(histogram_len, 0usize);

    for run in runs {
        let bucket = if run.value {
            true_run_lengths
                .get_mut(run.observations)
                .ok_or(PredicateDwellDistributionError::CountOverflow)?
        } else {
            false_run_lengths
                .get_mut(run.observations)
                .ok_or(PredicateDwellDistributionError::CountOverflow)?
        };
        *bucket = bucket
            .checked_add(1)
            .ok_or(PredicateDwellDistributionError::CountOverflow)?;
    }

    debug_assert_eq!(
        false_run_lengths.iter().copied().sum::<usize>(),
        summary.false_runs
    );
    debug_assert_eq!(
        true_run_lengths.iter().copied().sum::<usize>(),
        summary.true_runs
    );

    Ok(PredicateDwellDistribution {
        observations: summary.observations,
        transitions: summary.transitions,
        false_run_lengths,
        true_run_lengths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_complete_run_length_distribution() {
        let runs = [
            PredicateDwellRun {
                value: false,
                start_observation: 0,
                observations: 2,
            },
            PredicateDwellRun {
                value: true,
                start_observation: 2,
                observations: 3,
            },
            PredicateDwellRun {
                value: false,
                start_observation: 5,
                observations: 4,
            },
            PredicateDwellRun {
                value: true,
                start_observation: 9,
                observations: 3,
            },
        ];

        let distribution = predicate_dwell_distribution(&runs).unwrap();
        assert_eq!(distribution.observations, 12);
        assert_eq!(distribution.transitions, 3);
        assert_eq!(distribution.false_run_lengths, vec![0, 0, 1, 0, 1]);
        assert_eq!(distribution.true_run_lengths, vec![0, 0, 0, 2, 0]);
    }

    #[test]
    fn single_value_trajectory_keeps_other_histogram_empty_of_runs() {
        let runs = [PredicateDwellRun {
            value: true,
            start_observation: 0,
            observations: 5,
        }];

        let distribution = predicate_dwell_distribution(&runs).unwrap();
        assert_eq!(distribution.observations, 5);
        assert_eq!(distribution.transitions, 0);
        assert_eq!(distribution.false_run_lengths, vec![0; 6]);
        assert_eq!(distribution.true_run_lengths, vec![0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn malformed_public_runs_fail_closed_through_canonical_validator() {
        let runs = [
            PredicateDwellRun {
                value: false,
                start_observation: 0,
                observations: 2,
            },
            PredicateDwellRun {
                value: false,
                start_observation: 2,
                observations: 1,
            },
        ];

        assert_eq!(
            predicate_dwell_distribution(&runs),
            Err(PredicateDwellDistributionError::InvalidRuns(
                PredicateDwellSummaryError::NonMaximalRuns {
                    run_index: 1,
                    value: false,
                }
            ))
        );
    }

    #[test]
    fn histogram_run_counts_match_source_run_count() {
        let runs = [
            PredicateDwellRun {
                value: true,
                start_observation: 0,
                observations: 1,
            },
            PredicateDwellRun {
                value: false,
                start_observation: 1,
                observations: 2,
            },
            PredicateDwellRun {
                value: true,
                start_observation: 3,
                observations: 4,
            },
        ];
        let distribution = predicate_dwell_distribution(&runs).unwrap();
        let counted_runs = distribution.false_run_lengths.iter().sum::<usize>()
            + distribution.true_run_lengths.iter().sum::<usize>();
        assert_eq!(counted_runs, runs.len());
    }
}
