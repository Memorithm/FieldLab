#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Unit-vector state carried by one field node.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeState {
    values: Vec<f64>,
}

impl NodeState {
    /// Builds a state that is already unit length within `tolerance`.
    ///
    /// # Errors
    ///
    /// Returns an error for empty, non-finite or non-unit input, or for an invalid tolerance.
    pub fn try_unit(values: Vec<f64>, tolerance: f64) -> Result<Self, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError::EmptyVector);
        }
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(ValidationError::InvalidTolerance);
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ValidationError::NonFiniteValue);
        }
        let norm_sq = dot(&values, &values);
        if (norm_sq - 1.0).abs() > tolerance {
            return Err(ValidationError::NonUnitVector { norm_sq });
        }
        Ok(Self { values })
    }

    /// Normalizes a finite non-zero vector into a node state.
    ///
    /// # Errors
    ///
    /// Returns an error for empty, non-finite or effectively zero input.
    pub fn from_normalized(values: Vec<f64>) -> Result<Self, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError::EmptyVector);
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ValidationError::NonFiniteValue);
        }
        let norm_sq = dot(&values, &values);
        if norm_sq <= f64::EPSILON {
            return Err(ValidationError::ZeroVector);
        }
        let inv_norm = norm_sq.sqrt().recip();
        Ok(Self {
            values: values.into_iter().map(|value| value * inv_norm).collect(),
        })
    }

    /// Returns the state components.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns the vector dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.values.len()
    }
}

/// Complete field state with homogeneous node dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldState {
    nodes: Vec<NodeState>,
    dimension: usize,
}

impl FieldState {
    /// Builds a non-empty homogeneous field state.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty state or mixed node dimensions.
    pub fn new(nodes: Vec<NodeState>) -> Result<Self, ValidationError> {
        let first = nodes.first().ok_or(ValidationError::EmptyState)?;
        let dimension = first.dimension();
        if nodes.iter().any(|node| node.dimension() != dimension) {
            return Err(ValidationError::DimensionMismatch);
        }
        Ok(Self { nodes, dimension })
    }

    /// Returns all nodes in stable index order.
    #[must_use]
    pub fn nodes(&self) -> &[NodeState] {
        &self.nodes
    }

