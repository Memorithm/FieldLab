#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

const MATRIX_SYMMETRY_TOLERANCE: f64 = 1.0e-12;

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

/// Common conservative field-model contract used by deterministic dynamics.
pub trait FieldModel {
    /// Validates compatibility between a state and this model.
    ///
    /// # Errors
    ///
    /// Returns a validation error for incompatible states.
    fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError>;

    /// Evaluates the model energy at `state`.
    ///
    /// # Errors
    ///
    /// Returns a validation error for incompatible states.
    fn energy(&self, state: &FieldState) -> Result<f64, ValidationError>;

    /// Computes `-∂E/∂m_i` at every node.
    ///
    /// # Errors
    ///
    /// Returns a validation error for incompatible states.
    fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError>;
}

/// Signed undirected pair coupling used by the FL-E0 energy model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coupling {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

/// Validated finite graph of unique signed undirected pair couplings.
#[derive(Clone, Debug, PartialEq)]
pub struct CouplingGraph {
    node_count: usize,
    couplings: Vec<Coupling>,
}

impl CouplingGraph {
    /// Builds a graph whose endpoints all refer to distinct valid nodes.
    ///
    /// An undirected pair may occur only once. `(i, j)` and `(j, i)` identify
    /// the same physical edge and are therefore rejected as duplicates.
    ///
    /// # Errors
    ///
    /// Returns an error for zero nodes, invalid endpoints, self-coupling,
    /// duplicate undirected pairs or non-finite weights.
    pub fn new(node_count: usize, couplings: Vec<Coupling>) -> Result<Self, ValidationError> {
        if node_count == 0 {
            return Err(ValidationError::EmptyState);
        }
        let mut seen = BTreeSet::new();
        for coupling in &couplings {
            validate_endpoints(node_count, coupling.source, coupling.target)?;
            if !coupling.weight.is_finite() {
                return Err(ValidationError::NonFiniteValue);
            }
            let pair = canonical_pair(coupling.source, coupling.target);
            if !seen.insert(pair) {
                return Err(ValidationError::DuplicateCoupling {
                    source: pair.0,
                    target: pair.1,
                });
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

/// FL-E0: isotropic scalar pair energy with one external vector field per node.
///
/// The interaction sum is over unique undirected edges, not over every ordered
/// matrix pair. Therefore no `1/2` factor is required:
/// `E0 = -Σ_i h_i·m_i - Σ_{ {i,j}∈E } J_ij m_i·m_j`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnergyModel {
    graph: CouplingGraph,
    external_fields: Vec<Vec<f64>>,
    dimension: usize,
}

/// Explicit name for the historical FL-E0 model without breaking existing APIs.
pub type FlE0EnergyModel = EnergyModel;

impl EnergyModel {
    /// Builds an FL-E0 model with one external vector field per graph node.
    ///
    /// # Errors
    ///
    /// Returns an error for count/dimension mismatches, empty vectors or non-finite fields.
    pub fn new(
        graph: CouplingGraph,
        external_fields: Vec<Vec<f64>>,
    ) -> Result<Self, ValidationError> {
        let dimension = validate_external_fields(graph.node_count(), &external_fields)?;
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
        validate_state_shape(state, self.graph.node_count(), self.dimension)
    }

    /// Evaluates FL-E0 energy over unique undirected graph edges.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied state is incompatible with the model.
    pub fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        self.validate_state(state)?;
        let external = external_energy(state, &self.external_fields);
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

    /// Computes the FL-E0 effective vector field at every node.
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
            add_scaled(
                &mut result[coupling.source],
                target.values(),
                coupling.weight,
            );
            add_scaled(
                &mut result[coupling.target],
                source.values(),
                coupling.weight,
            );
        }
        Ok(result)
    }
}

impl FieldModel for EnergyModel {
    fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        Self::validate_state(self, state)
    }

    fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        Self::energy(self, state)
    }

    fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        Self::effective_fields(self, state)
    }
}

/// Operator-valued conservative pair coupling used by FL-E1.
///
/// The energy contribution is `-m_sourceᵀ K m_target`; consequently the
/// source receives `K m_target` and the target receives `Kᵀ m_source`.
#[derive(Clone, Debug, PartialEq)]
pub struct OperatorCoupling {
    pub source: usize,
    pub target: usize,
    pub operator: Vec<Vec<f64>>,
}

