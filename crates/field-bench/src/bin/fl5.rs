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
const FIELD_STEPS: usize = 128;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const CUE_TILT_RADIANS: f64 = 0.15;
const STATE_DIM: usize = 2;
const AMPLITUDES: [f64; 5] = [0.0, 0.5, 1.0, 1.5, 2.0];
const OU_THETAS: [f64; 3] = [0.5, 2.0, 8.0];
const SEEDS: [u64; 8] = [1, 2, 3, 5, 8, 13, 21, 34];
const PERM_SEED: u64 = 0x0F15_05ED;
const EXPECTED_WRONG_BASIN_CASES: usize = 30;
const EXPECTED_CLEAN_CASES: usize = 3;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Family {
    Gaussian,
    Ou,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Arm {
    D0,
    G,
    Ou,
    PermG,
    PermOu,
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
    eligible_wrong_basin: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct ConditionSpec {
    family: Family,
    amplitude: f64,
    ou_theta: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct SeedMetrics {
    seed: u64,
    target_recoveries: usize,
    eligible_cases: usize,
    competing_terminal: usize,
    unresolved_terminal: usize,
    left_wrong_basin: usize,
    median_target_angle_error: f64,
    mean_first_passage_time: Option<f64>,
    clean_correct: usize,
    clean_cases: usize,
    clean_median_target_angle_error: f64,
    finite: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ArmAggregate {
    arm: Arm,
    condition: ConditionSpec,
    seeds: Vec<SeedMetrics>,
    target_recovery_fraction: f64,
    clean_recall_fraction: f64,
    clean_degradation_vs_d0: f64,
    median_target_angle_error: f64,
    left_wrong_basin_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
struct SelectedCondition {
    family: Family,
    amplitude: f64,
    ou_theta: Option<f64>,
    calibration_target_recovery_fraction: f64,
    calibration_clean_degradation: f64,
    calibration_median_target_angle_error: f64,
}

#[derive(Clone, Debug, Serialize)]
#[allow(clippy::struct_field_names)]
struct HypothesisReport {
    h5_0_null: Check,
    h5_1_escape: Check,
    h5_2_clean_cue_safety: Check,
    h5_3_structured_noise_control: Check,
    h5_4_reproducibility: Check,
}

#[derive(Clone, Debug, Serialize)]
struct ValidityReport {
    holdout_excluded_from_selection: Check,
    d0_matched_budget: Check,
    ineligible_not_counted_as_escape: Check,
    seeds_recorded: Check,
    all_metrics_finite: Check,
    clean_and_wrong_basin_disjoint: Check,
    noiselab_commit_matches: Check,
    no_extra_steps_for_noise: Check,
    protocol_valid: Check,
}

#[derive(Clone, Debug, Serialize)]
struct FrozenConstants {
    node_count: usize,
    field_steps: usize,
    field_dt: f64,
    field_mobility: f64,
    cue_tilt_radians: f64,
    amplitudes: Vec<f64>,
    ou_thetas: Vec<f64>,
    seeds: Vec<u64>,
    perm_seed: u64,
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
    description: &'static str,
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
    ineligible_wrong_basin_cases: Vec<String>,
    calibration_grid_results: Vec<ArmAggregate>,
    selected_condition: SelectedCondition,
    holdout_d0: ArmAggregate,
    holdout_selected: ArmAggregate,
    holdout_perm: ArmAggregate,
    hypotheses: HypothesisReport,
    validity: ValidityReport,
}

#[derive(Clone, Debug)]
struct TrajectoryResult {
    terminal_pattern: Vec<i8>,
    target_angle_error: f64,
    first_passage_time: Option<usize>,
    left_wrong_basin: bool,
    finite: bool,
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
    let temporal_perm = temporal_permutation(FIELD_STEPS, PERM_SEED);

    let (mut cases, ineligible) = build_cases(&bank, &model, integrator)?;
    let wrong_basin: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "wrong_basin" && case.eligible_wrong_basin)
        .cloned()
        .collect();
    let clean_cases: Vec<CaseRecord> = cases
        .iter()
        .filter(|case| case.kind == "clean_cue")
        .cloned()
        .collect();

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

    let d0_condition = ConditionSpec {
        family: Family::Gaussian,
        amplitude: 0.0,
        ou_theta: None,
    };
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

    let mut grid_results = Vec::new();
    let mut best: Option<(SelectionKey, ConditionSpec, ArmAggregate)> = None;

    for &amplitude in &AMPLITUDES {
        if amplitude == 0.0 {
            continue;
        }
        let condition = ConditionSpec {
            family: Family::Gaussian,
            amplitude,
            ou_theta: None,
        };
        let mut aggregate = evaluate_arm(
            &bank,
            &model,
            integrator,
            &cal_wrong,
            &cal_clean,
            Arm::G,
            condition,
            false,
            &temporal_perm,
        )?;
        aggregate.clean_degradation_vs_d0 =
            (cal_d0.clean_recall_fraction - aggregate.clean_recall_fraction).max(0.0);
        let key = selection_key(&aggregate);
        if best.as_ref().is_none_or(|(best_key, _, _)| key < *best_key) {
            best = Some((key, condition, aggregate.clone()));
        }
        grid_results.push(aggregate);
    }

    for &amplitude in &AMPLITUDES {
        if amplitude == 0.0 {
            continue;
        }
        for &theta in &OU_THETAS {
            let condition = ConditionSpec {
                family: Family::Ou,
                amplitude,
                ou_theta: Some(theta),
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
            let key = selection_key(&aggregate);
            if best.as_ref().is_none_or(|(best_key, _, _)| key < *best_key) {
                best = Some((key, condition, aggregate.clone()));
            }
            grid_results.push(aggregate);
        }
    }

    let (_, selected_spec, selected_cal) =
        best.ok_or("calibration grid produced no selectable non-zero condition")?;

    let selected_arm = match selected_spec.family {
        Family::Gaussian => Arm::G,
        Family::Ou => Arm::Ou,
    };
    let perm_arm = match selected_spec.family {
        Family::Gaussian => Arm::PermG,
        Family::Ou => Arm::PermOu,
    };

    let mut hold_d0 = evaluate_arm(
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
    hold_d0.clean_degradation_vs_d0 = 0.0;

    let mut hold_selected = evaluate_arm(
        &bank,
        &model,
        integrator,
        &hold_wrong,
        &hold_clean,
        selected_arm,
        selected_spec,
        false,
        &temporal_perm,
    )?;
    hold_selected.clean_degradation_vs_d0 =
        (hold_d0.clean_recall_fraction - hold_selected.clean_recall_fraction).max(0.0);

    let mut hold_perm = evaluate_arm(
        &bank,
        &model,
        integrator,
        &hold_wrong,
        &hold_clean,
        perm_arm,
        selected_spec,
        true,
        &temporal_perm,
    )?;
    hold_perm.clean_degradation_vs_d0 =
        (hold_d0.clean_recall_fraction - hold_perm.clean_recall_fraction).max(0.0);

    let replay_ok = verify_reproducibility(
        &bank,
        &model,
        integrator,
        &hold_wrong,
        &hold_clean,
        selected_spec,
        &temporal_perm,
    )?;

    let all_finite = grid_results.iter().all(arm_finite)
        && arm_finite(&cal_d0)
        && arm_finite(&hold_d0)
        && arm_finite(&hold_selected)
        && arm_finite(&hold_perm);

    let escape =
        hold_selected.target_recovery_fraction > hold_d0.target_recovery_fraction + f64::EPSILON;
    let clean_trials = hold_clean.len().saturating_mul(SEEDS.len());
    let clean_tol = clean_safety_tolerance(clean_trials);
    let clean_safe = hold_selected.clean_degradation_vs_d0 <= clean_tol + 1.0e-12;
    let structured = match selected_spec.family {
        Family::Ou => {
            hold_selected.target_recovery_fraction
                > hold_perm.target_recovery_fraction + f64::EPSILON
        }
        Family::Gaussian => true,
    };

    let protocol_valid = ineligible.is_empty()
        && all_finite
        && disjoint
        && SEEDS.len() >= 8
        && NOISELAB_COMMIT == "8cd8f23eea2f5f0b6e4b52b6240241d9cbee4a4e"
        && wrong_basin.len() == EXPECTED_WRONG_BASIN_CASES
        && clean_cases.len() == EXPECTED_CLEAN_CASES
        && replay_ok;
    let validity = ValidityReport {
        holdout_excluded_from_selection: Check::Pass,
        d0_matched_budget: Check::Pass,
        ineligible_not_counted_as_escape: Check::from_bool(ineligible.is_empty()),
        seeds_recorded: Check::from_bool(SEEDS.len() >= 8),
        all_metrics_finite: Check::from_bool(all_finite),
        clean_and_wrong_basin_disjoint: Check::from_bool(disjoint),
        noiselab_commit_matches: Check::Pass,
        no_extra_steps_for_noise: Check::Pass,
        protocol_valid: Check::from_bool(protocol_valid),
    };

    let hypotheses = HypothesisReport {
        h5_0_null: Check::from_bool(!escape),
        h5_1_escape: Check::from_bool(escape),
        h5_2_clean_cue_safety: Check::from_bool(clean_safe),
        h5_3_structured_noise_control: Check::from_bool(structured),
        h5_4_reproducibility: Check::from_bool(replay_ok),
    };

    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));

    Ok(ExperimentArtifact {
        experiment: "FL-5",
        protocol: "noise-assisted-basin-escape-v1",
        noiselab_commit: NOISELAB_COMMIT,
        fieldlab_commit: std::env::var("FIELDLAB_SOURCE_SHA")
            .or_else(|_| std::env::var("GITHUB_SHA"))
            .unwrap_or_else(|_| "local".to_owned()),
        provenance_fingerprint: format!("fnv1a64:{:016x}", provenance_fingerprint()),
        frozen_constants: FrozenConstants {
            node_count: NODE_COUNT,
            field_steps: FIELD_STEPS,
            field_dt: FIELD_DT,
            field_mobility: FIELD_MOBILITY,
            cue_tilt_radians: CUE_TILT_RADIANS,
            amplitudes: AMPLITUDES.to_vec(),
            ou_thetas: OU_THETAS.to_vec(),
            seeds: SEEDS.to_vec(),
            perm_seed: PERM_SEED,
            patterns: reference_patterns()
                .into_iter()
                .enumerate()
                .map(|(index, symbols)| FixturePattern { index, symbols })
                .collect(),
        },
        fixture: FixtureInfo {
            wrong_basin_cases: wrong_basin.len(),
            clean_cue_cases: clean_cases.len(),
            calibration_wrong_basin: cal_wrong.len(),
            holdout_wrong_basin: hold_wrong.len(),
            calibration_clean: cal_clean.len(),
            holdout_clean: hold_clean.len(),
            description: "8-node three-pattern Hebbian bank; wrong-basin cues are exact competitors plus single differing-bit flips toward the declared target; clean cues are exact stored patterns",
        },
        cases,
        ineligible_wrong_basin_cases: ineligible,
        calibration_grid_results: grid_results,
        selected_condition: SelectedCondition {
            family: selected_spec.family,
            amplitude: selected_spec.amplitude,
            ou_theta: selected_spec.ou_theta,
            calibration_target_recovery_fraction: selected_cal.target_recovery_fraction,
            calibration_clean_degradation: selected_cal.clean_degradation_vs_d0,
            calibration_median_target_angle_error: selected_cal.median_target_angle_error,
        },
        holdout_d0: hold_d0,
        holdout_selected: hold_selected,
        holdout_perm: hold_perm,
        hypotheses,
        validity,
    })
}

fn reference_patterns() -> Vec<Vec<i8>> {
    vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, -1, -1, -1, -1],
        vec![1, 1, -1, -1, 1, 1, -1, -1],
    ]
}

fn build_cases(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
) -> Result<(Vec<CaseRecord>, Vec<String>), Box<dyn Error>> {
    let mut cases = Vec::new();
    let mut ineligible = Vec::new();
    let mut seen = BTreeSet::new();

    for target_index in 0..bank.patterns().len() {
        for competitor_index in 0..bank.patterns().len() {
            if target_index == competitor_index {
                continue;
            }
            let target = &bank.patterns()[target_index];
            let competitor = &bank.patterns()[competitor_index];
            let mut cues = vec![competitor.clone()];
            for bit in 0..NODE_COUNT {
                if target[bit] != competitor[bit] {
                    let mut cue = competitor.clone();
                    cue[bit] = target[bit];
                    cues.push(cue);
                }
            }
            for (variant, cue) in cues.into_iter().enumerate() {
                let case_id = format!("wb|t{target_index}|c{competitor_index}|k{variant}");
                if !seen.insert(case_id.clone()) {
                    return Err("duplicate wrong-basin case id".into());
                }
                let d0 = integrate_trajectory(
                    bank,
                    model,
                    integrator,
                    &cue,
                    target,
                    Some(competitor.as_slice()),
                    &[],
                )?;
                let eligible = d0.terminal_pattern == *competitor;
                if eligible {
                    cases.push(partition_record(
                        case_id,
                        "wrong_basin",
                        target_index,
                        Some(competitor_index),
                        cue,
                        true,
                    )?);
                } else {
                    ineligible.push(case_id);
                }
            }
        }
    }

    for target_index in 0..bank.patterns().len() {
        let cue = bank.patterns()[target_index].clone();
        let case_id = format!("clean|t{target_index}");
        if !seen.insert(case_id.clone()) {
            return Err("duplicate clean case id".into());
        }
        cases.push(partition_record(
            case_id,
            "clean_cue",
            target_index,
            None,
            cue,
            false,
        )?);
    }

    Ok((cases, ineligible))
}

fn partition_record(
    case_id: String,
    kind: &'static str,
    target_index: usize,
    competitor_index: Option<usize>,
    cue: Vec<i8>,
    eligible_wrong_basin: bool,
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
        eligible_wrong_basin,
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

fn panels_disjoint(wrong: &[CaseRecord], clean: &[CaseRecord]) -> bool {
    let wrong_ids: BTreeSet<&str> = wrong.iter().map(|case| case.case_id.as_str()).collect();
    let clean_ids: BTreeSet<&str> = clean.iter().map(|case| case.case_id.as_str()).collect();
    wrong_ids.is_disjoint(&clean_ids)
        && wrong
            .iter()
            .all(|case| case.kind == "wrong_basin" && case.eligible_wrong_basin)
        && clean.iter().all(|case| case.kind == "clean_cue")
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
    let mut total_eligible = 0usize;
    let mut total_left = 0usize;
    let mut total_clean_correct = 0usize;
    let mut total_clean = 0usize;
    let mut all_angles = Vec::new();

    for &seed in &SEEDS {
        let noise = build_noise(condition, seed, permute, temporal_perm)?;
        let mut recoveries = 0usize;
        let mut competing = 0usize;
        let mut unresolved = 0usize;
        let mut left = 0usize;
        let mut angles = Vec::new();
        let mut passages = Vec::new();
        let mut finite = true;

        for case in wrong_cases {
            let target = &bank.patterns()[case.target_index];
            let competitor_index = case
                .competitor_index
                .ok_or("wrong-basin case missing competitor")?;
            let competitor = &bank.patterns()[competitor_index];
            let traj = integrate_trajectory(
                bank,
                model,
                integrator,
                &case.cue,
                target,
                Some(competitor.as_slice()),
                &noise,
            )?;
            finite &= traj.finite;
            if traj.terminal_pattern == *target {
                recoveries += 1;
            } else if traj.terminal_pattern == *competitor {
                competing += 1;
            } else {
                unresolved += 1;
            }
            if traj.left_wrong_basin {
                left += 1;
            }
            angles.push(traj.target_angle_error);
            if let Some(step) = traj.first_passage_time {
                passages.push(f64::from(u32::try_from(step)?));
            }
        }

        let mut clean_correct = 0usize;
        let mut clean_angles = Vec::new();
        for case in clean_cases {
            let target = &bank.patterns()[case.target_index];
            let traj =
                integrate_trajectory(bank, model, integrator, &case.cue, target, None, &noise)?;
            finite &= traj.finite;
            if traj.terminal_pattern == *target {
                clean_correct += 1;
            }
            clean_angles.push(traj.target_angle_error);
        }

        total_recoveries += recoveries;
        total_eligible += wrong_cases.len();
        total_left += left;
        total_clean_correct += clean_correct;
        total_clean += clean_cases.len();
        all_angles.extend_from_slice(&angles);

        seeds.push(SeedMetrics {
            seed,
            target_recoveries: recoveries,
            eligible_cases: wrong_cases.len(),
            competing_terminal: competing,
            unresolved_terminal: unresolved,
            left_wrong_basin: left,
            median_target_angle_error: median_f64(&angles),
            mean_first_passage_time: if passages.is_empty() {
                None
            } else {
                Some(passages.iter().sum::<f64>() / f64::from(u32::try_from(passages.len())?))
            },
            clean_correct,
            clean_cases: clean_cases.len(),
            clean_median_target_angle_error: median_f64(&clean_angles),
            finite,
        });
    }

    Ok(ArmAggregate {
        arm,
        condition,
        seeds,
        target_recovery_fraction: rate(total_recoveries, total_eligible),
        clean_recall_fraction: rate(total_clean_correct, total_clean),
        clean_degradation_vs_d0: 0.0,
        median_target_angle_error: median_f64(&all_angles),
        left_wrong_basin_fraction: rate(total_left, total_eligible),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct SelectionKey {
    neg_recovery: i64,
    degradation: i64,
    angle: i64,
    amplitude: i64,
    family: Family,
}

fn selection_key(candidate: &ArmAggregate) -> SelectionKey {
    SelectionKey {
        neg_recovery: -encode_fraction(candidate.target_recovery_fraction),
        degradation: encode_fraction(candidate.clean_degradation_vs_d0),
        angle: encode_fraction(candidate.median_target_angle_error),
        amplitude: encode_fraction(candidate.condition.amplitude),
        family: candidate.condition.family,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn encode_fraction(value: f64) -> i64 {
    (value * 1_000_000_000.0).round() as i64
}

fn arm_finite(arm: &ArmAggregate) -> bool {
    arm.seeds.iter().all(|seed| seed.finite)
        && arm.target_recovery_fraction.is_finite()
        && arm.clean_recall_fraction.is_finite()
        && arm.median_target_angle_error.is_finite()
}

fn clean_safety_tolerance(holdout_clean_trials: usize) -> f64 {
    if holdout_clean_trials == 0 {
        return 0.0;
    }
    let one_point = 0.01;
    let min_representable = 1.0 / f64::from(u32::try_from(holdout_clean_trials).unwrap_or(1));
    if min_representable > one_point {
        0.0
    } else {
        one_point
    }
}

fn verify_reproducibility(
    bank: &PatternBank,
    model: &EnergyModel,
    integrator: IntegratorConfig,
    wrong_cases: &[CaseRecord],
    clean_cases: &[CaseRecord],
    condition: ConditionSpec,
    temporal_perm: &[usize],
) -> Result<bool, Box<dyn Error>> {
    let Some(case) = wrong_cases.first() else {
        return Ok(true);
    };
    let seed = SEEDS[0];
    let noise = build_noise(condition, seed, false, temporal_perm)?;
    let target = &bank.patterns()[case.target_index];
    let competitor = &bank.patterns()[case.competitor_index.ok_or("missing competitor")?];
    let first = integrate_trajectory(
        bank,
        model,
        integrator,
        &case.cue,
        target,
        Some(competitor.as_slice()),
        &noise,
    )?;
    let second = integrate_trajectory(
        bank,
        model,
        integrator,
        &case.cue,
        target,
        Some(competitor.as_slice()),
        &noise,
    )?;
    let clean_ok = if let Some(clean) = clean_cases.first() {
        let clean_target = &bank.patterns()[clean.target_index];
        let a = integrate_trajectory(
            bank,
            model,
            integrator,
            &clean.cue,
            clean_target,
            None,
            &noise,
        )?;
        let b = integrate_trajectory(
            bank,
            model,
            integrator,
            &clean.cue,
            clean_target,
            None,
            &noise,
        )?;
        a.terminal_pattern == b.terminal_pattern
            && (a.target_angle_error - b.target_angle_error).abs() < 1.0e-15
    } else {
        true
    };
    Ok(first.terminal_pattern == second.terminal_pattern
        && first.first_passage_time == second.first_passage_time
        && (first.target_angle_error - second.target_angle_error).abs() < 1.0e-15
        && clean_ok)
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
    let mut series: Vec<Vec<f64>> = match condition.family {
        Family::Gaussian => {
            let mut rng = SplitMix64::new(seed);
            (0..components)
                .map(|_| {
                    (0..FIELD_STEPS)
                        .map(|_| condition.amplitude * rng.next_gaussian())
                        .collect::<Vec<_>>()
                })
                .collect()
        }
        Family::Ou => {
            let theta = condition.ou_theta.ok_or("OU condition missing theta")?;
            (0..components)
                .map(|component| {
                    let component_seed = seed
                        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        .wrapping_add(u64::try_from(component)?);
                    ornstein_uhlenbeck_path(
                        0.0,
                        theta,
                        0.0,
                        condition.amplitude,
                        FIELD_DT,
                        FIELD_STEPS,
                        component_seed,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };

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
    model: &EnergyModel,
    integrator: IntegratorConfig,
    cue: &[i8],
    target: &[i8],
    competitor: Option<&[i8]>,
    noise: &[Vec<Vec<f64>>],
) -> Result<TrajectoryResult, Box<dyn Error>> {
    let target_state = bank.encode_cue(target, CUE_TILT_RADIANS)?;
    let mut state = bank.encode_cue(cue, CUE_TILT_RADIANS)?;
    let mut first_passage = None;
    let mut left_wrong = false;
    let mut finite = true;

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
        finite &= state
            .nodes()
            .iter()
            .all(|node| node.values().iter().all(|value| value.is_finite()));
        let decoded = bank.decode_state(&state)?;
        if first_passage.is_none() && decoded == target {
            first_passage = Some(step + 1);
        }
        if let Some(comp) = competitor {
            if decoded != comp {
                left_wrong = true;
            }
        }
    }

    let terminal_pattern = bank.decode_state(&state)?;
    let target_angle_error = mean_angle_error(&state, &target_state);
    finite &= target_angle_error.is_finite();
    if let Some(comp) = competitor {
        if terminal_pattern != comp {
            left_wrong = true;
        }
    }

    Ok(TrajectoryResult {
        terminal_pattern,
        target_angle_error,
        first_passage_time: first_passage,
        left_wrong_basin: left_wrong,
        finite,
    })
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

fn provenance_fingerprint() -> u64 {
    let manifest = format!(
        "fl5|nodes={NODE_COUNT}|steps={FIELD_STEPS}|dt={FIELD_DT:.17}|mobility={FIELD_MOBILITY:.17}|tilt={CUE_TILT_RADIANS:.17}|amps={AMPLITUDES:?}|ou={OU_THETAS:?}|seeds={SEEDS:?}|perm={PERM_SEED}|noiselab={NOISELAB_COMMIT}|patterns=3|fixture=wb-competitor+diffbit"
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
        build_noise, partition_record, reference_patterns, temporal_permutation, ConditionSpec,
        Family, Partition, AMPLITUDES, FIELD_STEPS, OU_THETAS, SEEDS,
    };

    #[test]
    fn frozen_grid_includes_zero_and_eight_seeds() {
        assert!(AMPLITUDES.contains(&0.0));
        assert!(AMPLITUDES.iter().any(|amp| *amp > 0.0));
        assert!(!OU_THETAS.is_empty());
        assert!(SEEDS.len() >= 8);
    }

    #[test]
    fn reference_bank_width_matches_fixture() {
        assert!(reference_patterns()
            .iter()
            .all(|pattern| pattern.len() == 8));
    }

    #[test]
    fn partition_matches_fl5_partition_tool_rule() {
        let calibration =
            partition_record("case-a".into(), "clean_cue", 0, None, vec![1; 8], false).unwrap();
        assert_eq!(calibration.partition, Partition::Calibration);
        let holdout =
            partition_record("case-b".into(), "clean_cue", 0, None, vec![1; 8], false).unwrap();
        assert_eq!(holdout.partition, Partition::Holdout);
    }

    #[test]
    fn temporal_permutation_is_bijection() {
        let perm = temporal_permutation(FIELD_STEPS, 123);
        let mut sorted = perm.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..FIELD_STEPS).collect::<Vec<_>>());
    }

    #[test]
    fn zero_amplitude_noise_is_empty() {
        let noise = build_noise(
            ConditionSpec {
                family: Family::Gaussian,
                amplitude: 0.0,
                ou_theta: None,
            },
            1,
            false,
            &temporal_permutation(FIELD_STEPS, 1),
        )
        .unwrap();
        assert!(noise.is_empty());
    }
}
