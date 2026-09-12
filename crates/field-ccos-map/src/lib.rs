#![forbid(unsafe_code)]

use std::cmp::Ordering;

/// CCOS-Core commit whose semantics FL-4A pins.
pub const PINNED_CCOS_COMMIT: &str = "a3c4d7e03744430c74dc337463ff3e944b4933ad";

/// Lifecycle contribution used by the pinned CCOS node score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Working,
    Orphan,
}

impl Lifecycle {
    #[must_use]
    pub const fn score_bias(self) -> f64 {
        match self {
            Self::Stable => 0.0,
            Self::Working => 0.3,
            Self::Orphan => -1.0,
        }
    }
}

/// Score and propagation coefficients mirrored from the pinned CCOS semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoringWeights {
    pub w_base: f64,
    pub w_failure: f64,
    pub w_recency: f64,
    pub w_access: f64,
    pub w_centrality: f64,
    pub w_trust: f64,
    pub failure_decay: f64,
    pub failure_fanout: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            w_base: 0.15,
            w_failure: 0.50,
            w_recency: 0.30,
            w_access: 0.05,
            w_centrality: 0.0,
            w_trust: 0.0,
            failure_decay: 0.8,
            failure_fanout: 6.0,
        }
    }
}

/// Minimal replayable CCOS node state required by FL-4A.
#[derive(Clone, Debug, PartialEq)]
pub struct CcosNode {
    pub id: String,
    pub base_importance: f64,
    pub failure_relevance: f64,
    pub recency: f64,
    pub access_count: u64,
    pub trust: f64,
    pub lifecycle: Lifecycle,
    pub content: String,
}

/// Directed weighted causal edge in insertion order.
#[derive(Clone, Debug, PartialEq)]
pub struct CausalEdge {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

/// Individual scalar-source terms of the field representation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarSources {
    pub base: f64,
    pub failure: f64,
    pub recency: f64,
    pub access: f64,
    pub centrality: f64,
    pub lifecycle: f64,
    pub distrust: f64,
}

impl ScalarSources {
    #[must_use]
    pub fn potential(self) -> f64 {
        self.base + self.failure + self.recency + self.access + self.centrality + self.lifecycle
            - self.distrust
    }

    #[must_use]
    pub fn activation(self) -> f64 {
        self.potential().clamp(0.0, 1.0)
    }
}

#[allow(clippy::cast_precision_loss)]
fn u64_as_f64(value: u64) -> f64 {
    value as f64
}

#[allow(clippy::cast_precision_loss)]
fn usize_as_f64(value: usize) -> f64 {
    value as f64
}

/// Exact pinned CCOS score formula, written independently of the field decomposition.
#[must_use]
pub fn reference_score(node: &CcosNode, in_degree: u32, weights: ScoringWeights) -> f64 {
    let base = node.base_importance * weights.w_base;
    let failure = node.failure_relevance * weights.w_failure;
    let recency = node.recency * weights.w_recency;
    let access = u64_as_f64(node.access_count.max(1)).ln() * weights.w_access;
    let centrality = if weights.w_centrality == 0.0 {
        0.0
    } else {
        (1.0 + f64::from(in_degree)).ln() * weights.w_centrality
    };
    let distrust = if weights.w_trust == 0.0 {
        0.0
    } else {
        (1.0 - node.trust.clamp(0.0, 1.0)) * weights.w_trust
    };
    (base + failure + recency + access + centrality + node.lifecycle.score_bias() - distrust)
        .clamp(0.0, 1.0)
}

/// Decomposes the same CCOS score into scalar field sources.
#[must_use]
pub fn field_sources(node: &CcosNode, in_degree: u32, weights: ScoringWeights) -> ScalarSources {
    ScalarSources {
        base: node.base_importance * weights.w_base,
        failure: node.failure_relevance * weights.w_failure,
        recency: node.recency * weights.w_recency,
        access: u64_as_f64(node.access_count.max(1)).ln() * weights.w_access,
        centrality: if weights.w_centrality == 0.0 {
            0.0
        } else {
            (1.0 + f64::from(in_degree)).ln() * weights.w_centrality
        },
        lifecycle: node.lifecycle.score_bias(),
        distrust: if weights.w_trust == 0.0 {
            0.0
        } else {
            (1.0 - node.trust.clamp(0.0, 1.0)) * weights.w_trust
        },
    }
}

