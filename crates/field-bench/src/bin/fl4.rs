#![forbid(unsafe_code)]

use field_ccos_map::{
    field_propagate_failure, field_qpage, field_sources, field_working_set, in_degrees,
    reference_propagate_failure, reference_qpage, reference_score, reference_working_set, CausalEdge,
    CcosNode, Lifecycle, ScoringWeights, PINNED_CCOS_COMMIT,
};

const TOLERANCE: f64 = 1.0e-12;
const PAGING_FLOOR: f64 = 0.10;
const MAX_DEPTH: u32 = 3;
const BUDGET_TOKENS: usize = 104;

#[derive(Clone, Debug, PartialEq)]
struct Report {
    max_default_score_error: f64,
    max_extended_score_error: f64,
    max_pressure_error: f64,
    working_set_equal: bool,
    working_set_tokens_equal: bool,
    reference_selection: Vec<String>,
    field_selection: Vec<String>,
    reference_tokens: usize,
    field_tokens: usize,
    max_qpage_error: f64,
}

impl Report {
    fn valid(&self) -> bool {
        self.max_default_score_error <= TOLERANCE
            && self.max_extended_score_error <= TOLERANCE
            && self.max_pressure_error <= TOLERANCE
            && self.working_set_equal
            && self.working_set_tokens_equal
            && self.max_qpage_error <= TOLERANCE
    }
}

fn main() {
    let first = run_campaign();
    let second = run_campaign();
    let replay_equal = first == second;
    let protocol_valid = first.valid() && replay_equal;
    let fingerprint = fnv1a64(manifest().as_bytes());

    println!("{{");
    println!("  \"experiment\": \"FL-4A\",");
    println!("  \"protocol\": \"ccos-semantic-field-mapping-v1\",");
    println!("  \"pinned_ccos_commit\": \"{PINNED_CCOS_COMMIT}\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    println!(
        "  \"max_default_score_error\": {:.17},",
        first.max_default_score_error
    );
    println!(
        "  \"max_extended_score_error\": {:.17},",
        first.max_extended_score_error
    );
    println!("  \"max_pressure_error\": {:.17},", first.max_pressure_error);
    println!("  \"working_set_equal\": {},", first.working_set_equal);
    println!(
        "  \"working_set_tokens_equal\": {},",
        first.working_set_tokens_equal
    );
    println!(
        "  \"reference_selection\": {},",
        json_strings(&first.reference_selection)
    );
    println!(
        "  \"field_selection\": {},",
        json_strings(&first.field_selection)
    );
    println!("  \"reference_tokens\": {},", first.reference_tokens);
    println!("  \"field_tokens\": {},", first.field_tokens);
    println!("  \"max_qpage_error\": {:.17},", first.max_qpage_error);
    println!("  \"replay_equal\": {replay_equal},");
    println!("  \"hypotheses\": {{");
    println!(
        "    \"H4_A1_score_equivalence\": {},",
        first.max_default_score_error <= TOLERANCE
            && first.max_extended_score_error <= TOLERANCE
    );
    println!(
        "    \"H4_A2_propagation_equivalence\": {},",
        first.max_pressure_error <= TOLERANCE
    );
    println!(
        "    \"H4_A3_working_set_equivalence\": {},",
        first.working_set_equal && first.working_set_tokens_equal
    );
    println!(
        "    \"H4_A4_qpage_equivalence\": {},",
        first.max_qpage_error <= TOLERANCE
    );
    println!("    \"H4_A5_replay\": {replay_equal}");
    println!("  }},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");

    if !protocol_valid {
        std::process::exit(1);
    }
}

fn run_campaign() -> Report {
    let (mut reference_nodes, edges) = fixture();
    let mut field_nodes = reference_nodes.clone();
    let degrees = in_degrees(reference_nodes.len(), &edges);
    let default_weights = ScoringWeights::default();
    let extended_weights = ScoringWeights {
        w_centrality: 0.12,
        w_trust: 0.25,
        ..default_weights
    };

    let max_default_score_error = score_error(&reference_nodes, &degrees, default_weights);
    let max_extended_score_error = score_error(&reference_nodes, &degrees, extended_weights);

    reference_propagate_failure(
        &mut reference_nodes,
        &edges,
        0,
        MAX_DEPTH,
        PAGING_FLOOR,
        default_weights,
    );
    field_propagate_failure(
        &mut field_nodes,
        &edges,
        0,
        MAX_DEPTH,
        PAGING_FLOOR,
        default_weights,
    );
    let max_pressure_error = reference_nodes
        .iter()
        .zip(&field_nodes)
        .map(|(reference, field)| {
            (reference.failure_relevance - field.failure_relevance).abs()
        })
        .fold(0.0_f64, f64::max);

    let (reference_selection, reference_tokens) =
        reference_working_set(&reference_nodes, &edges, BUDGET_TOKENS, extended_weights);
    let (field_selection, field_tokens) =
        field_working_set(&field_nodes, &edges, BUDGET_TOKENS, extended_weights);

    Report {
        max_default_score_error,
        max_extended_score_error,
        max_pressure_error,
        working_set_equal: reference_selection == field_selection,
        working_set_tokens_equal: reference_tokens == field_tokens,
        reference_selection,
        field_selection,
        reference_tokens,
        field_tokens,
        max_qpage_error: qpage_error(),
    }
}

