//! Exact threshold-profile evaluation for censor-aware Boolean dwell tails.
//!
//! This module evaluates a caller-declared, strictly increasing threshold grid
//! with the existing exact partial-identification contracts. It does not choose
//! thresholds from the observed data, fit a survival model, or turn a robust
//! ordering across thresholds into a field-stability or bifurcation verdict.

use crate::{
    predicate_censor_aware_dwell_tail_fraction_bounds, predicate_censor_aware_dwell_tail_ordering,
    CensorAwareDwellTailBoundsError, CensorAwarePredicateDwellTailFractionBounds,
    ObservationBoundary, PredicateDwellRun, PredicateDwellTailOrdering,
};

/// Versioned contract for one caller-declared censor-aware dwell-tail profile.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_PROFILE_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-profile.v1";

/// One exact profile point at one preregistered threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensorAwareDwellTailProfilePoint {
    /// Caller-declared strictly positive threshold in observation counts.
    pub threshold_observations: usize,
    /// Exact fraction bounds for both Boolean values at this threshold.
    pub fraction_bounds: CensorAwarePredicateDwellTailFractionBounds,
    /// Strict interval ordering classification at this threshold.
    pub ordering: PredicateDwellTailOrdering,
}

/// Failure while evaluating a fixed threshold profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CensorAwareDwellTailProfileError {
    /// A profile must contain at least one declared threshold.
    EmptyThresholdGrid,
    /// Thresholds must be strictly increasing so duplicate/reordered probes
    /// cannot be mistaken for independent robustness evidence.
    NonIncreasingThresholdGrid { previous: usize, current: usize },
    /// Canonical censor-aware tail validation failed.
    Tail(CensorAwareDwellTailBoundsError),
}

impl From<CensorAwareDwellTailBoundsError> for CensorAwareDwellTailProfileError {
    fn from(value: CensorAwareDwellTailBoundsError) -> Self {
        Self::Tail(value)
    }
}

/// Evaluate exact censor-aware tail bounds/orderings on a fixed threshold grid.
///
/// The caller owns threshold selection and must freeze it before interpreting
/// measurements. This function preserves every supplied threshold in order and
/// never searches for a threshold that makes one Boolean value look favorable.
///
/// # Errors
///
/// Returns an error for an empty grid, a non-strictly-increasing grid, or any
/// validation failure from the canonical censor-aware tail contracts (including
/// a zero threshold).
pub fn predicate_censor_aware_dwell_tail_profile(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    thresholds_observations: &[usize],
) -> Result<Vec<CensorAwareDwellTailProfilePoint>, CensorAwareDwellTailProfileError> {
    if thresholds_observations.is_empty() {
        return Err(CensorAwareDwellTailProfileError::EmptyThresholdGrid);
    }
    for pair in thresholds_observations.windows(2) {
        if pair[1] <= pair[0] {
            return Err(
                CensorAwareDwellTailProfileError::NonIncreasingThresholdGrid {
                    previous: pair[0],
                    current: pair[1],
                },
            );
        }
    }

    thresholds_observations
        .iter()
        .copied()
        .map(|threshold_observations| {
            let fraction_bounds = predicate_censor_aware_dwell_tail_fraction_bounds(
                runs,
                left_boundary,
                right_boundary,
                threshold_observations,
            )?;
            let ordering = predicate_censor_aware_dwell_tail_ordering(
                runs,
                left_boundary,
                right_boundary,
                threshold_observations,
            )?;
            Ok(CensorAwareDwellTailProfilePoint {
                threshold_observations,
                fraction_bounds,
                ordering,
            })
        })
        .collect()
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
    fn fixed_grid_preserves_every_exact_threshold_result() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 8),
            run(false, 10, 3),
            run(true, 13, 7),
        ];
        let profile = predicate_censor_aware_dwell_tail_profile(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            &[3, 5, 7],
        )
        .unwrap();

        assert_eq!(profile.len(), 3);
        assert_eq!(profile[0].threshold_observations, 3);
        assert_eq!(profile[1].threshold_observations, 5);
        assert_eq!(profile[2].threshold_observations, 7);
        assert_eq!(
            profile[1].ordering,
            PredicateDwellTailOrdering::TrueDefinitelyHigher
        );
        for point in profile {
            assert_eq!(
                point.fraction_bounds.threshold_observations,
                point.threshold_observations
            );
        }
    }

    #[test]
    fn censoring_can_leave_indeterminate_profile_points() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 7),
            run(false, 9, 6),
            run(true, 15, 2),
        ];
        let profile = predicate_censor_aware_dwell_tail_profile(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            &[3, 5],
        )
        .unwrap();
        assert_eq!(
            profile[1].ordering,
            PredicateDwellTailOrdering::IndeterminateOverlap
        );
    }

    #[test]
    fn empty_duplicate_or_reordered_threshold_grids_fail_closed() {
        let runs = [run(false, 0, 2), run(true, 2, 4)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[],
            ),
            Err(CensorAwareDwellTailProfileError::EmptyThresholdGrid)
        );
        assert_eq!(
            predicate_censor_aware_dwell_tail_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[2, 2],
            ),
            Err(
                CensorAwareDwellTailProfileError::NonIncreasingThresholdGrid {
                    previous: 2,
                    current: 2,
                }
            )
        );
        assert_eq!(
            predicate_censor_aware_dwell_tail_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[3, 2],
            ),
            Err(
                CensorAwareDwellTailProfileError::NonIncreasingThresholdGrid {
                    previous: 3,
                    current: 2,
                }
            )
        );
    }

    #[test]
    fn zero_threshold_is_rejected_by_canonical_tail_contract() {
        let runs = [run(false, 0, 2), run(true, 2, 4)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[0, 2],
            ),
            Err(CensorAwareDwellTailProfileError::Tail(
                CensorAwareDwellTailBoundsError::ZeroThreshold
            ))
        );
    }
}
