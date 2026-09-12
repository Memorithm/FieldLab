#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub struct NodeState {
    values: Vec<f64>,
}

impl NodeState {
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

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn dimension(&self) -> usize {
        self.values.len()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldState {
    nodes: Vec<NodeState>,
    dimension: usize,
}

impl FieldState {
    pub fn new(nodes: Vec<NodeState>) -> Result<Self, ValidationError> {
        let first = nodes.first().ok_or(ValidationError::EmptyState)?;
        let dimension = first.dimension();
        if nodes.iter().any(|node| node.dimension() != dimension) {
            return Err(ValidationError::DimensionMismatch);
        }
        Ok(Self { nodes, dimension })
    }

    pub fn nodes(&self) -> &[NodeState] {
        &self.nodes
    }

    pub fn node(&self, index: usize) -> Option<&NodeState> {
        self.nodes.get(index)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coupling {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CouplingGraph {
    node_count: usize,
    couplings: Vec<Coupling>,
}

impl CouplingGraph {
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

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn couplings(&self) -> &[Coupling] {
        &self.couplings
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnergyModel {
    graph: CouplingGraph,
    external_fields: Vec<Vec<f64>>,
    dimension: usize,
}

impl EnergyModel {
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

    pub fn graph(&self) -> &CouplingGraph {
        &self.graph
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        if state.node_count() != self.graph.node_count() {
            return Err(ValidationError::NodeCountMismatch);
        }
        if state.dimension() != self.dimension {
            return Err(ValidationError::DimensionMismatch);
        }
        Ok(())
    }

    pub fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        self.validate_state(state)?;
        let external = state
            .nodes()
            .iter()
            .zip(&self.external_fields)
            .map(|(node, field)| -dot(node.values(), field))
            .sum::<f64>();
        let interaction = self
            .graph
            .couplings()
            .iter()
            .map(|coupling| {
                let source = state.node(coupling.source).expect("validated source index");
                let target = state.node(coupling.target).expect("validated target index");
                -coupling.weight * dot(source.values(), target.values())
            })
            .sum::<f64>();
        Ok(external + interaction)
    }

    pub fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        self.validate_state(state)?;
        let mut result = self.external_fields.clone();
        for coupling in self.graph.couplings() {
            let source = state.node(coupling.source).expect("validated source index");
            let target = state.node(coupling.target).expect("validated target index");
            for axis in 0..self.dimension {
                result[coupling.source][axis] += coupling.weight * target.values()[axis];
                result[coupling.target][axis] += coupling.weight * source.values()[axis];
            }
        }
        Ok(result)
    }
}

pub fn dot(left: &[f64], right: &[f64]) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

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
