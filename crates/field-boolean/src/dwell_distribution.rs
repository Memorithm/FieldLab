//! Exact descriptive distributions for validated Boolean dwell runs.
//!
//! These histograms preserve the complete observed run-length distribution for
//! each Boolean value. They are observational only: no duration is interpreted
//! as stability, an attractor, a bifurcation, hysteresis, or a controller gate.

use crate::{summarize_predicate_dwell_runs, PredicateDwellRun, PredicateDwellSummaryError};

/// Versioned contract for exact Boolean dwell-run distributions.
pub const BOOLEAN_FIELD_DWELL_DISTRIBUTION_SCHEMA: &str = "fieldlab.boolean-dwell-distribution.v1";

/// One exact observed dwell length and the number of maximal runs having it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredicateDwellLengthCount {
    /// Exact run duration in observations.
    pub observations: usize,
    /// Number of maximal runs with this exact duration.
    pub runs: usize,
}

/// Exact sparse run-length distributions for one validated Boolean trajectory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateDwellDistribution {
    /// Number of observations represented by the source trajectory.
    pub observations: usize,
    /// Number of value changes between adjacent maximal runs.
    pub transitions: usize,
    /// Ascending exact run-length counts for maximal `false` runs.
    pub false_run_lengths: Vec<PredicateDwellLengthCount>,
    /// Ascending exact run-length counts for maximal `true` runs.
    pub true_run_lengths: Vec<PredicateDwellLengthCount>,
}

/// Failure while constructing an exact dwell distribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellDistributionError {
    /// The supplied public dwell-run values are not one canonical trajectory.
    InvalidRuns(PredicateDwellSummaryError),
    /// A run-count accumulator overflowed `usize`.
    CountOverflow,
    /// Storage proportional to the number of source runs could not be reserved.
    AllocationFailed,
}

impl From<PredicateDwellSummaryError> for PredicateDwellDistributionError {
    fn from(value: PredicateDwellSummaryError) -> Self {
        Self::InvalidRuns(value)
    }
}

/// Validates one maximal dwell-run sequence and returns exact sparse run-length
/// distributions for `false` and `true` runs.
///
/// Storage is bounded by the number of supplied runs, not by the largest
/// declared observation count. Only observed run lengths are represented and
/// entries are returned in ascending duration order.
///
/// # Errors
///
/// Returns a structural validation error for malformed public runs, an overflow
/// error when an exact frequency cannot be represented, or an allocation error
/// if run-count-bounded storage cannot be reserved.
pub fn predicate_dwell_distribution(
    runs: &[PredicateDwellRun],
) -> Result<PredicateDwellDistribution, PredicateDwellDistributionError> {
    let summary = summarize_predicate_dwell_runs(runs)?;
    let false_run_lengths = run_length_counts(runs, false)?;
    let true_run_lengths = run_length_counts(runs, true)?;

    debug_assert_eq!(
        false_run_lengths.iter().map(|entry| entry.runs).sum::<usize>(),
        summary.false_runs
    );
    debug_assert_eq!(
        true_run_lengths.iter().map(|entry| entry.runs).sum::<usize>(),
        summary.true_runs
    );

    Ok(PredicateDwellDistribution {
        observations: summary.observations,
        transitions: summary.transitions,
        false_run_lengths,
        true_run_lengths,
    })
}

fn run_length_counts(
    runs: &[PredicateDwellRun],
    value: bool,
) -> Result<Vec<PredicateDwellLengthCount>, PredicateDwellDistributionError> {
    let matching_runs = runs.iter().filter(|run| run.value == value).count();
    let mut lengths = Vec::new();
    lengths
        .try_reserve_exact(matching_runs)
        .map_err(|_| PredicateDwellDistributionError::AllocationFailed)?;
    lengths.extend(
        runs.iter()
            .filter(|run| run.value == value)
            .map(|run| run.observations),
    );
    lengths.sort_unstable();

    let mut counts = Vec::new();
    counts
        .try_reserve_exact(matching_runs)
        .map_err(|_| PredicateDwellDistributionError::AllocationFailed)?;
    for observations in lengths {
        match counts.last_mut() {
            Some(PredicateDwellLengthCount {
                observations: previous,
                runs: count,
            }) if *previous == observations => {
                *count = count
                    .checked_add(1)
                    .ok_or(PredicateDwellDistributionError::CountOverflow)?;
            }
            _ => counts.push(PredicateDwellLengthCount {
                observations,
                runs: 1,
            }),
        }
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(observations: usize, runs: usize) -> PredicateDwellLengthCount {
        PredicateDwellLengthCount { observations, runs }
    }

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
        assert_eq!(distribution.false_run_lengths, vec![count(2, 1), count(4, 1)]);
        assert_eq!(distribution.true_run_lengths, vec![count(3, 2)]);
    }

    #[test]
    fn single_value_trajectory_keeps_other_distribution_empty() {
        let runs = [PredicateDwellRun {
            value: true,
            start_observation: 0,
            observations: 5,
        }];

        let distribution = predicate_dwell_distribution(&runs).unwrap();
        assert_eq!(distribution.observations, 5);
        assert_eq!(distribution.transitions, 0);
        assert!(distribution.false_run_lengths.is_empty());
        assert_eq!(distribution.true_run_lengths, vec![count(5, 1)]);
    }

    #[test]
    fn very_large_duration_does_not_drive_dense_allocation() {
        let runs = [PredicateDwellRun {
            value: true,
            start_observation: 0,
            observations: usize::MAX,
        }];

        let distribution = predicate_dwell_distribution(&runs).unwrap();
        assert_eq!(distribution.observations, usize::MAX);
        assert!(distribution.false_run_lengths.is_empty());
        assert_eq!(distribution.true_run_lengths, vec![count(usize::MAX, 1)]);
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
    fn distribution_run_counts_match_source_run_count() {
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
        let counted_runs = distribution
            .false_run_lengths
            .iter()
            .chain(&distribution.true_run_lengths)
            .map(|entry| entry.runs)
            .sum::<usize>();
        assert_eq!(counted_runs, runs.len());
    }
}
