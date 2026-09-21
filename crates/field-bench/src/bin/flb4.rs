#![forbid(unsafe_code)]
#![allow(clippy::float_cmp)] // Frozen protocol constants use exact identity checks.
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]

use field_bench::qualification::AxialHessian;
use field_boolean::{
    annotate_predicate_dwell_censoring, evaluate_predicate_transition_trace, evaluate_predicates,
    predicate_censor_aware_dwell_tail_contrast_profile, predicate_censor_aware_dwell_tail_ordering,
    predicate_dwell_runs, ComponentThresholdPredicate, ExactFractionSign, ExactSignedRunFraction,
    ObservationBoundary, PredicateDwellRun, PredicateDwellTailOrdering, ThresholdRelation,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_CONTRAST_PROFILE_SCHEMA,
    BOOLEAN_FIELD_CENSOR_AWARE_TAIL_ORDERING_SCHEMA, BOOLEAN_FIELD_DWELL_CENSORING_SCHEMA,
    BOOLEAN_FIELD_DWELL_RUN_SCHEMA, BOOLEAN_FIELD_PREDICATE_SCHEMA,
};
use field_core::{
    CouplingGraph, FieldState, LocalAnisotropy, NodeState, OperatorCoupling, OperatorEnergyModel,
};
use field_dynamics::{heun_step, IntegratorConfig};
use field_memory::PatternBank;
use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Debug;

fn box_err<E: Debug>(error: E) -> Box<dyn Error> {
    format!("{error:?}").into()
}

const REPORT_SCHEMA: &str = "fieldlab.flb4-report.v1";
const CONSTRUCTION_TOLERANCE: f64 = 1.0e-12;
const VALIDITY_NORM_SQUARED_TOLERANCE: f64 = 1.0e-10;
const POSITIVE_THRESHOLD: f64 = 0.9;
const NEGATIVE_THRESHOLD: f64 = -0.9;
const STRESS_RADIUS: f64 = 0.50;
const DT: f64 = 0.05;
const STEPS: usize = 64;
const OBSERVATION_COUNT: usize = STEPS + 1;
const STIFFNESS_MARGIN: f64 = 0.25;
const THRESHOLDS: [usize; 5] = [2, 4, 8, 16, 32];
const PRIMARY_THRESHOLD: usize = 8;
const PATTERNS: [[i8; 8]; 3] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, -1, -1],
    [1, 1, 1, 1, -1, -1, 1, 1],
];