#[must_use]
pub fn in_degrees(node_count: usize, edges: &[CausalEdge]) -> Vec<u32> {
    let mut degrees = vec![0_u32; node_count];
    for edge in edges {
        if edge.target < node_count && edge.source < node_count {
            degrees[edge.target] = degrees[edge.target].saturating_add(1);
        }
    }
    degrees
}

/// Mirrors the pinned recursive CCOS pressure propagation.
pub fn reference_propagate_failure(
    nodes: &mut [CcosNode],
    edges: &[CausalEdge],
    origin: usize,
    max_depth: u32,
    paging_floor: f64,
    weights: ScoringWeights,
) {
    reference_recurse(nodes, edges, origin, 0, max_depth, paging_floor, weights);
}

fn reference_recurse(
    nodes: &mut [CcosNode],
    edges: &[CausalEdge],
    origin: usize,
    depth: u32,
    max_depth: u32,
    floor: f64,
    weights: ScoringWeights,
) {
    if depth > max_depth || origin >= nodes.len() {
        return;
    }
    let base = nodes[origin].failure_relevance;
    let targets: Vec<(usize, f64)> = edges
        .iter()
        .filter(|edge| edge.source == origin)
        .map(|edge| (edge.target, edge.weight))
        .collect();
    let fanout = weights.failure_fanout.max(1.0);
    let damp = (fanout / usize_as_f64(targets.len()).max(fanout)).min(1.0);
    for (target, edge_weight) in targets {
        let delta = base * edge_weight * weights.failure_decay.powi(depth.cast_signed()) * damp;
        if let Some(node) = nodes.get_mut(target) {
            node.failure_relevance = (node.failure_relevance + delta).clamp(0.0, 1.0);
            node.recency = 1.0;
        }
        if delta > floor {
            reference_recurse(nodes, edges, target, depth + 1, max_depth, floor, weights);
        }
    }
}

/// Scalar-field emission implementation of the same ordered propagation semantics.
pub fn field_propagate_failure(
    nodes: &mut [CcosNode],
    edges: &[CausalEdge],
    origin: usize,
    max_depth: u32,
    paging_floor: f64,
    weights: ScoringWeights,
) {
    emit_field(nodes, edges, origin, 0, max_depth, paging_floor, weights);
}

fn emit_field(
    nodes: &mut [CcosNode],
    edges: &[CausalEdge],
    source: usize,
    depth: u32,
    max_depth: u32,
    floor: f64,
    weights: ScoringWeights,
) {
    if depth > max_depth || source >= nodes.len() {
        return;
    }
    let source_amplitude = nodes[source].failure_relevance;
    let outgoing: Vec<&CausalEdge> = edges.iter().filter(|edge| edge.source == source).collect();
    let fanout_limit = weights.failure_fanout.max(1.0);
    let distribution = (fanout_limit / usize_as_f64(outgoing.len()).max(fanout_limit)).min(1.0);
    for edge in outgoing {
        let emission = source_amplitude
            * edge.weight
            * weights.failure_decay.powi(depth.cast_signed())
            * distribution;
        if let Some(target) = nodes.get_mut(edge.target) {
            target.failure_relevance = (target.failure_relevance + emission).clamp(0.0, 1.0);
            target.recency = 1.0;
        }
        if emission > floor {
            emit_field(
                nodes,
                edges,
                edge.target,
                depth + 1,
                max_depth,
                floor,
                weights,
            );
        }
    }
}

/// CCOS-like working-set assembly: score descending, URI ascending, then budget truncation.
#[must_use]
pub fn reference_working_set(
    nodes: &[CcosNode],
    edges: &[CausalEdge],
    budget_tokens: usize,
    weights: ScoringWeights,
) -> (Vec<String>, usize) {
    let degrees = in_degrees(nodes.len(), edges);
    assemble(nodes, budget_tokens, |index, node| {
        reference_score(node, degrees[index], weights)
    })
}

