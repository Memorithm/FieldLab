#![forbid(unsafe_code)]

use field_core::{Coupling, CouplingGraph, EnergyModel, FieldState, NodeState, ValidationError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternBank {
    patterns: Vec<Vec<i8>>,
    node_count: usize,
}

impl PatternBank {
    /// Builds a validated bank of bipolar patterns using values in `{-1, +1}`.
    ///
    /// # Errors
    ///
    /// Returns an error when the bank is empty, patterns are empty, pattern lengths differ,
    /// or any symbol is not `-1` or `+1`.
    pub fn new(patterns: Vec<Vec<i8>>) -> Result<Self, MemoryError> {
        let first = patterns.first().ok_or(MemoryError::EmptyBank)?;
        if first.is_empty() {
            return Err(MemoryError::EmptyPattern);
        }
        let node_count = first.len();
        for pattern in &patterns {
            validate_pattern(pattern, node_count)?;
        }
        Ok(Self {
            patterns,
            node_count,
        })
    }

    #[must_use]
    pub fn patterns(&self) -> &[Vec<i8>] {
        &self.patterns
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Constructs symmetric Hebbian pair couplings with zero self-coupling.
    ///
    /// The weight for `i != j` is `(1 / N) * sum_mu p_mu[i] * p_mu[j]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the node count cannot be represented safely for the floating-point
    /// normalization factor or if the resulting graph violates `field-core` invariants.
    pub fn hebbian_graph(&self) -> Result<CouplingGraph, MemoryError> {
        let node_count_u32 =
            u32::try_from(self.node_count).map_err(|_| MemoryError::TooManyNodes)?;
        let denominator = f64::from(node_count_u32);
        let mut couplings = Vec::new();

        for source in 0..self.node_count {
            for target in (source + 1)..self.node_count {
                let signed_sum = self
                    .patterns
                    .iter()
                    .map(|pattern| i32::from(pattern[source]) * i32::from(pattern[target]))
                    .sum::<i32>();
                let weight = f64::from(signed_sum) / denominator;
                couplings.push(Coupling {
                    source,
                    target,
                    weight,
                });
            }
        }

        Ok(CouplingGraph::new(self.node_count, couplings)?)
    }

    /// Constructs the zero-external-field associative-memory energy model.
    ///
    /// # Errors
    ///
    /// Returns an error if the Hebbian graph or energy model cannot be constructed.
    pub fn energy_model(&self) -> Result<EnergyModel, MemoryError> {
        let graph = self.hebbian_graph()?;
        let external_fields = vec![vec![0.0, 0.0]; self.node_count];
        Ok(EnergyModel::new(graph, external_fields)?)
    }

    /// Encodes a bipolar cue into two-dimensional unit vectors with a fixed positive tilt.
    ///
    /// The first component carries the bipolar symbol. The second component is the same positive
    /// symmetry-breaking tilt for every node, preventing an exactly collinear cue from having zero
    /// tangent velocity under the projected field dynamics.
    ///
    /// # Errors
    ///
    /// Returns an error if the cue is invalid or the tilt is not finite and strictly inside
    /// `(0, pi/2)`.
    pub fn encode_cue(&self, cue: &[i8], tilt_radians: f64) -> Result<FieldState, MemoryError> {
        validate_pattern(cue, self.node_count)?;
        if !tilt_radians.is_finite()
            || tilt_radians <= 0.0
            || tilt_radians >= std::f64::consts::FRAC_PI_2
        {
            return Err(MemoryError::InvalidTilt);
        }

        let (tilt_sine, tilt_cosine) = tilt_radians.sin_cos();
        let nodes = cue
            .iter()
            .map(|symbol| {
                NodeState::try_unit(vec![f64::from(*symbol) * tilt_cosine, tilt_sine], 1.0e-12)
                    .map_err(MemoryError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FieldState::new(nodes)?)
    }

    /// Decodes the sign of the first vector component back into a bipolar pattern.
    ///
    /// # Errors
    ///
    /// Returns an error if the state shape does not match the pattern bank or is not two-dimensional.
    pub fn decode_state(&self, state: &FieldState) -> Result<Vec<i8>, MemoryError> {
        if state.node_count() != self.node_count {
            return Err(MemoryError::PatternLengthMismatch);
        }
        if state.dimension() != 2 {
            return Err(MemoryError::InvalidStateDimension);
        }
        Ok(state
            .nodes()
            .iter()
            .map(|node| if node.values()[0] >= 0.0 { 1 } else { -1 })
            .collect())
    }

    /// Returns the deterministic nearest template, breaking equal-distance ties by bank order.
    ///
    /// # Errors
    ///
    /// Returns an error if the cue is not a valid bipolar pattern of the bank's width.
    pub fn nearest_neighbor_index(&self, cue: &[i8]) -> Result<usize, MemoryError> {
        validate_pattern(cue, self.node_count)?;
        self.patterns
            .iter()
            .enumerate()
            .min_by_key(|(_, pattern)| hamming_distance(pattern, cue))
            .map(|(index, _)| index)
            .ok_or(MemoryError::EmptyBank)
    }

    /// Runs a deterministic asynchronous Hopfield update in ascending node order using exactly
    /// the same Hebbian coupling matrix as the field model.
    ///
    /// # Errors
    ///
    /// Returns an error if the cue is invalid, `max_sweeps` is zero, or the coupling matrix cannot
    /// be constructed safely.
    pub fn hopfield_recover(&self, cue: &[i8], max_sweeps: usize) -> Result<Vec<i8>, MemoryError> {
        validate_pattern(cue, self.node_count)?;
        if max_sweeps == 0 {
            return Err(MemoryError::ZeroSweeps);
        }
        let matrix = self.coupling_matrix()?;
        let mut state = cue.to_vec();

        for _ in 0..max_sweeps {
            let previous = state.clone();
            for node_index in 0..self.node_count {
                let local_field = matrix[node_index]
                    .iter()
                    .zip(&state)
                    .map(|(weight, symbol)| weight * f64::from(*symbol))
                    .sum::<f64>();
                if local_field > 0.0 {
                    state[node_index] = 1;
                } else if local_field < 0.0 {
                    state[node_index] = -1;
                }
            }
            if state == previous {
                break;
            }
        }
        Ok(state)
    }

    fn coupling_matrix(&self) -> Result<Vec<Vec<f64>>, MemoryError> {
        let graph = self.hebbian_graph()?;
        let mut matrix = vec![vec![0.0; self.node_count]; self.node_count];
        for coupling in graph.couplings() {
            matrix[coupling.source][coupling.target] = coupling.weight;
            matrix[coupling.target][coupling.source] = coupling.weight;
        }
        Ok(matrix)
    }
}

fn validate_pattern(pattern: &[i8], expected_len: usize) -> Result<(), MemoryError> {
    if pattern.is_empty() {
        return Err(MemoryError::EmptyPattern);
    }
    if pattern.len() != expected_len {
        return Err(MemoryError::PatternLengthMismatch);
    }
    if pattern.iter().any(|symbol| !matches!(*symbol, -1 | 1)) {
        return Err(MemoryError::InvalidSymbol);
    }
    Ok(())
}

fn hamming_distance(left: &[i8], right: &[i8]) -> usize {
    left.iter()
        .zip(right)
        .filter(|(left_symbol, right_symbol)| left_symbol != right_symbol)
        .count()
}

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryError {
    EmptyBank,
    EmptyPattern,
    PatternLengthMismatch,
    InvalidSymbol,
    InvalidTilt,
    InvalidStateDimension,
    ZeroSweeps,
    TooManyNodes,
    Core(ValidationError),
}

impl Display for MemoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for MemoryError {}

impl From<ValidationError> for MemoryError {
    fn from(error: ValidationError) -> Self {
        Self::Core(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bank() -> PatternBank {
        PatternBank::new(vec![vec![1, 1, 1, 1], vec![1, 1, -1, -1]]).unwrap()
    }

    #[test]
    fn invalid_symbol_is_rejected() {
        let error = PatternBank::new(vec![vec![1, 0, -1]]).unwrap_err();
        assert_eq!(error, MemoryError::InvalidSymbol);
    }

    #[test]
    fn nearest_neighbor_uses_hamming_distance_and_bank_order_for_ties() {
        let bank = bank();
        assert_eq!(bank.nearest_neighbor_index(&[1, 1, -1, 1]).unwrap(), 0);
    }

    #[test]
    fn stored_pattern_is_hopfield_fixed_point() {
        let bank = bank();
        let recovered = bank.hopfield_recover(&[1, 1, -1, -1], 16).unwrap();
        assert_eq!(recovered, vec![1, 1, -1, -1]);
    }

    #[test]
    fn cue_encoding_is_unit_norm_and_decodable() {
        let bank = bank();
        let encoded = bank.encode_cue(&[1, -1, 1, -1], 0.15).unwrap();
        assert_eq!(bank.decode_state(&encoded).unwrap(), vec![1, -1, 1, -1]);
    }
}
