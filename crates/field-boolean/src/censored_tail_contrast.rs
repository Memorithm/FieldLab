//! Exact true-minus-false contrast bounds for censor-aware Boolean dwell tails.
//!
//! The contrast is derived only from the canonical partial-identification
//! fraction bounds. It does not impute censored durations, fit a survival model,
//! choose a threshold, or turn a contrast into a stability/bifurcation verdict.

use crate::{
    predicate_censor_aware_dwell_tail_fraction_bounds, CensorAwareDwellTailBoundsError,
    ExactRunFraction, ObservationBoundary, PredicateDwellRun,
};

/// Versioned contract for exact censor-aware true-minus-false tail contrasts.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_CONTRAST_BOUNDS_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-contrast-bounds.v1";

/// Sign of an exact rational contrast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactFractionSign {
    /// The exact value is strictly negative.
    Negative,
    /// The exact value is exactly zero.
    Zero,
    /// The exact value is strictly positive.
    Positive,
}

/// Canonical exact signed rational derived from empirical run-count fractions.
///
/// `numerator` is always a non-negative magnitude. Zero is canonicalized to
/// `0/1`; non-zero values are reduced by the greatest common divisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactSignedRunFraction {
    /// Exact sign.
    pub sign: ExactFractionSign,
    /// Reduced non-negative numerator magnitude.
    pub numerator: u128,
    /// Reduced strictly positive denominator.
    pub denominator: u128,
}

/// Exact partial-identification bounds for `true_tail_fraction - false_tail_fraction`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensorAwarePredicateDwellTailContrastBounds {
    /// Caller-declared strictly positive threshold in observation counts.
    pub threshold_observations: usize,
    /// Number of observed `false` dwell runs contributing to the empirical contrast.
    pub false_source_runs: usize,
    /// Number of observed `true` dwell runs contributing to the empirical contrast.
    pub true_source_runs: usize,
    /// Lower contrast bound, or `None` when either Boolean value is absent.
    pub true_minus_false_lower: Option<ExactSignedRunFraction>,
    /// Upper contrast bound, or `None` when either Boolean value is absent.
    pub true_minus_false_upper: Option<ExactSignedRunFraction>,
}

/// Compute exact censor-aware bounds for the true-minus-false dwell-tail contrast.
///
/// If `T = [T_lower, T_upper]` and `F = [F_lower, F_upper]` are the canonical
/// empirical tail-fraction intervals, this returns the exact interval
/// `[T_lower - F_upper, T_upper - F_lower]`. The caller owns threshold selection.
/// An absent Boolean value yields `None` bounds rather than an invented zero.
///
/// # Errors
///
/// Propagates validation errors from the canonical censor-aware tail contract,
/// including a zero threshold or malformed dwell/censoring declaration.
pub fn predicate_censor_aware_dwell_tail_contrast_bounds(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    threshold_observations: usize,
) -> Result<CensorAwarePredicateDwellTailContrastBounds, CensorAwareDwellTailBoundsError> {
    let fractions = predicate_censor_aware_dwell_tail_fraction_bounds(
        runs,
        left_boundary,
        right_boundary,
        threshold_observations,
    )?;

    let (lower, upper) = match (
        fractions.true_runs.lower,
        fractions.true_runs.upper,
        fractions.false_runs.lower,
        fractions.false_runs.upper,
    ) {
        (Some(true_lower), Some(true_upper), Some(false_lower), Some(false_upper)) => (
            Some(exact_fraction_difference(true_lower, false_upper)),
            Some(exact_fraction_difference(true_upper, false_lower)),
        ),
        _ => (None, None),
    };

    Ok(CensorAwarePredicateDwellTailContrastBounds {
        threshold_observations: fractions.threshold_observations,
        false_source_runs: fractions.false_runs.source_runs,
        true_source_runs: fractions.true_runs.source_runs,
        true_minus_false_lower: lower,
        true_minus_false_upper: upper,
    })
}

fn exact_fraction_difference(
    left: ExactRunFraction,
    right: ExactRunFraction,
) -> ExactSignedRunFraction {
    debug_assert!(left.denominator > 0);
    debug_assert!(right.denominator > 0);

    let left_scaled = (left.numerator as u128) * (right.denominator as u128);
    let right_scaled = (right.numerator as u128) * (left.denominator as u128);
    let denominator = (left.denominator as u128) * (right.denominator as u128);

    match left_scaled.cmp(&right_scaled) {
        std::cmp::Ordering::Equal => ExactSignedRunFraction {
            sign: ExactFractionSign::Zero,
            numerator: 0,
            denominator: 1,
        },
        std::cmp::Ordering::Greater => reduced_signed_fraction(
            ExactFractionSign::Positive,
            left_scaled - right_scaled,
            denominator,
        ),
        std::cmp::Ordering::Less => reduced_signed_fraction(
            ExactFractionSign::Negative,
            right_scaled - left_scaled,
            denominator,
        ),
    }
}

