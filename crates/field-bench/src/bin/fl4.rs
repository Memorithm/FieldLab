#![forbid(unsafe_code)]

use field_ccos_map::{
    field_propagate_failure, field_qpage, field_sources, field_working_set, in_degrees,
    reference_propagate_failure, reference_qpage, reference_score, reference_working_set,
    CausalEdge, CcosNode, Lifecycle, ScoringWeights, PINNED_CCOS_COMMIT,
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

#[derive(Clone, Copy)]
struct NodeSpec {
    id: &'static str,
    base: f64,
    failure: f64,
    recency: f64,
    access: u64,
    trust: f64,
    lifecycle: Lifecycle,
    fill: char,
    chars: usize,
}

impl NodeSpec {
    fn build(self) -> CcosNode {
        CcosNode {
            id: self.id.to_owned(),
            base_importance: self.base,
            failure_relevance: self.failure,
            recency: self.recency,
            access_count: self.access,
            trust: self.trust,
            lifecycle: self.lifecycle,
            content: self.fill.to_string().repeat(self.chars),
        }
    }
}

const fn spec(
    id: &'static str,
    scores: [f64; 4],
    access: u64,
    lifecycle: Lifecycle,
    fill: char,
    chars: usize,
) -> NodeSpec {
    NodeSpec {
        id,
        base: scores[0],
        failure: scores[1],
        recency: scores[2],
        access,
        trust: scores[3],
        lifecycle,
        fill,
        chars,
    }
}

fn main() {
    let first = run_campaign();
    let replay_equal = first == run_campaign();
    let protocol_valid = first.valid() && replay_equal;
    let fingerprint = fnv1a64(manifest().as_bytes());
    print_report(&first, replay_equal, protocol_valid, fingerprint);
    if !protocol_valid {
        std::process::exit(1);
    }
}

fn print_report(report: &Report, replay_equal: bool, protocol_valid: bool, fingerprint: u64) {
    println!("{{");
    println!("  \"experiment\": \"FL-4A\",");
    println!("  \"protocol\": \"ccos-semantic-field-mapping-v1\",");
    println!("  \"pinned_ccos_commit\": \"{PINNED_CCOS_COMMIT}\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    println!(
        "  \"max_default_score_error\": {:.17},",
        report.max_default_score_error
    );
    println!(
        "  \"max_extended_score_error\": {:.17},",
        report.max_extended_score_error
    );
    println!(
        "  \"max_pressure_error\": {:.17},",
        report.max_pressure_error
    );
    println!("  \"working_set_equal\": {},", report.working_set_equal);
    println!(
        "  \"working_set_tokens_equal\": {},",
        report.working_set_tokens_equal
    );
    println!(
        "  \"reference_selection\": {},",
        json_strings(&report.reference_selection)
    );
    println!(
        "  \"field_selection\": {},",
        json_strings(&report.field_selection)
    );
    println!("  \"reference_tokens\": {},", report.reference_tokens);
    println!("  \"field_tokens\": {},", report.field_tokens);
    println!("  \"max_qpage_error\": {:.17},", report.max_qpage_error);
    println!("  \"replay_equal\": {replay_equal},");
    print_hypotheses(report, replay_equal);
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");
}

fn print_hypotheses(report: &Report, replay_equal: bool) {
    println!("  \"hypotheses\": {{");
    println!(
        "    \"H4_A1_score_equivalence\": {},",
        report.max_default_score_error <= TOLERANCE && report.max_extended_score_error <= TOLERANCE
    );
    println!(
        "    \"H4_A2_propagation_equivalence\": {},",
        report.max_pressure_error <= TOLERANCE
    );
    println!(
        "    \"H4_A3_working_set_equivalence\": {},",
        report.working_set_equal && report.working_set_tokens_equal
    );
    println!(
        "    \"H4_A4_qpage_equivalence\": {},",
        report.max_qpage_error <= TOLERANCE
    );
    println!("    \"H4_A5_replay\": {replay_equal}");
    println!("  }},");
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
        .map(|(reference, field)| (reference.failure_relevance - field.failure_relevance).abs())
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

#[rustfmt::skip]
fn fixture() -> (Vec<CcosNode>, Vec<CausalEdge>) {
    let specs = [
        spec("file:failure.rs", [0.90, 0.95, 1.00, 1.00], 8, Lifecycle::Working, 'A', 144),
        spec("file:db.rs", [0.80, 0.05, 0.90, 1.00], 12, Lifecycle::Stable, 'B', 120),
        spec("file:api.rs", [0.65, 0.00, 0.70, 0.90], 3, Lifecycle::Stable, 'C', 96),
        spec("file:cache.rs", [0.55, 0.10, 0.60, 0.75], 2, Lifecycle::Stable, 'D', 112),
        spec("file:old.rs", [0.30, 0.00, 0.10, 1.00], 1, Lifecycle::Orphan, 'E', 80),
        spec("file:worker.rs", [0.70, 0.02, 0.85, 0.60], 20, Lifecycle::Stable, 'F', 128),
        spec("file:parser.rs", [0.75, 0.00, 0.50, 0.25], 5, Lifecycle::Stable, 'G', 88),
        spec("dep:std", [0.95, 0.00, 1.00, 1.00], 30, Lifecycle::Stable, 'H', 64),
        spec("file:model.rs", [0.60, 0.15, 0.40, 0.85], 7, Lifecycle::Stable, 'I', 136),
        spec("file:root.rs", [0.85, 0.00, 0.75, 1.00], 10, Lifecycle::Stable, 'J', 104),
    ];
    let nodes = specs.into_iter().map(NodeSpec::build).collect();
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
