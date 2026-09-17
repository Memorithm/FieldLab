//! Exact fraction bounds derived from censor-aware Boolean dwell-tail counts.
//!
//! Fractions remain exact count ratios. No survival model, hazard assumption,
//! extrapolation, or stability verdict is introduced. A Boolean value with no
//! observed runs has no empirical fraction and is represented by `None`.

use crate::{
    predicate_censor_aware_dwell_tail_bounds, CensorAwareDwellTailBoundsError, ObservationBoundary,
    PredicateDwellRun, PredicateDwellTailCountBounds,
};

/// Versioned contract for exact censor-aware dwell-tail fraction bounds.
pub const BOOLEAN_FIELD_CENSOR_AWARE_TAIL_FRACTION_BOUNDS_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-tail-fraction-bounds.v1";

/// An exact count fraction retaining the original run-count denominator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactRunFraction {
    /// Exact numerator count.
    pub numerator: usize,
    /// Exact source-run denominator. Strictly positive when present.
    pub denominator: usize,
}

/// Exact empirical tail-fraction bounds for one Boolean value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredicateDwellTailFractionBounds {
    /// Number of source runs with this Boolean value.
    pub source_runs: usize,
    /// Lower bound, or `None` if no run of this value exists.
    pub lower: Option<ExactRunFraction>,
    /// Upper bound, or `None` if no run of this value exists.
    pub upper: Option<ExactRunFraction>,
}

/// Exact empirical tail-fraction bounds for both Boolean values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensorAwarePredicateDwellTailFractionBounds {
    /// Caller-declared strictly positive threshold in observation counts.
    pub threshold_observations: usize,
    /// Fraction bounds for `false` dwell runs.
    pub false_runs: PredicateDwellTailFractionBounds,
    /// Fraction bounds for `true` dwell runs.
    pub true_runs: PredicateDwellTailFractionBounds,
}

/// Construct exact empirical fraction bounds for a censor-aware dwell tail.
///
/// Boundary-censored runs observed below the threshold widen only the upper
/// bound. When a Boolean value is absent, both fractions are `None`; absence of
/// evidence is not converted into an empirical zero.
///
/// # Errors
///
/// Propagates the canonical tail-count validation error, including a zero
/// threshold or malformed dwell/censoring declaration.
pub fn predicate_censor_aware_dwell_tail_fraction_bounds(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
    threshold_observations: usize,
) -> Result<CensorAwarePredicateDwellTailFractionBounds, CensorAwareDwellTailBoundsError> {
    let counts = predicate_censor_aware_dwell_tail_bounds(
        runs,
        left_boundary,
        right_boundary,
        threshold_observations,
    )?;

    Ok(CensorAwarePredicateDwellTailFractionBounds {
        threshold_observations: counts.threshold_observations,
        false_runs: exact_fraction_bounds(counts.false_runs),
        true_runs: exact_fraction_bounds(counts.true_runs),
    })
}

fn exact_fraction_bounds(
    counts: PredicateDwellTailCountBounds,
) -> PredicateDwellTailFractionBounds {
    if counts.source_runs == 0 {
        return PredicateDwellTailFractionBounds {
            source_runs: 0,
            lower: None,
            upper: None,
        };
    }

    debug_assert!(counts.definite_at_least_threshold <= counts.possible_at_least_threshold);
    debug_assert!(counts.possible_at_least_threshold <= counts.source_runs);
    PredicateDwellTailFractionBounds {
        source_runs: counts.source_runs,
        lower: Some(ExactRunFraction {
            numerator: counts.definite_at_least_threshold,
            denominator: counts.source_runs,
        }),
        upper: Some(ExactRunFraction {
            numerator: counts.possible_at_least_threshold,
            denominator: counts.source_runs,
        }),
    }
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
    fn censored_short_runs_widen_exact_fraction_without_float_rounding() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 5),
            run(false, 7, 4),
            run(true, 11, 2),
        ];
        let bounds = predicate_censor_aware_dwell_tail_fraction_bounds(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
            4,
        )
        .unwrap();

        let expected = PredicateDwellTailFractionBounds {
            source_runs: 2,
            lower: Some(ExactRunFraction {
                numerator: 1,
                denominator: 2,
            }),
            upper: Some(ExactRunFraction {
                numerator: 2,
                denominator: 2,
            }),
        };
        assert_eq!(bounds.false_runs, expected);
        assert_eq!(bounds.true_runs, expected);
    }

    #[test]
    fn complete_runs_collapse_lower_and_upper_to_the_same_fraction() {
        let runs = [run(false, 0, 2), run(true, 2, 5), run(false, 7, 3)];
        let bounds = predicate_censor_aware_dwell_tail_fraction_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            4,
        )
        .unwrap();

        assert_eq!(bounds.false_runs.source_runs, 2);
        assert_eq!(bounds.false_runs.lower.unwrap().numerator, 0);
        assert_eq!(bounds.false_runs.upper.unwrap().numerator, 0);
        assert_eq!(bounds.false_runs.lower.unwrap().denominator, 2);
        assert_eq!(bounds.true_runs.source_runs, 1);
        assert_eq!(bounds.true_runs.lower.unwrap().numerator, 1);
        assert_eq!(bounds.true_runs.upper.unwrap().numerator, 1);
    }

    #[test]
    fn absent_boolean_value_has_undefined_fraction_not_zero_percent() {
        let runs = [run(true, 0, 7)];
        let bounds = predicate_censor_aware_dwell_tail_fraction_bounds(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
            5,
        )
        .unwrap();

        assert_eq!(bounds.false_runs.source_runs, 0);
        assert_eq!(bounds.false_runs.lower, None);
        assert_eq!(bounds.false_runs.upper, None);
        assert_eq!(bounds.true_runs.lower.unwrap().numerator, 1);
        assert_eq!(bounds.true_runs.upper.unwrap().denominator, 1);
    }
    #[test]
    fn zero_threshold_remains_rejected_by_canonical_tail_contract() {
        let runs = [run(true, 0, 1)];
        assert_eq!(
            predicate_censor_aware_dwell_tail_fraction_bounds(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
                0,
            ),
            Err(CensorAwareDwellTailBoundsError::ZeroThreshold)
        );
    }
}
