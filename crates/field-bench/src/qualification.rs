//! Small deterministic qualification helpers for field experiments.
//!
//! The curvature certificate is sufficient, not necessary. A nonpositive row
//! bound is inconclusive; it must not be reported as an instability proof.

use field_core::CouplingGraph;
use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Tangent Hessian of a zero-field scalar graph at a two-dimensional axial state.
/// The local energy is `-a/2 * sum_i m_ix^2`, with one shared nonnegative `a`.
#[derive(Clone, Debug, PartialEq)]
pub struct AxialHessian {
    matrix: Vec<Vec<f64>>,
    lower_bound: f64,
}

impl AxialHessian {
    /// Uses the angular chart `m_i(q) = s_i (cos(q_i), sin(q_i))`.
    ///
    /// # Errors
    /// Rejects invalid signs, dimensions, stiffness and non-finite arithmetic.
    pub fn new(
        graph: &CouplingGraph,
        signs: &[i8],
        stiffness: f64,
    ) -> Result<Self, QualificationError> {
        if signs.len() != graph.node_count() {
            return Err(QualificationError::Dimension);
        }
        if signs.iter().any(|s| !matches!(*s, -1 | 1)) {
            return Err(QualificationError::InvalidSign);
        }
        if !stiffness.is_finite() || stiffness < 0.0 {
            return Err(QualificationError::InvalidStiffness);
        }
        let mut matrix = vec![vec![0.0; signs.len()]; signs.len()];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[i] = stiffness;
        }
        for edge in graph.couplings() {
            let i = edge.source;
            let j = edge.target;
            let b = edge.weight * f64::from(signs[i]) * f64::from(signs[j]);
            matrix[i][i] += b;
            matrix[j][j] += b;
            matrix[i][j] -= b;
            matrix[j][i] -= b;
        }
        if matrix.iter().flatten().any(|x| !x.is_finite()) {
            return Err(QualificationError::NonFinite);
        }
        let mut lower_bound = f64::INFINITY;
        for (i, row) in matrix.iter().enumerate() {
            let radius: f64 = row
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, x)| x.abs())
                .sum();
            let bound = row[i] - radius;
            if !bound.is_finite() {
                return Err(QualificationError::NonFinite);
            }
            lower_bound = lower_bound.min(bound);
        }
        Ok(Self { matrix, lower_bound })
    }

    /// Symmetric angular Hessian, in graph node order.
    #[must_use]
    pub fn matrix(&self) -> &[Vec<f64>] {
        &self.matrix
    }

    /// Gershgorin lower bound; a strictly positive value certifies local curvature.
    /// This is floating-point arithmetic, not an interval enclosure.
    #[must_use]
    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    /// Computes `v^T B v`; the caller controls normalization of `v`.
    ///
    /// # Errors
    /// Rejects dimension mismatches, non-finite vectors or arithmetic overflow.
    pub fn quadratic_form(&self, direction: &[f64]) -> Result<f64, QualificationError> {
        if direction.len() != self.matrix.len() {
            return Err(QualificationError::Dimension);
        }
        if direction.iter().any(|x| !x.is_finite()) {
            return Err(QualificationError::NonFinite);
        }
        let result: f64 = self
            .matrix
            .iter()
            .zip(direction)
            .map(|(row, v)| {
                v * row.iter().zip(direction).map(|(b, w)| b * w).sum::<f64>()
            })
            .sum();
        if !result.is_finite() {
            return Err(QualificationError::NonFinite);
        }
        Ok(result)
    }
}

/// All labels attached to one exact observable input, without case-ID aliases.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObservableGroup<K, L: Ord> {
    pub observable: K,
    pub label_counts: BTreeMap<L, usize>,
}

impl<K, L: Ord> ObservableGroup<K, L> {
    /// Whether identical observables request incompatible exact labels.
    #[must_use]
    pub fn conflicts(&self) -> bool {
        self.label_counts.len() > 1
    }

    /// Maximum exact-label successes for one deterministic output in this group.
    #[must_use]
    pub fn maximum_correct(&self) -> usize {
        self.label_counts.values().copied().max().unwrap_or(0)
    }
}

