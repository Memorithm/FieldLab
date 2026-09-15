//! Exact Boolean dwell runs reconstructed from already-observed transitions.
//!
//! This module is descriptive only. It validates that adjacent transition
//! observations form one contiguous Boolean trajectory, then records exact run
//! lengths. It does not infer thresholds, debounce observations, add hysteresis,
//! choose regimes, or control field dynamics.

use crate::PredicateTransition;

/// Versioned contract for exact Boolean dwell-run reconstruction.
pub const BOOLEAN_FIELD_DWELL_RUN_SCHEMA: &str = "fieldlab.boolean-dwell-run.v1";

/// One maximal contiguous run of an already-observed Boolean predicate value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredicateDwellRun {
    /// Boolean value observed throughout this run.
    pub value: bool,
    /// Zero-based field-observation index where this run starts.
    pub start_observation: usize,
    /// Number of consecutive field observations in this run.
    pub observations: usize,
}

impl PredicateDwellRun {
    /// Inclusive zero-based field-observation index where this run ends.
    ///
    /// Construction guarantees `observations >= 1`, so this arithmetic cannot
    /// underflow. The enclosing reconstruction checks the `trace.len() + 1`
    /// observation count before building any run.
    #[must_use]
    pub const fn end_observation(self) -> usize {
        self.start_observation + self.observations - 1
    }
}

/// Failure while reconstructing exact dwell runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellRunError {
    /// `trace.len() + 1` cannot be represented as `usize`.
    ObservationCountOverflow,
    /// The exact number of maximal runs exceeds the caller-declared bound.
    RunLimitExceeded {
        required_runs: usize,
        max_runs: usize,
    },
    /// A transition's declared previous value does not match the trajectory
    /// implied by earlier transitions.
    DiscontinuousTrace {
        transition_index: usize,
        expected_previous: bool,
        actual_previous: bool,
    },
}

/// Reconstruct exact maximal Boolean dwell runs from an ordered transition trace.
///
/// `initial_value` is the predicate value at field observation zero. A trace of
/// `n` transitions represents exactly `n + 1` field observations. The function
/// first validates the complete trajectory and computes the exact number of runs;
/// only then does it allocate the output vector. `max_runs` therefore bounds the
/// allocation before construction.
///
/// This function never interprets a long dwell as stability evidence or a state
/// change as a regime switch. Those meanings require a separately declared
/// experiment or controller contract.
///
/// # Errors
///
/// Returns [`PredicateDwellRunError::ObservationCountOverflow`] if the trace
/// length cannot be extended by the initial observation,
/// [`PredicateDwellRunError::DiscontinuousTrace`] if adjacent transition
/// categories cannot form one contiguous Boolean trajectory, or
/// [`PredicateDwellRunError::RunLimitExceeded`] before allocation when the exact
/// run count is larger than `max_runs`.
pub fn predicate_dwell_runs(
    initial_value: bool,
    trace: &[PredicateTransition],
    max_runs: usize,
) -> Result<Vec<PredicateDwellRun>, PredicateDwellRunError> {
    let observation_count = trace
        .len()
        .checked_add(1)
        .ok_or(PredicateDwellRunError::ObservationCountOverflow)?;

    let mut current = initial_value;
    let mut required_runs = 1_usize;
    for (transition_index, transition) in trace.iter().copied().enumerate() {
        let (previous, next) = transition_values(transition);
        if previous != current {
            return Err(PredicateDwellRunError::DiscontinuousTrace {
                transition_index,
                expected_previous: current,
                actual_previous: previous,
            });
        }
        if next != current {
            required_runs = required_runs
                .checked_add(1)
                .ok_or(PredicateDwellRunError::ObservationCountOverflow)?;
        }
        current = next;
    }

    if required_runs > max_runs {
        return Err(PredicateDwellRunError::RunLimitExceeded {
            required_runs,
            max_runs,
        });
    }

    let mut runs = Vec::with_capacity(required_runs);
    let mut run_value = initial_value;
    let mut run_start = 0_usize;
    let mut run_observations = 1_usize;

    for (transition_index, transition) in trace.iter().copied().enumerate() {
        let (_, next) = transition_values(transition);
        if next == run_value {
            run_observations += 1;
        } else {
            runs.push(PredicateDwellRun {
                value: run_value,
                start_observation: run_start,
                observations: run_observations,
            });
            run_value = next;
            run_start = transition_index + 1;
            run_observations = 1;
        }
    }

    runs.push(PredicateDwellRun {
        value: run_value,
        start_observation: run_start,
        observations: run_observations,
    });

    debug_assert_eq!(runs.len(), required_runs);
    debug_assert_eq!(
        runs.iter().map(|run| run.observations).sum::<usize>(),
        observation_count
    );
    Ok(runs)
}

const fn transition_values(transition: PredicateTransition) -> (bool, bool) {
    match transition {
        PredicateTransition::StableFalse => (false, false),
        PredicateTransition::Rising => (false, true),
        PredicateTransition::Falling => (true, false),
        PredicateTransition::StableTrue => (true, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trace_retains_the_initial_observation() {
        assert_eq!(
            predicate_dwell_runs(true, &[], 1),
            Ok(vec![PredicateDwellRun {
                value: true,
                start_observation: 0,
                observations: 1,
            }])
        );
    }

    #[test]
    fn reconstructs_exact_maximal_runs() {
        let trace = [
            PredicateTransition::StableFalse,
            PredicateTransition::Rising,
            PredicateTransition::StableTrue,
            PredicateTransition::StableTrue,
            PredicateTransition::Falling,
            PredicateTransition::StableFalse,
        ];
        let runs = predicate_dwell_runs(false, &trace, 3).expect("three exact runs");

        assert_eq!(
            runs,
            vec![
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
                    observations: 2,
                },
            ]
        );
        assert_eq!(runs[0].end_observation(), 1);
        assert_eq!(runs[1].end_observation(), 4);
        assert_eq!(runs[2].end_observation(), 6);
    }

    #[test]
    fn rejects_a_discontinuous_transition_trace() {
        let trace = [
            PredicateTransition::Rising,
            PredicateTransition::StableFalse,
        ];
        assert_eq!(
            predicate_dwell_runs(false, &trace, 2),
            Err(PredicateDwellRunError::DiscontinuousTrace {
                transition_index: 1,
                expected_previous: true,
                actual_previous: false,
            })
        );
    }

    #[test]
    fn run_budget_fails_before_output_construction() {
        let trace = [PredicateTransition::Rising, PredicateTransition::Falling];
        assert_eq!(
            predicate_dwell_runs(false, &trace, 2),
            Err(PredicateDwellRunError::RunLimitExceeded {
                required_runs: 3,
                max_runs: 2,
            })
        );
    }

    #[test]
    fn all_stable_observations_form_one_run() {
        let trace = [
            PredicateTransition::StableTrue,
            PredicateTransition::StableTrue,
            PredicateTransition::StableTrue,
        ];
        assert_eq!(
            predicate_dwell_runs(true, &trace, 1),
            Ok(vec![PredicateDwellRun {
                value: true,
                start_observation: 0,
                observations: 4,
            }])
        );
    }
}