/// Symmetric local quadratic anisotropy used by FL-E1.
///
/// Its energy contribution is `-1/2 mᵀ A m` and its effective field is `A m`.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalAnisotropy {
    pub node: usize,
    pub matrix: Vec<Vec<f64>>,
}

/// FL-E1 conservative energy model with operator-valued pair couplings and
/// optional local quadratic anisotropy.
///
/// `E1 = -Σ_i h_i·m_i - Σ_{ {i,j} } m_iᵀ K_ij m_j
///       - 1/2 Σ_i m_iᵀ A_i m_i`.
#[derive(Clone, Debug, PartialEq)]
pub struct OperatorEnergyModel {
    external_fields: Vec<Vec<f64>>,
    couplings: Vec<OperatorCoupling>,
    anisotropies: Vec<LocalAnisotropy>,
    dimension: usize,
}

impl OperatorEnergyModel {
    /// Builds a validated FL-E1 model.
    ///
    /// Pair operators must be finite square matrices matching the state
    /// dimension. Local anisotropy matrices additionally must be symmetric.
    /// Undirected endpoint pairs and anisotropy nodes must be unique.
    ///
    /// # Errors
    ///
    /// Returns a validation error for malformed fields, operators, endpoints,
    /// duplicate interactions or non-symmetric anisotropy matrices.
    pub fn new(
        external_fields: Vec<Vec<f64>>,
        couplings: Vec<OperatorCoupling>,
        anisotropies: Vec<LocalAnisotropy>,
    ) -> Result<Self, ValidationError> {
        let node_count = external_fields.len();
        let dimension = validate_external_fields(node_count, &external_fields)?;

        let mut seen_pairs = BTreeSet::new();
        for coupling in &couplings {
            validate_endpoints(node_count, coupling.source, coupling.target)?;
            validate_square_matrix(&coupling.operator, dimension)?;
            let pair = canonical_pair(coupling.source, coupling.target);
            if !seen_pairs.insert(pair) {
                return Err(ValidationError::DuplicateCoupling {
                    source: pair.0,
                    target: pair.1,
                });
            }
        }

        let mut seen_anisotropy = BTreeSet::new();
        for anisotropy in &anisotropies {
            if anisotropy.node >= node_count {
                return Err(ValidationError::NodeOutOfBounds);
            }
            validate_square_matrix(&anisotropy.matrix, dimension)?;
            validate_symmetric_matrix(&anisotropy.matrix)?;
            if !seen_anisotropy.insert(anisotropy.node) {
                return Err(ValidationError::DuplicateAnisotropy {
                    node: anisotropy.node,
                });
            }
        }

        Ok(Self {
            external_fields,
            couplings,
            anisotropies,
            dimension,
        })
    }

    /// Lifts an FL-E0 model into FL-E1 using `K_ij = J_ij I` and no anisotropy.
    ///
    /// This conversion is intended as the exact backward-compatibility bridge
    /// and as an ablation control for FL-E1 experiments.
    #[must_use]
    pub fn from_e0(model: &EnergyModel) -> Self {
        let couplings = model
            .graph
            .couplings()
            .iter()
            .map(|coupling| OperatorCoupling {
                source: coupling.source,
                target: coupling.target,
                operator: scaled_identity(model.dimension, coupling.weight),
            })
            .collect();
        Self {
            external_fields: model.external_fields.clone(),
            couplings,
            anisotropies: Vec::new(),
            dimension: model.dimension,
        }
    }

    /// Returns the state-vector dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns operator-valued pair couplings in insertion order.
    #[must_use]
    pub fn couplings(&self) -> &[OperatorCoupling] {
        &self.couplings
    }

    /// Returns local anisotropy terms in insertion order.
    #[must_use]
    pub fn anisotropies(&self) -> &[LocalAnisotropy] {
        &self.anisotropies
    }