/// Groups by actual observable inputs, not record identifiers or desired labels.
/// Include all model-visible context in `K`; do not include hidden target labels.
///
/// # Errors
/// Rejects an empty panel, which cannot support an accuracy claim.
pub fn group_observables<K: Ord + Clone, L: Ord + Clone>(
    records: &[(K, L)],
) -> Result<Vec<ObservableGroup<K, L>>, QualificationError> {
    if records.is_empty() {
        return Err(QualificationError::EmptyPanel);
    }
    let mut groups: BTreeMap<K, BTreeMap<L, usize>> = BTreeMap::new();
    for (key, label) in records {
        *groups.entry(key.clone()).or_default().entry(label.clone()).or_default() += 1;
    }
    Ok(groups
        .into_iter()
        .map(|(observable, label_counts)| ObservableGroup { observable, label_counts })
        .collect())
}

/// Malformed qualification input or failed finite arithmetic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QualificationError {
    Dimension,
    InvalidSign,
    InvalidStiffness,
    NonFinite,
    EmptyPanel,
}

impl Display for QualificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for QualificationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use field_core::Coupling;

    fn graph(weight: f64) -> CouplingGraph {
        CouplingGraph::new(2, vec![Coupling { source: 0, target: 1, weight }]).unwrap()
    }

    #[test]
    fn global_rotation_is_neutral_without_anisotropy() {
        let h = AxialHessian::new(&graph(1.0), &[1, 1], 0.0).unwrap();
        assert_eq!(h.quadratic_form(&[1.0, 1.0]).unwrap().to_bits(), 0.0_f64.to_bits());
        assert_eq!(h.lower_bound().to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn antiparallel_positive_edge_has_negative_curvature() {
        let h = AxialHessian::new(&graph(1.0), &[1, -1], 0.0).unwrap();
        assert!(h.quadratic_form(&[1.0, -1.0]).unwrap() < 0.0);
    }

    #[test]
    fn easy_axis_shifts_hessian_by_identity() {
        let h0 = AxialHessian::new(&graph(1.0), &[1, -1], 0.0).unwrap();
        let h3 = AxialHessian::new(&graph(1.0), &[1, -1], 3.0).unwrap();
        assert!((h3.lower_bound() - 1.0).abs() < 1e-12);
        let v = [0.3, -0.4];
        let delta = h3.quadratic_form(&v).unwrap() - h0.quadratic_form(&v).unwrap();
        assert!((delta - 0.75).abs() < 1e-12);
    }

    #[test]
    fn malformed_and_overflowing_inputs_fail_closed() {
        assert!(AxialHessian::new(&graph(1.0), &[1], 0.0).is_err());
        assert!(AxialHessian::new(&graph(1.0), &[1, 0], 0.0).is_err());
        for a in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(AxialHessian::new(&graph(1.0), &[1, 1], a).is_err());
        }
        assert!(AxialHessian::new(&graph(f64::MAX), &[1, 1], f64::MAX).is_err());
        let h = AxialHessian::new(&graph(1.0), &[1, 1], 0.0).unwrap();
        assert!(h.quadratic_form(&[1.0]).is_err());
        assert!(h.quadratic_form(&[f64::NAN, 0.0]).is_err());
        assert!(h.quadratic_form(&[f64::MAX, 0.0]).is_err());
    }

    #[test]
    fn duplicate_observables_do_not_create_independent_groups() {
        let groups = group_observables(&[(vec![1, -1], 0), (vec![1, -1], 0)]).unwrap();
        assert_eq!(groups.len(), 1);
        assert!(!groups[0].conflicts());
        assert_eq!(groups[0].maximum_correct(), 2);
    }

    #[test]
    fn contradictory_labels_are_reported_with_accuracy_bound() {
        let groups = group_observables(&[(vec![1], 0), (vec![1], 1), (vec![-1], 1)]).unwrap();
        assert_eq!(groups.iter().filter(|g| g.conflicts()).count(), 1);
        assert_eq!(groups.iter().map(ObservableGroup::maximum_correct).sum::<usize>(), 2);
        assert!(group_observables::<Vec<i8>, usize>(&[]).is_err());
    }
}