    /// Returns one node by index.
    #[must_use]
    pub fn node(&self, index: usize) -> Option<&NodeState> {
        self.nodes.get(index)
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the common node dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Signed undirected pair coupling used by the FL-0 energy model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coupling {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

/// Validated finite graph of signed pair couplings.
#[derive(Clone, Debug, PartialEq)]
pub struct CouplingGraph {
    node_count: usize,
    couplings: Vec<Coupling>,
}

impl CouplingGraph {
    /// Builds a graph whose endpoints all refer to distinct valid nodes.
    ///
    /// # Errors
    ///
    /// Returns an error for zero nodes, invalid endpoints, self-coupling or non-finite weights.
    pub fn new(node_count: usize, couplings: Vec<Coupling>) -> Result<Self, ValidationError> {
        if node_count == 0 {
            return Err(ValidationError::EmptyState);
        }
        for coupling in &couplings {
            if coupling.source >= node_count || coupling.target >= node_count {
                return Err(ValidationError::NodeOutOfBounds);
            }
            if coupling.source == coupling.target {
                return Err(ValidationError::SelfCoupling);
            }
            if !coupling.weight.is_finite() {
                return Err(ValidationError::NonFiniteValue);
            }
        }
        Ok(Self {
            node_count,
            couplings,
        })
    }

    /// Returns the declared graph node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns couplings in deterministic insertion order.
    #[must_use]
    pub fn couplings(&self) -> &[Coupling] {
        &self.couplings
    }
}

/// FL-0 energy model containing graph couplings and external fields.
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyModel {
    graph: CouplingGraph,
    external_fields: Vec<Vec<f64>>,
    dimension: usize,
}

impl EnergyModel {
    /// Builds an energy model with one external vector field per graph node.
    ///
    /// # Errors
    ///
    /// Returns an error for count/dimension mismatches, empty vectors or non-finite fields.
    pub fn new(
        graph: CouplingGraph,
        external_fields: Vec<Vec<f64>>,
    ) -> Result<Self, ValidationError> {
        if external_fields.len() != graph.node_count() {
            return Err(ValidationError::NodeCountMismatch);
        }
        let first = external_fields.first().ok_or(ValidationError::EmptyState)?;
        if first.is_empty() {
            return Err(ValidationError::EmptyVector);
        }
        let dimension = first.len();
        if external_fields.iter().any(|field| field.len() != dimension) {
            return Err(ValidationError::DimensionMismatch);
        }
        if external_fields
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            return Err(ValidationError::NonFiniteValue);
        }
        Ok(Self {
            graph,
            external_fields,
            dimension,
        })
    }

    /// Returns the coupling graph.
    #[must_use]
    pub fn graph(&self) -> &CouplingGraph {
        &self.graph
    }

    /// Returns the field vector dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Validates compatibility between a state and this energy model.
    ///
    /// # Errors
    ///
    /// Returns an error when node count or vector dimension differs.
    pub fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        if state.node_count() != self.graph.node_count() {
            return Err(ValidationError::NodeCountMismatch);
        }
        if state.dimension() != self.dimension {
            return Err(ValidationError::DimensionMismatch);
        }
        Ok(())
    }

    /// Evaluates `-Σ h_i·m_i - Σ J_ij m_i·m_j`.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied state is incompatible with the model.
    pub fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        self.validate_state(state)?;
        let external = state
            .nodes()
            .iter()
            .zip(&self.external_fields)
            .map(|(node, field)| -dot(node.values(), field))
            .sum::<f64>();
        let mut interaction = 0.0;
        for coupling in self.graph.couplings() {
            let source = state
                .node(coupling.source)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let target = state
                .node(coupling.target)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            interaction -= coupling.weight * dot(source.values(), target.values());
        }
        Ok(external + interaction)
    }

    /// Computes the effective vector field at every node.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied state is incompatible or a graph endpoint is invalid.
    pub fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        self.validate_state(state)?;
        let mut result = self.external_fields.clone();
        for coupling in self.graph.couplings() {
            let source = state
                .node(coupling.source)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let target = state
                .node(coupling.target)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            for (axis, target_value) in target.values().iter().enumerate() {
                result[coupling.source][axis] += coupling.weight * target_value;
            }
            for (axis, source_value) in source.values().iter().enumerate() {
                result[coupling.target][axis] += coupling.weight * source_value;
            }
        }
        Ok(result)
    }
}

/// Computes a dot product for equal-length vectors.
#[must_use]
pub fn dot(left: &[f64], right: &[f64]) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

/// Validation failures surfaced by the deterministic field kernel.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationError {
    EmptyState,
    EmptyVector,
    ZeroVector,
    NonFiniteValue,
    InvalidTolerance,
    NonUnitVector { norm_sq: f64 },
    DimensionMismatch,
    NodeCountMismatch,
    NodeOutOfBounds,
    SelfCoupling,
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_unit_state() {
        let error = NodeState::try_unit(vec![2.0, 0.0], 1.0e-12).unwrap_err();
        assert!(matches!(error, ValidationError::NonUnitVector { .. }));
    }

    #[test]
    fn signed_coupling_changes_energy_preference() {
        let aligned = FieldState::new(vec![
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
        ])
        .unwrap();
        let attractive = EnergyModel::new(
            CouplingGraph::new(
                2,
                vec![Coupling {
                    source: 0,
                    target: 1,
                    weight: 1.0,
                }],
            )
            .unwrap(),
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
        )
        .unwrap();
        let repulsive = EnergyModel::new(
            CouplingGraph::new(
                2,
                vec![Coupling {
                    source: 0,
                    target: 1,
                    weight: -1.0,
                }],
            )
            .unwrap(),
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
        )
        .unwrap();

        assert!((attractive.energy(&aligned).unwrap() + 1.0).abs() < 1.0e-12);
        assert!((repulsive.energy(&aligned).unwrap() - 1.0).abs() < 1.0e-12);
    }
}