    /// Validates compatibility between a state and this energy model.
    ///
    /// # Errors
    ///
    /// Returns an error when node count or vector dimension differs.
    pub fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        validate_state_shape(state, self.external_fields.len(), self.dimension)
    }

    /// Evaluates the FL-E1 conservative energy.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied state is incompatible with the model.
    pub fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        self.validate_state(state)?;
        let mut energy = external_energy(state, &self.external_fields);

        for coupling in &self.couplings {
            let source = state
                .node(coupling.source)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let target = state
                .node(coupling.target)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let transformed = matrix_vector(&coupling.operator, target.values());
            energy -= dot(source.values(), &transformed);
        }

        for anisotropy in &self.anisotropies {
            let node = state
                .node(anisotropy.node)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let transformed = matrix_vector(&anisotropy.matrix, node.values());
            energy -= 0.5 * dot(node.values(), &transformed);
        }

        Ok(energy)
    }

    /// Computes the FL-E1 effective field `-∂E/∂m_i` at every node.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied state is incompatible with the model.
    pub fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        self.validate_state(state)?;
        let mut result = self.external_fields.clone();

        for coupling in &self.couplings {
            let source = state
                .node(coupling.source)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let target = state
                .node(coupling.target)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let source_field = matrix_vector(&coupling.operator, target.values());
            let target_field = matrix_transpose_vector(&coupling.operator, source.values());
            add_scaled(&mut result[coupling.source], &source_field, 1.0);
            add_scaled(&mut result[coupling.target], &target_field, 1.0);
        }

        for anisotropy in &self.anisotropies {
            let node = state
                .node(anisotropy.node)
                .ok_or(ValidationError::NodeOutOfBounds)?;
            let local_field = matrix_vector(&anisotropy.matrix, node.values());
            add_scaled(&mut result[anisotropy.node], &local_field, 1.0);
        }

        Ok(result)
    }
}

impl FieldModel for OperatorEnergyModel {
    fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        Self::validate_state(self, state)
    }

    fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        Self::energy(self, state)
    }

    fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        Self::effective_fields(self, state)
    }
}

/// Computes a dot product for equal-length vectors.
#[must_use]
pub fn dot(left: &[f64], right: &[f64]) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn validate_external_fields(
    node_count: usize,
    external_fields: &[Vec<f64>],
) -> Result<usize, ValidationError> {
    if node_count == 0 || external_fields.is_empty() {
        return Err(ValidationError::EmptyState);
    }
    if external_fields.len() != node_count {
        return Err(ValidationError::NodeCountMismatch);
    }
    let dimension = external_fields[0].len();
    if dimension == 0 {
        return Err(ValidationError::EmptyVector);
    }
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
    Ok(dimension)
}

fn validate_state_shape(
    state: &FieldState,
    node_count: usize,
    dimension: usize,
) -> Result<(), ValidationError> {
    if state.node_count() != node_count {
        return Err(ValidationError::NodeCountMismatch);
    }
    if state.dimension() != dimension {
        return Err(ValidationError::DimensionMismatch);
    }
    Ok(())
}

fn validate_endpoints(
    node_count: usize,
    source: usize,
    target: usize,
) -> Result<(), ValidationError> {
    if source >= node_count || target >= node_count {
        return Err(ValidationError::NodeOutOfBounds);
    }
    if source == target {
        return Err(ValidationError::SelfCoupling);
    }
    Ok(())
}

fn validate_square_matrix(matrix: &[Vec<f64>], dimension: usize) -> Result<(), ValidationError> {
    if matrix.len() != dimension || matrix.iter().any(|row| row.len() != dimension) {
        return Err(ValidationError::InvalidMatrixShape);
    }
    if matrix.iter().flatten().any(|value| !value.is_finite()) {
        return Err(ValidationError::NonFiniteValue);
    }
    Ok(())
}

fn validate_symmetric_matrix(matrix: &[Vec<f64>]) -> Result<(), ValidationError> {
    for (row, row_values) in matrix.iter().enumerate() {
        for (column, column_values) in matrix.iter().enumerate().skip(row + 1) {
            if (row_values[column] - column_values[row]).abs() > MATRIX_SYMMETRY_TOLERANCE {
                return Err(ValidationError::NonSymmetricMatrix { row, column });
            }
        }
    }
    Ok(())
}

