#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

const PINNED_CCOS_COMMIT: &str = "a3c4d7e03744430c74dc337463ff3e944b4933ad";
const EXPECTED_BUDGET: usize = 2048;
const EXPECTED_DEPTH: usize = 3;
const TRACE_LEN: usize = 32;
const THRESHOLDS: [f64; 6] = [0.00, 0.01, 0.02, 0.04, 0.08, 0.16];
const ANCHOR_FILE_URIS: [(&str, &str); 4] = [
    ("A", "file:src/external_memory.rs"),
    ("B", "file:src/agent_session.rs"),
    ("C", "file:src/migrate.rs"),
    ("D", "file:src/region_metrics.rs"),
];
const CALIBRATION_STIMULUS: [&str; TRACE_LEN] = [
    "A", "A", "C", "A", "A", "B", "A", "A", "C", "D", "C", "C", "A", "C", "C", "C",
    "B", "B", "A", "B", "D", "B", "B", "B", "D", "C", "D", "D", "D", "A", "D", "D",
];
const CALIBRATION_TRUTH: [&str; TRACE_LEN] = [
    "A", "A", "A", "A", "A", "A", "A", "A", "C", "C", "C", "C", "C", "C", "C", "C",
    "B", "B", "B", "B", "B", "B", "B", "B", "D", "D", "D", "D", "D", "D", "D", "D",
];
const HOLDOUT_STIMULUS: [&str; TRACE_LEN] = [
    "B", "D", "B", "B", "B", "A", "B", "B", "D", "D", "C", "D", "B", "D", "D", "D",
    "A", "C", "A", "A", "D", "A", "A", "A", "C", "B", "C", "D", "C", "C", "C", "C",
];
const HOLDOUT_TRUTH: [&str; TRACE_LEN] = [
    "B", "B", "B", "B", "B", "B", "B", "B", "D", "D", "D", "D", "D", "D", "D", "D",
    "A", "A", "A", "A", "A", "A", "A", "A", "C", "C", "C", "C", "C", "C", "C", "C",
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
    truth: String,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
struct FocusMetrics {
    errors: usize,
    unresolved: usize,
    false_switches: usize,
    switches: usize,
    max_transition_latency: usize,
    resolved_focus_present: usize,
    max_tokens: usize,
}

impl FocusMetrics {
    const fn resolved(self) -> usize {
        TRACE_LEN - self.unresolved
    }

    const fn observability_valid(self) -> bool {
        self.resolved_focus_present == self.resolved()
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
struct CalibrationResult {
    threshold: f64,
    metrics: FocusMetrics,
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
    baseline_calibration: FocusMetrics,
    calibration_grid: Vec<CalibrationResult>,
    chosen_threshold: f64,
    baseline_holdout: FocusMetrics,
    hysteretic_holdout: FocusMetrics,
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

    let baseline_calibration = evaluate_memoryless(&first.calibration);
    let mut calibration_grid = Vec::with_capacity(THRESHOLDS.len());
    for threshold in THRESHOLDS {
        calibration_grid.push(CalibrationResult {
            threshold,
            metrics: evaluate_hysteretic(&first.calibration, threshold),
        });
    }
    let chosen_threshold = select_threshold(&calibration_grid);
    let baseline_holdout = evaluate_memoryless(&first.holdout);
    let hysteretic_holdout = evaluate_hysteretic(&first.holdout, chosen_threshold);
    let protocol_valid = validate_protocol(
        &first,
        &external_commit,
        replay_equal,
        &calibration_grid,
        chosen_threshold,
        baseline_calibration,
        baseline_holdout,
        hysteretic_holdout,
    );
    let hypotheses = evaluate_hypotheses(baseline_holdout, hysteretic_holdout);

    let output = ResultReport {
        experiment: "FL-4E",
        protocol: "external-native-window-focus-hysteresis-v1",
        ccos_commit: external_commit,
        trace_a_sha256: sha256_hex(&first_bytes),
        trace_b_sha256: sha256_hex(&second_bytes),
        source_path: first.source_path.clone(),
        files: first.files,
        budget: first.budget,
        depth: first.depth,
        baseline_calibration,
        calibration_grid,
        chosen_threshold,
        baseline_holdout,
        hysteretic_holdout,
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
    let first = args.next().ok_or("missing first FL-4E trace path")?;
    let second = args.next().ok_or("missing second FL-4E trace path")?;
    let commit = args.next().ok_or("missing CCOS commit")?;
    if args.next().is_some() {
        return Err("unexpected extra FL-4E arguments".into());
    }
    Ok((first, second, commit))
}

#[allow(clippy::too_many_arguments)]
fn validate_protocol(
    report: &TraceReport,
    external_commit: &str,
    replay_equal: bool,
    calibration_grid: &[CalibrationResult],
    chosen_threshold: f64,
    baseline_calibration: FocusMetrics,
    baseline_holdout: FocusMetrics,
    hysteretic_holdout: FocusMetrics,
) -> bool {
    external_commit == PINNED_CCOS_COMMIT
        && report.ccos_commit == PINNED_CCOS_COMMIT
        && report.experiment == "FL-4E"
        && report.protocol == "external-native-window-focus-hysteresis-v1"
        && replay_equal
        && report.files > 0
        && report.budget == EXPECTED_BUDGET
        && report.depth == EXPECTED_DEPTH
        && anchors_valid(&report.anchors)
        && trace_valid(
            &report.calibration,
            &CALIBRATION_STIMULUS,
            &CALIBRATION_TRUTH,
            &report.anchors,
        )
        && trace_valid(
            &report.holdout,
            &HOLDOUT_STIMULUS,
            &HOLDOUT_TRUTH,
            &report.anchors,
        )
        && calibration_grid.len() == THRESHOLDS.len()
        && calibration_grid.iter().zip(THRESHOLDS).all(|(result, expected)| {
            result.threshold.to_bits() == expected.to_bits() && result.metrics.observability_valid()
        })
        && THRESHOLDS
            .iter()
            .any(|threshold| threshold.to_bits() == chosen_threshold.to_bits())
        && baseline_calibration.observability_valid()
        && baseline_holdout.observability_valid()
        && hysteretic_holdout.observability_valid()
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
    stimulus: &[&str; TRACE_LEN],
    truth: &[&str; TRACE_LEN],
    anchors: &BTreeMap<String, String>,
) -> bool {
    trace.len() == TRACE_LEN
        && trace.iter().enumerate().all(|(step, observation)| {
            let expected_stimulus = stimulus[step];
            let expected_truth = truth[step];
            observation.step == step
                && observation.stimulus == expected_stimulus
                && observation.truth == expected_truth
                && anchors
                    .get(expected_stimulus)
                    .is_some_and(|anchor| observation.anchor == *anchor)
                && observation.affected >= 0
                && tokens_within_budget(observation.tokens)
                && observation.items.iter().all(|item| {
                    !item.uri.is_empty() && !item.kind.is_empty() && item.score.is_finite()
                })
        })
}

fn evaluate_memoryless(trace: &[Observation]) -> FocusMetrics {
    evaluate_policy(trace, |_| None)
}

fn evaluate_hysteretic(trace: &[Observation], threshold: f64) -> FocusMetrics {
    evaluate_policy(trace, |state| choose_hysteretic(state, threshold))
}

struct PolicyState<'a> {
    observation: &'a Observation,
    previous_focus: Option<&'static str>,
    memoryless_focus: Option<&'static str>,
}

fn evaluate_policy<F>(trace: &[Observation], mut hysteretic_choice: F) -> FocusMetrics
where
    F: FnMut(&PolicyState<'_>) -> Option<&'static str>,
{
    let memoryless = trace
        .iter()
        .map(memoryless_focus)
        .collect::<Vec<Option<&'static str>>>();
    let use_memoryless = std::any::type_name::<F>().contains("evaluate_memoryless");
    let mut metrics = FocusMetrics::default();
    let mut previous_focus: Option<&'static str> = None;
    let mut previous_truth: Option<&str> = None;
    let mut transition_start: Option<usize> = None;

    for (step, observation) in trace.iter().enumerate() {
        let baseline_focus = memoryless[step];
        let state = PolicyState {
            observation,
            previous_focus,
            memoryless_focus: baseline_focus,
        };
        let focus = if use_memoryless {
            baseline_focus
        } else {
            hysteretic_choice(&state)
        };
        update_metrics(
            &mut metrics,
            observation,
            previous_focus,
            previous_truth,
            &mut transition_start,
            focus,
            step,
        );
        previous_focus = focus;
        previous_truth = Some(&observation.truth);
    }
    finish_latency(&mut metrics, transition_start, trace.len());
    metrics
}

fn choose_hysteretic(state: &PolicyState<'_>, threshold: f64) -> Option<&'static str> {
    let candidate = state.memoryless_focus?;
    let Some(previous) = state.previous_focus else {
        return Some(candidate);
    };
    let Some(previous_score) = visible_score(state.observation, previous) else {
        return Some(candidate);
    };
    if previous == candidate {
        return Some(previous);
    }
    let candidate_score = visible_score(state.observation, candidate)
        .expect("memoryless candidate is visible by construction");
    if candidate_score - previous_score >= threshold {
        Some(candidate)
    } else {
        Some(previous)
    }
}

fn memoryless_focus(observation: &Observation) -> Option<&'static str> {
    observation
        .items
        .iter()
        .find_map(|item| symbol_for_uri(&item.uri))
}

fn visible_score(observation: &Observation, symbol: &str) -> Option<f64> {
    let uri = ANCHOR_FILE_URIS
        .iter()
        .find_map(|(candidate, uri)| (*candidate == symbol).then_some(*uri))?;
    observation
        .items
        .iter()
        .find_map(|item| (item.uri == uri).then_some(item.score))
}

fn symbol_for_uri(uri: &str) -> Option<&'static str> {
    ANCHOR_FILE_URIS
        .iter()
        .find_map(|(symbol, candidate_uri)| (*candidate_uri == uri).then_some(*symbol))
}

#[allow(clippy::too_many_arguments)]
fn update_metrics(
    metrics: &mut FocusMetrics,
    observation: &Observation,
    previous_focus: Option<&str>,
    previous_truth: Option<&str>,
    transition_start: &mut Option<usize>,
    focus: Option<&str>,
    step: usize,
) {
    metrics.max_tokens = metrics.max_tokens.max(usize::try_from(observation.tokens).unwrap_or(0));
    match focus {
        Some(symbol) => {
            if symbol != observation.truth {
                metrics.errors += 1;
            }
            if visible_score(observation, symbol).is_some() {
                metrics.resolved_focus_present += 1;
            }
        }
        None => {
            metrics.errors += 1;
            metrics.unresolved += 1;
        }
    }

    if let (Some(previous), Some(current)) = (previous_focus, focus) {
        if previous != current {
            metrics.switches += 1;
            if previous_truth.is_some_and(|truth| truth == observation.truth) {
                metrics.false_switches += 1;
            }
        }
    }

    if previous_truth.is_some_and(|truth| truth != observation.truth) {
        *transition_start = Some(step);
    }
    if focus.is_some_and(|symbol| symbol == observation.truth) {
        if let Some(start) = transition_start.take() {
            metrics.max_transition_latency = metrics.max_transition_latency.max(step - start);
        }
    }
}

fn finish_latency(metrics: &mut FocusMetrics, transition_start: Option<usize>, len: usize) {
    if let Some(start) = transition_start {
        metrics.max_transition_latency = metrics.max_transition_latency.max(len - start);
    }
}

fn select_threshold(grid: &[CalibrationResult]) -> f64 {
    grid.iter()
        .min_by(|left, right| {
            let left_key = (
                left.metrics.errors,
                left.metrics.false_switches,
                left.metrics.switches,
                left.metrics.max_transition_latency,
            );
            let right_key = (
                right.metrics.errors,
                right.metrics.false_switches,
                right.metrics.switches,
                right.metrics.max_transition_latency,
            );
            left_key
                .cmp(&right_key)
                .then_with(|| left.threshold.total_cmp(&right.threshold))
        })
        .map_or(THRESHOLDS[0], |result| result.threshold)
}

fn evaluate_hypotheses(
    baseline: FocusMetrics,
    hysteretic: FocusMetrics,
) -> BTreeMap<&'static str, bool> {
    let truth_tracking = hysteretic.errors < baseline.errors;
    let anti_thrash = hysteretic.false_switches < baseline.false_switches;
    let transition_cost = hysteretic.max_transition_latency
        <= baseline.max_transition_latency.saturating_add(1);
    let observability = baseline.max_tokens <= EXPECTED_BUDGET
        && hysteretic.max_tokens <= EXPECTED_BUDGET
        && hysteretic.observability_valid();
    BTreeMap::from([
        ("H4_E1_holdout_truth_tracking", truth_tracking),
        ("H4_E2_anti_thrash", anti_thrash),
        ("H4_E3_bounded_transition_cost", transition_cost),
        ("H4_E4_exact_budget_observability", observability),
        ("H4_E5_calibration_transfer", truth_tracking && anti_thrash),
    ])
}

fn tokens_within_budget(tokens: i64) -> bool {
    usize::try_from(tokens).is_ok_and(|tokens| tokens <= EXPECTED_BUDGET)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