fn score_error(nodes: &[CcosNode], degrees: &[u32], weights: ScoringWeights) -> f64 {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let reference = reference_score(node, degrees[index], weights);
            let field = field_sources(node, degrees[index], weights).activation();
            (reference - field).abs()
        })
        .fold(0.0_f64, f64::max)
}

fn qpage_error() -> f64 {
    [
        (0.0, 0.0),
        (1.0, 0.0),
        (0.0, 1.0),
        (1.0, 1.0),
        (4.0, 4.0),
        (4.0, 1.0),
        (1.0, 4.0),
        (0.2, 2.5),
    ]
    .into_iter()
    .map(|(support, contradiction)| {
        let reference = reference_qpage(support, contradiction);
        let field = field_qpage(support, contradiction);
        (reference.belief - field.belief)
            .abs()
            .max((reference.conflict - field.conflict).abs())
    })
    .fold(0.0_f64, f64::max)
}

fn fixture() -> (Vec<CcosNode>, Vec<CausalEdge>) {
    let mut nodes = vec![
        node("file:failure.rs", 0.90, 0.95, 1.00, 8, 1.00, Lifecycle::Working, 'A', 144),
        node("file:db.rs", 0.80, 0.05, 0.90, 12, 1.00, Lifecycle::Stable, 'B', 120),
        node("file:api.rs", 0.65, 0.00, 0.70, 3, 0.90, Lifecycle::Stable, 'C', 96),
        node("file:cache.rs", 0.55, 0.10, 0.60, 2, 0.75, Lifecycle::Stable, 'D', 112),
        node("file:old.rs", 0.30, 0.00, 0.10, 1, 1.00, Lifecycle::Orphan, 'E', 80),
        node("file:worker.rs", 0.70, 0.02, 0.85, 20, 0.60, Lifecycle::Stable, 'F', 128),
        node("file:parser.rs", 0.75, 0.00, 0.50, 5, 0.25, Lifecycle::Stable, 'G', 88),
        node("dep:std", 0.95, 0.00, 1.00, 30, 1.00, Lifecycle::Stable, 'H', 64),
        node("file:model.rs", 0.60, 0.15, 0.40, 7, 0.85, Lifecycle::Stable, 'I', 136),
        node("file:root.rs", 0.85, 0.00, 0.75, 10, 1.00, Lifecycle::Stable, 'J', 104),
    ];
    nodes[0].failure_relevance = 0.95;
    let edges = vec![
        edge(0, 1, 0.90),
        edge(0, 2, 0.80),
        edge(0, 3, 0.70),
        edge(0, 4, 0.60),
        edge(0, 5, 0.50),
        edge(0, 6, 0.40),
        edge(0, 7, 0.30),
        edge(0, 8, 0.20),
        edge(1, 9, 0.85),
        edge(2, 9, 0.65),
        edge(9, 4, 0.50),
        edge(5, 9, 0.45),
    ];
    (nodes, edges)
}

fn node(
    id: &str,
    base_importance: f64,
    failure_relevance: f64,
    recency: f64,
    access_count: u64,
    trust: f64,
    lifecycle: Lifecycle,
    fill: char,
    chars: usize,
) -> CcosNode {
    CcosNode {
        id: id.to_owned(),
        base_importance,
        failure_relevance,
        recency,
        access_count,
        trust,
        lifecycle,
        content: fill.to_string().repeat(chars),
    }
}

const fn edge(source: usize, target: usize, weight: f64) -> CausalEdge {
    CausalEdge {
        source,
        target,
        weight,
    }
}

fn manifest() -> String {
    format!(
        "fl4a|ccos={PINNED_CCOS_COMMIT}|nodes=10|edges=12|origin=0|origin_pressure=0.95|max_depth={MAX_DEPTH}|paging_floor={PAGING_FLOOR:.17}|budget={BUDGET_TOKENS}|extended_centrality=0.12|extended_trust=0.25|qpage_pairs=8"
    )
}

fn json_strings(values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{body}]")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_exercises_fanout_damping() {
        let (_, edges) = fixture();
        assert_eq!(edges.iter().filter(|edge| edge.source == 0).count(), 8);
    }

    #[test]
    fn campaign_is_replay_stable() {
        assert_eq!(run_campaign(), run_campaign());
    }
}
