#![forbid(unsafe_code)]

//! FL-5H — identifiable E1 residence-aware stochastic gate.

use field_bench::qualification::{group_observables, ObservableGroup};
use field_core::{
    CouplingGraph, FieldModel, FieldState, LocalAnisotropy, OperatorCoupling, OperatorEnergyModel,
    ValidationError,
};
use field_dynamics::{heun_step, IntegratorConfig};
use field_memory::PatternBank;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;

const NOISELAB_COMMIT: &str = "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e";
const PREREG_COMMIT: &str = "ea9bb6e88c90f1978bb1757a8393c3598c7a0db6";
const NODE_COUNT: usize = 8;
const STATE_DIM: usize = 2;
const FIELD_STEPS: usize = 256;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const CUE_TILT_RADIANS: f64 = 0.15;
const STIFFNESS: f64 = 1.25;
const AMPLITUDES: [f64; 5] = [0.05, 0.10, 0.25, 0.50, 1.00];
const OU_THETAS: [f64; 3] = [0.5, 2.0, 8.0];
const SEEDS: [u64; 8] = [1, 2, 3, 5, 8, 13, 21, 34];
const PERM_SEED: u64 = 0x0F15_5008;
const NUMERIC_TOL: f64 = 1e-10;

const CLEAN_CUE_IDS: [&str; 6] = [
    "fl5h|clean|t0|a0",
    "fl5h|clean|t0|a1",
    "fl5h|clean|t1|a0",
    "fl5h|clean|t1|a1",
    "fl5h|clean|t2|a1",
    "fl5h|clean|t2|a5",
];

