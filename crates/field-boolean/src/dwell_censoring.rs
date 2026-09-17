//! Explicit observation-window censoring for exact Boolean dwell runs.
//!
//! A finite observation window can cut through a dwell that began before the
//! first retained observation or continues after the last retained observation.
//! This module records that boundary fact explicitly so descriptive dwell counts
//! are not silently reinterpreted as complete residence times.

use crate::{summarize_predicate_dwell_runs, PredicateDwellRun, PredicateDwellSummaryError};

/// Versioned contract for explicit dwell-run observation-window censoring.
pub const BOOLEAN_FIELD_DWELL_CENSORING_SCHEMA: &str = "fieldlab.boolean-dwell-censoring.v1";

/// Declared knowledge about one edge of the retained observation window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationBoundary {
    /// The experiment declares that the dwell is complete at this boundary
    /// (for example, a known reset/initialization or declared terminal event).
    Complete,
    /// The retained window cuts the trajectory at this edge, so the adjacent
    /// dwell may extend beyond the observed samples.
    ObservationCut,
}

/// One exact dwell run annotated only with declared window-boundary censoring.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensoredPredicateDwellRun {
    /// Original exact observed run.
    pub run: PredicateDwellRun,
    /// True only when the run touches a declared left observation cut.
    pub left_censored: bool,
    /// True only when the run touches a declared right observation cut.
    pub right_censored: bool,
}

/// Failure while applying an observation-window censoring declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateDwellCensoringError {
    /// Source runs are not one valid maximal canonical trajectory.
    InvalidRuns(PredicateDwellSummaryError),
    /// Run-count-bounded output storage could not be reserved.
    AllocationFailed,
}

impl From<PredicateDwellSummaryError> for PredicateDwellCensoringError {
    fn from(value: PredicateDwellSummaryError) -> Self {
        Self::InvalidRuns(value)
    }
}

/// Validate exact dwell runs and annotate boundary censoring without inferring it.
///
/// Only the first run can be left-censored and only the final run can be
/// right-censored. Interior runs are complete with respect to the supplied
/// observation sequence because each is bounded by an observed Boolean change.
/// A single-run trajectory may be censored at both edges.
///
/// The caller must declare whether each window boundary is complete or an
/// observation cut. This function never infers that declaration from run length,
/// transition count, field energy, or predicate value.
///
/// # Errors
///
/// Returns a canonical dwell-run validation error for malformed public inputs or
/// [`PredicateDwellCensoringError::AllocationFailed`] when output storage cannot
/// be reserved.
pub fn annotate_predicate_dwell_censoring(
    runs: &[PredicateDwellRun],
    left_boundary: ObservationBoundary,
    right_boundary: ObservationBoundary,
) -> Result<Vec<CensoredPredicateDwellRun>, PredicateDwellCensoringError> {
    summarize_predicate_dwell_runs(runs)?;

    let mut annotated = Vec::new();
    annotated
        .try_reserve_exact(runs.len())
        .map_err(|_| PredicateDwellCensoringError::AllocationFailed)?;

    let last_index = runs
        .len()
        .checked_sub(1)
        .ok_or(PredicateDwellCensoringError::InvalidRuns(
            PredicateDwellSummaryError::EmptyRuns,
        ))?;

    for (index, &run) in runs.iter().enumerate() {
        annotated.push(CensoredPredicateDwellRun {
            run,
            left_censored: index == 0 && left_boundary == ObservationBoundary::ObservationCut,
            right_censored: index == last_index
                && right_boundary == ObservationBoundary::ObservationCut,
        });
    }

    Ok(annotated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_runs() -> [PredicateDwellRun; 3] {
        [
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
        ]
    }

    #[test]
    fn complete_boundaries_do_not_invent_censoring() {
        let annotated = annotate_predicate_dwell_censoring(
            &three_runs(),
            ObservationBoundary::Complete,
            ObservationBoundary::Complete,
        )
        .unwrap();
        assert!(annotated
            .iter()
            .all(|run| !run.left_censored && !run.right_censored));
    }

    #[test]
    fn observation_cuts_mark_only_boundary_runs() {
        let annotated = annotate_predicate_dwell_censoring(
            &three_runs(),
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
        )
        .unwrap();
        assert_eq!(annotated.len(), 3);
        assert!(annotated[0].left_censored);
        assert!(!annotated[0].right_censored);
        assert!(!annotated[1].left_censored && !annotated[1].right_censored);
        assert!(!annotated[2].left_censored);
        assert!(annotated[2].right_censored);
    }

    #[test]
    fn single_run_can_be_censored_at_both_window_edges() {
        let runs = [PredicateDwellRun {
            value: true,
            start_observation: 0,
            observations: 7,
        }];
        let annotated = annotate_predicate_dwell_censoring(
            &runs,
            ObservationBoundary::ObservationCut,
            ObservationBoundary::ObservationCut,
        )
        .unwrap();
        assert_eq!(
            annotated,
            vec![CensoredPredicateDwellRun {
                run: runs[0],
                left_censored: true,
                right_censored: true,
            }]
        );
    }

    #[test]
    fn malformed_public_runs_fail_closed() {
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
            annotate_predicate_dwell_censoring(
                &runs,
                ObservationBoundary::Complete,
                ObservationBoundary::Complete,
            ),
            Err(PredicateDwellCensoringError::InvalidRuns(
                PredicateDwellSummaryError::NonMaximalRuns {
                    run_index: 1,
                    value: false
                }
            ))
        );
    }
}
