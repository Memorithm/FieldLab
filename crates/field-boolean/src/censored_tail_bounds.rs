//! Exact partial-identification bounds for boundary-censored Boolean dwell tails.
//!
//! A boundary-censored observed dwell is a lower bound on its unobserved complete
//! dwell length.  For a caller-declared positive observation threshold `t`, an
//! observed length at least `t` therefore proves that the complete dwell is at
//! least `t`; a censored observed length below `t` leaves that statement
//! unresolved.  This module records those exact count bounds without fitting a
//! survival distribution, imputing a censored duration, or declaring stability.

use crate::{
    annotate_predicate_dwell_censoring, ObservationBoundary, PredicateDwellCensoringError,
    PredicateDwellRun,
};

/// Versioned contract for exact censor-aware dwell-tail count bounds.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_BOUNDS_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-bounds.v1";

/// Exact count bounds for one Boolean value at one declared dwell threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredicateDwellTailCountBounds {
    /// Number of source runs with the selected Boolean value.
    pub source_runs: usize,
    /// Runs whose observed length already proves complete length `>= threshold`.
    pub definite_at_least_threshold: usize,
    /// Maximum runs that could have complete length `>= threshold` without
    /// contradicting the retained observations and censoring declarations.
    pub possible_at_least_threshold: usize,
    /// Complete runs observed to end below the threshold.
    pub complete_below_threshold: usize,
    /// Boundary-censored runs observed below the threshold; their complete length
    /// is unresolved and creates the gap between lower and upper counts.
    pub censored_below_threshold: usize,
}

/// Exact tail-count bounds for both Boolean values at one declared threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensorAwarePredicateDwellTailBounds {
    /// Caller-declared strictly positive threshold in observation counts.
    pub threshold_observations: usize,
    /// Bounds for `false` dwell runs.
    pub false_runs: PredicateDwellTailCountBounds,
    /// Bounds for `true` dwell runs.
    pub true_runs: PredicateDwellTailCountBounds,
}

/// Failure while constructing exact censor-aware dwell-tail bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CensorAwareDwellTailBoundsError {
    /// Threshold zero is vacuous and is rejected to keep the contract explicit.
    ZeroThreshold,
    /// Source maximal runs or their boundary declaration are invalid.
    InvalidSource(PredicateDwellCensoringError),
    /// An exact counter could not be represented as `usize`.
    CountOverflow,
}

impl From<PredicateDwellCensoringError> for CensorAwareDwellTailBoundsError {
    fn from(value: PredicateDwellCensoringError) -> Self {
        Self::InvalidSource(value)
    }
}

