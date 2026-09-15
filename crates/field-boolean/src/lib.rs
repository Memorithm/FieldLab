#![forbid(unsafe_code)]

pub mod transition;
pub mod transition_runs;
pub mod transition_summary;

use field_core::FieldState;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub use transition::{
    BOOLEAN_FIELD_TRANSITION_SCHEMA, BOOLEAN_FIELD_TRANSITION_TRACE_SCHEMA, PredicateTransition,
    PredicateTransitionTraceError, evaluate_predicate_transition, evaluate_predicate_transition_trace,
    evaluate_predicate_transitions,
};
pub use transition_runs::{
    BOOLEAN_FIELD_DWELL_RUN_SCHEMA, PredicateDwellRun, PredicateDwellRunError, predicate_dwell_runs,
};
pub use transition_summary::{
    BOOLEAN_FIELD_TRANSITION_SUMMARY_SCHEMA, PredicateTransitionSummary,
    summarize_predicate_transition_trace,
};

/// Versioned contract for field-to-Boolean predicate evaluation.
pub const BOOLEAN_FIELD_PREDICATE_SCHEMA: &str = "fieldlab.boolean-predicate.v1";

/// Versioned exact encoding for declared Ising-like spins.
pub const BOOLEAN_SPIN_ENCODING_SCHEMA: &str = "fieldlab.boolean-spin.v1";

/// Explicit comparison applied to one declared field-state component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThresholdRelation {
    /// The predicate is true when `value >= threshold`.
    AtLeast,
    /// The predicate is true when `value < threshold`.
    LessThan,
}

/// A declared scalar predicate over one component of one field node.
///
/// This type intentionally stores the node, component, threshold and comparison.
/// No threshold is inferred from data and evaluation does not mutate the field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentThresholdPredicate {
    node: usize,
    component: usize,
    threshold: f64,
    relation: ThresholdRelation,
}

impl ComponentThresholdPredicate {
    /// Builds an explicit predicate.
    ///
    /// # Errors
    ///
    /// Returns [`PredicateError::NonFiniteThreshold`] when `threshold` is NaN or infinite.
    pub fn new(
        node: usize,
        component: usize,
        threshold: f64,
        relation: ThresholdRelation,
    ) -> Result<Self, PredicateError> {
        if !threshold.is_finite() {
            return Err(PredicateError::NonFiniteThreshold);
        }
        Ok(Self {
            node,
            component,
            threshold,
            relation,
        })
    }

    /// Returns the declared node index.
    #[must_use]
    pub const fn node(&self) -> usize {
        self.node
    }

    /// Returns the declared component index.
    #[must_use]
    pub const fn component(&self) -> usize {
        self.component
    }

    /// Returns the declared threshold.
    #[must_use]
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Returns the declared comparison relation.
    #[must_use]
    pub const fn relation(&self) -> ThresholdRelation {
        self.relation
    }

    /// Evaluates the declared predicate on a field state without modifying it.
    ///
    /// # Errors
    ///
    /// Returns [`PredicateError::NodeOutOfBounds`] or
    /// [`PredicateError::ComponentOutOfBounds`] when the declared address is not
    /// present in `state`.
    pub fn evaluate(&self, state: &FieldState) -> Result<bool, PredicateError> {
        let node = state
            .nodes()
            .get(self.node)
            .ok_or(PredicateError::NodeOutOfBounds {
                node: self.node,
                nodes: state.nodes().len(),
            })?;
        let value = *node
            .components()
            .get(self.component)
            .ok_or(PredicateError::ComponentOutOfBounds {
                component: self.component,
                dimension: node.components().len(),
            })?;
        Ok(match self.relation {
            ThresholdRelation::AtLeast => value >= self.threshold,
            ThresholdRelation::LessThan => value < self.threshold,
        })
    }
}

/// Errors produced by explicit field-to-Boolean predicates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PredicateError {
    /// Thresholds must be finite binary64 values.
    NonFiniteThreshold,
    /// The declared node is absent from the supplied state.
    NodeOutOfBounds { node: usize, nodes: usize },
    /// The declared component is absent from the addressed node.
    ComponentOutOfBounds { component: usize, dimension: usize },
}

impl Display for PredicateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFiniteThreshold => f.write_str("predicate threshold must be finite"),
            Self::NodeOutOfBounds { node, nodes } => {
                write!(f, "predicate node index {node} is out of bounds for {nodes} nodes")
            }
            Self::ComponentOutOfBounds {
                component,
                dimension,
            } => write!(
                f,
                "predicate component index {component} is out of bounds for dimension {dimension}"
            ),
        }
    }
}

impl Error for PredicateError {}

