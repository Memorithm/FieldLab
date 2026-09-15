//! Exact descriptive summaries for validated Boolean dwell runs.
//!
//! The summary in this module is observational only. It revalidates public
//! [`crate::PredicateDwellRun`] values before counting runs and observations; it
//! does not interpret long runs as stability, introduce hysteresis, select a
//! regime, or modify field dynamics.

use crate::PredicateDwellRun;

/// Versioned contract for exact Boolean dwell-run summaries.
pub const BOOLEAN_FIELD_DWELL_SUMMARY_SCHEMA: &str = "fieldlab.boolean-dwell-summary.v1";

/// Exact counts derived from one canonical dwell-run sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredicateDwellSummary {
    /// Number of field observations represented by all runs.
    pub observations: usize,
    /// Number of maximal runs.
    pub runs: usize,
    /// Number of value changes between adjacent runs.
    pub transitions: usize,
    /// Number of observations carrying `false`.
    pub false_observations: usize,
    /// Number of observations carrying `true`.
    pub true_observations: usize,
    /// Number of maximal `false` runs.
    pub false_runs: usize,
    /// Number of maximal `true` runs.
    pub true_runs: usize,
    /// Longest observed `false` run, or zero if no `false` run exists.
    pub longest_false_run: usize,
    /// Longest observed `true` run, or zero if no `true` run exists.
    pub longest_true_run: usize,
}

/// Failure while validating or summarizing public dwell-run data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellSummaryError {
    /// A dwell-run trajectory must contain at least the initial observation.
    EmptyRuns,
    /// A run cannot contain zero field observations.
    EmptyRun { run_index: usize },
    /// The first run must start at observation zero.
    NonZeroStart { actual_start: usize },
    /// Adjacent runs are not contiguous in observation space.
    NonContiguousRun {
        run_index: usize,
        expected_start: usize,
        actual_start: usize,
    },
    /// Adjacent runs repeat the same value and are therefore not maximal.
    NonMaximalRuns { run_index: usize, value: bool },
    /// A run end or aggregate count cannot be represented as `usize`.
    ObservationCountOverflow,
}

/// Validates and summarizes an exact sequence of maximal Boolean dwell runs.
///
/// Public dwell-run values are revalidated because callers can construct them
/// without using [`crate::predicate_dwell_runs`]. A valid sequence must start at
/// observation zero, contain non-empty contiguous runs and alternate values.
/// Counts are accumulated with checked arithmetic.
///
/// The returned longest-run values are raw descriptive durations in observation
/// count. They are not a stability threshold, bifurcation result, controller
/// debounce window, or evidence of cognition.
///
/// # Errors
///
/// Returns a typed structural error when the supplied public runs cannot be one
/// canonical maximal trajectory, or an overflow error if exact counts cannot be
/// represented.
pub fn summarize_predicate_dwell_runs(
    runs: &[PredicateDwellRun],
) -> Result<PredicateDwellSummary, PredicateDwellSummaryError> {
    let first = runs.first().ok_or(PredicateDwellSummaryError::EmptyRuns)?;
    if first.start_observation != 0 {
        return Err(PredicateDwellSummaryError::NonZeroStart {
            actual_start: first.start_observation,
        });
    }

    let mut observations = 0usize;
    let mut false_observations = 0usize;
    let mut true_observations = 0usize;
    let mut false_runs = 0usize;
    let mut true_runs = 0usize;
    let mut longest_false_run = 0usize;
    let mut longest_true_run = 0usize;
    let mut expected_start = 0usize;
    let mut previous_value = None;

    for (run_index, run) in runs.iter().copied().enumerate() {
        if run.observations == 0 {
            return Err(PredicateDwellSummaryError::EmptyRun { run_index });
        }
        if run.start_observation != expected_start {
            return Err(PredicateDwellSummaryError::NonContiguousRun {
                run_index,
                expected_start,
                actual_start: run.start_observation,
            });
        }
        if previous_value == Some(run.value) {
            return Err(PredicateDwellSummaryError::NonMaximalRuns {
                run_index,
                value: run.value,
            });
        }

        observations = observations
            .checked_add(run.observations)
            .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;
        expected_start = run
            .start_observation
            .checked_add(run.observations)
            .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;

        if run.value {
            true_runs = true_runs
                .checked_add(1)
                .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;
            true_observations = true_observations
                .checked_add(run.observations)
                .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;
            longest_true_run = longest_true_run.max(run.observations);
        } else {
            false_runs = false_runs
                .checked_add(1)
                .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;
            false_observations = false_observations
                .checked_add(run.observations)
                .ok_or(PredicateDwellSummaryError::ObservationCountOverflow)?;
            longest_false_run = longest_false_run.max(run.observations);
        }

        previous_value = Some(run.value);
    }

    let transitions = runs
        .len()
        .checked_sub(1)
        .ok_or(PredicateDwellSummaryError::EmptyRuns)?;

    Ok(PredicateDwellSummary {
        observations,
        runs: runs.len(),
        transitions,
        false_observations,
        true_observations,
        false_runs,
        true_runs,
        longest_false_run,
        longest_true_run,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_exact_run_counts_and_longest_dwells() {
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
        ];

        assert_eq!(
            summarize_predicate_dwell_runs(&runs),
            Ok(PredicateDwellSummary {
                observations: 9,
                runs: 3,
                transitions: 2,
                false_observations: 6,
                true_observations: 3,
                false_runs: 2,
                true_runs: 1,
                longest_false_run: 4,
                longest_true_run: 3,
            })
        );
    }

    #[test]
    fn single_run_has_zero_transitions() {
        let runs = [PredicateDwellRun {
            value: true,
            start_observation: 0,
            observations: 5,
        }];

        assert_eq!(
            summarize_predicate_dwell_runs(&runs),
            Ok(PredicateDwellSummary {
                observations: 5,
                runs: 1,
                transitions: 0,
                false_observations: 0,
                true_observations: 5,
                false_runs: 0,
                true_runs: 1,
                longest_false_run: 0,
                longest_true_run: 5,
            })
        );
    }

    #[test]
    fn rejects_empty_public_run_sequences() {
        assert_eq!(
            summarize_predicate_dwell_runs(&[]),
            Err(PredicateDwellSummaryError::EmptyRuns)
        );
    }

    #[test]
    fn rejects_non_contiguous_public_runs() {
        let runs = [
            PredicateDwellRun {
                value: false,
                start_observation: 0,
                observations: 2,
            },
            PredicateDwellRun {
                value: true,
                start_observation: 3,
                observations: 1,
            },
        ];

        assert_eq!(
            summarize_predicate_dwell_runs(&runs),
            Err(PredicateDwellSummaryError::NonContiguousRun {
                run_index: 1,
                expected_start: 2,
                actual_start: 3,
            })
        );
    }

    #[test]
    fn rejects_adjacent_runs_with_the_same_value() {
        let runs = [
            PredicateDwellRun {
                value: true,
                start_observation: 0,
                observations: 1,
            },
            PredicateDwellRun {
                value: true,
                start_observation: 1,
                observations: 2,
            },
        ];

        assert_eq!(
            summarize_predicate_dwell_runs(&runs),
            Err(PredicateDwellSummaryError::NonMaximalRuns {
                run_index: 1,
                value: true,
            })
        );
    }

    #[test]
    fn rejects_zero_length_run_before_using_its_endpoint() {
        let runs = [PredicateDwellRun {
            value: false,
            start_observation: 0,
            observations: 0,
        }];

        assert_eq!(
            summarize_predicate_dwell_runs(&runs),
            Err(PredicateDwellSummaryError::EmptyRun { run_index: 0 })
        );
    }
}