/// Working set assembled from scalar field activation rather than the reference score function.
#[must_use]
pub fn field_working_set(
    nodes: &[CcosNode],
    edges: &[CausalEdge],
    budget_tokens: usize,
    weights: ScoringWeights,
) -> (Vec<String>, usize) {
    let degrees = in_degrees(nodes.len(), edges);
    assemble(nodes, budget_tokens, |index, node| {
        field_sources(node, degrees[index], weights).activation()
    })
}

fn assemble<F>(nodes: &[CcosNode], budget_tokens: usize, mut score: F) -> (Vec<String>, usize)
where
    F: FnMut(usize, &CcosNode) -> f64,
{
    let mut ranked: Vec<(usize, f64)> = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.id.starts_with("dep:") && !node.content.trim().is_empty())
        .map(|(index, node)| (index, score(index, node)))
        .collect();
    ranked.sort_by(|(left_index, left_score), (right_index, right_score)| {
        right_score
            .partial_cmp(left_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| nodes[*left_index].id.cmp(&nodes[*right_index].id))
    });

    let mut selected = Vec::new();
    let mut tokens = 0_usize;
    let mut seen_content: Vec<&str> = Vec::new();
    for (index, _) in ranked {
        let node = &nodes[index];
        if seen_content.contains(&node.content.as_str()) {
            continue;
        }
        let item_tokens = node.content.chars().count() / 4;
        if tokens + item_tokens > budget_tokens && !selected.is_empty() {
            break;
        }
        seen_content.push(node.content.as_str());
        tokens += item_tokens;
        selected.push(node.id.clone());
    }
    (selected, tokens)
}

/// Two-axis representation of CCOS Q-Page evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeliefField {
    pub belief: f64,
    pub conflict: f64,
}

#[must_use]
pub fn reference_qpage(support: f64, contradiction: f64) -> BeliefField {
    let denominator = support + contradiction + 1.0;
    BeliefField {
        belief: (support - contradiction) / denominator,
        conflict: 2.0 * (support * contradiction).sqrt() / denominator,
    }
}

#[must_use]
pub fn field_qpage(support: f64, contradiction: f64) -> BeliefField {
    let total_field = support + contradiction + 1.0;
    let longitudinal = support - contradiction;
    let transverse = 2.0 * (support * contradiction).sqrt();
    BeliefField {
        belief: longitudinal / total_field,
        conflict: transverse / total_field,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> CcosNode {
        CcosNode {
            id: id.to_owned(),
            base_importance: 0.5,
            failure_relevance: 0.2,
            recency: 0.7,
            access_count: 3,
            trust: 1.0,
            lifecycle: Lifecycle::Stable,
            content: "abcdefgh".to_owned(),
        }
    }

    #[test]
    fn scalar_sources_match_reference_score() {
        let n = node("n");
        let weights = ScoringWeights::default();
        let reference = reference_score(&n, 2, weights);
        let field = field_sources(&n, 2, weights).activation();
        assert!((reference - field).abs() <= f64::EPSILON);
    }

    #[test]
    fn qpage_mapping_is_exact() {
        assert_eq!(reference_qpage(2.0, 1.0), field_qpage(2.0, 1.0));
    }

    #[test]
    fn propagation_implementations_match() {
        let mut a = vec![node("a"), node("b"), node("c")];
        let mut b = a.clone();
        a[0].failure_relevance = 0.95;
        b[0].failure_relevance = 0.95;
        let edges = vec![
            CausalEdge {
                source: 0,
                target: 1,
                weight: 0.8,
            },
            CausalEdge {
                source: 1,
                target: 2,
                weight: 0.7,
            },
        ];
        reference_propagate_failure(&mut a, &edges, 0, 3, 0.1, ScoringWeights::default());
        field_propagate_failure(&mut b, &edges, 0, 3, 0.1, ScoringWeights::default());
        assert_eq!(a, b);
        assert!((a[1].recency - 1.0).abs() <= f64::EPSILON);
        assert!((a[2].recency - 1.0).abs() <= f64::EPSILON);
    }
}
