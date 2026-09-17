//! Exact fixed-threshold profiles for censor-aware Boolean dwell-tail contrasts.
//!
//! This module evaluates canonical true-minus-false contrast bounds over a
//! caller-declared, strictly increasing threshold grid. It never chooses a
//! threshold from observed data or turns a profile into a stability verdict.

use crate::{
    predicate_censor_aware_dwell_tail_contrast_bounds, CensorAwareDwellTailBoundsError,
    CensorAwarePredicateDwellTailContrastBounds, ObservationBoundary, PredicateDwellRun,
};

/// Versioned contract for one fixed censor-aware dwell-tail contrast profile.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_CONTRAST_PROFILE_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-contrast-profile.v1";

/// One exact contrast interval at one preregistered threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensorAwareDwellTailContrastProfilePoint {
    /// Caller-declared strictly positive threshold in observation counts.
    pub threshold_observations: usize,
    /// Exact partial-identification interval for true minus false tail fraction.
    pub contrast_bounds: CensorAwarePredicateDwellTailContrastBounds,
}

/// Failure while evaluating a fixed contrast threshold profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CensorAwareDwellTailContrastProfileError {
    /// A profile must contain at least one declared threshold.
    EmptyThresholdGrid,
    /// Thresholds must be strictly increasing.
    NonIncreasingThresholdGrid { previous: usize, current: usize },
    /// Canonical censor-aware tail validation failed.
    Tail(CensorAwareDwellTailBoundsError),
}

impl From<CensorAwareDwellTailBoundsError> for CensorAwareDwellTailContrastProfileError {
    fn from(value: CensorAwareDwellTailBoundsError) -> Self {
        Self::Tail(value)
    }
}

/// Evaluate exact true-minus-false contrast bounds on a fixed threshold grid.
///
/// Threshold selection remains caller-owned and must be frozen before result
/// interpretation. No threshold search, interpolation, smoothing, or sign
/// selection is performed here.
///
/// # Errors
///
/// Returns an error for an empty grid, a non-strictly-increasing grid, or any
/// validation failure from the canonical censor-aware tail contract.
pub fn predicate_censor_aware_dwell_tail_contrast_profile(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    thresholds_observations: &[usize],
) -> Result<Vec<CensorAwareDwellTailContrastProfilePoint>, CensorAwareDwellTailContrastProfileError>
{
    if thresholds_observations.is_empty() {
        return Err(CensorAwareDwellTailContrastProfileError::EmptyThresholdGrid);
    }
    for pair in thresholds_observations.windows(2) {
        if pair[1] <= pair[0] {
            return Err(
                CensorAwareDwellTailContrastProfileError::NonIncreasingThresholdGrid {
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
            let contrast_bounds = predicate_censor_aware_dwell_tail_contrast_bounds(
                runs,
                left_boundary,
                right_boundary,
                threshold_observations,
            )?;
            Ok(CensorAwareDwellTailContrastProfilePoint {
                threshold_observations,
                contrast_bounds,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExactFractionSign, ExactSignedRunFraction};

    fn run(value: bool, start_observation: usize, observations: usize) -> PredicateDwellRun {
        PredicateDwellRun {
            value,
            start_observation,
            observations,
        }
    }

    #[test]
    fn fixed_grid_matches_each_single_threshold_contrast() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 8),
            run(false, 10, 3),
            run(true, 13, 7),
        ];
        let thresholds = [3, 5, 7];
        let profile = predicate_censor_aware_dwell_tail_contrast_profile(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            &thresholds,
        )
        .unwrap();

        assert_eq!(profile.len(), thresholds.len());
        for (point, threshold) in profile.iter().zip(thresholds) {
            assert_eq!(point.threshold_observations, threshold);
            assert_eq!(point.contrast_bounds.threshold_observations, threshold);
            assert_eq!(
                point.contrast_bounds,
                predicate_censor_aware_dwell_tail_contrast_bounds(
                    &runs,
                    ObservationBoundary::Complete,
                    ObservationBoundary::Complete,
                    threshold,
                )
                .unwrap()
            );
        }
    }

    #[test]
    fn censoring_preserves_sign_indeterminate_interval() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 4),
            run(false, 6, 4),
            run(true, 10, 2),
        ];
        let profile = predicate_censor_aware_dwell_tail_contrast_profile(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            &[3, 4],
        )
        .unwrap();

        assert_eq!(
            profile[1].contrast_bounds.true_minus_false_lower,
            Some(ExactSignedRunFraction {
                sign: ExactFractionSign::Negative,
                numerator: 1,
                denominator: 2,
            })
        );
        assert_eq!(
            profile[1].contrast_bounds.true_minus_false_upper,
            Some(ExactSignedRunFraction {
                sign: ExactFractionSign::Positive,
                numerator: 1,
                denominator: 2,
            })
        );
    }

    #[test]
    fn absent_boolean_value_remains_undefined_at_every_threshold() {
        let runs = [run(true, 0, 8)];
        let profile = predicate_censor_aware_dwell_tail_contrast_profile(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            &[2, 4, 8],
        )
        .unwrap();

        for point in profile {
            assert_eq!(point.contrast_bounds.false_source_runs, 0);
            assert_eq!(point.contrast_bounds.true_source_runs, 1);
            assert_eq!(point.contrast_bounds.true_minus_false_lower, None);
            assert_eq!(point.contrast_bounds.true_minus_false_upper, None);
        }
    }

    #[test]
    fn invalid_threshold_grids_fail_closed() {
        let runs = [run(false, 0, 2), run(true, 2, 4)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_contrast_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[],
            ),
            Err(CensorAwareDwellTailContrastProfileError::EmptyThresholdGrid)
        );
        assert_eq!(
            predicate_censor_aware_dwell_tail_contrast_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[2, 2],
            ),
            Err(
                CensorAwareDwellTailContrastProfileError::NonIncreasingThresholdGrid {
                    previous: 2,
                    current: 2,
                }
            )
        );
        assert_eq!(
            predicate_censor_aware_dwell_tail_contrast_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[3, 2],
            ),
            Err(
                CensorAwareDwellTailContrastProfileError::NonIncreasingThresholdGrid {
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
            predicate_censor_aware_dwell_tail_contrast_profile(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                &[0, 2],
            ),
            Err(CensorAwareDwellTailContrastProfileError::Tail(
                CensorAwareDwellTailBoundsError::ZeroThreshold
            ))
        );
    }
}
