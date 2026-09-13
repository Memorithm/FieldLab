#![forbid(unsafe_code)]

use field_core::{EnergyModel, FieldState};
use field_dynamics::{heun_step, IntegratorConfig};
use field_memory::PatternBank;
use serde::Serialize;
use std::collections::BTreeSet;
use std::error::Error;

const NODE_COUNT: usize = 8;
const MAX_STEPS: usize = 1024;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const CUE_TILT_RADIANS: f64 = 0.15;
const CHECKPOINTS: [usize; 15] = [
    0, 16, 32, 64, 96, 128, 160, 192, 224, 256, 320, 384, 512, 768, 1024,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Check {
    Fail,
    Pass,
}

impl Check {
    const fn from_bool(value: bool) -> Self {
        if value {
            Self::Pass
        } else {
            Self::Fail
        }
    }

    const fn as_bool(self) -> bool {
        matches!(self, Self::Pass)
    }
}

impl Serialize for Check {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(self.as_bool())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaseKind {
    Clean,
    WrongBasin,
}

#[derive(Clone, Copy, Debug)]
struct CaseSpec {
    id: &'static str,
    kind: CaseKind,
    target: usize,
    competitor: Option<usize>,
    cue: [i8; NODE_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Segment {
    label: String,
    start_step: usize,
    end_step: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct CheckpointRecord {
    step: usize,
    decoded_label: String,
    nearest_template: usize,
    top_two_gap: f64,
    template_angles: Vec<f64>,
    energy: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct CaseReport {
    case_id: String,
    kind: String,
    target_index: usize,
    competitor_index: Option<usize>,
    cue: Vec<i8>,
    initial_label: String,
    final_label: String,
    first_clean_exit: Option<usize>,
    first_clean_return: Option<usize>,
    first_competitor_entry: Option<usize>,
    first_competitor_exit: Option<usize>,
    first_target_entry: Option<usize>,
    first_competitor_return: Option<usize>,
    total_label_transitions: usize,
    decoded_segments: Vec<Segment>,
    checkpoints: Vec<CheckpointRecord>,
    final_target_angle: f64,
    final_competitor_angle: Option<f64>,
    unit_norm_valid: bool,
    finite: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Serialize)]
struct Campaign {
    cases: Vec<CaseReport>,
    clean_common_stable_through: usize,
    wrong_common_competitor_through: usize,
    benchmark_common_stable_through: usize,
    largest_stable_checkpoint: usize,
    h5f_0_reference_128: bool,
    h5f_1_clean_permanence: bool,
    h5f_2_wrong_permanence: bool,
    h5f_3_128_256_invariance: bool,
    all_numeric_finite: bool,
    all_unit_norm_valid: bool,
    all_checkpoints_present: bool,
    case_set_exact: bool,
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Serialize)]
struct HypothesisReport {
    h5f_0_reference_128: Check,
    h5f_1_clean_permanence: Check,
    h5f_2_wrong_permanence: Check,
    h5f_3_128_256_invariance: Check,
    h5f_4_deterministic_replay: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ValidityReport {
    frozen_case_set_exact: Check,
    new_case_namespace: Check,
    reference_128_gate: Check,
    checkpoints_complete: Check,
    all_metrics_finite: Check,
    unit_norm_valid: Check,
    energy_evaluation_succeeded: Check,
    replay_exact: Check,
    protocol_valid: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ExperimentArtifact {
    experiment: &'static str,
    protocol: &'static str,
    fieldlab_commit: String,
    frozen_constants: FrozenConstants,
    campaign: Campaign,
    replay_equal: bool,
    hypotheses: HypothesisReport,
    validity: ValidityReport,
}

#[derive(Clone, Debug, Serialize)]
struct FrozenConstants {
    node_count: usize,
    max_steps: usize,
    field_dt: f64,
    field_mobility: f64,
    cue_tilt_radians: f64,
    checkpoints: Vec<usize>,
    patterns: Vec<Vec<i8>>,
    case_ids: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = run_campaign()?;
    let second = run_campaign()?;
    let replay_equal = first == second;
    let protocol_valid = first.case_set_exact
        && first.cases.iter().all(|case| case.case_id.starts_with("fl5f|"))
        && first.h5f_0_reference_128
        && first.all_checkpoints_present
        && first.all_numeric_finite
        && first.all_unit_norm_valid
        && replay_equal;

    let artifact = ExperimentArtifact {
        experiment: "FL-5F",
        protocol: "deterministic-basin-lifetime-v1",
        fieldlab_commit: std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_owned()),
        frozen_constants: FrozenConstants {
            node_count: NODE_COUNT,
            max_steps: MAX_STEPS,
            field_dt: FIELD_DT,
            field_mobility: FIELD_MOBILITY,
            cue_tilt_radians: CUE_TILT_RADIANS,
            checkpoints: CHECKPOINTS.to_vec(),
            patterns: reference_patterns(),
            case_ids: case_specs().iter().map(|case| case.id.to_owned()).collect(),
        },
        campaign: first.clone(),
        replay_equal,
        hypotheses: HypothesisReport {
            h5f_0_reference_128: Check::from_bool(first.h5f_0_reference_128),
            h5f_1_clean_permanence: Check::from_bool(first.h5f_1_clean_permanence),
            h5f_2_wrong_permanence: Check::from_bool(first.h5f_2_wrong_permanence),
            h5f_3_128_256_invariance: Check::from_bool(first.h5f_3_128_256_invariance),
            h5f_4_deterministic_replay: Check::from_bool(replay_equal),
        },
        validity: ValidityReport {
            frozen_case_set_exact: Check::from_bool(first.case_set_exact),
            new_case_namespace: Check::from_bool(
                first.cases.iter().all(|case| case.case_id.starts_with("fl5f|")),
            ),
            reference_128_gate: Check::from_bool(first.h5f_0_reference_128),
            checkpoints_complete: Check::from_bool(first.all_checkpoints_present),
            all_metrics_finite: Check::from_bool(first.all_numeric_finite),
            unit_norm_valid: Check::from_bool(first.all_unit_norm_valid),
            energy_evaluation_succeeded: Check::Pass,
            replay_exact: Check::from_bool(replay_equal),
            protocol_valid: Check::from_bool(protocol_valid),
        },
    };

    println!("{}", serde_json::to_string_pretty(&artifact)?);
    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn run_campaign() -> Result<Campaign, Box<dyn Error>> {
    let bank = PatternBank::new(reference_patterns())?;
    let model = bank.energy_model()?;
    let integrator = IntegratorConfig { dt: FIELD_DT, mobility: FIELD_MOBILITY };

    let specs = case_specs();
    let mut cases = Vec::with_capacity(specs.len());
    for spec in specs {
        cases.push(run_case(&bank, &model, integrator, spec)?);
    }

    let expected_ids: BTreeSet<&str> = case_specs().iter().map(|case| case.id).collect();
    let actual_ids: BTreeSet<&str> = cases.iter().map(|case| case.case_id.as_str()).collect();
    let case_set_exact = expected_ids == actual_ids && cases.len() == expected_ids.len();

    let clean_common_stable_through = cases.iter().filter(|case| case.kind == "clean").map(|case| case.first_clean_exit.map_or(MAX_STEPS, |step| step.saturating_sub(1))).min().unwrap_or(0);
    let wrong_common_competitor_through = cases.iter().filter(|case| case.kind == "wrong_basin").map(|case| case.first_competitor_exit.map_or(MAX_STEPS, |step| step.saturating_sub(1))).min().unwrap_or(0);
    let benchmark_common_stable_through = clean_common_stable_through.min(wrong_common_competitor_through);
    let largest_stable_checkpoint = CHECKPOINTS.iter().copied().filter(|step| *step <= benchmark_common_stable_through).max().unwrap_or(0);

    let h5f_0_reference_128 = cases.iter().all(|case| {
        let label = checkpoint_label(case, 128);
        if case.kind == "clean" { label == pattern_label(case.target_index) } else { case.competitor_index.is_some_and(|index| label == pattern_label(index)) }
    });
    let h5f_1_clean_permanence = cases.iter().filter(|case| case.kind == "clean").all(|case| case.first_clean_exit.is_none());
    let h5f_2_wrong_permanence = cases.iter().filter(|case| case.kind == "wrong_basin").all(|case| case.first_competitor_entry.is_some() && case.first_competitor_exit.is_none());
    let h5f_3_128_256_invariance = cases.iter().all(|case| checkpoint_label(case, 128) == checkpoint_label(case, 256));
    let all_numeric_finite = cases.iter().all(|case| case.finite);
    let all_unit_norm_valid = cases.iter().all(|case| case.unit_norm_valid);
    let all_checkpoints_present = cases.iter().all(|case| case.checkpoints.len() == CHECKPOINTS.len() && case.checkpoints.iter().zip(CHECKPOINTS).all(|(record, expected)| record.step == expected));

    Ok(Campaign { cases, clean_common_stable_through, wrong_common_competitor_through, benchmark_common_stable_through, largest_stable_checkpoint, h5f_0_reference_128, h5f_1_clean_permanence, h5f_2_wrong_permanence, h5f_3_128_256_invariance, all_numeric_finite, all_unit_norm_valid, all_checkpoints_present, case_set_exact })
}

fn run_case(bank: &PatternBank, model: &EnergyModel, integrator: IntegratorConfig, spec: CaseSpec) -> Result<CaseReport, Box<dyn Error>> {
    let mut state = bank.encode_cue(&spec.cue, CUE_TILT_RADIANS)?;
    let mut labels = Vec::with_capacity(MAX_STEPS + 1);
    let mut checkpoints = Vec::with_capacity(CHECKPOINTS.len());
    let mut unit_norm_valid = true;
    let mut finite = true;

    for step in 0..=MAX_STEPS {
        unit_norm_valid &= model.validate_state(&state).is_ok();
        let energy = model.energy(&state)?;
        finite &= energy.is_finite() && state.nodes().iter().all(|node| node.values().iter().all(|value| value.is_finite()));
        let decoded = bank.decode_state(&state)?;
        let label = classify_pattern(bank, &decoded);
        labels.push(label.clone());
        if CHECKPOINTS.contains(&step) { checkpoints.push(checkpoint_record(bank, model, &state, step, label, energy)?); }
        if step < MAX_STEPS { state = heun_step(&state, model, integrator)?; }
    }

    let segments = build_segments(&labels);
    let target_label = pattern_label(spec.target);
    let competitor_label = spec.competitor.map(pattern_label);
    let first_clean_exit = if spec.kind == CaseKind::Clean { first_step_not_label(&labels, &target_label, 0) } else { None };
    let first_clean_return = first_clean_exit.and_then(|exit| first_step_with_label(&labels, &target_label, exit.saturating_add(1)));
    let first_competitor_entry = competitor_label.as_ref().and_then(|label| first_step_with_label(&labels, label, 0));
    let first_competitor_exit = match (competitor_label.as_ref(), first_competitor_entry) { (Some(label), Some(entry)) => first_step_not_label(&labels, label, entry.saturating_add(1)), _ => None };
    let first_target_entry = first_step_with_label(&labels, &target_label, 0);
    let first_competitor_return = match (competitor_label.as_ref(), first_competitor_exit) { (Some(label), Some(exit)) => first_step_with_label(&labels, label, exit.saturating_add(1)), _ => None };

    let target_state = bank.encode_cue(&bank.patterns()[spec.target], CUE_TILT_RADIANS)?;
    let final_target_angle = mean_angle_error(&state, &target_state);
    let final_competitor_angle = match spec.competitor { Some(index) => { let template = bank.encode_cue(&bank.patterns()[index], CUE_TILT_RADIANS)?; Some(mean_angle_error(&state, &template)) }, None => None };
    finite &= final_target_angle.is_finite() && final_competitor_angle.is_none_or(f64::is_finite) && checkpoints.iter().all(checkpoint_finite);

    Ok(CaseReport { case_id: spec.id.to_owned(), kind: match spec.kind { CaseKind::Clean => "clean", CaseKind::WrongBasin => "wrong_basin" }.to_owned(), target_index: spec.target, competitor_index: spec.competitor, cue: spec.cue.to_vec(), initial_label: labels.first().cloned().unwrap_or_else(|| "other".to_owned()), final_label: labels.last().cloned().unwrap_or_else(|| "other".to_owned()), first_clean_exit, first_clean_return, first_competitor_entry, first_competitor_exit, first_target_entry, first_competitor_return, total_label_transitions: segments.len().saturating_sub(1), decoded_segments: segments, checkpoints, final_target_angle, final_competitor_angle, unit_norm_valid, finite })
}

fn checkpoint_record(bank: &PatternBank, model: &EnergyModel, state: &FieldState, step: usize, decoded_label: String, energy: f64) -> Result<CheckpointRecord, Box<dyn Error>> {
    let mut angles = Vec::with_capacity(bank.patterns().len());
    for pattern in bank.patterns() { let template = bank.encode_cue(pattern, CUE_TILT_RADIANS)?; angles.push(mean_angle_error(state, &template)); }
    let mut ranked: Vec<(usize, f64)> = angles.iter().copied().enumerate().collect();
    ranked.sort_by(|left, right| left.1.total_cmp(&right.1));
    let nearest_template = ranked.first().map_or(0, |entry| entry.0);
    let top_two_gap = if ranked.len() >= 2 { ranked[1].1 - ranked[0].1 } else { 0.0 };
    let verified_energy = model.energy(state)?;
    debug_assert_eq!(energy.to_bits(), verified_energy.to_bits());
    Ok(CheckpointRecord { step, decoded_label, nearest_template, top_two_gap, template_angles: angles, energy })
}

fn checkpoint_finite(record: &CheckpointRecord) -> bool { record.energy.is_finite() && record.top_two_gap.is_finite() && record.template_angles.iter().all(|value| value.is_finite()) }
fn classify_pattern(bank: &PatternBank, decoded: &[i8]) -> String { bank.patterns().iter().position(|pattern| pattern == decoded).map_or_else(|| "other".to_owned(), pattern_label) }
fn pattern_label(index: usize) -> String { format!("P{index}") }
fn first_step_with_label(labels: &[String], wanted: &str, start: usize) -> Option<usize> { labels.iter().enumerate().skip(start).find_map(|(step, label)| (label == wanted).then_some(step)) }
fn first_step_not_label(labels: &[String], wanted: &str, start: usize) -> Option<usize> { labels.iter().enumerate().skip(start).find_map(|(step, label)| (label != wanted).then_some(step)) }

fn build_segments(labels: &[String]) -> Vec<Segment> {
    let Some(first) = labels.first() else { return Vec::new(); };
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut current = first.clone();
    for (step, label) in labels.iter().enumerate().skip(1) {
        if label != &current {
            segments.push(Segment { label: current.clone(), start_step: start, end_step: step - 1 });
            current.clone_from(label);
            start = step;
        }
    }
    segments.push(Segment { label: current, start_step: start, end_step: labels.len() - 1 });
    segments
}

fn checkpoint_label(case: &CaseReport, step: usize) -> String { case.checkpoints.iter().find(|record| record.step == step).map_or_else(|| "missing".to_owned(), |record| record.decoded_label.clone()) }
fn mean_angle_error(state: &FieldState, target: &FieldState) -> f64 { let mut sum = 0.0; for (node, target_node) in state.nodes().iter().zip(target.nodes()) { let cosine = field_core::dot(node.values(), target_node.values()).clamp(-1.0, 1.0); sum += cosine.acos(); } sum / f64::from(u32::try_from(state.node_count()).unwrap_or(1)) }
fn reference_patterns() -> Vec<Vec<i8>> { vec![vec![1,1,1,1,1,1,1,1], vec![1,1,1,1,1,1,-1,-1], vec![1,1,1,1,-1,-1,1,1]] }

fn case_specs() -> [CaseSpec; 9] {
    [
        CaseSpec { id: "fl5f|clean|t0", kind: CaseKind::Clean, target: 0, competitor: None, cue: [1,1,1,1,1,1,1,1] },
        CaseSpec { id: "fl5f|clean|t1", kind: CaseKind::Clean, target: 1, competitor: None, cue: [1,1,1,1,1,1,-1,-1] },
        CaseSpec { id: "fl5f|clean|t2", kind: CaseKind::Clean, target: 2, competitor: None, cue: [1,1,1,1,-1,-1,1,1] },
        CaseSpec { id: "fl5f|wb|t0|c1|k0", kind: CaseKind::WrongBasin, target: 0, competitor: Some(1), cue: [1,1,1,1,1,1,-1,-1] },
        CaseSpec { id: "fl5f|wb|t0|c2|k0", kind: CaseKind::WrongBasin, target: 0, competitor: Some(2), cue: [1,1,1,1,-1,-1,1,1] },
        CaseSpec { id: "fl5f|wb|t1|c0|k1", kind: CaseKind::WrongBasin, target: 1, competitor: Some(0), cue: [1,1,1,1,1,1,-1,1] },
        CaseSpec { id: "fl5f|wb|t1|c0|k2", kind: CaseKind::WrongBasin, target: 1, competitor: Some(0), cue: [1,1,1,1,1,1,1,-1] },
        CaseSpec { id: "fl5f|wb|t2|c0|k1", kind: CaseKind::WrongBasin, target: 2, competitor: Some(0), cue: [1,1,1,1,-1,1,1,1] },
        CaseSpec { id: "fl5f|wb|t2|c0|k2", kind: CaseKind::WrongBasin, target: 2, competitor: Some(0), cue: [1,1,1,1,1,-1,1,1] },
    ]
}

#[cfg(test)]
mod tests {
    use super::{case_specs, CHECKPOINTS, MAX_STEPS};
    use std::collections::BTreeSet;
    #[test]
    fn frozen_checkpoints_are_sorted_and_end_at_max() { assert_eq!(CHECKPOINTS[0], 0); assert_eq!(*CHECKPOINTS.last().unwrap(), MAX_STEPS); assert!(CHECKPOINTS.windows(2).all(|pair| pair[0] < pair[1])); }
    #[test]
    fn frozen_cases_are_unique_and_namespaced() { let specs = case_specs(); let ids: BTreeSet<&str> = specs.iter().map(|case| case.id).collect(); assert_eq!(ids.len(), specs.len()); assert!(ids.iter().all(|id| id.starts_with("fl5f|"))); }
}