/// Bound the number of complete dwells at least as long as `threshold_observations`.
///
/// The bounds use only the exact observed run lengths and the caller-declared
/// observation-window censoring.  They do not assume a censoring distribution,
/// stationarity, independent censoring, a hazard model, or an attractor model.
/// A censored run observed below the threshold contributes only to the upper
/// bound; a censored run already observed at or above the threshold contributes
/// to both bounds because its complete dwell can only be at least as long.
///
/// # Errors
///
/// Returns [`CensorAwareDwellTailBoundsError::ZeroThreshold`] for threshold zero,
/// or propagates canonical dwell/censoring validation and exact-count overflow.
pub fn predicate_censor_aware_dwell_tail_bounds(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    threshold_observations: usize,
) -> Result<CensorAwarePredicateDwellTailBounds, CensorAwareDwellTailBoundsError> {
    if threshold_observations == 0 {
        return Err(CensorAwareDwellTailBoundsError::ZeroThreshold);
    }
    let annotated = annotate_predicate_dwell_censoring(runs, left_boundary, right_boundary)?;

    let mut false_counts = MutableTailCounts::default();
    let mut true_counts = MutableTailCounts::default();
    for item in annotated {
        let counts = if item.run.value {
            &mut true_counts
        } else {
            &mut false_counts
        };
        counts.source_runs = checked_add(counts.source_runs, 1)?;
        if item.run.observations >= threshold_observations {
            counts.definite_at_least_threshold =
                checked_add(counts.definite_at_least_threshold, 1)?;
            continue;
        }
        if item.left_censored || item.right_censored {
            counts.censored_below_threshold = checked_add(counts.censored_below_threshold, 1)?;
        } else {
            counts.complete_below_threshold = checked_add(counts.complete_below_threshold, 1)?;
        }
    }

    Ok(CensorAwarePredicateDwellTailBounds {
        threshold_observations,
        false_runs: false_counts.finish()?,
        true_runs: true_counts.finish()?,
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MutableTailCounts {
    source_runs: usize,
    definite_at_least_threshold: usize,
    complete_below_threshold: usize,
    censored_below_threshold: usize,
}

impl MutableTailCounts {
    fn finish(self) -> Result<PredicateDwellTailCountBounds, CensorAwareDwellTailBoundsError> {
        let possible_at_least_threshold = checked_add(
            self.definite_at_least_threshold,
            self.censored_below_threshold,
        )?;
        let classified_runs = checked_add(
            checked_add(
                self.definite_at_least_threshold,
                self.complete_below_threshold,
            )?,
            self.censored_below_threshold,
        )?;
        debug_assert_eq!(classified_runs, self.source_runs);
        Ok(PredicateDwellTailCountBounds {
            source_runs: self.source_runs,
            definite_at_least_threshold: self.definite_at_least_threshold,
            possible_at_least_threshold,
            complete_below_threshold: self.complete_below_threshold,
            censored_below_threshold: self.censored_below_threshold,
        })
    }
}

fn checked_add(left: usize, right: usize) -> Result<usize, CensorAwareDwellTailBoundsError> {
    left.checked_add(right)
        .ok_or(CensorAwareDwellTailBoundsError::CountOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(value: bool, start_observation: usize, observations: usize) -> PredicateDwellRun {
        PredicateDwellRun {
            value,
            start_observation,
            observations,
        }
    }

    #[test]
    fn censored_short_boundary_runs_widen_only_the_upper_count() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 5),
            run(false, 7, 4),
            run(true, 11, 2),
        ];
        let bounds = predicate_censor_aware_dwell_tail_bounds(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            4,
        )
        .unwrap();

        assert_eq!(
            bounds.false_runs,
            PredicateDwellTailCountBounds {
                source_runs: 2,
                definite_at_least_threshold: 1,
                possible_at_least_threshold: 2,
                complete_below_threshold: 0,
                censored_below_threshold: 1,
            }
        );
        assert_eq!(
            bounds.true_runs,
            PredicateDwellTailCountBounds {
                source_runs: 2,
                definite_at_least_threshold: 1,
                possible_at_least_threshold: 2,
                complete_below_threshold: 0,
                censored_below_threshold: 1,
            }
        );
    }

    #[test]
    fn complete_short_runs_are_definitely_below_threshold() {
        let runs = [run(false, 0, 2), run(true, 2, 5), run(false, 7, 3)];
        let bounds = predicate_censor_aware_dwell_tail_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            4,
        )
        .unwrap();
        assert_eq!(bounds.false_runs.definite_at_least_threshold, 0);
        assert_eq!(bounds.false_runs.possible_at_least_threshold, 0);
        assert_eq!(bounds.false_runs.complete_below_threshold, 2);
        assert_eq!(bounds.false_runs.censored_below_threshold, 0);
        assert_eq!(bounds.true_runs.definite_at_least_threshold, 1);
        assert_eq!(bounds.true_runs.possible_at_least_threshold, 1);
    }

    #[test]
    fn long_censored_run_is_already_a_definite_tail_member() {
        let runs = [run(true, 0, 7)];
        let bounds = predicate_censor_aware_dwell_tail_bounds(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            5,
        )
        .unwrap();
        assert_eq!(bounds.true_runs.source_runs, 1);
        assert_eq!(bounds.true_runs.definite_at_least_threshold, 1);
        assert_eq!(bounds.true_runs.possible_at_least_threshold, 1);
        assert_eq!(bounds.true_runs.censored_below_threshold, 0);
    }

    #[test]
    fn zero_threshold_fails_instead_of_creating_a_vacuous_bound() {
        let runs = [run(true, 0, 1)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_bounds(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                0,
            ),
            Err(CensorAwareDwellTailBoundsError::ZeroThreshold)
        );
    }

    #[test]
    fn malformed_runs_still_fail_through_canonical_validation() {
        let runs = [run(false, 0, 2), run(false, 2, 1)];
        assert!(matches!(
            predicate_censor_aware_dwell_tail_bounds(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                2,
            ),
            Err(CensorAwareDwellTailBoundsError::InvalidSource(_))
        ));
    }
}