#[derive(Clone, Debug, PartialEq, Serialize)]
struct PredicateRecord {
    index: usize,
    node: usize,
    component: usize,
    threshold_bits: u64,
    relation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ExactFractionRecord {
    sign: &'static str,
    numerator: u128,
    denominator: u128,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ContrastPointRecord {
    threshold_observations: usize,
    false_source_runs: usize,
    true_source_runs: usize,
    true_minus_false_lower: Option<ExactFractionRecord>,
    true_minus_false_upper: Option<ExactFractionRecord>,
    ordering: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct CaseRecord {
    case_id: String,
    memory: usize,
    predicate_index: usize,
    probe_node: usize,
    probe_direction: &'static str,
    bit_role: &'static str,
    run_count: usize,
    transition_count: usize,
    left_censored_runs: usize,
    right_censored_runs: usize,
    both_censored_runs: usize,
    contrast_profile: Vec<ContrastPointRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct MemorySignature {
    memory: usize,
    modal_orderings_at_primary_threshold: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct RoleMultiset {
    memory: usize,
    aligned_modal_orderings: Vec<&'static str>,
    anti_modal_orderings: Vec<&'static str>,
    multisets_differ: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
struct HypothesisOutcomes {
    hb4_0_protocol_valid: bool,
    hb4_1_determinate_contrast_exists: bool,
    hb4_2_memory_signatures_pairwise_distinct: bool,
    hb4_3_baselines_support_nonartifact: bool,
    hb4_4_bit_role_separation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Report {
    schema: &'static str,
    predicate_schema: &'static str,
    dwell_run_schema: &'static str,
    dwell_censoring_schema: &'static str,
    ordering_schema: &'static str,
    contrast_profile_schema: &'static str,
    source_revision: String,
    stiffness: f64,
    dt: f64,
    steps: usize,
    observation_count: usize,
    stress_radius: f64,
    thresholds_observations: [usize; 5],
    primary_threshold_observations: usize,
    left_boundary: &'static str,
    right_boundary: &'static str,
    predicates: Vec<PredicateRecord>,
    trajectory_count: usize,
    case_count: usize,
    cases: Vec<CaseRecord>,
    determinate_case_threshold_hits: usize,
    const_false_determinate_hits: usize,
    memory_signatures: Vec<MemorySignature>,
    perm_pred_signatures: Vec<MemorySignature>,
    role_multisets: Vec<RoleMultiset>,
    max_norm_squared_error: f64,
    replay_exact: bool,
    hypotheses: HypothesisOutcomes,
}

fn patterns() -> Vec<Vec<i8>> {
    PATTERNS.iter().map(Vec::from).collect()
}

fn predicate_bank() -> Result<Vec<ComponentThresholdPredicate>, Box<dyn Error>> {
    let mut predicates = Vec::with_capacity(16);
    for node in 0..8 {
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            POSITIVE_THRESHOLD,
            ThresholdRelation::AtLeast,
        )?);
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            NEGATIVE_THRESHOLD,
            ThresholdRelation::LessThan,
        )?);
    }
    if predicates.len() != 16 {
        return Err("FL-B4 predicate count drift".into());
    }
    Ok(predicates)
}

fn predicate_records(predicates: &[ComponentThresholdPredicate]) -> Vec<PredicateRecord> {
    predicates
        .iter()
        .enumerate()
        .map(|(index, predicate)| PredicateRecord {
            index,
            node: predicate.node(),
            component: predicate.component(),
            threshold_bits: predicate.threshold().to_bits(),
            relation: match predicate.relation() {
                ThresholdRelation::AtLeast => "at-least",
                ThresholdRelation::LessThan => "less-than",
            },
        })
        .collect()
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
        vec![vec![0.0; 2]; 8],
        couplings,
        anisotropies,
    )?)
}

fn angular_state(signs: &[i8], angles: &[f64]) -> Result<FieldState, Box<dyn Error>> {
    if signs.len() != angles.len() {
        return Err("angular chart dimension mismatch".into());
    }
    let nodes = signs
        .iter()
        .zip(angles)
        .map(|(sign, angle)| {
            let (sin, cos) = angle.sin_cos();
            NodeState::try_unit(
                vec![f64::from(*sign) * cos, f64::from(*sign) * sin],
                CONSTRUCTION_TOLERANCE,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
}

fn stress_probe_state(
    signs: &[i8],
    probe_node: usize,
    direction: f64,
) -> Result<FieldState, Box<dyn Error>> {
    let mut angles = vec![0.0; signs.len()];
    angles[probe_node] = direction * STRESS_RADIUS;
    angular_state(signs, &angles)
}

fn max_norm_squared_error(state: &FieldState) -> f64 {
    state
        .nodes()
        .iter()
        .map(|node| {
            let norm_squared = node.values().iter().map(|value| value * value).sum::<f64>();
            (norm_squared - 1.0).abs()
        })
        .fold(0.0_f64, f64::max)
}

fn ordering_name(ordering: PredicateDwellTailOrdering) -> &'static str {
    match ordering {
        PredicateDwellTailOrdering::UndefinedAbsentValue => "undefined-absent-value",
        PredicateDwellTailOrdering::TrueDefinitelyHigher => "true-definitely-higher",
        PredicateDwellTailOrdering::FalseDefinitelyHigher => "false-definitely-higher",
        PredicateDwellTailOrdering::IndeterminateOverlap => "indeterminate-overlap",
    }
}

fn fraction_record(fraction: ExactSignedRunFraction) -> ExactFractionRecord {
    ExactFractionRecord {
        sign: match fraction.sign {
            ExactFractionSign::Negative => "negative",
            ExactFractionSign::Zero => "zero",
            ExactFractionSign::Positive => "positive",
        },
        numerator: fraction.numerator,
        denominator: fraction.denominator,
    }
}

fn is_determinate(name: &str) -> bool {
    matches!(name, "true-definitely-higher" | "false-definitely-higher")
}

fn bit_role(clean_code: &[bool], predicate_index: usize) -> &'static str {
    if clean_code[predicate_index] {
        "aligned"
    } else {
        "anti"
    }
}

fn modal_ordering(counts: &BTreeMap<&'static str, usize>) -> &'static str {
    const ORDER: [&str; 4] = [
        "indeterminate-overlap",
        "undefined-absent-value",
        "true-definitely-higher",
        "false-definitely-higher",
    ];
    let mut best_name = ORDER[0];
    let mut best_count = 0_usize;
    for name in ORDER {
        let count = counts.get(name).copied().unwrap_or(0);
        if count > best_count {
            best_count = count;
            best_name = name;
        }
    }
    best_name
}

fn signatures_pairwise_distinct(signatures: &[MemorySignature]) -> bool {
    for left in 0..signatures.len() {
        for right in (left + 1)..signatures.len() {
            if signatures[left].modal_orderings_at_primary_threshold
                == signatures[right].modal_orderings_at_primary_threshold
            {
                return false;
            }
        }
    }
    true
}

fn analyze_runs(
    runs: &[PredicateDwellRun],
) -> Result<(Vec<ContrastPointRecord>, usize, usize, usize), Box<dyn Error>> {
    let annotated = annotate_predicate_dwell_censoring(
        runs,
        ObservationBoundary::Complete,
        ObservationBoundary::ObservationCut,
    )
    .map_err(box_err)?;
    let left_censored_runs = annotated.iter().filter(|run| run.left_censored).count();
    let right_censored_runs = annotated.iter().filter(|run| run.right_censored).count();
    let both_censored_runs = annotated
        .iter()
        .filter(|run| run.left_censored && run.right_censored)
        .count();
    let profile = predicate_censor_aware_dwell_tail_contrast_profile(
        runs,
        ObservationBoundary::Complete,
        ObservationBoundary::ObservationCut,
        &THRESHOLDS,
    )
    .map_err(box_err)?;
    let mut points = Vec::with_capacity(profile.len());
    for point in profile {
        let ordering = predicate_censor_aware_dwell_tail_ordering(
            runs,
            ObservationBoundary::Complete,
            ObservationBoundary::ObservationCut,
            point.threshold_observations,
        )
        .map_err(box_err)?;
        points.push(ContrastPointRecord {
            threshold_observations: point.threshold_observations,
            false_source_runs: point.contrast_bounds.false_source_runs,
            true_source_runs: point.contrast_bounds.true_source_runs,
            true_minus_false_lower: point
                .contrast_bounds
                .true_minus_false_lower
                .map(fraction_record),
            true_minus_false_upper: point
                .contrast_bounds
                .true_minus_false_upper
                .map(fraction_record),
            ordering: ordering_name(ordering),
        });
    }
    Ok((
        points,
        left_censored_runs,
        right_censored_runs,
        both_censored_runs,
    ))
}

fn run_once(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    if source_revision.trim().is_empty() {
        return Err("FIELDLAB_SOURCE_REVISION must be non-empty".into());
    }

    let predicates = predicate_bank()?;
    let predicate_records = predicate_records(&predicates);
    let bank = PatternBank::new(patterns())?;
    let graph = bank.hebbian_graph()?;
    let mut worst_bound = 0.0_f64;
    for signs in bank.patterns() {
        worst_bound = worst_bound.min(AxialHessian::new(&graph, signs, 0.0)?.lower_bound());
    }
    let stiffness = -worst_bound + STIFFNESS_MARGIN;
    if (stiffness - 1.25).abs() > 1.0e-12 {
        return Err(format!("FL-B4 expected stiffness 1.25, got {stiffness}").into());
    }
    let model = model_e1(&graph, stiffness)?;
    let config = IntegratorConfig {
        dt: DT,
        mobility: 1.0,
    };

    let mut clean_codes = Vec::with_capacity(PATTERNS.len());
    for pattern in &PATTERNS {
        let state = angular_state(pattern, &[0.0; 8])?;
        clean_codes.push(evaluate_predicates(&state, &predicates)?);
    }

    let mut cases = Vec::with_capacity(768);
    let mut max_error = 0.0_f64;
    let mut determinate_hits = 0_usize;
    let mut trajectory_count = 0_usize;
    let mut primary_orderings: Vec<Vec<Vec<&'static str>>> = (0..PATTERNS.len())
        .map(|_| (0..16).map(|_| Vec::with_capacity(16)).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    for (memory, pattern) in PATTERNS.iter().enumerate() {
        for probe_node in 0..8 {
            for direction in [-1.0_f64, 1.0_f64] {
                let direction_name = if direction.is_sign_negative() {
                    "minus"
                } else {
                    "plus"
                };
                let mut state = stress_probe_state(pattern, probe_node, direction)?;
                let mut states = Vec::with_capacity(OBSERVATION_COUNT);
                for step in 0..=STEPS {
                    let error = max_norm_squared_error(&state);
                    if !error.is_finite() || error > VALIDITY_NORM_SQUARED_TOLERANCE {
                        return Err(format!(
                            "FL-B4 unit-state validity failed for p{memory}/n{probe_node}/{direction_name} at step {step}: {error:e}"
                        )
                        .into());
                    }
                    max_error = max_error.max(error);
                    states.push(state.clone());
                    if step < STEPS {
                        state = heun_step(&state, &model, config)?;
                    }
                }
                if states.len() != OBSERVATION_COUNT {
                    return Err("FL-B4 observation count drift".into());
                }
                trajectory_count += 1;

                for (predicate_index, predicate) in predicates.iter().enumerate() {
                    let trace =
                        evaluate_predicate_transition_trace(predicate, &states, OBSERVATION_COUNT)
                            .map_err(box_err)?;
                    let initial = predicate.evaluate(&states[0])?;
                    let runs = predicate_dwell_runs(initial, &trace, OBSERVATION_COUNT)
                        .map_err(box_err)?;
                    let (profile, left_censored_runs, right_censored_runs, both_censored_runs) =
                        analyze_runs(&runs)?;
                    for point in &profile {
                        if is_determinate(point.ordering) {
                            determinate_hits += 1;
                        }
                    }
                    let primary = profile
                        .iter()
                        .find(|point| point.threshold_observations == PRIMARY_THRESHOLD)
                        .ok_or("missing primary threshold profile point")?;
                    primary_orderings[memory][predicate_index].push(primary.ordering);

                    cases.push(CaseRecord {
                        case_id: format!(
                            "flb4|p{memory}|pred{predicate_index}|n{probe_node}|d{direction_name}"
                        ),
                        memory,
                        predicate_index,
                        probe_node,
                        probe_direction: direction_name,
                        bit_role: bit_role(&clean_codes[memory], predicate_index),
                        run_count: runs.len(),
                        transition_count: trace
                            .iter()
                            .filter(|transition| transition.changed())
                            .count(),
                        left_censored_runs,
                        right_censored_runs,
                        both_censored_runs,
                        contrast_profile: profile,
                    });
                }
            }
        }
    }

    if trajectory_count != 48 {
        return Err("FL-B4 trajectory count drift".into());
    }
    if cases.len() != 768 {
        return Err("FL-B4 case count drift".into());
    }

    let const_runs = [PredicateDwellRun {
        value: false,
        start_observation: 0,
        observations: OBSERVATION_COUNT,
    }];
    let mut const_false_determinate_hits = 0_usize;
    for _ in 0..cases.len() {
        for threshold in THRESHOLDS {
            let ordering = predicate_censor_aware_dwell_tail_ordering(
                &const_runs,
                ObservationBoundary::Complete,
                ObservationBoundary::ObservationCut,
                threshold,
            )
            .map_err(box_err)?;
            if is_determinate(ordering_name(ordering)) {
                const_false_determinate_hits += 1;
            }
        }
    }

    let mut memory_signatures = Vec::with_capacity(PATTERNS.len());
    let mut role_multisets = Vec::with_capacity(PATTERNS.len());
    for memory in 0..PATTERNS.len() {
        let mut modal = Vec::with_capacity(16);
        for predicate_index in 0..16 {
            let mut counts = BTreeMap::<&'static str, usize>::new();
            for ordering in &primary_orderings[memory][predicate_index] {
                *counts.entry(*ordering).or_default() += 1;
            }
            modal.push(modal_ordering(&counts));
        }
        let mut aligned = Vec::new();
        let mut anti = Vec::new();
        for (predicate_index, ordering) in modal.iter().enumerate() {
            if clean_codes[memory][predicate_index] {
                aligned.push(*ordering);
            } else {
                anti.push(*ordering);
            }
        }
        let mut aligned_sorted = aligned.clone();
        let mut anti_sorted = anti.clone();
        aligned_sorted.sort_unstable();
        anti_sorted.sort_unstable();
        role_multisets.push(RoleMultiset {
            memory,
            aligned_modal_orderings: aligned,
            anti_modal_orderings: anti,
            multisets_differ: aligned_sorted != anti_sorted,
        });
        memory_signatures.push(MemorySignature {
            memory,
            modal_orderings_at_primary_threshold: modal,
        });
    }

    let mut perm_pred_signatures = Vec::with_capacity(PATTERNS.len());
    for signature in &memory_signatures {
        let mut shifted = Vec::with_capacity(16);
        for predicate_index in 0..16 {
            shifted
                .push(signature.modal_orderings_at_primary_threshold[(predicate_index + 1) % 16]);
        }
        perm_pred_signatures.push(MemorySignature {
            memory: signature.memory,
            modal_orderings_at_primary_threshold: shifted,
        });
    }

    let hb4_1 = determinate_hits > 0;
    let hb4_2 = signatures_pairwise_distinct(&memory_signatures);
    let perm_distinct = signatures_pairwise_distinct(&perm_pred_signatures);
    let mut perm_role_separation = false;
    for memory in 0..PATTERNS.len() {
        let mut aligned = Vec::new();
        let mut anti = Vec::new();
        for predicate_index in 0..16 {
            let source = (predicate_index + 1) % 16;
            let ordering = memory_signatures[memory].modal_orderings_at_primary_threshold[source];
            if clean_codes[memory][predicate_index] {
                aligned.push(ordering);
            } else {
                anti.push(ordering);
            }
        }
        aligned.sort_unstable();
        anti.sort_unstable();
        if aligned != anti {
            perm_role_separation = true;
        }
    }
    let hb4_3 = const_false_determinate_hits == 0
        && determinate_hits > const_false_determinate_hits
        && (!perm_distinct || !perm_role_separation);
    let hb4_4 = role_multisets.iter().any(|entry| entry.multisets_differ);
    let hb4_0 = cases.len() == 768
        && trajectory_count == 48
        && max_error <= VALIDITY_NORM_SQUARED_TOLERANCE
        && predicates.len() == 16;

    Ok(Report {
        schema: REPORT_SCHEMA,
        predicate_schema: BOOLEAN_FIELD_PREDICATE_SCHEMA,
        dwell_run_schema: BOOLEAN_FIELD_DWELL_RUN_SCHEMA,
        dwell_censoring_schema: BOOLEAN_FIELD_DWELL_CENSORING_SCHEMA,
        ordering_schema: BOOLEAN_FIELD_CENSOR_AWARE_TAIL_ORDERING_SCHEMA,
        contrast_profile_schema: BOOLEAN_FIELD_CENSOR_AWARE_TAIL_CONTRAST_PROFILE_SCHEMA,
        source_revision: source_revision.to_owned(),
        stiffness,
        dt: DT,
        steps: STEPS,
        observation_count: OBSERVATION_COUNT,
        stress_radius: STRESS_RADIUS,
        thresholds_observations: THRESHOLDS,
        primary_threshold_observations: PRIMARY_THRESHOLD,
        left_boundary: "complete",
        right_boundary: "observation-cut",
        predicates: predicate_records,
        trajectory_count,
        case_count: cases.len(),
        cases,
        determinate_case_threshold_hits: determinate_hits,
        const_false_determinate_hits,
        memory_signatures,
        perm_pred_signatures,
        role_multisets,
        max_norm_squared_error: max_error,
        replay_exact: false,
        hypotheses: HypothesisOutcomes {
            hb4_0_protocol_valid: hb4_0,
            hb4_1_determinate_contrast_exists: hb4_1,
            hb4_2_memory_signatures_pairwise_distinct: hb4_2,
            hb4_3_baselines_support_nonartifact: hb4_3,
            hb4_4_bit_role_separation: hb4_4,
        },
    })
}

fn run_replayed(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    let first = run_once(source_revision)?;
    let second = run_once(source_revision)?;
    if first != second {
        return Err("FL-B4 semantic replay mismatch".into());
    }
    let mut report = first;
    report.replay_exact = true;
    report.hypotheses.hb4_0_protocol_valid =
        report.hypotheses.hb4_0_protocol_valid && report.replay_exact;
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_revision = std::env::var("FIELDLAB_SOURCE_REVISION")
        .map_err(|_| "FIELDLAB_SOURCE_REVISION must identify the executed repository revision")?;
    let report = run_replayed(&source_revision)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.hypotheses.hb4_0_protocol_valid {
        return Err("FL-B4 execution gate failed; retain output for diagnosis".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{predicate_bank, run_replayed, PATTERNS, PRIMARY_THRESHOLD, THRESHOLDS};

    #[test]
    fn frozen_protocol_has_declared_shape() {
        assert_eq!(predicate_bank().expect("frozen predicates").len(), 16);
        assert_eq!(PATTERNS.len(), 3);
        assert_eq!(THRESHOLDS, [2, 4, 8, 16, 32]);
        assert_eq!(PRIMARY_THRESHOLD, 8);
    }

    #[test]
    fn campaign_shape_and_replay_are_valid_without_asserting_scientific_outcomes() {
        let report = run_replayed("test-revision").expect("protocol-valid campaign");
        assert_eq!(report.trajectory_count, 48);
        assert_eq!(report.case_count, 768);
        assert_eq!(report.cases.len(), 768);
        assert_eq!(report.predicates.len(), 16);
        assert!(report.replay_exact);
        assert!(report.hypotheses.hb4_0_protocol_valid);
    }
}