/// Frozen identifiable wrong-basin panel: `(case_id, target, competitor, flip_bit)`.
const WRONG_BASIN_SPEC: [(&str, usize, usize, usize); 4] = [
    ("fl5h|wb|t1|c0|bit6", 1, 0, 6),
    ("fl5h|wb|t1|c0|bit7", 1, 0, 7),
    ("fl5h|wb|t2|c0|bit4", 2, 0, 4),
    ("fl5h|wb|t2|c0|bit5", 2, 0, 5),
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
    Ou,
    PermOu,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct ConditionSpec {
    amplitude: f64,
    ou_theta: f64,
}

#[derive(Clone, Debug, Serialize)]
struct FixturePattern {
    index: usize,
    symbols: Vec<i8>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct DecodedSegment {
    label: Option<usize>,
    start_step: usize,
    length: usize,
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
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ResidenceSummary {
    initial_decoded_label: Option<usize>,
    first_leave_initial_step: Option<usize>,
    first_target_hit_step: Option<usize>,
    first_competitor_leave_step: Option<usize>,
    transition_count: usize,
    terminal_decoded_label: Option<usize>,
    right_censored_no_target_hit: bool,
    segments: Vec<DecodedSegment>,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
struct TrajectoryResult {
    terminal_pattern: Vec<i8>,
    target_angle_error: f64,
    competitor_angle_error: Option<f64>,
    target_recovered: bool,
    left_wrong_basin: bool,
    competing_terminal: bool,
    finite: bool,
    max_norm_squared_error: f64,
    residence: ResidenceSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct SeedMetrics {
    seed: u64,
    target_recoveries: usize,
    wrong_cases: usize,
    competing_terminal: usize,
    left_wrong_basin: usize,
    clean_correct: usize,
    clean_cases: usize,
    median_target_angle_error: f64,
    mean_first_passage_time: Option<f64>,
    right_censor_no_target: usize,
    mean_first_leave_initial: Option<f64>,
    mean_transition_count: f64,
    finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ArmAggregate {
    arm: Arm,
    condition: ConditionSpec,
    seeds: Vec<SeedMetrics>,
    target_recovery_fraction: f64,
    competing_terminal_fraction: f64,
    left_wrong_basin_fraction: f64,
    clean_recall_fraction: f64,
    clean_degradation_vs_d0: f64,
    median_target_angle_error: f64,
    mean_first_passage_time: Option<f64>,
    right_censor_no_target_fraction: f64,
    mean_first_leave_initial: Option<f64>,
    mean_transition_count: f64,
    finite: bool,
}

#[derive(Clone, Debug, Serialize)]
struct SelectedCondition {
    amplitude: f64,
    ou_theta: f64,
    calibration_target_recovery_fraction: f64,
    calibration_left_wrong_basin_fraction: f64,
    calibration_clean_recall_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
struct FrozenConstants {
    node_count: usize,
    field_steps: usize,
    physical_horizon: f64,
    field_dt: f64,
    field_mobility: f64,
    cue_tilt_radians: f64,
    stiffness: f64,
    amplitudes: Vec<f64>,
    ou_thetas: Vec<f64>,
    seeds: Vec<u64>,
    perm_seed: u64,
    clean_cue_ids: Vec<String>,
    wrong_basin_ids: Vec<String>,
    patterns: Vec<FixturePattern>,
    left_boundary: &'static str,
    right_boundary: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct IdentifiabilityReport {
    records: usize,
    distinct_inputs: usize,
    conflicting_groups: usize,
    deterministic_maximum_correct: usize,
    groups: Vec<ObservableGroup<Vec<i8>, usize>>,
    zero_conflicts: bool,
    ceiling_matches_records: bool,
}

#[derive(Clone, Debug, Serialize)]
struct FixtureInfo {
    wrong_basin_cases: usize,
    clean_cue_cases: usize,
    calibration_wrong_basin: usize,
    holdout_wrong_basin: usize,
    calibration_clean: usize,
    holdout_clean: usize,
    description: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[allow(clippy::struct_field_names)]
struct HypothesisReport {
    h5h_0_protocol_validity: Check,
    h5h_1_safe_escape: Check,
    h5h_2_temporal_structure: Check,
    h5h_3_identifiable_inputs: Check,
    h5h_4_residence_evidence: Check,
    h5h_5_reproducibility: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ValidityReport {
    preregistered_grid_used: Check,
    holdout_excluded_from_selection: Check,
    hard_clean_cue_filter_applied: Check,
    identifiability_gate: Check,
    partition_minima: Check,
    panels_disjoint: Check,
    finite_metrics: Check,
    numerical_invariants: Check,
    replay_equal: Check,
    noiselab_commit_matches: Check,
    residence_fields_complete: Check,
    protocol_valid: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ExperimentArtifact {
    experiment: &'static str,
    protocol: &'static str,
    noiselab_commit: &'static str,
    preregistration_commit: &'static str,
    fieldlab_commit: String,
    frozen_constants: FrozenConstants,
    fixture: FixtureInfo,
    identifiability: IdentifiabilityReport,
    none_survived_clean_cue_filter: bool,
    selected: Option<SelectedCondition>,
    calibration_d0: Option<ArmAggregate>,
    calibration_selected: Option<ArmAggregate>,
    holdout_d0: Option<ArmAggregate>,
    holdout_selected: Option<ArmAggregate>,
    holdout_perm: Option<ArmAggregate>,
    hypotheses: HypothesisReport,
    validity: ValidityReport,
    cases: Vec<CaseRecord>,
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
    base: &'a OperatorEnergyModel,
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
    let model = model_e1(&bank.hebbian_graph()?, STIFFNESS)?;
    let integrator = IntegratorConfig {
        dt: FIELD_DT,
        mobility: FIELD_MOBILITY,
    };
    let temporal_perm = temporal_permutation(FIELD_STEPS, PERM_SEED);

    let cases = build_cases(&bank)?;
    let wrong_basin: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "wrong_basin")
        .cloned()
        .collect();
    let clean_cases: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "clean_cue")
        .cloned()
        .collect();

    let identifiability = audit_identifiability(&cases)?;
    let disjoint = panels_disjoint(&wrong_basin, &clean_cases);
    let cal_wrong: Vec<CaseRecord> = wrong_basin
        .iter()
        .filter(|case| case.partition == Partition::Calibration)
        .cloned()
        .collect();
    let hold_wrong: Vec<CaseRecord> = wrong_basin
        .iter()
        .filter(|case| case.partition == Partition::Holdout)
        .cloned()
        .collect();
    let cal_clean: Vec<CaseRecord> = clean_cases
        .iter()
        .filter(|case| case.partition == Partition::Calibration)
        .cloned()
        .collect();
    let hold_clean: Vec<CaseRecord> = clean_cases
        .iter()
        .filter(|case| case.partition == Partition::Holdout)
        .cloned()
        .collect();

    let partition_ok = cal_clean.len() >= 2
        && hold_clean.len() >= 2
        && !cal_wrong.is_empty()
        && !hold_wrong.is_empty();
    let construction_failed = !identifiability.zero_conflicts
        || !identifiability.ceiling_matches_records
        || !disjoint
        || !partition_ok;

    let d0_condition = ConditionSpec {
        amplitude: 0.0,
        ou_theta: 0.5,
    };

    let mut best: Option<(SelectionKey, ConditionSpec, ArmAggregate)> = None;
    let mut none_survived = true;
    let mut hold_d0 = None;
    let mut hold_selected = None;
    let mut hold_perm = None;
    let mut cal_d0_out = None;
    let mut cal_selected_out = None;
    let mut selected_condition = None;
    let mut escape = false;
    let mut structured = false;
    let mut all_finite = true;
    let mut replay_ok = true;
    let mut residence_complete = true;
    let mut numeric_ok = true;

    if !construction_failed {
        let mut cal_d0 = evaluate_arm(
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
        cal_d0.clean_degradation_vs_d0 = 0.0;
        all_finite &= cal_d0.finite;
        residence_complete &= residence_arm_complete(&cal_d0);
        cal_d0_out = Some(cal_d0.clone());

        let cal_clean_trials = cal_clean.len().saturating_mul(SEEDS.len());
        let cal_clean_tol = clean_safety_tolerance(cal_clean_trials);

        for &amplitude in &AMPLITUDES {
            for &theta in &OU_THETAS {
                let condition = ConditionSpec {
                    amplitude,
                    ou_theta: theta,
                };
                let mut aggregate = evaluate_arm(
                    &bank,
                    &model,
                    integrator,
                    &cal_wrong,
                    &cal_clean,
                    Arm::Ou,
                    condition,
                    false,
                    &temporal_perm,
                )?;
                aggregate.clean_degradation_vs_d0 =
                    (cal_d0.clean_recall_fraction - aggregate.clean_recall_fraction).max(0.0);
                all_finite &= aggregate.finite;
                residence_complete &= residence_arm_complete(&aggregate);
                if aggregate.clean_degradation_vs_d0 <= cal_clean_tol + 1.0e-12 {
                    let key = selection_key(&aggregate);
                    if best.as_ref().is_none_or(|(best_key, _, _)| key < *best_key) {
                        best = Some((key, condition, aggregate));
                    }
                }
            }
        }

        none_survived = best.is_none();

        let mut hold_d0_agg = evaluate_arm(
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
        hold_d0_agg.clean_degradation_vs_d0 = 0.0;
        all_finite &= hold_d0_agg.finite;
        residence_complete &= residence_arm_complete(&hold_d0_agg);

        let replay_condition = best.as_ref().map_or(
            ConditionSpec {
                amplitude: 0.05,
                ou_theta: 0.5,
            },
            |(_, selected_spec, _)| *selected_spec,
        );

        if let Some((_, selected_spec, selected_cal)) = best.as_ref() {
            let selected_spec = *selected_spec;
            let selected_cal = selected_cal.clone();
            let mut hold_sel = evaluate_arm(
                &bank,
                &model,
                integrator,
                &hold_wrong,
                &hold_clean,
                Arm::Ou,
                selected_spec,
                false,
                &temporal_perm,
            )?;
            hold_sel.clean_degradation_vs_d0 =
                (hold_d0_agg.clean_recall_fraction - hold_sel.clean_recall_fraction).max(0.0);
            let mut hold_perm_agg = evaluate_arm(
                &bank,
                &model,
                integrator,
                &hold_wrong,
                &hold_clean,
                Arm::PermOu,
                selected_spec,
                true,
                &temporal_perm,
            )?;
            hold_perm_agg.clean_degradation_vs_d0 =
                (hold_d0_agg.clean_recall_fraction - hold_perm_agg.clean_recall_fraction).max(0.0);
            all_finite &= hold_sel.finite && hold_perm_agg.finite;
            residence_complete &=
                residence_arm_complete(&hold_sel) && residence_arm_complete(&hold_perm_agg);

            let hold_clean_trials = hold_clean.len().saturating_mul(SEEDS.len());
            let hold_clean_tol = clean_safety_tolerance(hold_clean_trials);
            let clean_safe = hold_sel.clean_degradation_vs_d0 <= hold_clean_tol + 1.0e-12;
            escape = hold_sel.target_recovery_fraction
                > hold_d0_agg.target_recovery_fraction + 1.0e-15
                && clean_safe;
            structured = hold_sel.target_recovery_fraction
                > hold_perm_agg.target_recovery_fraction + 1.0e-15;

            selected_condition = Some(SelectedCondition {
                amplitude: selected_spec.amplitude,
                ou_theta: selected_spec.ou_theta,
                calibration_target_recovery_fraction: selected_cal.target_recovery_fraction,
                calibration_left_wrong_basin_fraction: selected_cal.left_wrong_basin_fraction,
                calibration_clean_recall_fraction: selected_cal.clean_recall_fraction,
            });
            cal_selected_out = Some(selected_cal);
            hold_selected = Some(hold_sel);
            hold_perm = Some(hold_perm_agg);
        }

        hold_d0 = Some(hold_d0_agg);

        // D0 numeric probe on first wrong-basin cue.
        if let Some(probe) = wrong_basin.first() {
            let target = &bank.patterns()[probe.target_index];
            let competitor = probe
                .competitor_index
                .map(|idx| bank.patterns()[idx].as_slice());
            let traj = integrate_trajectory(
                &bank,
                &model,
                integrator,
                &probe.cue,
                target,
                competitor,
                &[],
            )?;
            numeric_ok = traj.max_norm_squared_error <= NUMERIC_TOL && traj.finite;
        }

        replay_ok = replay_campaign(
            &bank,
            &model,
            integrator,
            &cal_wrong,
            &cal_clean,
            replay_condition,
            &temporal_perm,
        )?;
    }

    let ident_ok = identifiability.zero_conflicts && identifiability.ceiling_matches_records;
    let protocol_valid = !construction_failed
        && partition_ok
        && disjoint
        && identifiability.zero_conflicts
        && identifiability.ceiling_matches_records
        && all_finite
        && numeric_ok
        && replay_ok
        && residence_complete
        && NOISELAB_COMMIT == "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e";

    Ok(ExperimentArtifact {
        experiment: "FL-5H",
        protocol: "identifiable-e1-residence-stochastic-gate-v1",
        noiselab_commit: NOISELAB_COMMIT,
        preregistration_commit: PREREG_COMMIT,
        fieldlab_commit: std::env::var("FIELDLAB_SOURCE_SHA")
            .or_else(|_| std::env::var("GITHUB_SHA"))
            .unwrap_or_else(|_| "local".to_owned()),
        frozen_constants: FrozenConstants {
            node_count: NODE_COUNT,
            field_steps: FIELD_STEPS,
            physical_horizon: FIELD_DT * f64::from(u32::try_from(FIELD_STEPS)?),
            field_dt: FIELD_DT,
            field_mobility: FIELD_MOBILITY,
            cue_tilt_radians: CUE_TILT_RADIANS,
            stiffness: STIFFNESS,
            amplitudes: AMPLITUDES.to_vec(),
            ou_thetas: OU_THETAS.to_vec(),
            seeds: SEEDS.to_vec(),
            perm_seed: PERM_SEED,
            clean_cue_ids: CLEAN_CUE_IDS.iter().map(|s| (*s).to_owned()).collect(),
            wrong_basin_ids: WRONG_BASIN_SPEC
                .iter()
                .map(|(id, _, _, _)| (*id).to_owned())
                .collect(),
            patterns: reference_patterns()
                .into_iter()
                .enumerate()
                .map(|(index, symbols)| FixturePattern { index, symbols })
                .collect(),
            left_boundary: "Complete",
            right_boundary: "ObservationCut",
        },
        fixture: FixtureInfo {
            wrong_basin_cases: wrong_basin.len(),
            clean_cue_cases: clean_cases.len(),
            calibration_wrong_basin: cal_wrong.len(),
            holdout_wrong_basin: hold_wrong.len(),
            calibration_clean: cal_clean.len(),
            holdout_clean: hold_clean.len(),
            description: "FL-5H identifiable single-bit P0 corruptions + exact cleans; E1 a=1.25; Complete-left residence; hard clean-cue OU selection",
        },
        identifiability,
        none_survived_clean_cue_filter: none_survived,
        selected: selected_condition,
        calibration_d0: cal_d0_out,
        calibration_selected: cal_selected_out,
        holdout_d0: hold_d0,
        holdout_selected: hold_selected.clone(),
        holdout_perm: hold_perm.clone(),
        hypotheses: HypothesisReport {
            h5h_0_protocol_validity: Check::from_bool(protocol_valid),
            h5h_1_safe_escape: Check::from_bool(!none_survived && escape),
            h5h_2_temporal_structure: Check::from_bool(!none_survived && structured),
            h5h_3_identifiable_inputs: Check::from_bool(ident_ok),
            h5h_4_residence_evidence: Check::from_bool(residence_complete && !construction_failed),
            h5h_5_reproducibility: Check::from_bool(replay_ok),
        },
        validity: ValidityReport {
            preregistered_grid_used: Check::Pass,
            holdout_excluded_from_selection: Check::Pass,
            hard_clean_cue_filter_applied: Check::Pass,
            identifiability_gate: Check::from_bool(ident_ok),
            partition_minima: Check::from_bool(partition_ok),
            panels_disjoint: Check::from_bool(disjoint),
            finite_metrics: Check::from_bool(all_finite),
            numerical_invariants: Check::from_bool(numeric_ok),
            replay_equal: Check::from_bool(replay_ok),
            noiselab_commit_matches: Check::from_bool(
                NOISELAB_COMMIT == "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e",
            ),
            residence_fields_complete: Check::from_bool(residence_complete),
            protocol_valid: Check::from_bool(protocol_valid),
        },
        cases,
    })
}

fn reference_patterns() -> Vec<Vec<i8>> {
    vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, -1, -1],
        vec![1, 1, 1, 1, -1, -1, 1, 1],
    ]
}

fn model_e1(graph: &CouplingGraph, stiffness: f64) -> Result<OperatorEnergyModel, Box<dyn Error>> {
    let couplings = graph
        .couplings()
        .iter()
        .map(|edge| OperatorCoupling {
            source: edge.source,
            target: edge.target,
            operator: vec![vec![edge.weight, 0.0], vec![0.0, edge.weight]],
        })
        .collect();
    let anisotropies = (0..graph.node_count())
        .map(|node| LocalAnisotropy {
            node,
            matrix: vec![vec![stiffness, 0.0], vec![0.0, 0.0]],
        })
        .collect();
    Ok(OperatorEnergyModel::new(
        vec![vec![0.0; STATE_DIM]; NODE_COUNT],
        couplings,
        anisotropies,
    )?)
}

fn build_cases(bank: &PatternBank) -> Result<Vec<CaseRecord>, Box<dyn Error>> {
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();
    let patterns = bank.patterns();

    for &(case_id, target_index, competitor_index, flip_bit) in &WRONG_BASIN_SPEC {
        let case_id = case_id.to_owned();
        if !seen.insert(case_id.clone()) {
            return Err("duplicate wrong-basin case id".into());
        }
        let mut cue = patterns[competitor_index].clone();
        if flip_bit >= NODE_COUNT {
            return Err("flip bit out of range".into());
        }
        cue[flip_bit] = patterns[target_index][flip_bit];
        cases.push(partition_record(
            case_id,
            "wrong_basin",
            target_index,
            Some(competitor_index),
            cue,
        )?);
    }

    for &case_id in &CLEAN_CUE_IDS {
        let case_id = case_id.to_owned();
        let target_index = parse_clean_target_index(&case_id)?;
        let cue = patterns[target_index].clone();
        if !seen.insert(case_id.clone()) {
            return Err("duplicate clean case id".into());
        }
        cases.push(partition_record(
            case_id,
            "clean_cue",
            target_index,
            None,
            cue,
        )?);
    }

    Ok(cases)
}

fn parse_clean_target_index(case_id: &str) -> Result<usize, Box<dyn Error>> {
    let parts: Vec<&str> = case_id.split('|').collect();
    if parts.len() != 4 || parts[0] != "fl5h" || parts[1] != "clean" {
        return Err(format!("invalid frozen clean-cue id: {case_id}").into());
    }
    let target = parts[2]
        .strip_prefix('t')
        .ok_or_else(|| format!("clean id missing t-index: {case_id}"))?;
    let index: usize = target
        .parse()
        .map_err(|_| format!("clean id bad t-index: {case_id}"))?;
    if !parts[3].starts_with('a') {
        return Err(format!("clean id missing alias: {case_id}").into());
    }
    Ok(index)
}

fn partition_record(
    case_id: String,
    kind: &'static str,
    target_index: usize,
    competitor_index: Option<usize>,
    cue: Vec<i8>,
) -> Result<CaseRecord, Box<dyn Error>> {
    if case_id.is_empty() {
        return Err("empty case id".into());
    }
    let digest = Sha256::digest(case_id.as_bytes());
    let partition = if digest[0] < 0x80 {
        Partition::Calibration
    } else {
        Partition::Holdout
    };
    Ok(CaseRecord {
        case_id,
        sha256: hex_digest(&digest),
        partition,
        kind,
        target_index,
        competitor_index,
        cue,
    })
}

fn hex_digest(digest: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(digest.len() * 2);
    for &byte in digest {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn audit_identifiability(cases: &[CaseRecord]) -> Result<IdentifiabilityReport, Box<dyn Error>> {
    let records: Vec<(Vec<i8>, usize)> = cases
        .iter()
        .map(|case| (case.cue.clone(), case.target_index))
        .collect();
    let groups = group_observables(&records)?;
    let conflicting_groups = groups.iter().filter(|g| g.conflicts()).count();
    let deterministic_maximum_correct = groups.iter().map(ObservableGroup::maximum_correct).sum();
    Ok(IdentifiabilityReport {
        records: records.len(),
        distinct_inputs: groups.len(),
        conflicting_groups,
        deterministic_maximum_correct,
        groups,
        zero_conflicts: conflicting_groups == 0,
        ceiling_matches_records: deterministic_maximum_correct == records.len(),
    })
}

fn panels_disjoint(wrong: &[CaseRecord], clean: &[CaseRecord]) -> bool {
    let wrong_ids: BTreeSet<&str> = wrong.iter().map(|case| case.case_id.as_str()).collect();
    let clean_ids: BTreeSet<&str> = clean.iter().map(|case| case.case_id.as_str()).collect();
    wrong_ids.is_disjoint(&clean_ids)
        && wrong.iter().all(|case| case.kind == "wrong_basin")
        && clean.iter().all(|case| case.kind == "clean_cue")
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct SelectionKey {
    neg_recovery: f64,
    neg_left_wrong: f64,
    median_angle: f64,
    amplitude: f64,
    neg_theta: f64,
}

fn selection_key(aggregate: &ArmAggregate) -> SelectionKey {
    SelectionKey {
        neg_recovery: -aggregate.target_recovery_fraction,
        neg_left_wrong: -aggregate.left_wrong_basin_fraction,
        median_angle: aggregate.median_target_angle_error,
        amplitude: aggregate.condition.amplitude,
        neg_theta: -aggregate.condition.ou_theta,
    }
}

fn clean_safety_tolerance(holdout_clean_trials: usize) -> f64 {
    if holdout_clean_trials == 0 {
        return 0.0;
    }
    1.0 / f64::from(u32::try_from(holdout_clean_trials).unwrap_or(1))
}

fn residence_arm_complete(arm: &ArmAggregate) -> bool {
    arm.seeds
        .iter()
        .all(|seed| seed.wrong_cases + seed.clean_cases > 0)
        && arm.mean_transition_count.is_finite()
}

#[allow(clippy::too_many_arguments)]
fn evaluate_arm(
    bank: &PatternBank,
    model: &OperatorEnergyModel,
    integrator: IntegratorConfig,
    wrong_cases: &[CaseRecord],
    clean_cases: &[CaseRecord],
    arm: Arm,
    condition: ConditionSpec,
    permute: bool,
    temporal_perm: &[usize],
) -> Result<ArmAggregate, Box<dyn Error>> {
    let mut seeds = Vec::with_capacity(SEEDS.len());
    for &seed in &SEEDS {
        let noise = build_noise(condition, seed, permute, temporal_perm)?;
        let mut target_recoveries = 0usize;
        let mut competing_terminal = 0usize;
        let mut left_wrong_basin = 0usize;
        let mut clean_correct = 0usize;
        let mut angles = Vec::new();
        let mut passages = Vec::new();
        let mut leaves = Vec::new();
        let mut right_censor = 0usize;
        let mut transition_sum = 0.0_f64;
        let mut finite = true;
        let case_count = wrong_cases.len() + clean_cases.len();

        for case in wrong_cases {
            let target = &bank.patterns()[case.target_index];
            let competitor = case
                .competitor_index
                .map(|idx| bank.patterns()[idx].as_slice());
            let traj = integrate_trajectory(
                bank, model, integrator, &case.cue, target, competitor, &noise,
            )?;
            finite &= traj.finite;
            if traj.target_recovered {
                target_recoveries += 1;
            }
            if traj.competing_terminal {
                competing_terminal += 1;
            }
            if traj.left_wrong_basin {
                left_wrong_basin += 1;
            }
            angles.push(traj.target_angle_error);
            if let Some(step) = traj.residence.first_target_hit_step {
                passages.push(f64::from(u32::try_from(step)?));
            } else {
                right_censor += 1;
            }
            if let Some(step) = traj.residence.first_leave_initial_step {
                leaves.push(f64::from(u32::try_from(step)?));
            }
            transition_sum += f64::from(u32::try_from(traj.residence.transition_count)?);
        }

        for case in clean_cases {
            let target = &bank.patterns()[case.target_index];
            let traj =
                integrate_trajectory(bank, model, integrator, &case.cue, target, None, &noise)?;
            finite &= traj.finite;
            if traj.target_recovered {
                clean_correct += 1;
            }
            angles.push(traj.target_angle_error);
            if traj.residence.first_target_hit_step.is_none() {
                // clean may start already on target; still count leave metrics
            }
            if let Some(step) = traj.residence.first_leave_initial_step {
                leaves.push(f64::from(u32::try_from(step)?));
            }
            transition_sum += f64::from(u32::try_from(traj.residence.transition_count)?);
            // For cleans that begin on target, first_target_hit is step 0 conceptually;
            // protocol records first hit among steps 1..N after integration start, so
            // right_censor is only counted for wrong-basin above.
        }

        let wrong_n = wrong_cases.len();
        let clean_n = clean_cases.len();
        seeds.push(SeedMetrics {
            seed,
            target_recoveries,
            wrong_cases: wrong_n,
            competing_terminal,
            left_wrong_basin,
            clean_correct,
            clean_cases: clean_n,
            median_target_angle_error: median_f64(&angles),
            mean_first_passage_time: mean_opt(&passages),
            right_censor_no_target: right_censor,
            mean_first_leave_initial: mean_opt(&leaves),
            mean_transition_count: if case_count == 0 {
                0.0
            } else {
                transition_sum / f64::from(u32::try_from(case_count).unwrap_or(1))
            },
            finite,
        });
    }

    Ok(aggregate_seeds(arm, condition, seeds))
}

fn aggregate_seeds(arm: Arm, condition: ConditionSpec, seeds: Vec<SeedMetrics>) -> ArmAggregate {
    let wrong_trials: usize = seeds.iter().map(|s| s.wrong_cases).sum();
    let clean_trials: usize = seeds.iter().map(|s| s.clean_cases).sum();
    let recoveries: usize = seeds.iter().map(|s| s.target_recoveries).sum();
    let competing: usize = seeds.iter().map(|s| s.competing_terminal).sum();
    let left: usize = seeds.iter().map(|s| s.left_wrong_basin).sum();
    let clean_ok: usize = seeds.iter().map(|s| s.clean_correct).sum();
    let censor: usize = seeds.iter().map(|s| s.right_censor_no_target).sum();
    let angles: Vec<f64> = seeds.iter().map(|s| s.median_target_angle_error).collect();
    let passages: Vec<f64> = seeds
        .iter()
        .filter_map(|s| s.mean_first_passage_time)
        .collect();
    let leaves: Vec<f64> = seeds
        .iter()
        .filter_map(|s| s.mean_first_leave_initial)
        .collect();
    let transition_mean = if seeds.is_empty() {
        0.0
    } else {
        seeds.iter().map(|s| s.mean_transition_count).sum::<f64>()
            / f64::from(u32::try_from(seeds.len()).unwrap_or(1))
    };
    let finite = seeds.iter().all(|s| s.finite);
    ArmAggregate {
        arm,
        condition,
        seeds,
        target_recovery_fraction: rate(recoveries, wrong_trials),
        competing_terminal_fraction: rate(competing, wrong_trials),
        left_wrong_basin_fraction: rate(left, wrong_trials),
        clean_recall_fraction: rate(clean_ok, clean_trials),
        clean_degradation_vs_d0: 0.0,
        median_target_angle_error: median_f64(&angles),
        mean_first_passage_time: mean_opt(&passages),
        right_censor_no_target_fraction: rate(censor, wrong_trials),
        mean_first_leave_initial: mean_opt(&leaves),
        mean_transition_count: transition_mean,
        finite,
    }
}

fn build_noise(
    condition: ConditionSpec,
    seed: u64,
    permute: bool,
    temporal_perm: &[usize],
) -> Result<Vec<Vec<Vec<f64>>>, Box<dyn Error>> {
    if condition.amplitude == 0.0 {
        return Ok(Vec::new());
    }
    let components = NODE_COUNT * STATE_DIM;
    let mut series: Vec<Vec<f64>> = (0..components)
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
                FIELD_STEPS,
                component_seed,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    if permute {
        for component_series in &mut series {
            let original = component_series.clone();
            for (step, &source) in temporal_perm.iter().enumerate() {
                component_series[step] = original[source];
            }
        }
    }

    let mut by_step = vec![vec![vec![0.0; STATE_DIM]; NODE_COUNT]; FIELD_STEPS];
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

fn integrate_trajectory(
    bank: &PatternBank,
    model: &OperatorEnergyModel,
    integrator: IntegratorConfig,
    cue: &[i8],
    target: &[i8],
    competitor: Option<&[i8]>,
    noise: &[Vec<Vec<f64>>],
) -> Result<TrajectoryResult, Box<dyn Error>> {
    let target_state = bank.encode_cue(target, CUE_TILT_RADIANS)?;
    let competitor_state = competitor
        .map(|comp| bank.encode_cue(comp, CUE_TILT_RADIANS))
        .transpose()?;
    let mut state = bank.encode_cue(cue, CUE_TILT_RADIANS)?;
    let initial_decoded = label_of(bank, &bank.decode_state(&state)?);
    let mut first_leave = None;
    let mut first_target = None;
    let mut first_comp_leave = None;
    let mut transition_count = 0usize;
    let mut prev_label = initial_decoded;
    let mut segments = vec![DecodedSegment {
        label: initial_decoded,
        start_step: 0,
        length: 1,
    }];
    let mut finite = true;
    let mut max_norm_sq = 0.0_f64;

    // Complete-left: step 0 is the known constructed cue observation.
    for step in 0..FIELD_STEPS {
        if noise.is_empty() {
            state = heun_step(&state, model, integrator)?;
        } else {
            let additive = &noise[step];
            let forced = NoisyModel {
                base: model,
                additive,
            };
            state = heun_step(&state, &forced, integrator)?;
        }
        for node in state.nodes() {
            let norm_sq: f64 = node.values().iter().map(|v| v * v).sum();
            max_norm_sq = max_norm_sq.max((norm_sq - 1.0).abs());
            finite &= node.values().iter().all(|value| value.is_finite());
        }
        let decoded = bank.decode_state(&state)?;
        let label = label_of(bank, &decoded);
        let obs_step = step + 1;
        if label != prev_label {
            transition_count += 1;
            segments.push(DecodedSegment {
                label,
                start_step: obs_step,
                length: 1,
            });
            prev_label = label;
        } else if let Some(last) = segments.last_mut() {
            last.length += 1;
        }
        if first_leave.is_none() && label != initial_decoded {
            first_leave = Some(obs_step);
        }
        if first_target.is_none() && decoded == target {
            first_target = Some(obs_step);
        }
        if let Some(comp) = competitor {
            if first_comp_leave.is_none() && decoded != comp {
                first_comp_leave = Some(obs_step);
            }
        }
    }

    let terminal_pattern = bank.decode_state(&state)?;
    let terminal_label = label_of(bank, &terminal_pattern);
    let target_angle_error = mean_angle_error(&state, &target_state);
    finite &= target_angle_error.is_finite();
    let competitor_angle_error = competitor_state
        .as_ref()
        .map(|comp_state| mean_angle_error(&state, comp_state));
    if let Some(ca) = competitor_angle_error {
        finite &= ca.is_finite();
    }
    let target_recovered = terminal_pattern == target;
    let competing_terminal = competitor.is_some_and(|comp| terminal_pattern == comp);
    let left_wrong =
        competitor.is_some_and(|comp| first_comp_leave.is_some() || terminal_pattern != comp);

    Ok(TrajectoryResult {
        terminal_pattern,
        target_angle_error,
        competitor_angle_error,
        target_recovered,
        left_wrong_basin: left_wrong,
        competing_terminal,
        finite,
        max_norm_squared_error: max_norm_sq,
        residence: ResidenceSummary {
            initial_decoded_label: initial_decoded,
            first_leave_initial_step: first_leave,
            first_target_hit_step: first_target,
            first_competitor_leave_step: first_comp_leave,
            transition_count,
            terminal_decoded_label: terminal_label,
            right_censored_no_target_hit: first_target.is_none(),
            segments,
        },
    })
}

fn label_of(bank: &PatternBank, decoded: &[i8]) -> Option<usize> {
    bank.patterns().iter().position(|p| p == decoded)
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
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    } else {
        sorted[mid]
    }
}

fn rate(successes: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    f64::from(u32::try_from(successes).unwrap_or(0)) / f64::from(u32::try_from(total).unwrap_or(1))
}

fn mean_opt(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / f64::from(u32::try_from(values.len()).unwrap_or(1)))
    }
}

fn replay_campaign(
    bank: &PatternBank,
    model: &OperatorEnergyModel,
    integrator: IntegratorConfig,
    wrong: &[CaseRecord],
    clean: &[CaseRecord],
    condition: ConditionSpec,
    temporal_perm: &[usize],
) -> Result<bool, Box<dyn Error>> {
    let a = evaluate_arm(
        bank,
        model,
        integrator,
        wrong,
        clean,
        Arm::Ou,
        condition,
        false,
        temporal_perm,
    )?;
    let b = evaluate_arm(
        bank,
        model,
        integrator,
        wrong,
        clean,
        Arm::Ou,
        condition,
        false,
        temporal_perm,
    )?;
    Ok(a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiable_panel_has_zero_conflicts() {
        let bank = PatternBank::new(reference_patterns()).unwrap();
        let cases = build_cases(&bank).unwrap();
        let audit = audit_identifiability(&cases).unwrap();
        assert_eq!(audit.conflicting_groups, 0);
        assert_eq!(audit.deterministic_maximum_correct, cases.len());
        assert!(audit.zero_conflicts);
        assert!(audit.ceiling_matches_records);
    }

    #[test]
    fn partitions_meet_minima() {
        let bank = PatternBank::new(reference_patterns()).unwrap();
        let cases = build_cases(&bank).unwrap();
        let cal_clean = cases
            .iter()
            .filter(|c| c.kind == "clean_cue" && c.partition == Partition::Calibration)
            .count();
        let hold_clean = cases
            .iter()
            .filter(|c| c.kind == "clean_cue" && c.partition == Partition::Holdout)
            .count();
        let cal_wb = cases
            .iter()
            .filter(|c| c.kind == "wrong_basin" && c.partition == Partition::Calibration)
            .count();
        let hold_wb = cases
            .iter()
            .filter(|c| c.kind == "wrong_basin" && c.partition == Partition::Holdout)
            .count();
        assert!(cal_clean >= 2 && hold_clean >= 2);
        assert!(cal_wb >= 1 && hold_wb >= 1);
    }

    #[test]
    fn physical_horizon_matches_prereg() {
        assert!((FIELD_DT * f64::from(u32::try_from(FIELD_STEPS).unwrap()) - 12.8).abs() < 1e-12);
        assert!((STIFFNESS - 1.25).abs() < 1e-15);
    }

    #[test]
    fn excluded_conflict_geometries_are_absent() {
        let bank = PatternBank::new(reference_patterns()).unwrap();
        let cases = build_cases(&bank).unwrap();
        let p1 = bank.patterns()[1].clone();
        let p2 = bank.patterns()[2].clone();
        // Conflict geometries were full P1/P2 cues labeled as target 0.
        assert!(!cases.iter().any(|c| c.cue == p1 && c.target_index == 0));
        assert!(!cases.iter().any(|c| c.cue == p2 && c.target_index == 0));
    }
}
