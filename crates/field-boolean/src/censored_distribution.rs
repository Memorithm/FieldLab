//! Censor-aware descriptive distributions for exact Boolean dwell runs.
//!
//! Boundary-censored dwells are retained as censored counts but are excluded
//! from the complete-run histogram.  This prevents a finite observation cut
//! from being silently treated as an observed residence completion.  No
//! survival model, extrapolation, hazard estimate, or stability verdict is
//! computed here.

use crate::{
    annotate_predicate_dwell_censoring, ObservationBoundary, PredicateDwellCensoringError,
    PredicateDwellLengthCount, PredicateDwellRun,
};

/// Versioned contract for censor-aware exact Boolean dwell distributions.
pub const BOOLEAN_FIELD_CENSOR_AWARE_DISTRIBUTION_SCHEMA: &str =
    "fieldlab.boolean-censor-aware-dwell-distribution.v1";

/// Exact descriptive distribution split between complete and boundary-censored runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CensorAwarePredicateDwellDistribution {
    /// Number of source maximal runs before censoring classification.
    pub source_runs: usize,
    /// Number of runs known complete at both retained window boundaries.
    pub complete_runs: usize,
    /// Number of runs censored at the left observation boundary.
    pub left_censored_runs: usize,
    /// Number of runs censored at the right observation boundary.
    pub right_censored_runs: usize,
    /// Number of runs censored at both boundaries (possible only for one-run traces).
    pub both_censored_runs: usize,
    /// Exact complete-run lengths for `false`, excluding all censored runs.
    pub complete_false_run_lengths: Vec<PredicateDwellLengthCount>,
    /// Exact complete-run lengths for `true`, excluding all censored runs.
    pub complete_true_run_lengths: Vec<PredicateDwellLengthCount>,
}

/// Failure while constructing a censor-aware dwell distribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CensorAwareDwellDistributionError {
    /// Source maximal runs or their boundary declaration are invalid.
    InvalidSource(PredicateDwellCensoringError),
    /// An exact counter could not be represented as `usize`.
    CountOverflow,
    /// Storage proportional to the number of source runs could not be reserved.
    AllocationFailed,
}

impl From<PredicateDwellCensoringError> for CensorAwareDwellDistributionError {
    fn from(value: PredicateDwellCensoringError) -> Self {
        match value {
            PredicateDwellCensoringError::InvalidRuns(error) => {
                Self::InvalidSource(PredicateDwellCensoringError::InvalidRuns(error))
            }
            PredicateDwellCensoringError::AllocationFailed => Self::AllocationFailed,
        }
    }
}

/// Return complete-run histograms while preserving explicit censoring counts.
///
/// The source run sequence is revalidated through the canonical censoring path.
/// A run contributes to a complete-run histogram only when neither boundary is
/// censored.  Censored run lengths are intentionally not imputed, extrapolated,
/// or mixed into the complete-run distribution.
///
/// # Errors
///
/// Returns an error if the source run sequence is malformed, exact counters
/// overflow, or run-count-bounded allocation fails.
pub fn predicate_censor_aware_dwell_distribution(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
) -> Result<CensorAwarePredicateDwellDistribution, CensorAwareDwellDistributionError> {
    let annotated = annotate_predicate_dwell_censoring(runs, left_boundary, right_boundary)?;

    let mut complete_false = Vec::new();
    let mut complete_true = Vec::new();
    complete_false
        .try_reserve_exact(annotated.len())
        .map_err(|_| CensorAwareDwellDistributionError::AllocationFailed)?;
    complete_true
        .try_reserve_exact(annotated.len())
        .map_err(|_| CensorAwareDwellDistributionError::AllocationFailed)?;

    let mut complete_runs = 0usize;
    let mut left_censored_runs = 0usize;
    let mut right_censored_runs = 0usize;
    let mut both_censored_runs = 0usize;

    for annotated_run in &annotated {
        left_censored_runs = checked_increment(left_censored_runs, annotated_run.left_censored)?;
        right_censored_runs = checked_increment(right_censored_runs, annotated_run.right_censored)?;
        both_censored_runs = checked_increment(
            both_censored_runs,
            annotated_run.left_censored && annotated_run.right_censored,
        )?;

        if annotated_run.left_censored || annotated_run.right_censored {
            continue;
        }
        complete_runs = complete_runs
            .checked_add(1)
            .ok_or(CensorAwareDwellDistributionError::CountOverflow)?;
        if annotated_run.run.value {
            complete_true.push(annotated_run.run.observations);
        } else {
            complete_false.push(annotated_run.run.observations);
        }
    }

    Ok(CensorAwarePredicateDwellDistribution {
        source_runs: annotated.len(),
        complete_runs,
        left_censored_runs,
        right_censored_runs,
        both_censored_runs,
        complete_false_run_lengths: exact_counts(complete_false)?,
        complete_true_run_lengths: exact_counts(complete_true)?,
    })
}

