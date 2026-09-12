#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;

const PINNED_CCOS_COMMIT: &str = "a3c4d7e03744430c74dc337463ff3e944b4933ad";
const EXPECTED_BUDGET: usize = 2048;
const EXPECTED_DEPTH: usize = 3;
const TRACE_LEN: usize = 24;
const CALIBRATION: [&str; TRACE_LEN] = [
    "A", "A", "A", "B", "A", "A", "A", "A", "B", "B", "B", "C", "B", "B", "B",
    "B", "C", "C", "C", "D", "C", "C", "C", "C",
];
const HOLDOUT: [&str; TRACE_LEN] = [
    "C", "C", "A", "C", "C", "C", "C", "C", "A", "A", "D", "A", "A", "A", "A",
    "A", "D", "D", "B", "D", "D", "D", "D", "D",
];

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TraceReport {
    experiment: String,
    protocol: String,
    ccos_commit: String,
    source_path: String,
    files: usize,
    budget: usize,
    depth: usize,
    anchors: BTreeMap<String, String>,
    calibration: Vec<Observation>,
    holdout: Vec<Observation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Observation {
    step: usize,
    stimulus: String,
    anchor: String,
    affected: i64,
    tokens: i64,
    items: Vec<TraceItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TraceItem {
    uri: String,
    score: f64,
    kind: String,
}

#[derive(Debug, Serialize)]
struct ResultReport {
    experiment: &'static str,
    protocol: &'static str,
    ccos_commit: String,
    trace_a_sha256: String,
    trace_b_sha256: String,
    source_path: String,
    files: usize,
    budget: usize,
    depth: usize,
    calibration_observations: usize,
    holdout_observations: usize,
    calibration_distinct_snapshots: usize,
    holdout_distinct_snapshots: usize,
    hypotheses: BTreeMap<&'static str, bool>,
    replay_equal: bool,
    protocol_valid: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (first_path, second_path, external_commit) = arguments()?;
    let first_bytes = fs::read(&first_path)?;
    let second_bytes = fs::read(&second_path)?;
    let first: TraceReport = serde_json::from_slice(&first_bytes)?;
    let second: TraceReport = serde_json::from_slice(&second_bytes)?;
    let replay_equal = first == second;
    let protocol_valid = validate_protocol(&first, &external_commit, replay_equal);
    let hypotheses = evaluate_hypotheses(&first, replay_equal);
    let output = ResultReport {
        experiment: "FL-4D",
        protocol: "native-temporal-ccos-trace-v1",
        ccos_commit: external_commit,
        trace_a_sha256: sha256_hex(&first_bytes),
        trace_b_sha256: sha256_hex(&second_bytes),
        source_path: first.source_path.clone(),
        files: first.files,
        budget: first.budget,
        depth: first.depth,
        calibration_observations: first.calibration.len(),
        holdout_observations: first.holdout.len(),
        calibration_distinct_snapshots: distinct_snapshots(&first.calibration),
        holdout_distinct_snapshots: distinct_snapshots(&first.holdout),
        hypotheses,
        replay_equal,
        protocol_valid,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn arguments() -> Result<(String, String, String), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let first = args.next().ok_or("missing first FL-4D trace path")?;
    let second = args.next().ok_or("missing second FL-4D trace path")?;
    let commit = args.next().ok_or("missing CCOS commit")?;
    if args.next().is_some() {
        return Err("unexpected extra FL-4D arguments".into());
    }
    Ok((first, second, commit))
}

fn validate_protocol(report: &TraceReport, external_commit: &str, replay_equal: bool) -> bool {
    external_commit == PINNED_CCOS_COMMIT
        && report.ccos_commit == PINNED_CCOS_COMMIT
        && report.experiment == "FL-4D"
        && report.protocol == "native-temporal-ccos-trace-v1"
        && replay_equal
        && report.files > 0
        && report.budget == EXPECTED_BUDGET
        && report.depth == EXPECTED_DEPTH
        && anchors_valid(&report.anchors)
        && trace_valid(&report.calibration, &CALIBRATION, &report.anchors)
        && trace_valid(&report.holdout, &HOLDOUT, &report.anchors)
}

fn anchors_valid(anchors: &BTreeMap<String, String>) -> bool {
    anchors
        == &BTreeMap::from([
            ("A".to_owned(), "src/external_memory.rs".to_owned()),
            ("B".to_owned(), "src/agent_session.rs".to_owned()),
            ("C".to_owned(), "src/migrate.rs".to_owned()),
            ("D".to_owned(), "src/region_metrics.rs".to_owned()),
        ])
}

fn trace_valid(
    trace: &[Observation],
    schedule: &[&str; TRACE_LEN],
    anchors: &BTreeMap<String, String>,
) -> bool {
    trace.len() == TRACE_LEN
        && trace.iter().enumerate().all(|(step, observation)| {
            let expected_stimulus = schedule[step];
            let expected_anchor = anchors.get(expected_stimulus);
            observation.step == step
                && observation.stimulus == expected_stimulus
                && expected_anchor.is_some_and(|anchor| observation.anchor == *anchor)
                && observation.affected >= 0
                && (0..=i64::try_from(EXPECTED_BUDGET).expect("budget fits i64"))
                    .contains(&observation.tokens)
                && observation.items.iter().all(|item| {
                    !item.uri.is_empty() && !item.kind.is_empty() && item.score.is_finite()
                })
        })
}

fn evaluate_hypotheses(
    report: &TraceReport,
    replay_equal: bool,
) -> BTreeMap<&'static str, bool> {
    let all_observations = report.calibration.iter().chain(&report.holdout);
    let hard_budget = all_observations
        .clone()
        .all(|observation| observation.tokens >= 0 && observation.tokens as usize <= EXPECTED_BUDGET);
    let scored_evidence = all_observations.clone().all(|observation| {
        observation
            .items
            .iter()
            .all(|item| item.score.is_finite() && !item.uri.is_empty() && !item.kind.is_empty())
    });
    let stimulus_sensitivity = has_stimulus_sensitivity(&report.calibration)
        || has_stimulus_sensitivity(&report.holdout);
    let anchor_diversity = report.anchors.values().all(|anchor| {
        all_observations
            .clone()
            .any(|observation| observation.items.iter().any(|item| item.uri.contains(anchor)))
    });
    BTreeMap::from([
        ("H4_D1_deterministic_native_temporal_replay", replay_equal),
        ("H4_D2_stimulus_sensitivity", stimulus_sensitivity),
        ("H4_D3_hard_budget_preservation", hard_budget),
        ("H4_D4_scored_evidence_availability", scored_evidence),
        ("H4_D5_anchor_diversity", anchor_diversity),
    ])
}

fn has_stimulus_sensitivity(trace: &[Observation]) -> bool {
    trace.windows(2).any(|pair| {
        pair[0].stimulus != pair[1].stimulus && snapshot_signature(&pair[0]) != snapshot_signature(&pair[1])
    })
}

fn distinct_snapshots(trace: &[Observation]) -> usize {
    trace
        .iter()
        .map(snapshot_signature)
        .collect::<BTreeSet<_>>()
        .len()
}

fn snapshot_signature(observation: &Observation) -> String {
    let mut signature = format!("{}|", observation.tokens);
    for item in &observation.items {
        signature.push_str(&item.uri);
        signature.push('@');
        signature.push_str(&item.score.to_bits().to_string());
        signature.push('|');
    }
    signature
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
