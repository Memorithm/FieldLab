#![forbid(unsafe_code)]

use field_boolean::{
    evaluate_predicates, ComponentThresholdPredicate, ThresholdRelation,
    BOOLEAN_FIELD_PREDICATE_SCHEMA,
};
use field_core::{FieldState, NodeState};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

const REPORT_SCHEMA: &str = "fieldlab.flb1-report.v1";
const CONSTRUCTION_TOLERANCE: f64 = 1.0e-12;
const VALIDITY_NORM_SQUARED_TOLERANCE: f64 = 1.0e-10;
const POSITIVE_THRESHOLD: f64 = 0.9;
const NEGATIVE_THRESHOLD: f64 = -0.9;
const PATTERNS: [[i8; 8]; 3] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, -1, -1],
    [1, 1, 1, 1, -1, -1, 1, 1],
];
const RADII: [f64; 5] = [0.01, 0.05, 0.10, 0.25, 0.50];

#[derive(Clone, Debug, PartialEq, Serialize)]
struct PredicateRecord {
    node: usize,
    component: usize,
    threshold_bits: u64,
    relation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct CleanObservation {
    case_id: String,
    label: usize,
    code: Vec<bool>,
    continuous_bits: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ObservationGroup {
    members: Vec<usize>,
    label_purity: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct HammingRecord {
    left_label: usize,
    right_label: usize,
    distance: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct RadiusRetention {
    radius_index: usize,
    radius: f64,
    retained: usize,
    total: usize,
    rate: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct FirstChange {
    label: usize,
    radius_index: Option<usize>,
    radius: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct HypothesisOutcomes {
    hb1_1_clean_codes_separated: bool,
    hb1_2_all_declared_local_perturbations_retained: bool,
    hb1_3_clean_boolean_collision_count: usize,
    hb1_4_nonzero_empirical_information: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Report {
    schema: &'static str,
    predicate_schema: &'static str,
    source_revision: String,
    construction_tolerance: f64,
    validity_norm_squared_tolerance: f64,
    predicates: Vec<PredicateRecord>,
    clean: Vec<CleanObservation>,
    boolean_groups: Vec<ObservationGroup>,
    continuous_bitwise_groups: Vec<Vec<usize>>,
    boolean_induced_collision_groups: Vec<Vec<usize>>,
    hamming: Vec<HammingRecord>,
    minimum_clean_hamming_distance: usize,
    radius_retention: Vec<RadiusRetention>,
    first_code_change: Vec<FirstChange>,
    clean_label_entropy_bits: f64,
    conditional_entropy_given_boolean_bits: f64,
    mutual_information_bits: f64,
    constant_baseline_mutual_information_bits: f64,
    max_norm_squared_error: f64,
    clean_case_count: usize,
    perturbation_case_count: usize,
    replay_exact: bool,
    hypotheses: HypothesisOutcomes,
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

    let mut seen = BTreeSet::new();
    for predicate in &predicates {
        let relation = match predicate.relation() {
            ThresholdRelation::AtLeast => 0_u8,
            ThresholdRelation::LessThan => 1_u8,
        };
        let identity = (
            predicate.node(),
            predicate.component(),
            predicate.threshold().to_bits(),
            relation,
        );
        if !seen.insert(identity) {
            return Err("duplicate FL-B1.0 predicate".into());
        }
    }
    if predicates.len() != 16 {
        return Err("FL-B1.0 predicate count drift".into());
    }
    Ok(predicates)
}

fn predicate_records(predicates: &[ComponentThresholdPredicate]) -> Vec<PredicateRecord> {
    predicates
        .iter()
        .map(|predicate| PredicateRecord {
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

fn state_from_pattern(pattern: &[i8; 8]) -> Result<FieldState, Box<dyn Error>> {
    let nodes = pattern
        .iter()
        .map(|sign| {
            NodeState::try_unit(
                vec![f64::from(*sign), 0.0],
                CONSTRUCTION_TOLERANCE,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
}

fn perturbed_state(
    pattern: &[i8; 8],
    node_index: usize,
    radius: f64,
    direction: f64,
) -> Result<FieldState, Box<dyn Error>> {
    let q = direction * radius;
    let nodes = pattern
        .iter()
        .enumerate()
        .map(|(index, sign)| {
            let sign = f64::from(*sign);
            let values = if index == node_index {
                vec![sign * q.cos(), sign * q.sin()]
            } else {
                vec![sign, 0.0]
            };
            NodeState::try_unit(values, CONSTRUCTION_TOLERANCE)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
}

fn continuous_key(state: &FieldState) -> Vec<u64> {
    state
        .nodes()
        .iter()
        .flat_map(|node| node.values().iter().map(|value| value.to_bits()))
        .collect()
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

fn hamming(left: &[bool], right: &[bool]) -> usize {
    left.iter()
        .zip(right)
        .filter(|(left_bit, right_bit)| left_bit != right_bit)
        .count()
}

fn entropy_bits(counts: impl IntoIterator<Item = usize>, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    counts
        .into_iter()
        .filter(|count| *count != 0)
        .map(|count| {
            let probability = count as f64 / total as f64;
            -probability * probability.log2()
        })
        .sum()
}

fn label_purity(members: &[usize]) -> f64 {
    if members.is_empty() {
        return 0.0;
    }
    let mut counts = BTreeMap::<usize, usize>::new();
    for label in members {
        *counts.entry(*label).or_default() += 1;
    }
    let maximum = counts.values().copied().max().unwrap_or(0);
    maximum as f64 / members.len() as f64
}

fn conditional_entropy(groups: &[Vec<usize>], total: usize) -> f64 {
    groups
        .iter()
        .map(|members| {
            let weight = members.len() as f64 / total as f64;
            let mut counts = BTreeMap::<usize, usize>::new();
            for label in members {
                *counts.entry(*label).or_default() += 1;
            }
            weight * entropy_bits(counts.values().copied(), members.len())
        })
        .sum()
}

fn run_once(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    if source_revision.trim().is_empty() {
        return Err("FIELDLAB_SOURCE_REVISION must be non-empty".into());
    }

    let predicates = predicate_bank()?;
    let predicate_records = predicate_records(&predicates);
    let mut clean = Vec::with_capacity(PATTERNS.len());
    let mut clean_states = Vec::with_capacity(PATTERNS.len());
    let mut max_error = 0.0_f64;

    for (label, pattern) in PATTERNS.iter().enumerate() {
        let state = state_from_pattern(pattern)?;
        max_error = max_error.max(max_norm_squared_error(&state));
        let code = evaluate_predicates(&state, &predicates)?;
        let continuous_bits = continuous_key(&state);
        clean.push(CleanObservation {
            case_id: format!("flb1|clean|p{label}"),
            label,
            code,
            continuous_bits,
        });
        clean_states.push(state);
    }

    let mut boolean_map = BTreeMap::<Vec<bool>, Vec<usize>>::new();
    let mut continuous_map = BTreeMap::<Vec<u64>, Vec<usize>>::new();
    for observation in &clean {
        boolean_map
            .entry(observation.code.clone())
            .or_default()
            .push(observation.label);
        continuous_map
            .entry(observation.continuous_bits.clone())
            .or_default()
            .push(observation.label);
    }

    let boolean_group_members = boolean_map.values().cloned().collect::<Vec<_>>();
    let continuous_groups = continuous_map.values().cloned().collect::<Vec<_>>();
    let boolean_groups = boolean_group_members
        .iter()
        .map(|members| ObservationGroup {
            members: members.clone(),
            label_purity: label_purity(members),
        })
        .collect::<Vec<_>>();

    let mut boolean_induced_collision_groups = Vec::new();
    for members in &boolean_group_members {
        if members.len() <= 1 {
            continue;
        }
        let distinct_continuous = members
            .iter()
            .map(|label| clean[*label].continuous_bits.clone())
            .collect::<BTreeSet<_>>()
            .len();
        if distinct_continuous > 1 {
            boolean_induced_collision_groups.push(members.clone());
        }
    }

    let mut hamming_records = Vec::new();
    for left in 0..clean.len() {
        for right in (left + 1)..clean.len() {
            hamming_records.push(HammingRecord {
                left_label: left,
                right_label: right,
                distance: hamming(&clean[left].code, &clean[right].code),
            });
        }
    }
    let minimum_clean_hamming_distance = hamming_records
        .iter()
        .map(|record| record.distance)
        .min()
        .unwrap_or(0);

    let mut retained_per_radius = [0_usize; RADII.len()];
    let mut total_per_radius = [0_usize; RADII.len()];
    let mut first_change_index = [None::<usize>; PATTERNS.len()];
    let mut perturbation_case_count = 0_usize;

    for (label, pattern) in PATTERNS.iter().enumerate() {
        for (radius_index, radius) in RADII.iter().copied().enumerate() {
            for node_index in 0..8 {
                for direction in [-1.0_f64, 1.0_f64] {
                    let state = perturbed_state(pattern, node_index, radius, direction)?;
                    let error = max_norm_squared_error(&state);
                    if !error.is_finite() || error > VALIDITY_NORM_SQUARED_TOLERANCE {
                        return Err(format!(
                            "FL-B1.0 unit-state validity failed for p{label}/r{radius_index}/n{node_index}: {error:e}"
                        )
                        .into());
                    }
                    max_error = max_error.max(error);
                    let code = evaluate_predicates(&state, &predicates)?;
                    let retained = code == clean[label].code;
                    total_per_radius[radius_index] += 1;
                    if retained {
                        retained_per_radius[radius_index] += 1;
                    } else if first_change_index[label].is_none() {
                        first_change_index[label] = Some(radius_index);
                    }
                    perturbation_case_count += 1;
                }
            }
        }
    }

    if perturbation_case_count != 240 {
        return Err("FL-B1.0 perturbation panel count drift".into());
    }
    if max_error > VALIDITY_NORM_SQUARED_TOLERANCE {
        return Err("FL-B1.0 campaign unit-state tolerance exceeded".into());
    }

    let radius_retention = RADII
        .iter()
        .copied()
        .enumerate()
        .map(|(radius_index, radius)| RadiusRetention {
            radius_index,
            radius,
            retained: retained_per_radius[radius_index],
            total: total_per_radius[radius_index],
            rate: retained_per_radius[radius_index] as f64
                / total_per_radius[radius_index] as f64,
        })
        .collect::<Vec<_>>();

    let first_code_change = first_change_index
        .iter()
        .copied()
        .enumerate()
        .map(|(label, radius_index)| FirstChange {
            label,
            radius_index,
            radius: radius_index.map(|index| RADII[index]),
        })
        .collect::<Vec<_>>();

    let clean_label_entropy_bits = entropy_bits([1_usize; PATTERNS.len()], PATTERNS.len());
    let conditional_entropy_given_boolean_bits =
        conditional_entropy(&boolean_group_members, PATTERNS.len());
    let mutual_information_bits =
        clean_label_entropy_bits - conditional_entropy_given_boolean_bits;
    let constant_baseline_mutual_information_bits =
        clean_label_entropy_bits - conditional_entropy(&[(0..PATTERNS.len()).collect()], PATTERNS.len());

    let hb1_1 = boolean_group_members.iter().all(|members| members.len() == 1);
    let hb1_2 = radius_retention
        .iter()
        .filter(|record| record.radius_index <= 3)
        .all(|record| record.retained == record.total);
    let collision_count = boolean_group_members
        .iter()
        .filter(|members| members.len() > 1)
        .count();

    Ok(Report {
        schema: REPORT_SCHEMA,
        predicate_schema: BOOLEAN_FIELD_PREDICATE_SCHEMA,
        source_revision: source_revision.to_owned(),
        construction_tolerance: CONSTRUCTION_TOLERANCE,
        validity_norm_squared_tolerance: VALIDITY_NORM_SQUARED_TOLERANCE,
        predicates: predicate_records,
        clean,
        boolean_groups,
        continuous_bitwise_groups: continuous_groups,
        boolean_induced_collision_groups,
        hamming: hamming_records,
        minimum_clean_hamming_distance,
        radius_retention,
        first_code_change,
        clean_label_entropy_bits,
        conditional_entropy_given_boolean_bits,
        mutual_information_bits,
        constant_baseline_mutual_information_bits,
        max_norm_squared_error: max_error,
        clean_case_count: PATTERNS.len(),
        perturbation_case_count,
        replay_exact: false,
        hypotheses: HypothesisOutcomes {
            hb1_1_clean_codes_separated: hb1_1,
            hb1_2_all_declared_local_perturbations_retained: hb1_2,
            hb1_3_clean_boolean_collision_count: collision_count,
            hb1_4_nonzero_empirical_information: mutual_information_bits > 0.0,
        },
    })
}

fn run_replayed(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    let first = run_once(source_revision)?;
    let second = run_once(source_revision)?;
    if first != second {
        return Err("FL-B1.0 semantic replay mismatch".into());
    }
    let mut report = first;
    report.replay_exact = true;
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_revision = std::env::var("FIELDLAB_SOURCE_REVISION")
        .map_err(|_| "FIELDLAB_SOURCE_REVISION must identify the executed repository revision")?;
    let report = run_replayed(&source_revision)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{predicate_bank, run_replayed, PATTERNS, RADII};

    #[test]
    fn frozen_protocol_has_declared_shape() {
        assert_eq!(predicate_bank().expect("frozen predicates").len(), 16);
        assert_eq!(PATTERNS.len(), 3);
        assert_eq!(RADII, [0.01, 0.05, 0.10, 0.25, 0.50]);
    }

    #[test]
    fn campaign_shape_and_replay_are_valid_without_asserting_scientific_outcomes() {
        let report = run_replayed("test-revision").expect("protocol-valid campaign");
        assert_eq!(report.clean_case_count, 3);
        assert_eq!(report.perturbation_case_count, 240);
        assert_eq!(report.predicates.len(), 16);
        assert!(report.replay_exact);
    }
}