fn checked_increment(
    value: usize,
    condition: bool,
) -> Result<usize, CensorAwareDwellDistributionError> {
    if condition {
        value
            .checked_add(1)
            .ok_or(CensorAwareDwellDistributionError::CountOverflow)
    } else {
        Ok(value)
    }
}

fn exact_counts(
    mut lengths: Vec<usize>,
) -> Result<Vec<PredicateDwellLengthCount>, CensorAwareDwellDistributionError> {
    lengths.sort_unstable();
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(lengths.len())
        .map_err(|_| CensorAwareDwellDistributionError::AllocationFailed)?;
    for observations in lengths {
        match counts.last_mut() {
            Some(PredicateDwellLengthCount {
                observations: previous,
                runs,
            }) if *previous == observations => {
                *runs = runs
                    .checked_add(1)
                    .ok_or(CensorAwareDwellDistributionError::CountOverflow)?;
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

    fn run(value: bool, start_observation: usize, observations: usize) -> PredicateDwellRun {
        PredicateDwellRun {
            value,
            start_observation,
            observations,
        }
    }

    #[test]
    fn boundary_cuts_are_counted_but_not_misclassified_as_complete_dwells() {
        let runs = [
            run(false, 0, 2),
            run(true, 2, 3),
            run(false, 5, 4),
            run(true, 9, 3),
        ];
        let distribution = predicate_censor_aware_dwell_distribution(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
        )
        .unwrap();

        assert_eq!(distribution.source_runs, 4);
        assert_eq!(distribution.complete_runs, 2);
        assert_eq!(distribution.left_censored_runs, 1);
        assert_eq!(distribution.right_censored_runs, 1);
        assert_eq!(distribution.both_censored_runs, 0);
        assert_eq!(
            distribution.complete_false_run_lengths,
            vec![PredicateDwellLengthCount {
                observations: 4,
                runs: 1,
            }]
        );
        assert_eq!(
            distribution.complete_true_run_lengths,
            vec![PredicateDwellLengthCount {
                observations: 3,
                runs: 1,
            }]
        );
    }

    #[test]
    fn complete_boundaries_preserve_all_exact_run_lengths() {
        let runs = [run(true, 0, 2), run(false, 2, 5), run(true, 7, 2)];
        let distribution = predicate_censor_aware_dwell_distribution(
            &runs,
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
        )
        .unwrap();
        assert_eq!(distribution.complete_runs, 3);
        assert_eq!(distribution.left_censored_runs, 0);
        assert_eq!(distribution.right_censored_runs, 0);
        assert_eq!(
            distribution.complete_true_run_lengths,
            vec![PredicateDwellLengthCount {
                observations: 2,
                runs: 2,
            }]
        );
    }

    #[test]
    fn single_window_spanning_run_is_retained_only_as_double_censored() {
        let runs = [run(true, 0, 7)];
        let distribution = predicate_censor_aware_dwell_distribution(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
        )
        .unwrap();
        assert_eq!(distribution.source_runs, 1);
        assert_eq!(distribution.complete_runs, 0);
        assert_eq!(distribution.left_censored_runs, 1);
        assert_eq!(distribution.right_censored_runs, 1);
        assert_eq!(distribution.both_censored_runs, 1);
        assert!(distribution.complete_false_run_lengths.is_empty());
        assert!(distribution.complete_true_run_lengths.is_empty());
    }

    #[test]
    fn malformed_source_still_fails_through_canonical_validator() {
        let runs = [run(false, 0, 2), run(false, 2, 1)];
        assert!(matches!(
            predicate_censor_aware_dwell_distribution(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
            ),
            Err(CensorAwareDwellDistributionError::InvalidSource(_))
        ));
    }

    #[test]
    fn nested_allocation_failure_remains_retryable_resource_failure() {
        assert_eq!(
            CensorAwareDwellDistributionError::from(PredicateDwellCensoringError::AllocationFailed),
            CensorAwareDwellDistributionError::AllocationFailed
        );
    }
}
