#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

const PINNED_CCOS_COMMIT: &str = "a3c4d7e03744430c74dc337463ff3e944b4933ad";
const EXPECTED_BUDGET: usize = 2048;
const EXPECTED_DEPTH: usize = 3;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct ProbeReport {
    crate_src: String,
    files: usize,
    all_src_tokens: usize,
    budget: usize,
    depth: usize,
    duplication_factor: f64,
    anchors: BTreeMap<String, AnchorReport>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct AnchorReport {
    deps: Vec<String>,
    deps_in_window: Vec<String>,
    affected: i64,
    window_tokens: usize,
    pct_all_src: f64,
    noise_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ImportedAnchor {
    anchor: String,
    direct_dependencies: usize,
    covered_dependencies: usize,
    affected: i64,
    window_tokens: usize,
    pct_all_src: f64,
    noise_files: usize,
    full_dependency_coverage: bool,
}

#[derive(Debug, Serialize)]
struct ResultReport {
    experiment: &'static str,
    protocol: &'static str,
    ccos_commit: String,
    probe_a_sha256: String,
    probe_b_sha256: String,
    source_path: String,
    files: usize,
    all_src_tokens: usize,
    budget: usize,
    depth: usize,
    duplication_factor: f64,
    anchors: Vec<ImportedAnchor>,
    hypotheses: BTreeMap<&'static str, bool>,
    replay_equal: bool,
    protocol_valid: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (first_path, second_path, external_commit) = arguments()?;
    let first_bytes = fs::read(&first_path)?;
    let second_bytes = fs::read(&second_path)?;
    let first: ProbeReport = serde_json::from_slice(&first_bytes)?;
    let second: ProbeReport = serde_json::from_slice(&second_bytes)?;
    let replay_equal = first == second;
    let protocol_valid = validate_protocol(&first, &external_commit, replay_equal);
    let anchors = import_anchors(&first);
    let hypotheses = evaluate_hypotheses(&first, &anchors, replay_equal);
    let output = ResultReport {
        experiment: "FL-4C",
        protocol: "external-ccos-runtime-replay-v1",
        ccos_commit: external_commit,
        probe_a_sha256: sha256_hex(&first_bytes),
        probe_b_sha256: sha256_hex(&second_bytes),
        source_path: first.crate_src.clone(),
        files: first.files,
        all_src_tokens: first.all_src_tokens,
        budget: first.budget,
        depth: first.depth,
        duplication_factor: first.duplication_factor,
        anchors,
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
    let first = args.next().ok_or("missing first probe path")?;
    let second = args.next().ok_or("missing second probe path")?;
    let commit = args.next().ok_or("missing CCOS commit")?;
    if args.next().is_some() {
        return Err("unexpected extra FL-4C arguments".into());
    }
    Ok((first, second, commit))
}

fn validate_protocol(report: &ProbeReport, external_commit: &str, replay_equal: bool) -> bool {
    external_commit == PINNED_CCOS_COMMIT
        && replay_equal
        && report.files > 0
        && report.all_src_tokens > 0
        && report.budget == EXPECTED_BUDGET
        && report.depth == EXPECTED_DEPTH
        && !report.anchors.is_empty()
        && report.duplication_factor.is_finite()
        && report.duplication_factor >= 0.0
        && report.anchors.values().all(anchor_protocol_valid)
}

fn anchor_protocol_valid(anchor: &AnchorReport) -> bool {
    anchor.deps.len() >= 2
        && anchor.affected >= 0
        && anchor.window_tokens <= EXPECTED_BUDGET
        && anchor.pct_all_src.is_finite()
        && anchor.pct_all_src >= 0.0
}

fn import_anchors(report: &ProbeReport) -> Vec<ImportedAnchor> {
    report
        .anchors
        .iter()
        .map(|(name, anchor)| {
            let covered = anchor
                .deps
                .iter()
                .filter(|dependency| anchor.deps_in_window.contains(*dependency))
                .count();
            ImportedAnchor {
                anchor: name.clone(),
                direct_dependencies: anchor.deps.len(),
                covered_dependencies: covered,
                affected: anchor.affected,
                window_tokens: anchor.window_tokens,
                pct_all_src: anchor.pct_all_src,
                noise_files: anchor.noise_files.len(),
                full_dependency_coverage: covered == anchor.deps.len(),
            }
        })
        .collect()
}

fn evaluate_hypotheses(
    report: &ProbeReport,
    anchors: &[ImportedAnchor],
    replay_equal: bool,
) -> BTreeMap<&'static str, bool> {
    let full_coverage = anchors.iter().all(|anchor| anchor.full_dependency_coverage);
    let causal_pressure = anchors.iter().all(|anchor| anchor.affected > 1);
    let bounded_recall = anchors
        .iter()
        .all(|anchor| anchor.window_tokens < report.all_src_tokens);
    let low_noise_count = anchors
        .iter()
        .filter(|anchor| anchor.noise_files <= anchor.covered_dependencies)
        .count();
    let low_noise_majority = low_noise_count > anchors.len() / 2;
    BTreeMap::from([
        ("H4_C1_direct_dependency_coverage", full_coverage),
        ("H4_C2_nontrivial_causal_pressure", causal_pressure),
        ("H4_C3_bounded_external_recall", bounded_recall),
        ("H4_C4_low_noise_majority", low_noise_majority),
        ("H4_C5_deterministic_external_replay", replay_equal),
    ])
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
