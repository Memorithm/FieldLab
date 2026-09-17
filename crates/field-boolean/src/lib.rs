#![forbid(unsafe_code)]

pub mod censored_distribution;
pub mod censored_tail_bounds;
pub mod censored_tail_contrast;
pub mod censored_tail_fractions;
pub mod censored_tail_ordering;
pub mod censored_tail_profile;
pub mod dwell_censoring;
pub mod dwell_distribution;
pub mod dwell_summary;
pub mod transition;
pub mod transition_runs;
pub mod transition_summary;

use field_core::FieldState;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub use censored_distribution::{
    predicate_censor_aware_dwell_distribution, CensorAwareDwellDistributionError,
    CensorAwarePredicateDwellDistribution, BOOLEAN_FIELD_CENSOR_AWARE_DISTRIBUTION_SCHEMA,
};
pub use censored_tail_bounds::{
    predicate_censor_aware_dwell_tail_bounds, CensorAwareDwellTailBoundsError,
    CensorAwarePredicateDwellTailBounds, PredicateDwellTailCountBounds,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_BOUNDS_SCHEMA,
};
pub use censored_tail_contrast::{
    predicate_censor_aware_dwell_tail_contrast_bounds, CensorAwarePredicateDwellTailContrastBounds,
    ExactFractionSign, ExactSignedRunFraction,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_CONTRAST_BOUNDS_SCHEMA,
};
pub use censored_tail_fractions::{
    predicate_censor_aware_dwell_tail_fraction_bounds, CensorAwarePredicateDwellTailFractionBounds,
    ExactRunFraction, PredicateDwellTailFractionBounds,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_FRACTION_BOUNDS_SCHEMA,
};
pub use censored_tail_ordering::{
    predicate_censor_aware_dwell_tail_ordering, PredicateDwellTailOrdering,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_ORDERING_SCHEMA,
};
pub use censored_tail_profile::{
    predicate_censor_aware_dwell_tail_profile, CensorAwareDwellTailProfileError,
    CensorAwareDwellTailProfilePoint, BOOLEAN_FIELD_CENSOR_AWARE_TAIL_PROFILE_SCHEMA,
};
pub use dwell_censoring::{
    annotate_predicate_dwell_censoring, CensoredPredicateDwellRun, ObservationBoundary,
    PredicateDwellCensoringError, BOOLEAN_FIELD_DWELL_CENSORING_SCHEMA,
};
pub use dwell_distribution::{
    predicate_dwell_distribution, PredicateDwellDistribution, PredicateDwellDistributionError,
    PredicateDwellLengthCount, BOOLEAN_FIELD_DWELL_DISTRIBUTION_SCHEMA,
};
pub use dwell_summary::{
    summarize_predicate_dwell_runs, PredicateDwellSummary, PredicateDwellSummaryError,
    BOOLEAN_FIELD_DWELL_SUMMARY_SCHEMA,
};
pub use transition::{
    evaluate_predicate_transition, evaluate_predicate_transition_trace,
    evaluate_predicate_transitions, PredicateTransition, PredicateTransitionTraceError,
    BOOLEAN_FIELD_TRANSITION_SCHEMA, BOOLEAN_FIELD_TRANSITION_TRACE_SCHEMA,
};
pub use transition_runs::{
    predicate_dwell_runs, PredicateDwellRun, PredicateDwellRunError, BOOLEAN_FIELD_DWELL_RUN_SCHEMA,
};
pub use transition_summary::{
    summarize_predicate_transition_trace, PredicateTransitionSummary,
    BOOLEAN_FIELD_TRANSITION_SUMMARY_SCHEMA,
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

/// Encodes an ordered exact spin vector using `-1.0 -> false`, `+1.0 -> true`.
///
/// This is an exact representation contract, not thresholding. Values such as
/// `0.999`, signed zero, NaN, infinities, or any other continuous field value
/// are rejected rather than rounded or classified.
///
/// # Errors
///
/// Returns [`SpinEncodingError::NonBinarySpin`] at the first input that is not
/// exactly `-1.0` or `+1.0` in IEEE-754 binary64 representation.
pub fn encode_spins(spins: &[f64]) -> Result<Vec<bool>, SpinEncodingError> {
    spins
        .iter()
        .enumerate()
        .map(|(index, value)| match value.to_bits() {
            bits if bits == 1.0_f64.to_bits() => Ok(true),
            bits if bits == (-1.0_f64).to_bits() => Ok(false),
            bits => Err(SpinEncodingError::NonBinarySpin { index, bits }),
        })
        .collect()
}

/// Decodes Boolean bits using the inverse exact spin convention.
#[must_use]
pub fn decode_spins(bits: &[bool]) -> Vec<f64> {
    bits.iter()
        .map(|bit| if *bit { 1.0 } else { -1.0 })
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

/// Errors for exact `{-1,+1}` spin encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinEncodingError {
    /// The indexed value is not exactly one of the two declared spin values.
    NonBinarySpin { index: usize, bits: u64 },
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

impl Display for SpinEncodingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonBinarySpin { index, bits } => write!(
                formatter,
                "spin at index {index} is not exact -1.0 or +1.0 (bits=0x{bits:016x})"
            ),
        }
    }
}

impl Error for SpinEncodingError {}

#[cfg(test)]
mod tests {
    use super::{
        decode_spins, encode_spins, evaluate_predicates, ComponentThresholdPredicate,
        PredicateError, SpinEncodingError, ThresholdRelation,
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

    #[test]
    fn exact_spin_encoding_round_trips_in_order() {
        let spins = [-1.0, 1.0, 1.0, -1.0, -1.0];
        let bits = encode_spins(&spins).expect("exact declared spins");
        assert_eq!(bits, vec![false, true, true, false, false]);
        assert_eq!(decode_spins(&bits), spins);
    }

    #[test]
    fn spin_encoding_rejects_continuous_values_without_thresholding() {
        assert_eq!(
            encode_spins(&[-1.0, 0.999, 1.0]),
            Err(SpinEncodingError::NonBinarySpin {
                index: 1,
                bits: 0.999_f64.to_bits(),
            })
        );
        assert!(matches!(
            encode_spins(&[-0.0]),
            Err(SpinEncodingError::NonBinarySpin { index: 0, .. })
        ));
        assert!(matches!(
            encode_spins(&[f64::NAN]),
            Err(SpinEncodingError::NonBinarySpin { index: 0, .. })
        ));
    }

    #[test]
    fn empty_spin_vector_round_trips_without_inventing_state() {
        assert_eq!(
            encode_spins(&[]).expect("empty exact vector"),
            Vec::<bool>::new()
        );
        assert_eq!(decode_spins(&[]), Vec::<f64>::new());
    }
}