/// Evaluates predicates in caller-supplied order.
///
/// # Errors
///
/// Returns the first [`PredicateError`] encountered in predicate order.
pub fn evaluate_predicates(
    predicates: &[ComponentThresholdPredicate],
    state: &FieldState,
) -> Result<Vec<bool>, PredicateError> {
    predicates.iter().map(|predicate| predicate.evaluate(state)).collect()
}

/// Error while converting exact Ising-like spins into Boolean bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpinEncodingError {
    /// The source value was neither exact `-1.0` nor exact `+1.0`.
    NonBinarySpin { index: usize, value: f64 },
}

impl Display for SpinEncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonBinarySpin { index, value } => write!(
                f,
                "spin at index {index} must be exactly -1.0 or +1.0, got {value}"
            ),
        }
    }
}

impl Error for SpinEncodingError {}

/// Converts exact Ising-like spins to bits (`-1 -> false`, `+1 -> true`).
///
/// No rounding, sign test, zero handling, thresholding, or normalization is
/// performed. Callers with continuous field components must declare their
/// discretization separately before using this exact bridge.
///
/// # Errors
///
/// Returns [`SpinEncodingError::NonBinarySpin`] at the first source value that
/// is not exactly `-1.0` or `+1.0`.
pub fn spins_to_bits(spins: &[f64]) -> Result<Vec<bool>, SpinEncodingError> {
    spins
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| {
            if value == -1.0 {
                Ok(false)
            } else if value == 1.0 {
                Ok(true)
            } else {
                Err(SpinEncodingError::NonBinarySpin { index, value })
            }
        })
        .collect()
}

/// Converts Boolean bits to exact Ising-like spins (`false -> -1`, `true -> +1`).
#[must_use]
pub fn bits_to_spins(bits: &[bool]) -> Vec<f64> {
    bits.iter()
        .map(|&bit| if bit { 1.0 } else { -1.0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use field_core::NodeState;

    fn state(values: [f64; 2]) -> FieldState {
        FieldState::new(vec![
            NodeState::try_unit(values.to_vec(), 1.0e-12).expect("unit state")
        ])
        .expect("homogeneous field state")
    }

    #[test]
    fn declared_predicates_preserve_order() {
        let field = state([0.6, 0.8]);
        let predicates = [
            ComponentThresholdPredicate::new(0, 0, 0.5, ThresholdRelation::AtLeast).unwrap(),
            ComponentThresholdPredicate::new(0, 1, 0.9, ThresholdRelation::LessThan).unwrap(),
            ComponentThresholdPredicate::new(0, 1, 0.8, ThresholdRelation::AtLeast).unwrap(),
        ];

        assert_eq!(evaluate_predicates(&predicates, &field), Ok(vec![true, true, true]));
    }

    #[test]
    fn threshold_relation_is_explicit() {
        let field = state([0.6, 0.8]);
        let at_least =
            ComponentThresholdPredicate::new(0, 0, 0.6, ThresholdRelation::AtLeast).unwrap();
        let less_than =
            ComponentThresholdPredicate::new(0, 0, 0.6, ThresholdRelation::LessThan).unwrap();

        assert_eq!(at_least.evaluate(&field), Ok(true));
        assert_eq!(less_than.evaluate(&field), Ok(false));
    }

    #[test]
    fn invalid_addresses_fail_closed() {
        let field = state([0.6, 0.8]);
        let invalid_node =
            ComponentThresholdPredicate::new(1, 0, 0.0, ThresholdRelation::AtLeast).unwrap();
        let invalid_component =
            ComponentThresholdPredicate::new(0, 2, 0.0, ThresholdRelation::AtLeast).unwrap();

        assert_eq!(
            invalid_node.evaluate(&field),
            Err(PredicateError::NodeOutOfBounds { node: 1, nodes: 1 })
        );
        assert_eq!(
            invalid_component.evaluate(&field),
            Err(PredicateError::ComponentOutOfBounds {
                component: 2,
                dimension: 2,
            })
        );
    }

    #[test]
    fn non_finite_thresholds_are_rejected() {
        for threshold in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                ComponentThresholdPredicate::new(0, 0, threshold, ThresholdRelation::AtLeast),
                Err(PredicateError::NonFiniteThreshold)
            );
        }
    }

    #[test]
    fn exact_spin_encoding_round_trips() {
        let spins = [-1.0, 1.0, 1.0, -1.0];
        let bits = spins_to_bits(&spins).expect("exact spins are accepted");
        assert_eq!(bits, vec![false, true, true, false]);
        assert_eq!(bits_to_spins(&bits), spins);
    }

    #[test]
    fn spin_encoding_rejects_continuous_values() {
        for value in [-0.999, -0.0, 0.0, 0.75, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                spins_to_bits(&[value]),
                Err(SpinEncodingError::NonBinarySpin { index: 0, .. })
            ));
        }
    }
}
