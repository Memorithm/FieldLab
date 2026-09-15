//! Exact summaries of already-observed Boolean field transitions.
//!
//! This module reduces a transition trace to deterministic counts. It does not
//! infer thresholds, smooth the trace, choose a regime, or control field
//! dynamics. The summary is descriptive evidence only.

use crate::PredicateTransition;

/// Versioned contract for exact transition-trace summaries.
pub const BOOLEAN_FIELD_TRANSITION_SUMMARY_SCHEMA: &str = "fieldlab.boolean-transition-summary.v1";

/// Exact count summary for one ordered transition trace.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PredicateTransitionSummary {
    /// Total number of adjacent transitions summarized.
    pub transitions: usize,
    /// Number of `false -> false` observations.
    pub stable_false: usize,
    /// Number of `false -> true` observations.
    pub rising: usize,
    /// Number of `true -> false` observations.
    pub falling: usize,
    /// Number of `true -> true` observations.
    pub stable_true: usize,
}

impl PredicateTransitionSummary {
    /// Counts an already-observed transition trace exactly.
    ///
    /// No temporal weighting, threshold tuning, debounce, or hysteresis is
    /// applied. The four category counts always sum to [`Self::transitions`].
    #[must_use]
    pub fn from_trace(trace: &[PredicateTransition]) -> Self {
        let mut summary = Self {
            transitions: trace.len(),
            ..Self::default()
        };

        for transition in trace {
            match transition {
                PredicateTransition::StableFalse => summary.stable_false += 1,
                PredicateTransition::Rising => summary.rising += 1,
                PredicateTransition::Falling => summary.falling += 1,
                PredicateTransition::StableTrue => summary.stable_true += 1,
            }
        }

        summary
    }

    /// Returns the exact number of value-changing transitions.
    #[must_use]
    pub const fn changed(self) -> usize {
        self.rising + self.falling
    }

    /// Returns the exact number of stable transitions.
    #[must_use]
    pub const fn stable(self) -> usize {
        self.stable_false + self.stable_true
    }

    /// Returns whether the trace contains at least one state change.
    #[must_use]
    pub const fn has_change(self) -> bool {
        self.changed() != 0
    }
}

/// Summarizes an already-observed transition trace without interpreting it.
#[must_use]
pub fn summarize_predicate_transition_trace(
    trace: &[PredicateTransition],
) -> PredicateTransitionSummary {
    PredicateTransitionSummary::from_trace(trace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trace_has_zero_exact_counts() {
        let summary = summarize_predicate_transition_trace(&[]);
        assert_eq!(summary, PredicateTransitionSummary::default());
        assert_eq!(summary.changed(), 0);
        assert_eq!(summary.stable(), 0);
        assert!(!summary.has_change());
    }

    #[test]
    fn counts_all_transition_categories_without_reordering() {
        let trace = [
            PredicateTransition::StableTrue,
            PredicateTransition::Falling,
            PredicateTransition::StableFalse,
            PredicateTransition::Rising,
            PredicateTransition::Rising,
            PredicateTransition::StableTrue,
        ];
        let summary = summarize_predicate_transition_trace(&trace);

        assert_eq!(summary.transitions, 6);
        assert_eq!(summary.stable_false, 1);
        assert_eq!(summary.rising, 2);
        assert_eq!(summary.falling, 1);
        assert_eq!(summary.stable_true, 2);
        assert_eq!(summary.changed(), 3);
        assert_eq!(summary.stable(), 3);
        assert!(summary.has_change());
        assert_eq!(
            summary.stable_false + summary.rising + summary.falling + summary.stable_true,
            summary.transitions
        );
    }

    #[test]
    fn stable_only_trace_does_not_invent_a_switch() {
        let summary = summarize_predicate_transition_trace(&[
            PredicateTransition::StableFalse,
            PredicateTransition::StableFalse,
            PredicateTransition::StableTrue,
        ]);

        assert_eq!(summary.changed(), 0);
        assert_eq!(summary.stable(), 3);
        assert!(!summary.has_change());
    }
}
