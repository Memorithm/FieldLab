#![forbid(unsafe_code)]

use field_ccos_map::{
    field_working_set, reference_working_set, CcosNode, Lifecycle, ScoringWeights,
    PINNED_CCOS_COMMIT,
};
use field_hysteresis::{Relay, RelayState};
use std::error::Error;

const NODE_COUNT: usize = 8;
const RELEVANT_COUNT: usize = 3;
const TRACE_LEN: usize = 64;
const BUDGET_TOKENS: usize = 48;
const ITEM_CHARS: usize = 64;
const GAIN: f64 = 0.12;
const THRESHOLDS: [f64; 5] = [0.05, 0.10, 0.15, 0.20, 0.25];
const IDS: [&str; NODE_COUNT] = [
    "file:a.rs",
    "file:b.rs",
    "file:c.rs",
    "file:d.rs",
    "file:e.rs",
    "file:f.rs",
    "file:g.rs",
    "file:h.rs",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraceKind {
    Calibration,
    Holdout,
}

#[derive(Clone, Debug, PartialEq)]
struct Observation {
    evidence: [f64; NODE_COUNT],
    truth: [bool; NODE_COUNT],
    transition: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Metrics {
    misses: usize,
    exact_sets: usize,
    replacements: usize,
    max_transition_latency: usize,
    max_tokens: usize,
    total_tokens: usize,
    cardinality_ok: bool,
    budget_ok: bool,
}

impl Metrics {
    fn exact_accuracy(self) -> f64 {
        usize_to_f64(self.exact_sets) / usize_to_f64(TRACE_LEN)
    }

    const fn protocol_valid(self) -> bool {
        self.cardinality_ok && self.budget_ok
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CalibrationResult {
    threshold: f64,
    metrics: Metrics,
}

#[derive(Clone, Debug, PartialEq)]
struct Campaign {
    baseline_calibration: Metrics,
    calibration_grid: Vec<CalibrationResult>,
    chosen_threshold: f64,
    baseline_holdout: Metrics,
    hysteretic_holdout: Metrics,
}

impl Campaign {
    fn protocol_valid(&self, replay_equal: bool) -> bool {
        replay_equal
            && self.baseline_calibration.protocol_valid()
            && self
                .calibration_grid
                .iter()
                .all(|result| result.metrics.protocol_valid())
            && self.baseline_holdout.protocol_valid()
            && self.hysteretic_holdout.protocol_valid()
            && self.chosen_threshold.is_finite()
            && THRESHOLDS
                .iter()
                .any(|candidate| candidate.to_bits() == self.chosen_threshold.to_bits())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run_campaign()?;
    let second = run_campaign()?;
    let replay_equal = first == second;
    let protocol_valid = first.protocol_valid(replay_equal);
    let hypotheses = hypotheses(&first);
    let fingerprint = fnv1a64(manifest().as_bytes());
    print_report(
        &first,
        replay_equal,
        protocol_valid,
        hypotheses,
        fingerprint,
    );
    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn run_campaign() -> Result<Campaign, Box<dyn Error>> {
    let calibration = build_trace(TraceKind::Calibration);
    let holdout = build_trace(TraceKind::Holdout);
    let baseline_calibration = evaluate_baseline(&calibration);

    let mut calibration_grid = Vec::with_capacity(THRESHOLDS.len());
    for threshold in THRESHOLDS {
        calibration_grid.push(CalibrationResult {
            threshold,
            metrics: evaluate_hysteretic(&calibration, threshold)?,
        });
    }
    let chosen_threshold = select_threshold(&calibration_grid);

    Ok(Campaign {
        baseline_calibration,
        calibration_grid,
        chosen_threshold,
        baseline_holdout: evaluate_baseline(&holdout),
        hysteretic_holdout: evaluate_hysteretic(&holdout, chosen_threshold)?,
    })
}

fn hypotheses(campaign: &Campaign) -> [bool; 5] {
    let baseline = campaign.baseline_holdout;
    let hysteretic = campaign.hysteretic_holdout;
    let quality = hysteretic.misses < baseline.misses;
    let anti_thrash = hysteretic.replacements < baseline.replacements;
    let switching_cost =
        hysteretic.max_transition_latency <= baseline.max_transition_latency.saturating_add(1);
    let budget = baseline.budget_ok && hysteretic.budget_ok;
    let transfer = quality && anti_thrash;
    [quality, anti_thrash, switching_cost, budget, transfer]
}

fn select_threshold(grid: &[CalibrationResult]) -> f64 {
    grid.iter()
        .min_by(|left, right| {
            (left.metrics.misses, left.metrics.replacements)
                .cmp(&(right.metrics.misses, right.metrics.replacements))
                .then_with(|| left.threshold.total_cmp(&right.threshold))
        })
        .map_or(THRESHOLDS[0], |result| result.threshold)
}

fn evaluate_baseline(trace: &[Observation]) -> Metrics {
    evaluate_trace(trace, |observation, _| {
        let nodes = nodes_from_scores(observation.evidence);
        reference_working_set(&nodes, &[], BUDGET_TOKENS, score_weights())
    })
}

fn evaluate_hysteretic(trace: &[Observation], threshold: f64) -> Result<Metrics, Box<dyn Error>> {
    let first = trace.first().ok_or("FL-4B trace must not be empty")?;
    let mut relays = first
        .evidence
        .iter()
        .map(|score| {
            let initial = if *score >= 0.5 {
                RelayState::Positive
            } else {
                RelayState::Negative
            };
            Relay::symmetric(threshold, initial)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(evaluate_trace(trace, |observation, _| {
        let mut adjusted = observation.evidence;
        for (index, relay) in relays.iter_mut().enumerate() {
            let state = relay
                .update(observation.evidence[index] - 0.5)
                .expect("finite preregistered FL-4B evidence");
            adjusted[index] = (observation.evidence[index] + GAIN * state.signed()).clamp(0.0, 1.0);
        }
        let nodes = nodes_from_scores(adjusted);
        field_working_set(&nodes, &[], BUDGET_TOKENS, score_weights())
    }))
}

fn evaluate_trace<F>(trace: &[Observation], mut select: F) -> Metrics
where
    F: FnMut(&Observation, usize) -> (Vec<String>, usize),
{
    let mut metrics = Metrics {
        cardinality_ok: true,
        budget_ok: true,
        ..Metrics::default()
    };
    let mut previous: Option<Vec<String>> = None;
    let mut pending_transition: Option<usize> = None;

    for (time, observation) in trace.iter().enumerate() {
        let (selected, tokens) = select(observation, time);
        let misses = missed_relevant(&selected, &observation.truth);
        let exact = misses == 0 && selected.len() == RELEVANT_COUNT;
        metrics.misses += misses;
        metrics.exact_sets += usize::from(exact);
        metrics.max_tokens = metrics.max_tokens.max(tokens);
        metrics.total_tokens += tokens;
        metrics.cardinality_ok &= selected.len() == RELEVANT_COUNT;
        metrics.budget_ok &= tokens <= BUDGET_TOKENS;

        if let Some(prior) = &previous {
            metrics.replacements += prior.iter().filter(|id| !selected.contains(id)).count();
        }

        update_transition_latency(
            &mut metrics,
            &mut pending_transition,
            observation.transition,
            exact,
            time,
        );
        previous = Some(selected);
    }

    if let Some(start) = pending_transition {
        metrics.max_transition_latency = metrics
            .max_transition_latency
            .max(trace.len().saturating_sub(start));
    }
    metrics
}

fn update_transition_latency(
    metrics: &mut Metrics,
    pending: &mut Option<usize>,
    transition: bool,
    exact: bool,
    time: usize,
) {
    if transition {
        *pending = Some(time);
    }
    if exact {
        if let Some(start) = pending.take() {
            metrics.max_transition_latency = metrics
                .max_transition_latency
                .max(time.saturating_sub(start));
        }
    }
}

fn missed_relevant(selected: &[String], truth: &[bool; NODE_COUNT]) -> usize {
    truth
        .iter()
        .enumerate()
        .filter(|(index, relevant)| **relevant && !selected.iter().any(|id| id == IDS[*index]))
        .count()
}

fn build_trace(kind: TraceKind) -> Vec<Observation> {
    (0..TRACE_LEN)
        .map(|time| build_observation(kind, time))
        .collect()
}

fn build_observation(kind: TraceKind, time: usize) -> Observation {
    let truth = truth_at(kind, time);
    let transition = transition_at(kind, time);
    let mut evidence = std::array::from_fn(|index| if truth[index] { 0.80 } else { 0.20 });

    if transition {
        let previous = truth_at(kind, time.saturating_sub(1));
        let (leaving, entering) = transition_levels(kind);
        for index in 0..NODE_COUNT {
            evidence[index] = match (previous[index], truth[index]) {
                (true, false) => leaving,
                (false, true) => entering,
                (true, true) => 0.80,
                (false, false) => 0.20,
            };
        }
    }

    if let Some((relevant, irrelevant)) = disturbance_pair(kind, time) {
        let (low, high) = disturbance_levels(kind);
        debug_assert!(truth[relevant]);
        debug_assert!(!truth[irrelevant]);
        evidence[relevant] = low;
        evidence[irrelevant] = high;
    }

    Observation {
        evidence,
        truth,
        transition,
    }
}

fn truth_at(kind: TraceKind, time: usize) -> [bool; NODE_COUNT] {
    let relevant: [usize; RELEVANT_COUNT] = match kind {
        TraceKind::Calibration => match time {
            0..=15 => [0, 1, 2],
            16..=31 => [3, 4, 5],
            32..=47 => [1, 5, 6],
            _ => [0, 6, 7],
        },
        TraceKind::Holdout => match time {
            0..=13 => [0, 3, 7],
            14..=29 => [1, 2, 6],
            30..=46 => [0, 4, 5],
            _ => [2, 5, 7],
        },
    };
    std::array::from_fn(|index| relevant.contains(&index))
}

const fn transition_at(kind: TraceKind, time: usize) -> bool {
    match kind {
        TraceKind::Calibration => matches!(time, 16 | 32 | 48),
        TraceKind::Holdout => matches!(time, 14 | 30 | 47),
    }
}

const fn transition_levels(kind: TraceKind) -> (f64, f64) {
    match kind {
        TraceKind::Calibration => (0.35, 0.65),
        TraceKind::Holdout => (0.34, 0.66),
    }
}

const fn disturbance_levels(kind: TraceKind) -> (f64, f64) {
    match kind {
        TraceKind::Calibration => (0.42, 0.58),
        TraceKind::Holdout => (0.41, 0.59),
    }
}

const fn disturbance_pair(kind: TraceKind, time: usize) -> Option<(usize, usize)> {
    match kind {
        TraceKind::Calibration => match time {
            4 => Some((0, 3)),
            8 => Some((1, 4)),
            12 => Some((2, 5)),
            20 => Some((3, 0)),
            24 => Some((4, 1)),
            28 => Some((5, 2)),
            36 => Some((1, 0)),
            40 => Some((5, 2)),
            44 => Some((6, 3)),
            52 => Some((0, 1)),
            56 => Some((6, 2)),
            60 => Some((7, 3)),
            _ => None,
        },
        TraceKind::Holdout => match time {
            3 => Some((0, 1)),
            7 => Some((3, 2)),
            11 => Some((7, 4)),
            18 => Some((1, 0)),
            22 => Some((2, 3)),
            26 => Some((6, 4)),
            34 => Some((0, 1)),
            38 => Some((4, 2)),
            42 => Some((5, 3)),
            51 => Some((2, 0)),
            55 => Some((5, 1)),
            59 => Some((7, 3)),
            _ => None,
        },
    }
}

fn nodes_from_scores(scores: [f64; NODE_COUNT]) -> Vec<CcosNode> {
    scores
        .into_iter()
        .enumerate()
        .map(|(index, score)| CcosNode {
            id: IDS[index].to_owned(),
            base_importance: score,
            failure_relevance: 0.0,
            recency: 0.0,
            access_count: 1,
            trust: 1.0,
            lifecycle: Lifecycle::Stable,
            content: char::from(b'A' + u8::try_from(index).expect("FL-4B index fits u8"))
                .to_string()
                .repeat(ITEM_CHARS),
        })
        .collect()
}

fn score_weights() -> ScoringWeights {
    ScoringWeights {
        w_base: 1.0,
        w_failure: 0.0,
        w_recency: 0.0,
        w_access: 0.0,
        w_centrality: 0.0,
        w_trust: 0.0,
        ..ScoringWeights::default()
    }
}

fn print_report(
    campaign: &Campaign,
    replay_equal: bool,
    protocol_valid: bool,
    hypotheses: [bool; 5],
    fingerprint: u64,
) {
    println!("{{");
    println!("  \"experiment\": \"FL-4B\",");
    println!("  \"protocol\": \"hysteretic-bounded-working-set-v1\",");
    println!("  \"pinned_ccos_commit\": \"{PINNED_CCOS_COMMIT}\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    print_metrics("baseline_calibration", campaign.baseline_calibration, true);
    println!("  \"calibration_grid\": [");
    for (index, result) in campaign.calibration_grid.iter().enumerate() {
        let suffix = if index + 1 == campaign.calibration_grid.len() {
            ""
        } else {
            ","
        };
        print!("    {{\"threshold\": {:.2}, ", result.threshold);
        print_metrics_inline(result.metrics);
        println!("}}{suffix}");
    }
    println!("  ],");
    println!("  \"chosen_threshold\": {:.2},", campaign.chosen_threshold);
    print_metrics("baseline_holdout", campaign.baseline_holdout, true);
    print_metrics("hysteretic_holdout", campaign.hysteretic_holdout, true);
    println!("  \"replay_equal\": {replay_equal},");
    println!("  \"hypotheses\": {{");
    println!("    \"H4_B1_holdout_quality\": {},", hypotheses[0]);
    println!("    \"H4_B2_anti_thrash\": {},", hypotheses[1]);
    println!("    \"H4_B3_bounded_switching_cost\": {},", hypotheses[2]);
    println!(
        "    \"H4_B4_exact_budget_preservation\": {},",
        hypotheses[3]
    );
    println!("    \"H4_B5_calibration_transfer\": {}", hypotheses[4]);
    println!("  }},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");
}

fn print_metrics(label: &str, metrics: Metrics, trailing_comma: bool) {
    print!("  \"{label}\": {{");
    print_metrics_inline(metrics);
    let suffix = if trailing_comma { "," } else { "" };
    println!("}}{suffix}");
}

fn print_metrics_inline(metrics: Metrics) {
    print!(
        "\"misses\": {}, \"exact_sets\": {}, \"exact_accuracy\": {:.12}, \"replacements\": {}, \"max_transition_latency\": {}, \"max_tokens\": {}, \"total_tokens\": {}, \"cardinality_ok\": {}, \"budget_ok\": {}",
        metrics.misses,
        metrics.exact_sets,
        metrics.exact_accuracy(),
        metrics.replacements,
        metrics.max_transition_latency,
        metrics.max_tokens,
        metrics.total_tokens,
        metrics.cardinality_ok,
        metrics.budget_ok,
    );
}

fn manifest() -> String {
    "fl4b|ccos=a3c4d7e03744430c74dc337463ff3e944b4933ad|nodes=8|trace=64|budget=48|item_chars=64|gain=0.12|theta=0.05,0.10,0.15,0.20,0.25|cal_regimes=ABC,DEF,BFG,AGH|cal_transitions=16,32,48|cal_pulses=4,8,12,20,24,28,36,40,44,52,56,60|hold_regimes=ADH,BCG,AEF,CFH|hold_transitions=14,30,47|hold_pulses=3,7,11,18,22,26,34,38,42,51,55,59".to_owned()
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("FL-4B bounded count fits u32"))
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
    fn every_observation_has_three_relevant_nodes() {
        for kind in [TraceKind::Calibration, TraceKind::Holdout] {
            for observation in build_trace(kind) {
                assert_eq!(
                    observation.truth.into_iter().filter(|value| *value).count(),
                    RELEVANT_COUNT
                );
            }
        }
    }

    #[test]
    fn pulses_never_overlap_true_transitions() {
        for kind in [TraceKind::Calibration, TraceKind::Holdout] {
            for time in 0..TRACE_LEN {
                if disturbance_pair(kind, time).is_some() {
                    assert!(!transition_at(kind, time));
                }
            }
        }
    }

    #[test]
    fn campaign_replays_exactly() {
        assert_eq!(run_campaign().unwrap(), run_campaign().unwrap());
    }
}
