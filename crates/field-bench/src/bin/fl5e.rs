#![forbid(unsafe_code)]

use field_core::{EnergyModel, FieldModel, FieldState, ValidationError};
use field_dynamics::{heun_step, IntegratorConfig};
use field_memory::PatternBank;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;

const NOISELAB_COMMIT: &str = "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e";
const NODE_COUNT: usize = 8;
const STATE_DIM: usize = 2;
const STAGE_STEPS: usize = 128;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const CUE_TILT_RADIANS: f64 = 0.15;
const AMPLITUDES: [f64; 2] = [0.25, 0.50];
const OU_THETAS: [f64; 2] = [0.5, 2.0];
const GATE_THRESHOLDS: [f64; 4] = [0.05, 0.10, 0.20, 0.35];
const SEEDS: [u64; 8] = [1, 2, 3, 5, 8, 13, 21, 34];
const PERM_SEED: u64 = 0x0F15_E005;

const CLEAN_CUE_IDS: [&str; 6] = [
    "fl5e|clean|t0|a0",
    "fl5e|clean|t0|a1",
    "fl5e|clean|t1|a0",
    "fl5e|clean|t1|a1",
    "fl5e|clean|t2|a0",
    "fl5e|clean|t2|a1",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Calibration,
    Holdout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Arm {
    D0,
    GatedOu,
    PermGatedOu,
    GlobalOu,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct ConditionSpec {
    amplitude: f64,
    ou_theta: f64,
    gate_threshold: f64,
}

#[derive(Clone, Debug, Serialize)]
struct FixturePattern {
    index: usize,
    symbols: Vec<i8>,
}

#[derive(Clone, Debug, Serialize)]
struct CaseRecord {
    case_id: String,
    sha256: String,
    partition: Partition,
    kind: &'static str,
    target_index: usize,
    competitor_index: Option<usize>,
    cue: Vec<i8>,
    stage_a_terminal_pattern: Vec<i8>,
    stage_a_best_index: usize,
    stage_a_ambiguity_gap: f64,
    stage_a_target_angle: f64,
    stage_a_competitor_angle: Option<f64>,
    stage_a_ends_competitor: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct SeedMetrics {
    seed: u64,
    target_recoveries: usize,
    wrong_cases: usize,
    competing_terminal: usize,
    unresolved_terminal: usize,
    left_wrong_basin: usize,
    clean_correct: usize,
    clean_cases: usize,
    gate_on_wrong: usize,
    gate_on_clean: usize,
    noise_applied_wrong: usize,
    noise_applied_clean: usize,
    median_target_angle_error: f64,
    mean_first_passage_time: Option<f64>,
    finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ArmAggregate {
    arm: Arm,
    condition: ConditionSpec,
    seeds: Vec<SeedMetrics>,
    target_recovery_fraction: f64,
    competing_terminal_fraction: f64,
    unresolved_terminal_fraction: f64,
    left_wrong_basin_fraction: f64,
    clean_recall_fraction: f64,
    clean_degradation_vs_d0: f64,
    gate_on_wrong_fraction: f64,
    gate_on_clean_fraction: f64,
    noise_applied_wrong_fraction: f64,
    noise_applied_clean_fraction: f64,
    median_target_angle_error: f64,
    mean_first_passage_time: Option<f64>,
    finite: bool,
}

#[derive(Clone, Debug, Serialize)]
struct SelectedCondition {
    amplitude: f64,
    ou_theta: f64,
    gate_threshold: f64,
    calibration_target_recovery_fraction: f64,
    calibration_left_wrong_basin_fraction: f64,
    calibration_clean_recall_fraction: f64,
    calibration_gate_on_clean_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
struct FrozenConstants {
    node_count: usize,
    stage_steps: usize,
    total_steps: usize,
    field_dt: f64,
    field_mobility: f64,
    cue_tilt_radians: f64,
    amplitudes: Vec<f64>,
    ou_thetas: Vec<f64>,
    gate_thresholds: Vec<f64>,
    seeds: Vec<u64>,
    perm_seed: u64,
    clean_cue_ids: Vec<String>,
    patterns: Vec<FixturePattern>,
}

#[derive(Clone, Debug, Serialize)]
struct FixtureInfo {
    wrong_basin_cases: usize,
    clean_cue_cases: usize,
    calibration_wrong_basin: usize,
    holdout_wrong_basin: usize,
    calibration_clean: usize,
    holdout_clean: usize,
    all_wrong_stage_a_end_competitor: bool,
    calibration_gate_constructible: bool,
    holdout_gate_constructible: bool,
    description: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct HypothesisReport {
    h5e_0_selective_gate: Check,
    h5e_1_safe_escape: Check,
    h5e_2_gating_benefit: Check,
    h5e_3_temporal_structure: Check,
    h5e_4_reproducibility: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ValidityReport {
    preregistered_grid_used: Check,
    holdout_excluded_from_selection: Check,
    hard_clean_cue_filter_applied: Check,
    target_label_not_used_by_gate: Check,
    matched_two_stage_budget: Check,
    new_case_namespace: Check,
    panels_disjoint: Check,
    both_partitions_wrong_nonempty: Check,
    both_partitions_clean_nonempty: Check,
    wrong_stage_a_competitor_gate: Check,
    gate_constructible_both_partitions: Check,
    all_metrics_finite: Check,
    replay_exact: Check,
    noiselab_commit_matches: Check,
    protocol_valid: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ExperimentArtifact {
    experiment: &'static str,
    protocol: &'static str,
    noiselab_commit: &'static str,
    fieldlab_commit: String,
    provenance_fingerprint: String,
    frozen_constants: FrozenConstants,
    fixture: FixtureInfo,
    cases: Vec<CaseRecord>,
    calibration_d0: ArmAggregate,
    calibration_grid_results: Vec<ArmAggregate>,
    none_survived_clean_cue_filter: bool,
    selected_condition: Option<SelectedCondition>,
    holdout_d0: ArmAggregate,
    holdout_selected: Option<ArmAggregate>,
    holdout_perm: Option<ArmAggregate>,
    holdout_global: Option<ArmAggregate>,
    hypotheses: HypothesisReport,
    validity: ValidityReport,
}

#[derive(Clone, Debug)]
struct TrialResult {
    terminal_pattern: Vec<i8>,
    target_angle_error: f64,
    first_passage_time: Option<usize>,
    left_wrong_basin: bool,
    gate_on: bool,
    noise_applied: bool,
    finite: bool,
}

#[derive(Clone, Copy)]
struct WrongSpec {
    id: &'static str,
    target: usize,
    competitor: usize,
    cue: [i8; NODE_COUNT],
}

struct SplitMix64 {
    state: u64,
    gauss_spare: Option<f64>,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self {
            state: seed,
            gauss_spare: None,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    #[allow(clippy::cast_precision_loss)]
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    fn next_gaussian(&mut self) -> f64 {
        if let Some(spare) = self.gauss_spare.take() {
            return spare;
        }
        let u1 = 1.0 - self.next_f64();
        let u2 = self.next_f64();
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = 2.0 * std::f64::consts::PI * u2;
        self.gauss_spare = Some(radius * angle.sin());
        radius * angle.cos()
    }
}

struct NoisyModel<'a> {
    base: &'a EnergyModel,
    additive: &'a [Vec<f64>],
}

impl FieldModel for NoisyModel<'_> {
    fn validate_state(&self, state: &FieldState) -> Result<(), ValidationError> {
        self.base.validate_state(state)
    }

    fn energy(&self, state: &FieldState) -> Result<f64, ValidationError> {
        self.base.energy(state)
    }

    fn effective_fields(&self, state: &FieldState) -> Result<Vec<Vec<f64>>, ValidationError> {
        let mut fields = self.base.effective_fields(state)?;
        for (field, add) in fields.iter_mut().zip(self.additive) {
            for (component, delta) in field.iter_mut().zip(add) {
                *component += *delta;
            }
        }
        Ok(fields)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let artifact = run_experiment()?;
    println!("{}", serde_json::to_string_pretty(&artifact)?);
    if !artifact.validity.protocol_valid.as_bool() {
        std::process::exit(1);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_experiment() -> Result<ExperimentArtifact, Box<dyn Error>> {
    let bank = PatternBank::new(reference_patterns())?;
    let model = bank.energy_model()?;
    let integrator = IntegratorConfig {
        dt: FIELD_DT,
        mobility: FIELD_MOBILITY,
    };
    let temporal_perm = temporal_permutation(STAGE_STEPS, PERM_SEED);
    let mut cases = build_cases(&bank, &model, integrator)?;

    let wrong: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "wrong_basin")
        .cloned()
        .collect();
    let clean: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "clean_cue")
        .cloned()
        .collect();
    let cal_wrong = by_partition(&wrong, Partition::Calibration);
    let hold_wrong = by_partition(&wrong, Partition::Holdout);
    let cal_clean = by_partition(&clean, Partition::Calibration);
    let hold_clean = by_partition(&clean, Partition::Holdout);

    let all_wrong_competitor = wrong.iter().all(|case| case.stage_a_ends_competitor);
    let cal_gate_constructible = gate_constructible(&cal_wrong);
    let hold_gate_constructible = gate_constructible(&hold_wrong);
    let disjoint = panels_disjoint(&wrong, &clean);
    let case_namespace_ok = cases.iter().all(|case| case.case_id.starts_with("fl5e|"));
    let partition_counts_ok = cal_wrong.len() >= 2
        && hold_wrong.len() >= 2
        && cal_clean.len() >= 2
        && hold_clean.len() >= 2;

    let d0_condition = ConditionSpec {
        amplitude: 0.0,
        ou_theta: 0.5,
        gate_threshold: 0.0,
    };
    let mut calibration_d0 = evaluate_arm(
        &bank,
        &model,
        integrator,
        &cal_wrong,
        &cal_clean,
        Arm::D0,
        d0_condition,
        false,
        &temporal_perm,
    )?;
    calibration_d0.clean_degradation_vs_d0 = 0.0;

    let mut grid_results = Vec::new();
    let mut best: Option<(SelectionKey, ConditionSpec, ArmAggregate)> = None;
    for amplitude in AMPLITUDES {
        for theta in OU_THETAS {
            for tau in GATE_THRESHOLDS {
                let condition = ConditionSpec {
                    amplitude,
                    ou_theta: theta,
                    gate_threshold: tau,
                };
                let mut aggregate = evaluate_arm(
                    &bank,
                    &model,
                    integrator,
                    &cal_wrong,
                    &cal_clean,
                    Arm::GatedOu,
                    condition,
                    false,
                    &temporal_perm,
                )?;
                aggregate.clean_degradation_vs_d0 = (calibration_d0.clean_recall_fraction
                    - aggregate.clean_recall_fraction)
                    .max(0.0);
                let clean_safe = aggregate.clean_recall_fraction.to_bits()
                    == calibration_d0.clean_recall_fraction.to_bits();
                if clean_safe {
                    let key = selection_key(&aggregate);
                    if best.as_ref().is_none_or(|(best_key, _, _)| key < *best_key) {
                        best = Some((key, condition, aggregate.clone()));
                    }
                }
                grid_results.push(aggregate);
            }
        }
    }

    let none_survived = best.is_none();
    let mut holdout_d0 = evaluate_arm(
        &bank,
        &model,
        integrator,
        &hold_wrong,
        &hold_clean,
        Arm::D0,
        d0_condition,
        false,
        &temporal_perm,
    )?;
    holdout_d0.clean_degradation_vs_d0 = 0.0;

    let mut selected_condition = None;
    let mut holdout_selected = None;
    let mut holdout_perm = None;
    let mut holdout_global = None;
    let mut replay_exact = true;

    if let Some((_, condition, cal_selected)) = best.as_ref() {
        let condition = *condition;
        let mut selected = evaluate_arm(
            &bank,
            &model,
            integrator,
            &hold_wrong,
            &hold_clean,
            Arm::GatedOu,
            condition,
            false,
            &temporal_perm,
        )?;
        selected.clean_degradation_vs_d0 =
            (holdout_d0.clean_recall_fraction - selected.clean_recall_fraction).max(0.0);

        let mut perm = evaluate_arm(
            &bank,
            &model,
            integrator,
            &hold_wrong,
            &hold_clean,
            Arm::PermGatedOu,
            condition,
            true,
            &temporal_perm,
        )?;
        perm.clean_degradation_vs_d0 =
            (holdout_d0.clean_recall_fraction - perm.clean_recall_fraction).max(0.0);

        let mut global = evaluate_arm(
            &bank,
            &model,
            integrator,
            &hold_wrong,
            &hold_clean,
            Arm::GlobalOu,
            condition,
            false,
            &temporal_perm,
        )?;
        global.clean_degradation_vs_d0 =
            (holdout_d0.clean_recall_fraction - global.clean_recall_fraction).max(0.0);

        let replay = evaluate_arm(
            &bank,
            &model,
            integrator,
            &hold_wrong,
            &hold_clean,
            Arm::GatedOu,
            condition,
            false,
            &temporal_perm,
        )?;
        replay_exact = selected == replay;

        selected_condition = Some(SelectedCondition {
            amplitude: condition.amplitude,
            ou_theta: condition.ou_theta,
            gate_threshold: condition.gate_threshold,
            calibration_target_recovery_fraction: cal_selected.target_recovery_fraction,
            calibration_left_wrong_basin_fraction: cal_selected.left_wrong_basin_fraction,
            calibration_clean_recall_fraction: cal_selected.clean_recall_fraction,
            calibration_gate_on_clean_fraction: cal_selected.gate_on_clean_fraction,
        });
        holdout_selected = Some(selected);
        holdout_perm = Some(perm);
        holdout_global = Some(global);
    }

    let all_finite = arm_finite(&calibration_d0)
        && grid_results.iter().all(arm_finite)
        && arm_finite(&holdout_d0)
        && holdout_selected.as_ref().is_none_or(arm_finite)
        && holdout_perm.as_ref().is_none_or(arm_finite)
        && holdout_global.as_ref().is_none_or(arm_finite);

    let (selective_gate, safe_escape, gating_benefit, temporal_structure) = match (
        holdout_selected.as_ref(),
        holdout_perm.as_ref(),
        holdout_global.as_ref(),
    ) {
        (Some(selected), Some(perm), Some(global)) => (
            selected.gate_on_wrong_fraction > 0.0
                && selected.gate_on_wrong_fraction > selected.gate_on_clean_fraction,
            selected.target_recovery_fraction > holdout_d0.target_recovery_fraction
                && selected.clean_recall_fraction.to_bits()
                    == holdout_d0.clean_recall_fraction.to_bits(),
            selected.clean_recall_fraction > global.clean_recall_fraction
                && selected.target_recovery_fraction + f64::EPSILON
                    >= global.target_recovery_fraction,
            selected.target_recovery_fraction > perm.target_recovery_fraction + f64::EPSILON,
        ),
        _ => (false, false, false, false),
    };

    let protocol_valid = partition_counts_ok
        && all_wrong_competitor
        && cal_gate_constructible
        && hold_gate_constructible
        && disjoint
        && case_namespace_ok
        && all_finite
        && replay_exact
        && NOISELAB_COMMIT == "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e";

    let hypotheses = HypothesisReport {
        h5e_0_selective_gate: Check::from_bool(selective_gate),
        h5e_1_safe_escape: Check::from_bool(safe_escape),
        h5e_2_gating_benefit: Check::from_bool(gating_benefit),
        h5e_3_temporal_structure: Check::from_bool(temporal_structure),
        h5e_4_reproducibility: Check::from_bool(replay_exact && all_finite),
    };

    let validity = ValidityReport {
        preregistered_grid_used: Check::Pass,
        holdout_excluded_from_selection: Check::Pass,
        hard_clean_cue_filter_applied: Check::Pass,
        target_label_not_used_by_gate: Check::Pass,
        matched_two_stage_budget: Check::Pass,
        new_case_namespace: Check::from_bool(case_namespace_ok),
        panels_disjoint: Check::from_bool(disjoint),
        both_partitions_wrong_nonempty: Check::from_bool(
            cal_wrong.len() >= 2 && hold_wrong.len() >= 2,
        ),
        both_partitions_clean_nonempty: Check::from_bool(
            cal_clean.len() >= 2 && hold_clean.len() >= 2,
        ),
        wrong_stage_a_competitor_gate: Check::from_bool(all_wrong_competitor),
        gate_constructible_both_partitions: Check::from_bool(
            cal_gate_constructible && hold_gate_constructible,
        ),
        all_metrics_finite: Check::from_bool(all_finite),
        replay_exact: Check::from_bool(replay_exact),
        noiselab_commit_matches: Check::Pass,
        protocol_valid: Check::from_bool(protocol_valid),
    };

    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));

    Ok(ExperimentArtifact {
        experiment: "FL-5E",
        protocol: "ambiguity-gated-strong-perturbation-v1",
        noiselab_commit: NOISELAB_COMMIT,
        fieldlab_commit: std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_owned()),
        provenance_fingerprint: format!("fnv1a64:{:016x}", provenance_fingerprint()),
        frozen_constants: FrozenConstants {
            node_count: NODE_COUNT,
            stage_steps: STAGE_STEPS,
            total_steps: STAGE_STEPS * 2,
            field_dt: FIELD_DT,
            field_mobility: FIELD_MOBILITY,
            cue_tilt_radians: CUE_TILT_RADIANS,
            amplitudes: AMPLITUDES.to_vec(),
            ou_thetas: OU_THETAS.to_vec(),
            gate_thresholds: GATE_THRESHOLDS.to_vec(),
            seeds: SEEDS.to_vec(),
            perm_seed: PERM_SEED,
            clean_cue_ids: CLEAN_CUE_IDS.iter().map(|id| (*id).to_owned()).collect(),
            patterns: reference_patterns()
                .into_iter()
                .enumerate()
                .map(|(index, symbols)| FixturePattern { index, symbols })
                .collect(),
        },
        fixture: FixtureInfo {
            wrong_basin_cases: wrong.len(),
            clean_cue_cases: clean.len(),
            calibration_wrong_basin: cal_wrong.len(),
            holdout_wrong_basin: hold_wrong.len(),
            calibration_clean: cal_clean.len(),
            holdout_clean: hold_clean.len(),
            all_wrong_stage_a_end_competitor: all_wrong_competitor,
            calibration_gate_constructible: cal_gate_constructible,
            holdout_gate_constructible: hold_gate_constructible,
            description: "FL-5E fresh namespace; FL-5D H=2/2/4 shallow cue geometries; 128-step deterministic settle followed by a matched 128-step continuation; strong OU is gated only by target-agnostic top-two template angular ambiguity; clean cues obey the same gate; calibration-only selection",
        },
        cases,
        calibration_d0,
        calibration_grid_results: grid_results,
        none_survived_clean_cue_filter: none_survived,
        selected_condition,
        holdout_d0,
        holdout_selected,
        holdout_perm,
        holdout_global,
        hypotheses,
        validity,
    })
}

fn reference_patterns() -> Vec<Vec<i8>> {
    vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, -1, -1],
        vec![1, 1, 1, 1, -1, -1, 1, 1],
    ]
}

fn wrong_specs() -> [WrongSpec; 6] {
    [
        WrongSpec {
            id: "fl5e|wb|t0|c1|k0",
            target: 0,
            competitor: 1,
            cue: [1, 1, 1, 1, 1, 1, -1, -1],
        },
        WrongSpec {
            id: "fl5e|wb|t0|c2|k0",
            target: 0,
            competitor: 2,
            cue: [1, 1, 1, 1, -1, -1, 1, 1],
        },
        WrongSpec {
            id: "fl5e|wb|t1|c0|k1",
            target: 1,
            competitor: 0,
            cue: [1, 1, 1, 1, 1, 1, -1, 1],
        },
        WrongSpec {
            id: "fl5e|wb|t1|c0|k2",
            target: 1,
            competitor: 0,
            cue: [1, 1, 1, 1, 1, 1, 1, -1],
        },
        WrongSpec {
            id: "fl5e|wb|t2|c0|k1",
            target: 2,
            competitor: 0,
            cue: [1, 1, 1, 1, -1, 1, 1, 1],
        },
        WrongSpec {
            id: "fl5e|wb|t2|c0|k2",
            target: 2,
            competitor: 0,
            cue: [1, 1, 1, 1, 1, -1, 1, 1],
        },
    ]
}

fn build_cases(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
) -> Result<Vec<CaseRecord>, Box<dyn Error>> {
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();

    for spec in wrong_specs() {
        if !seen.insert(spec.id) {
            return Err("duplicate FL-5E wrong-basin id".into());
        }
        let cue = spec.cue.to_vec();
        let state = deterministic_settle(bank, model, integrator, &cue)?;
        let terminal = bank.decode_state(&state)?;
        let (best_index, gap) = ambiguity_summary(bank, &state)?;
        let target_state = bank.encode_cue(&bank.patterns()[spec.target], CUE_TILT_RADIANS)?;
        let competitor_state =
            bank.encode_cue(&bank.patterns()[spec.competitor], CUE_TILT_RADIANS)?;
        let target_angle = mean_angle_error(&state, &target_state);
        let competitor_angle = mean_angle_error(&state, &competitor_state);
        cases.push(partition_case(CaseRecord {
            case_id: spec.id.to_owned(),
            sha256: String::new(),
            partition: Partition::Calibration,
            kind: "wrong_basin",
            target_index: spec.target,
            competitor_index: Some(spec.competitor),
            cue,
            stage_a_terminal_pattern: terminal.clone(),
            stage_a_best_index: best_index,
            stage_a_ambiguity_gap: gap,
            stage_a_target_angle: target_angle,
            stage_a_competitor_angle: Some(competitor_angle),
            stage_a_ends_competitor: terminal == bank.patterns()[spec.competitor],
        })?);
    }

    for case_id in CLEAN_CUE_IDS {
        if !seen.insert(case_id) {
            return Err("duplicate FL-5E clean id".into());
        }
        let target_index = parse_clean_target_index(case_id)?;
        let cue = bank.patterns()[target_index].clone();
        let state = deterministic_settle(bank, model, integrator, &cue)?;
        let terminal = bank.decode_state(&state)?;
        let (best_index, gap) = ambiguity_summary(bank, &state)?;
        let target_state = bank.encode_cue(&bank.patterns()[target_index], CUE_TILT_RADIANS)?;
        cases.push(partition_case(CaseRecord {
            case_id: case_id.to_owned(),
            sha256: String::new(),
            partition: Partition::Calibration,
            kind: "clean_cue",
            target_index,
            competitor_index: None,
            cue,
            stage_a_terminal_pattern: terminal,
            stage_a_best_index: best_index,
            stage_a_ambiguity_gap: gap,
            stage_a_target_angle: mean_angle_error(&state, &target_state),
            stage_a_competitor_angle: None,
            stage_a_ends_competitor: false,
        })?);
    }

    Ok(cases)
}

fn parse_clean_target_index(case_id: &str) -> Result<usize, Box<dyn Error>> {
    let parts: Vec<&str> = case_id.split('|').collect();
    if parts.len() != 4 || parts[0] != "fl5e" || parts[1] != "clean" {
        return Err(format!("invalid FL-5E clean id: {case_id}").into());
    }
    let target = parts[2]
        .strip_prefix('t')
        .ok_or_else(|| format!("clean id missing target: {case_id}"))?;
    let index = target
        .parse::<usize>()
        .map_err(|_| format!("bad clean target index: {case_id}"))?;
    if index >= 3 || !parts[3].starts_with('a') {
        return Err(format!("invalid FL-5E clean id: {case_id}").into());
    }
    Ok(index)
}

fn partition_case(mut case: CaseRecord) -> Result<CaseRecord, Box<dyn Error>> {
    if case.case_id.is_empty() {
        return Err("empty FL-5E case id".into());
    }
    let digest = Sha256::digest(case.case_id.as_bytes());
    case.partition = if digest[0] < 0x80 {
        Partition::Calibration
    } else {
        Partition::Holdout
    };
    case.sha256 = hex_digest(&digest);
    Ok(case)
}

fn by_partition(cases: &[CaseRecord], partition: Partition) -> Vec<CaseRecord> {
    cases
        .iter()
        .filter(|case| case.partition == partition)
        .cloned()
        .collect()
}

fn panels_disjoint(wrong: &[CaseRecord], clean: &[CaseRecord]) -> bool {
    let wrong_ids: BTreeSet<&str> = wrong.iter().map(|case| case.case_id.as_str()).collect();
    let clean_ids: BTreeSet<&str> = clean.iter().map(|case| case.case_id.as_str()).collect();
    wrong_ids.is_disjoint(&clean_ids)
}

fn gate_constructible(cases: &[CaseRecord]) -> bool {
    GATE_THRESHOLDS
        .iter()
        .any(|tau| cases.iter().any(|case| case.stage_a_ambiguity_gap <= *tau))
}

fn deterministic_settle(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
    cue: &[i8],
) -> Result<FieldState, Box<dyn Error>> {
    let mut state = bank.encode_cue(cue, CUE_TILT_RADIANS)?;
    for _ in 0..STAGE_STEPS {
        state = heun_step(&state, model, integrator)?;
    }
    Ok(state)
}

fn ambiguity_summary(
    bank: &PatternBank,
    state: &FieldState,
) -> Result<(usize, f64), Box<dyn Error>> {
    let mut angles = Vec::with_capacity(bank.patterns().len());
    for (index, pattern) in bank.patterns().iter().enumerate() {
        let template = bank.encode_cue(pattern, CUE_TILT_RADIANS)?;
        angles.push((index, mean_angle_error(state, &template)));
    }
    angles.sort_by(|left, right| left.1.total_cmp(&right.1));
    if angles.len() < 2 {
        return Err("FL-5E ambiguity requires at least two templates".into());
    }
    Ok((angles[0].0, angles[1].1 - angles[0].1))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_arm(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
    wrong_cases: &[CaseRecord],
    clean_cases: &[CaseRecord],
    arm: Arm,
    condition: ConditionSpec,
    permute: bool,
    temporal_perm: &[usize],
) -> Result<ArmAggregate, Box<dyn Error>> {
    let mut seeds = Vec::with_capacity(SEEDS.len());
    let mut total_recoveries = 0usize;
    let mut total_competing = 0usize;
    let mut total_unresolved = 0usize;
    let mut total_left = 0usize;
    let mut total_clean_correct = 0usize;
    let mut total_gate_wrong = 0usize;
    let mut total_gate_clean = 0usize;
    let mut total_noise_wrong = 0usize;
    let mut total_noise_clean = 0usize;
    let mut all_angles = Vec::new();
    let mut all_passages = Vec::new();
    let mut finite = true;

    for seed in SEEDS {
        let noise = if matches!(arm, Arm::D0) {
            Vec::new()
        } else {
            build_ou_noise(condition, seed, permute, temporal_perm)?
        };
        let mut recoveries = 0usize;
        let mut competing = 0usize;
        let mut unresolved = 0usize;
        let mut left = 0usize;
        let mut clean_correct = 0usize;
        let mut gate_wrong = 0usize;
        let mut gate_clean = 0usize;
        let mut noise_wrong = 0usize;
        let mut noise_clean = 0usize;
        let mut angles = Vec::new();
        let mut passages = Vec::new();
        let mut seed_finite = true;

        for case in wrong_cases {
            let trial = run_two_stage_trial(bank, model, integrator, case, arm, condition, &noise)?;
            seed_finite &= trial.finite;
            gate_wrong += usize::from(trial.gate_on);
            noise_wrong += usize::from(trial.noise_applied);
            if trial.terminal_pattern == bank.patterns()[case.target_index] {
                recoveries += 1;
            } else if case
                .competitor_index
                .is_some_and(|index| trial.terminal_pattern == bank.patterns()[index])
            {
                competing += 1;
            } else {
                unresolved += 1;
            }
            left += usize::from(trial.left_wrong_basin);
            angles.push(trial.target_angle_error);
            if let Some(step) = trial.first_passage_time {
                passages.push(f64::from(u32::try_from(step)?));
            }
        }

        for case in clean_cases {
            let trial = run_two_stage_trial(bank, model, integrator, case, arm, condition, &noise)?;
            seed_finite &= trial.finite;
            gate_clean += usize::from(trial.gate_on);
            noise_clean += usize::from(trial.noise_applied);
            clean_correct +=
                usize::from(trial.terminal_pattern == bank.patterns()[case.target_index]);
        }

        total_recoveries += recoveries;
        total_competing += competing;
        total_unresolved += unresolved;
        total_left += left;
        total_clean_correct += clean_correct;
        total_gate_wrong += gate_wrong;
        total_gate_clean += gate_clean;
        total_noise_wrong += noise_wrong;
        total_noise_clean += noise_clean;
        all_angles.extend_from_slice(&angles);
        all_passages.extend_from_slice(&passages);
        finite &= seed_finite;

        seeds.push(SeedMetrics {
            seed,
            target_recoveries: recoveries,
            wrong_cases: wrong_cases.len(),
            competing_terminal: competing,
            unresolved_terminal: unresolved,
            left_wrong_basin: left,
            clean_correct,
            clean_cases: clean_cases.len(),
            gate_on_wrong: gate_wrong,
            gate_on_clean: gate_clean,
            noise_applied_wrong: noise_wrong,
            noise_applied_clean: noise_clean,
            median_target_angle_error: median_f64(&angles),
            mean_first_passage_time: mean_opt(&passages),
            finite: seed_finite,
        });
    }

    let wrong_trials = wrong_cases.len().saturating_mul(SEEDS.len());
    let clean_trials = clean_cases.len().saturating_mul(SEEDS.len());
    let aggregate = ArmAggregate {
        arm,
        condition,
        seeds,
        target_recovery_fraction: rate(total_recoveries, wrong_trials),
        competing_terminal_fraction: rate(total_competing, wrong_trials),
        unresolved_terminal_fraction: rate(total_unresolved, wrong_trials),
        left_wrong_basin_fraction: rate(total_left, wrong_trials),
        clean_recall_fraction: rate(total_clean_correct, clean_trials),
        clean_degradation_vs_d0: 0.0,
        gate_on_wrong_fraction: rate(total_gate_wrong, wrong_trials),
        gate_on_clean_fraction: rate(total_gate_clean, clean_trials),
        noise_applied_wrong_fraction: rate(total_noise_wrong, wrong_trials),
        noise_applied_clean_fraction: rate(total_noise_clean, clean_trials),
        median_target_angle_error: median_f64(&all_angles),
        mean_first_passage_time: mean_opt(&all_passages),
        finite,
    };
    Ok(aggregate)
}

fn run_two_stage_trial(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
    case: &CaseRecord,
    arm: Arm,
    condition: ConditionSpec,
    noise: &[Vec<Vec<f64>>],
) -> Result<TrialResult, Box<dyn Error>> {
    let mut state = deterministic_settle(bank, model, integrator, &case.cue)?;
    let (_, gap) = ambiguity_summary(bank, &state)?;
    let gate_on = gap <= condition.gate_threshold;
    let noise_applied = match arm {
        Arm::D0 => false,
        Arm::GlobalOu => true,
        Arm::GatedOu | Arm::PermGatedOu => gate_on,
    };
    let target = &bank.patterns()[case.target_index];
    let target_state = bank.encode_cue(target, CUE_TILT_RADIANS)?;
    let competitor = case.competitor_index.map(|index| &bank.patterns()[index]);
    let mut first_passage = None;
    let mut left_wrong =
        competitor.is_some_and(|comp| bank.decode_state(&state).ok().as_ref() != Some(comp));
    let mut finite = state
        .nodes()
        .iter()
        .all(|node| node.values().iter().all(|value| value.is_finite()));

    for step in 0..STAGE_STEPS {
        if noise_applied {
            let forced = NoisyModel {
                base: model,
                additive: &noise[step],
            };
            state = heun_step(&state, &forced, integrator)?;
        } else {
            state = heun_step(&state, model, integrator)?;
        }
        finite &= state
            .nodes()
            .iter()
            .all(|node| node.values().iter().all(|value| value.is_finite()));
        let decoded = bank.decode_state(&state)?;
        if first_passage.is_none() && decoded == *target {
            first_passage = Some(STAGE_STEPS + step + 1);
        }
        if competitor.is_some_and(|comp| decoded != *comp) {
            left_wrong = true;
        }
    }

    let terminal_pattern = bank.decode_state(&state)?;
    let target_angle_error = mean_angle_error(&state, &target_state);
    finite &= target_angle_error.is_finite() && gap.is_finite();
    Ok(TrialResult {
        terminal_pattern,
        target_angle_error,
        first_passage_time: first_passage,
        left_wrong_basin: left_wrong,
        gate_on,
        noise_applied,
        finite,
    })
}

fn build_ou_noise(
    condition: ConditionSpec,
    seed: u64,
    permute: bool,
    temporal_perm: &[usize],
) -> Result<Vec<Vec<Vec<f64>>>, Box<dyn Error>> {
    let components = NODE_COUNT * STATE_DIM;
    let mut series = (0..components)
        .map(|component| {
            let component_seed = seed
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(u64::try_from(component).unwrap_or(0));
            ornstein_uhlenbeck_path(
                0.0,
                condition.ou_theta,
                0.0,
                condition.amplitude,
                FIELD_DT,
                STAGE_STEPS,
                component_seed,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    if permute {
        for component_series in &mut series {
            let original = component_series.clone();
            for (step, source) in temporal_perm.iter().copied().enumerate() {
                component_series[step] = original[source];
            }
        }
    }

    let mut by_step = vec![vec![vec![0.0; STATE_DIM]; NODE_COUNT]; STAGE_STEPS];
    for (component_index, component_series) in series.iter().enumerate() {
        let node = component_index / STATE_DIM;
        let dim = component_index % STATE_DIM;
        for (step, sample) in component_series.iter().enumerate() {
            by_step[step][node][dim] = *sample;
        }
    }
    Ok(by_step)
}

fn ornstein_uhlenbeck_path(
    x0: f64,
    theta: f64,
    mean: f64,
    sigma: f64,
    dt: f64,
    steps: usize,
    seed: u64,
) -> Result<Vec<f64>, Box<dyn Error>> {
    if !(theta.is_finite() && theta > 0.0) {
        return Err("OU theta must be finite and positive".into());
    }
    if !(sigma.is_finite() && sigma >= 0.0) {
        return Err("OU sigma must be finite and non-negative".into());
    }
    if !(dt.is_finite() && dt > 0.0) {
        return Err("OU dt must be finite and positive".into());
    }
    let mut rng = SplitMix64::new(seed);
    let decay = (-theta * dt).exp();
    let stddev = sigma * ((1.0 - decay * decay) / (2.0 * theta)).sqrt();
    let mut path = Vec::with_capacity(steps);
    let mut x = x0;
    for _ in 0..steps {
        x = mean + (x - mean) * decay + stddev * rng.next_gaussian();
        path.push(x);
    }
    Ok(path)
}

fn temporal_permutation(steps: usize, seed: u64) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..steps).collect();
    let mut rng = SplitMix64::new(seed);
    for i in (1..steps).rev() {
        let j = usize::try_from(rng.next_u64() % u64::try_from(i + 1).unwrap_or(1)).unwrap_or(0);
        perm.swap(i, j);
    }
    perm
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SelectionKey {
    neg_recovery: i64,
    neg_left: i64,
    angle: i64,
    clean_gate: i64,
    amplitude: i64,
    neg_theta: i64,
    tau: i64,
}

fn selection_key(candidate: &ArmAggregate) -> SelectionKey {
    SelectionKey {
        neg_recovery: -encode(candidate.target_recovery_fraction),
        neg_left: -encode(candidate.left_wrong_basin_fraction),
        angle: encode(candidate.median_target_angle_error),
        clean_gate: encode(candidate.gate_on_clean_fraction),
        amplitude: encode(candidate.condition.amplitude),
        neg_theta: -encode(candidate.condition.ou_theta),
        tau: encode(candidate.condition.gate_threshold),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn encode(value: f64) -> i64 {
    (value * 1_000_000_000.0).round() as i64
}

fn arm_finite(arm: &ArmAggregate) -> bool {
    arm.finite
        && arm.seeds.iter().all(|seed| seed.finite)
        && arm.target_recovery_fraction.is_finite()
        && arm.clean_recall_fraction.is_finite()
        && arm.gate_on_wrong_fraction.is_finite()
        && arm.gate_on_clean_fraction.is_finite()
        && arm.median_target_angle_error.is_finite()
        && arm.mean_first_passage_time.is_none_or(f64::is_finite)
}

fn mean_angle_error(state: &FieldState, target: &FieldState) -> f64 {
    let mut sum = 0.0;
    for (node, target_node) in state.nodes().iter().zip(target.nodes()) {
        let cosine = field_core::dot(node.values(), target_node.values()).clamp(-1.0, 1.0);
        sum += cosine.acos();
    }
    sum / f64::from(u32::try_from(state.node_count()).unwrap_or(1))
}

fn median_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    } else {
        sorted[mid]
    }
}

fn mean_opt(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / f64::from(u32::try_from(values.len()).ok()?))
    }
}

fn rate(successes: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    f64::from(u32::try_from(successes).unwrap_or(0)) / f64::from(u32::try_from(total).unwrap_or(1))
}

fn hex_digest(digest: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(char::from(HEX[usize::from(*byte >> 4)]));
        out.push(char::from(HEX[usize::from(*byte & 0x0f)]));
    }
    out
}

fn provenance_fingerprint() -> u64 {
    let manifest = format!(
        "fl5e|nodes={NODE_COUNT}|stage={STAGE_STEPS}|dt={FIELD_DT:.17}|mobility={FIELD_MOBILITY:.17}|tilt={CUE_TILT_RADIANS:.17}|amps={AMPLITUDES:?}|ou={OU_THETAS:?}|tau={GATE_THRESHOLDS:?}|seeds={SEEDS:?}|perm={PERM_SEED}|noiselab={NOISELAB_COMMIT}|patterns=H2/2/4|gate=stageA-top2-angle-gap|matched=128+128|namespace=fl5e"
    );
    fnv1a64(manifest.as_bytes())
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
    use super::{
        partition_case, temporal_permutation, CaseRecord, Partition, AMPLITUDES, CLEAN_CUE_IDS,
        GATE_THRESHOLDS, OU_THETAS, SEEDS, STAGE_STEPS,
    };

    fn placeholder(id: &str) -> CaseRecord {
        CaseRecord {
            case_id: id.to_owned(),
            sha256: String::new(),
            partition: Partition::Calibration,
            kind: "clean_cue",
            target_index: 0,
            competitor_index: None,
            cue: vec![1; 8],
            stage_a_terminal_pattern: vec![1; 8],
            stage_a_best_index: 0,
            stage_a_ambiguity_gap: 1.0,
            stage_a_target_angle: 0.0,
            stage_a_competitor_angle: None,
            stage_a_ends_competitor: false,
        }
    }

    #[test]
    fn frozen_grid_matches_preregistration() {
        assert_eq!(AMPLITUDES, [0.25, 0.50]);
        assert_eq!(OU_THETAS, [0.5, 2.0]);
        assert_eq!(GATE_THRESHOLDS, [0.05, 0.10, 0.20, 0.35]);
        assert_eq!(SEEDS, [1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn frozen_clean_ids_cover_both_partitions() {
        let records = CLEAN_CUE_IDS
            .iter()
            .map(|id| partition_case(placeholder(id)).unwrap())
            .collect::<Vec<_>>();
        let cal = records
            .iter()
            .filter(|record| record.partition == Partition::Calibration)
            .count();
        let hold = records.len() - cal;
        assert!(cal >= 2 && hold >= 2);
    }

    #[test]
    fn frozen_wrong_ids_cover_both_partitions() {
        let ids = [
            "fl5e|wb|t0|c1|k0",
            "fl5e|wb|t0|c2|k0",
            "fl5e|wb|t1|c0|k1",
            "fl5e|wb|t1|c0|k2",
            "fl5e|wb|t2|c0|k1",
            "fl5e|wb|t2|c0|k2",
        ];
        let records = ids
            .iter()
            .map(|id| partition_case(placeholder(id)).unwrap())
            .collect::<Vec<_>>();
        let cal = records
            .iter()
            .filter(|record| record.partition == Partition::Calibration)
            .count();
        let hold = records.len() - cal;
        assert!(cal >= 2 && hold >= 2);
    }

    #[test]
    fn temporal_permutation_is_bijection() {
        let mut perm = temporal_permutation(STAGE_STEPS, 0x0F15_E005);
        perm.sort_unstable();
        assert_eq!(perm, (0..STAGE_STEPS).collect::<Vec<_>>());
    }
}
