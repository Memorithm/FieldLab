#![forbid(unsafe_code)]

use field_core::FieldState;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Versioned contract for field-to-Boolean predicate evaluation.
pub const BOOLEAN_FIELD_PREDICATE_SCHEMA: &str = "fieldlab.boolean-predicate.v1";

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

    /// Evaluates this predicate against `state` without changing it.
    ///
    /// # Errors
    ///
    /// Returns a typed bounds error when the declared node or component does not
    /// exist in the supplied field state.
    pub fn evaluate(&self, state: &FieldState) -> Result<bool, PredicateError> {
        let node = state
            .node(self.node)
            .ok_or(PredicateError::NodeOutOfBounds {
                node: self.node,
                node_count: state.node_count(),
            })?;
        let value =
            *node
                .values()
                .get(self.component)
                .ok_or(PredicateError::ComponentOutOfBounds {
                    component: self.component,
                    dimension: node.dimension(),
                })?;

        Ok(match self.relation {
            ThresholdRelation::AtLeast => value >= self.threshold,
            ThresholdRelation::LessThan => value < self.threshold,
        })
    }
}

/// Evaluates an ordered predicate set into an equally ordered Boolean vector.
///
/// The order is caller-owned and is preserved exactly. This function neither
/// tunes predicates nor interprets the resulting bits as a physical claim.
///
/// # Errors
///
/// Returns the first typed predicate error in input order.
pub fn evaluate_predicates(
    state: &FieldState,
    predicates: &[ComponentThresholdPredicate],
) -> Result<Vec<bool>, PredicateError> {
    predicates
        .iter()
        .map(|predicate| predicate.evaluate(state))
        .collect()
}

/// Validation errors for the explicit field-to-Boolean bridge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PredicateError {
    /// Predicate thresholds must be finite.
    NonFiniteThreshold,
    /// The declared node does not exist in the evaluated state.
    NodeOutOfBounds { node: usize, node_count: usize },
    /// The declared component does not exist in the selected node.
    ComponentOutOfBounds { component: usize, dimension: usize },
}

impl Display for PredicateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFiniteThreshold => write!(formatter, "predicate threshold must be finite"),
            Self::NodeOutOfBounds { node, node_count } => write!(
                formatter,
                "predicate node index {node} is outside field state with {node_count} nodes"
            ),
            Self::ComponentOutOfBounds {
                component,
                dimension,
            } => write!(
                formatter,
                "predicate component index {component} is outside node dimension {dimension}"
            ),
        }
    }
}

impl Error for PredicateError {}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_predicates, ComponentThresholdPredicate, PredicateError, ThresholdRelation,
    };
    use field_core::{FieldState, NodeState};

    fn state() -> FieldState {
        FieldState::new(vec![
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).expect("unit state"),
            NodeState::try_unit(vec![0.0, -1.0], 1.0e-12).expect("unit state"),
        ])
        .expect("homogeneous field state")
    }

    #[test]
    fn evaluates_declared_thresholds_in_stable_order() {
        let predicates = [
            ComponentThresholdPredicate::new(0, 0, 1.0, ThresholdRelation::AtLeast)
                .expect("finite threshold"),
            ComponentThresholdPredicate::new(0, 1, 0.0, ThresholdRelation::LessThan)
                .expect("finite threshold"),
            ComponentThresholdPredicate::new(1, 1, -0.5, ThresholdRelation::LessThan)
                .expect("finite threshold"),
        ];

        assert_eq!(
            evaluate_predicates(&state(), &predicates).expect("valid addresses"),
            vec![true, false, true]
        );
    }

    #[test]
    fn equality_boundary_is_explicit() {
        let field = state();
        let at_least = ComponentThresholdPredicate::new(0, 1, 0.0, ThresholdRelation::AtLeast)
            .expect("finite threshold");
        let less_than = ComponentThresholdPredicate::new(0, 1, 0.0, ThresholdRelation::LessThan)
            .expect("finite threshold");

        assert!(at_least.evaluate(&field).expect("valid address"));
        assert!(!less_than.evaluate(&field).expect("valid address"));
    }

    #[test]
    fn rejects_non_finite_thresholds() {
        assert_eq!(
            ComponentThresholdPredicate::new(0, 0, f64::NAN, ThresholdRelation::AtLeast),
            Err(PredicateError::NonFiniteThreshold)
        );
        assert_eq!(
            ComponentThresholdPredicate::new(0, 0, f64::INFINITY, ThresholdRelation::AtLeast),
            Err(PredicateError::NonFiniteThreshold)
        );
    }

    #[test]
    fn rejects_out_of_bounds_addresses() {
        let field = state();
        let node = ComponentThresholdPredicate::new(2, 0, 0.0, ThresholdRelation::AtLeast)
            .expect("finite threshold");
        assert_eq!(
            node.evaluate(&field),
            Err(PredicateError::NodeOutOfBounds {
                node: 2,
                node_count: 2,
            })
        );

        let component = ComponentThresholdPredicate::new(0, 2, 0.0, ThresholdRelation::AtLeast)
            .expect("finite threshold");
        assert_eq!(
            component.evaluate(&field),
            Err(PredicateError::ComponentOutOfBounds {
                component: 2,
                dimension: 2,
            })
        );
    }

    #[test]
    fn empty_predicate_set_is_an_empty_snapshot() {
        assert_eq!(
            evaluate_predicates(&state(), &[]).expect("no predicates to invalidate"),
            Vec::<bool>::new()
        );
    }
}