fn canonical_pair(source: usize, target: usize) -> (usize, usize) {
    if source < target {
        (source, target)
    } else {
        (target, source)
    }
}

fn external_energy(state: &FieldState, external_fields: &[Vec<f64>]) -> f64 {
    state
        .nodes()
        .iter()
        .zip(external_fields)
        .map(|(node, field)| -dot(node.values(), field))
        .sum()
}

fn add_scaled(destination: &mut [f64], source: &[f64], scale: f64) {
    for (destination_value, source_value) in destination.iter_mut().zip(source) {
        *destination_value += scale * source_value;
    }
}

fn scaled_identity(dimension: usize, scale: f64) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; dimension]; dimension];
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = scale;
    }
    matrix
}

fn matrix_vector(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
    matrix.iter().map(|row| dot(row, vector)).collect()
}

fn matrix_transpose_vector(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
    let dimension = vector.len();
    (0..dimension)
        .map(|column| {
            matrix
                .iter()
                .zip(vector)
                .map(|(row, value)| row[column] * value)
                .sum()
        })
        .collect()
}

/// Validation failures surfaced by deterministic field models.
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
    DuplicateCoupling { source: usize, target: usize },
    InvalidMatrixShape,
    NonSymmetricMatrix { row: usize, column: usize },
    DuplicateAnisotropy { node: usize },
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

    fn pair_state() -> FieldState {
        FieldState::new(vec![
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
            NodeState::try_unit(vec![0.0, 1.0], 1.0e-12).unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn rejects_non_unit_state() {
        let error = NodeState::try_unit(vec![2.0, 0.0], 1.0e-12).unwrap_err();
        assert!(matches!(error, ValidationError::NonUnitVector { .. }));
    }

    #[test]
    fn rejects_duplicate_undirected_edge_in_either_orientation() {
        let error = CouplingGraph::new(
            2,
            vec![
                Coupling {
                    source: 0,
                    target: 1,
                    weight: 1.0,
                },
                Coupling {
                    source: 1,
                    target: 0,
                    weight: 2.0,
                },
            ],
        )
        .unwrap_err();
        assert_eq!(
            error,
            ValidationError::DuplicateCoupling {
                source: 0,
                target: 1
            }
        );
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

    #[test]
    fn e1_scalar_identity_lift_matches_e0() {
        let state = pair_state();
        let e0 = EnergyModel::new(
            CouplingGraph::new(
                2,
                vec![Coupling {
                    source: 0,
                    target: 1,
                    weight: -0.75,
                }],
            )
            .unwrap(),
            vec![vec![0.2, -0.1], vec![-0.3, 0.4]],
        )
        .unwrap();
        let e1 = OperatorEnergyModel::from_e0(&e0);

        assert!((e0.energy(&state).unwrap() - e1.energy(&state).unwrap()).abs() < 1.0e-12);
        let fields0 = e0.effective_fields(&state).unwrap();
        let fields1 = e1.effective_fields(&state).unwrap();
        for (left, right) in fields0.iter().flatten().zip(fields1.iter().flatten()) {
            assert!((left - right).abs() < 1.0e-12);
        }
    }

    #[test]
    fn e1_operator_can_map_between_axes() {
        let state = FieldState::new(vec![
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
        ])
        .unwrap();
        let model = OperatorEnergyModel::new(
            vec![vec![0.0, 0.0]; 2],
            vec![OperatorCoupling {
                source: 0,
                target: 1,
                operator: vec![vec![0.0, -1.0], vec![1.0, 0.0]],
            }],
            Vec::new(),
        )
        .unwrap();
        let fields = model.effective_fields(&state).unwrap();
        assert_eq!(fields[0], vec![0.0, 1.0]);
        assert_eq!(fields[1], vec![0.0, -1.0]);
    }

    #[test]
    fn e1_rejects_non_symmetric_local_anisotropy() {
        let error = OperatorEnergyModel::new(
            vec![vec![0.0, 0.0]],
            Vec::new(),
            vec![LocalAnisotropy {
                node: 0,
                matrix: vec![vec![1.0, 1.0], vec![0.0, 1.0]],
            }],
        )
        .unwrap_err();
        assert!(matches!(error, ValidationError::NonSymmetricMatrix { .. }));
    }
}