fn reduced_signed_fraction(
    sign: ExactFractionSign,
    numerator: u128,
    denominator: u128,
) -> ExactSignedRunFraction {
    debug_assert_ne!(sign, ExactFractionSign::Zero);
    debug_assert!(numerator > 0);
    debug_assert!(denominator > 0);

    let divisor = gcd_u128(numerator, denominator);
    ExactSignedRunFraction {
        sign,
        numerator: numerator / divisor,
        denominator: denominator / divisor,
    }
}

fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
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
    fn complete_runs_can_yield_exact_positive_unit_contrast() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 7),
            run(false, 9, 3),
            run(true, 12, 8),
        ];
        let contrast = predicate_censor_aware_dwell_tail_contrast_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            5,
        )
        .unwrap();

        let one = ExactSignedRunFraction {
            sign: ExactFractionSign::Positive,
            numerator: 1,
            denominator: 1,
        };
        assert_eq!(contrast.true_minus_false_lower, Some(one));
        assert_eq!(contrast.true_minus_false_upper, Some(one));
    }

    #[test]
    fn unequal_source_counts_reduce_exact_difference() {
        let runs = [
            run(false, 0, 5),
            run(true, 5, 5),
            run(false, 10, 2),
            run(true, 12, 2),
            run(false, 14, 2),
        ];
        let contrast = predicate_censor_aware_dwell_tail_contrast_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            5,
        )
        .unwrap();

        let one_sixth = ExactSignedRunFraction {
            sign: ExactFractionSign::Positive,
            numerator: 1,
            denominator: 6,
        };
        assert_eq!(contrast.false_source_runs, 3);
        assert_eq!(contrast.true_source_runs, 2);
        assert_eq!(contrast.true_minus_false_lower, Some(one_sixth));
        assert_eq!(contrast.true_minus_false_upper, Some(one_sixth));
    }

    #[test]
    fn censoring_can_leave_a_sign_indeterminate_contrast_interval() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 4),
            run(false, 6, 4),
            run(true, 10, 2),
        ];
        let contrast = predicate_censor_aware_dwell_tail_contrast_bounds(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            4,
        )
        .unwrap();

        assert_eq!(
            contrast.true_minus_false_lower,
            Some(ExactSignedRunFraction {
                sign: ExactFractionSign::Negative,
                numerator: 1,
                denominator: 2,
            })
        );
        assert_eq!(
            contrast.true_minus_false_upper,
            Some(ExactSignedRunFraction {
                sign: ExactFractionSign::Positive,
                numerator: 1,
                denominator: 2,
            })
        );
    }

    #[test]
    fn exact_equal_fractions_canonicalize_to_zero_over_one() {
        let runs = [
            run(false, 0, 5),
            run(true, 5, 5),
            run(false, 10, 2),
            run(true, 12, 2),
        ];
        let contrast = predicate_censor_aware_dwell_tail_contrast_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            5,
        )
        .unwrap();

        let zero = ExactSignedRunFraction {
            sign: ExactFractionSign::Zero,
            numerator: 0,
            denominator: 1,
        };
        assert_eq!(contrast.true_minus_false_lower, Some(zero));
        assert_eq!(contrast.true_minus_false_upper, Some(zero));
    }

    #[test]
    fn absent_boolean_value_keeps_contrast_undefined() {
        let runs = [run(true, 0, 7)];
        let contrast = predicate_censor_aware_dwell_tail_contrast_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            5,
        )
        .unwrap();

        assert_eq!(contrast.false_source_runs, 0);
        assert_eq!(contrast.true_source_runs, 1);
        assert_eq!(contrast.true_minus_false_lower, None);
        assert_eq!(contrast.true_minus_false_upper, None);
    }

    #[test]
    fn zero_threshold_remains_rejected_by_canonical_tail_contract() {
        let runs = [run(false, 0, 2), run(true, 2, 2)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_contrast_bounds(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                0,
            ),
            Err(CensorAwareDwellTailBoundsError::ZeroThreshold)
        );
    }
}
