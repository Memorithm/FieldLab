//! Exact transition observations for declared Boolean field predicates.
//!
//! This module classifies changes in already-declared field predicates. It does
//! not infer thresholds, select a controller, mutate field dynamics, or assign
//! physical meaning to a Boolean transition.

use crate::{ComponentThresholdPredicate, PredicateError};
use field_core::FieldState;

/// Versioned contract for two-state predicate transition observations.
pub const BOOLEAN_FIELD_TRANSITION_SCHEMA: &str = "fieldlab.boolean-transition.v1";

/// Exact transition of one Boolean predicate between two field observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateTransition {
    /// Predicate was false and remains false.
    StableFalse,
    /// Predicate changed from false to true.
    Rising,
    /// Predicate changed from true to false.
    Falling,
    /// Predicate was true and remains true.
    StableTrue,
}

impl PredicateTransition {
    /// Classifies an already-evaluated pair of Boolean observations.
    #[must_use]
    pub const fn from_values(previous: bool, current: bool) -> Self {
        match (previous, current) {
            (false, false) => Self::StableFalse,
            (false, true) => Self::Rising,
            (true, false) => Self::Falling,
            (true, true) => Self::StableTrue,
        }
    }

    /// Returns whether the predicate changed value between observations.
    #[must_use]
    pub const fn changed(self) -> bool {
        matches!(self, Self::Rising | Self::Falling)
    }
}

/// Evaluates one declared predicate on two field states and classifies its transition.
///
/// Evaluation order is deterministic: `previous` is evaluated first, then
/// `current`. No threshold or regime is inferred from either state.
///
/// # Errors
///
/// Returns the first [`PredicateError`] produced by the declared predicate on
/// either state.
pub fn evaluate_predicate_transition(
    predicate: &ComponentThresholdPredicate,
    previous: &FieldState,
    current: &FieldState,
) -> Result<PredicateTransition, PredicateError> {
    let previous_value = predicate.evaluate(previous)?;
    let current_value = predicate.evaluate(current)?;
    Ok(PredicateTransition::from_values(
        previous_value,
        current_value,
    ))
}

/// Evaluates an ordered predicate set on two field states.
///
/// Output order is exactly the caller-supplied predicate order. This is an
/// observation primitive only: transitions are not ranked, filtered, or used
/// to choose a field regime.
///
/// # Errors
///
/// Returns the first [`PredicateError`] in predicate order. For each predicate,
/// `previous` is evaluated before `current`.
pub fn evaluate_predicate_transitions(
    predicates: &[ComponentThresholdPredicate],
    previous: &FieldState,
    current: &FieldState,
) -> Result<Vec<PredicateTransition>, PredicateError> {
    predicates
        .iter()
        .map(|predicate| evaluate_predicate_transition(predicate, previous, current))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThresholdRelation;
    use field_core::NodeState;

    fn state(values: [f64; 2]) -> FieldState {
        FieldState::new(vec![
            NodeState::try_unit(values.to_vec(), 1.0e-12).expect("unit state"),
        ])
        .expect("homogeneous field state")
    }

    #[test]
    fn classifies_all_boolean_transition_states() {
        assert_eq!(
            PredicateTransition::from_values(false, false),
            PredicateTransition::StableFalse
        );
        assert_eq!(
            PredicateTransition::from_values(false, true),
            PredicateTransition::Rising
        );
        assert_eq!(
            PredicateTransition::from_values(true, false),
            PredicateTransition::Falling
        );
        assert_eq!(
            PredicateTransition::from_values(true, true),
            PredicateTransition::StableTrue
        );
        assert!(PredicateTransition::Rising.changed());
        assert!(PredicateTransition::Falling.changed());
        assert!(!PredicateTransition::StableFalse.changed());
        assert!(!PredicateTransition::StableTrue.changed());
    }

    #[test]
    fn observes_rising_and_falling_transitions_without_inferred_thresholds() {
        let previous = state([1.0, 0.0]);
        let current = state([0.0, 1.0]);
        let rising = ComponentThresholdPredicate::new(0, 1, 0.5, ThresholdRelation::AtLeast)
            .expect("finite threshold");
        let falling = ComponentThresholdPredicate::new(0, 0, 0.5, ThresholdRelation::AtLeast)
            .expect("finite threshold");

        assert_eq!(
            evaluate_predicate_transition(&rising, &previous, &current),
            Ok(PredicateTransition::Rising)
        );
        assert_eq!(
            evaluate_predicate_transition(&falling, &previous, &current),
            Ok(PredicateTransition::Falling)
        );
    }

    #[test]
    fn ordered_batch_preserves_stable_and_changed_observations() {
        let previous = state([1.0, 0.0]);
        let current = state([0.0, 1.0]);
        let predicates = [
            ComponentThresholdPredicate::new(0, 1, 0.5, ThresholdRelation::AtLeast)
                .expect("finite threshold"),
            ComponentThresholdPredicate::new(0, 0, 0.5, ThresholdRelation::AtLeast)
                .expect("finite threshold"),
            ComponentThresholdPredicate::new(0, 0, -0.5, ThresholdRelation::AtLeast)
                .expect("finite threshold"),
            ComponentThresholdPredicate::new(0, 1, 1.5, ThresholdRelation::AtLeast)
                .expect("finite threshold"),
        ];

        assert_eq!(
            evaluate_predicate_transitions(&predicates, &previous, &current),
            Ok(vec![
                PredicateTransition::Rising,
                PredicateTransition::Falling,
                PredicateTransition::StableTrue,
                PredicateTransition::StableFalse,
            ])
        );
    }

    #[test]
    fn transition_evaluation_fails_closed_on_invalid_current_state_address() {
        let previous = state([1.0, 0.0]);
        let current = FieldState::new(vec![
            NodeState::try_unit(vec![1.0], 1.0e-12).expect("unit state"),
        ])
        .expect("one-dimensional field state");
        let predicate =
            ComponentThresholdPredicate::new(0, 1, 0.0, ThresholdRelation::AtLeast)
                .expect("finite threshold");

        assert_eq!(
            evaluate_predicate_transition(&predicate, &previous, &current),
            Err(PredicateError::ComponentOutOfBounds {
                component: 1,
                dimension: 1,
            })
        );
    }

    #[test]
    fn empty_predicate_batch_returns_empty_transition_snapshot() {
        let previous = state([1.0, 0.0]);
        let current = state([0.0, 1.0]);
        assert_eq!(
            evaluate_predicate_transitions(&[], &previous, &current)
                .expect("no predicates to invalidate"),
            Vec::<PredicateTransition>::new()
        );
    }
}
