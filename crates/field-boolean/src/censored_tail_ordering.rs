//! Exact ordering classification for censor-aware Boolean dwell-tail fractions.
//!
//! This module compares the partial-identification intervals already produced by
//! the canonical censor-aware tail-fraction contract. It does not fit a survival
//! model, impute censored durations, or turn an interval ordering into a field
//! stability/bifurcation verdict.

use crate::{
    predicate_censor_aware_dwell_tail_fraction_bounds, CensorAwareDwellTailBoundsError,
    ExactRunFraction, ObservationBoundary, PredicateDwellRun,
};

/// Versioned contract for exact ordering of censor-aware Boolean tail fractions.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_ORDERING_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-ordering.v1";

/// Exact interval-order relationship between `true` and `false` dwell-tail fractions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellTailOrdering {
    /// At least one Boolean value has no observed run, so no empirical contrast exists.
    UndefinedAbsentValue,
    /// The complete lower bound for `true` is strictly above the possible upper bound for `false`.
    TrueDefinitelyHigher,
    /// The complete lower bound for `false` is strictly above the possible upper bound for `true`.
    FalseDefinitelyHigher,
    /// The two exact partial-identification intervals overlap or touch.
    IndeterminateOverlap,
}

/// Compare exact censor-aware empirical tail-fraction bounds without floating point.
///
/// A strict ordering is reported only when the entire interval for one Boolean
/// value lies above the entire interval for the other. Touching boundaries remain
/// indeterminate. The comparison uses `u128` cross-products, which exactly hold
/// products of two `usize` values on supported Rust targets.
///
/// # Errors
///
/// Propagates validation errors from the canonical censor-aware tail contracts.
pub fn predicate_censor_aware_dwell_tail_ordering(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    threshold_observations: usize,
) -> Result<PredicateDwellTailOrdering, CensorAwareDwellTailBoundsError> {
    let bounds = predicate_censor_aware_dwell_tail_fraction_bounds(
        runs,
        left_boundary,
        right_boundary,
        threshold_observations,
    )?;

    let (Some(false_lower), Some(false_upper), Some(true_lower), Some(true_upper)) = (
        bounds.false_runs.lower,
        bounds.false_runs.upper,
        bounds.true_runs.lower,
        bounds.true_runs.upper,
    ) else {
        return Ok(PredicateDwellTailOrdering::UndefinedAbsentValue);
    };

    if fraction_gt(true_lower, false_upper) {
        return Ok(PredicateDwellTailOrdering::TrueDefinitelyHigher);
    }
    if fraction_gt(false_lower, true_upper) {
        return Ok(PredicateDwellTailOrdering::FalseDefinitelyHigher);
    }

    Ok(PredicateDwellTailOrdering::IndeterminateOverlap)
}

fn fraction_gt(left: ExactRunFraction, right: ExactRunFraction) -> bool {
    debug_assert!(left.denominator > 0);
    debug_assert!(right.denominator > 0);
    (left.numerator as u128) * (right.denominator as u128)
        > (right.numerator as u128) * (left.denominator as u128)
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
    fn complete_intervals_can_prove_true_tail_higher() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 7),
            run(false, 9, 3),
            run(true, 12, 8),
        ];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                5,
            )
            .unwrap(),
            PredicateDwellTailOrdering::TrueDefinitelyHigher
        );
    }

    #[test]
    fn complete_intervals_can_prove_false_tail_higher() {
        let runs = [
            run(false, 0, 7),
            run(true, 7, 2),
            run(false, 9, 8),
            run(true, 17, 3),
        ];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                5,
            )
            .unwrap(),
            PredicateDwellTailOrdering::FalseDefinitelyHigher
        );
    }

    #[test]
    fn censoring_overlap_blocks_an_ordering_claim() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 7),
            run(false, 9, 6),
            run(true, 15, 2),
        ];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::ObservationCut,
                ObservationBoundary::ObservationCut,
                5,
            )
            .unwrap(),
            PredicateDwellTailOrdering::IndeterminateOverlap
        );
    }

    #[test]
    fn touching_intervals_are_not_promoted_to_strict_ordering() {
        let runs = [
            run(false, 0, 6),
            run(true, 6, 6),
            run(false, 12, 2),
            run(true, 14, 2),
        ];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                5,
            )
            .unwrap(),
            PredicateDwellTailOrdering::IndeterminateOverlap
        );
    }

    #[test]
    fn absent_boolean_value_is_undefined_not_zero() {
        let runs = [run(true, 0, 7)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                5,
            )
            .unwrap(),
            PredicateDwellTailOrdering::UndefinedAbsentValue
        );
    }

    #[test]
    fn zero_threshold_remains_rejected() {
        let runs = [run(false, 0, 2), run(true, 2, 2)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_ordering(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                0,
            ),
            Err(CensorAwareDwellTailBoundsError::ZeroThreshold)
        );
    }
}
