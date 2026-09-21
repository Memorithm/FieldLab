#![forbid(unsafe_code)]
#![allow(clippy::float_cmp)] // Frozen protocol constants use exact identity checks.
#![allow(clippy::struct_excessive_bools)]

use field_bench::qualification::group_observables;
use field_boolean::{
    evaluate_predicates, ComponentThresholdPredicate, ThresholdRelation,
    BOOLEAN_FIELD_PREDICATE_SCHEMA,
};
use field_core::{FieldState, NodeState};
use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;

const REPORT_SCHEMA: &str = "fieldlab.flb3-report.v1";
const CONSTRUCTION_TOLERANCE: f64 = 1.0e-12;
const VALIDITY_NORM_SQUARED_TOLERANCE: f64 = 1.0e-10;
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
struct CaseRecord {
    case_id: String,
    construction_index: usize,
    signs: Vec<i8>,
    label: usize,
    b1_code: Vec<bool>,
    rich_code: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct GroupRecord {
    member_construction_indices: Vec<usize>,
    labels: Vec<usize>,
    conflicts: bool,
    maximum_correct: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct HypothesisOutcomes {
    hb3_0_protocol_valid: bool,
    hb3_1_b1_collisions_exist: bool,
    hb3_2_conflict_coverage: bool,
    hb3_3_ceiling_match: bool,
    hb3_4_richer_bank_does_not_invent_identity: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Report {
    schema: &'static str,
    predicate_schema: &'static str,
    source_revision: String,
    case_count: usize,
    b1_predicates: Vec<PredicateRecord>,
    rich_predicates: Vec<PredicateRecord>,
    cases: Vec<CaseRecord>,
    sign_groups: Vec<GroupRecord>,
    b1_groups: Vec<GroupRecord>,
    rich_groups: Vec<GroupRecord>,
    sign_ceiling: usize,
    b1_ceiling: usize,
    rich_ceiling: usize,
    sign_conflict_group_count: usize,
    b1_collision_group_count: usize,
    rich_collision_group_count: usize,
    covered_sign_conflict_groups_b1: usize,
    covered_sign_conflict_groups_rich: usize,
    max_norm_squared_error: f64,
    replay_exact: bool,
    hypotheses: HypothesisOutcomes,
}

fn b1_bank() -> Result<Vec<ComponentThresholdPredicate>, Box<dyn Error>> {
    let mut predicates = Vec::with_capacity(16);
    for node in 0..8 {
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            0.9,
            ThresholdRelation::AtLeast,
        )?);
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            -0.9,
            ThresholdRelation::LessThan,
        )?);
    }
    Ok(predicates)
}

fn rich_bank() -> Result<Vec<ComponentThresholdPredicate>, Box<dyn Error>> {
    let mut predicates = Vec::with_capacity(32);
    for node in 0..8 {
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            0.9,
            ThresholdRelation::AtLeast,
        )?);
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            -0.9,
            ThresholdRelation::LessThan,
        )?);
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            0.5,
            ThresholdRelation::AtLeast,
        )?);
        predicates.push(ComponentThresholdPredicate::new(
            node,
            0,
            -0.5,
            ThresholdRelation::LessThan,
        )?);
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

fn panel_records() -> Vec<(Vec<i8>, usize)> {
    let p0 = PATTERNS[0].to_vec();
    let p1 = PATTERNS[1].to_vec();
    let p2 = PATTERNS[2].to_vec();
    let mut records = vec![(p0.clone(), 0), (p1.clone(), 1), (p2.clone(), 2)];
    records.push((p1, 0));
    records.push((p2, 0));
    for bit in [6_usize, 7] {
        let mut cue = p0.clone();
        cue[bit] = -1;
        records.push((cue, 1));
    }
    for bit in [4_usize, 5] {
        let mut cue = p0.clone();
        cue[bit] = -1;
        records.push((cue, 2));
    }
    records
}

fn state_from_signs(signs: &[i8]) -> Result<FieldState, Box<dyn Error>> {
    let nodes = signs
        .iter()
        .map(|sign| NodeState::try_unit(vec![f64::from(*sign), 0.0], CONSTRUCTION_TOLERANCE))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
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

fn group_records_from_keys(keys_and_labels: &[(Vec<bool>, usize)]) -> Vec<GroupRecord> {
    // Preserve construction indices by grouping manually with indices.
    let mut map: BTreeMap<Vec<bool>, Vec<(usize, usize)>> = BTreeMap::new();
    for (index, (key, label)) in keys_and_labels.iter().enumerate() {
        map.entry(key.clone()).or_default().push((index, *label));
    }
    map.into_values()
        .map(|members| {
            let mut label_counts: BTreeMap<usize, usize> = BTreeMap::new();
            let mut member_construction_indices = Vec::new();
            let mut labels = Vec::new();
            for (index, label) in members {
                member_construction_indices.push(index);
                labels.push(label);
                *label_counts.entry(label).or_default() += 1;
            }
            let maximum_correct = label_counts.values().copied().max().unwrap_or(0);
            GroupRecord {
                member_construction_indices,
                labels,
                conflicts: label_counts.len() > 1,
                maximum_correct,
            }
        })
        .collect()
}

fn sign_group_records(records: &[(Vec<i8>, usize)]) -> Result<Vec<GroupRecord>, Box<dyn Error>> {
    let groups = group_observables(records)?;
    // Reconstruct construction indices for each sign group.
    Ok(groups
        .into_iter()
        .map(|group| {
            let members: Vec<usize> = records
                .iter()
                .enumerate()
                .filter(|(_, (signs, _))| signs == &group.observable)
                .map(|(index, _)| index)
                .collect();
            let labels: Vec<usize> = members.iter().map(|&index| records[index].1).collect();
            GroupRecord {
                member_construction_indices: members,
                labels,
                conflicts: group.conflicts(),
                maximum_correct: group.maximum_correct(),
            }
        })
        .collect())
}

fn ceiling(groups: &[GroupRecord]) -> usize {
    groups.iter().map(|group| group.maximum_correct).sum()
}

fn conflict_groups(groups: &[GroupRecord]) -> Vec<&GroupRecord> {
    groups.iter().filter(|group| group.conflicts).collect()
}

fn boolean_covers_sign_conflicts(
    sign_groups: &[GroupRecord],
    bool_groups: &[GroupRecord],
) -> usize {
    let mut covered = 0_usize;
    for sign_group in conflict_groups(sign_groups) {
        let sign_members: std::collections::BTreeSet<usize> = sign_group
            .member_construction_indices
            .iter()
            .copied()
            .collect();
        let ok = bool_groups.iter().any(|bool_group| {
            let bool_members: std::collections::BTreeSet<usize> = bool_group
                .member_construction_indices
                .iter()
                .copied()
                .collect();
            sign_members.is_subset(&bool_members)
        });
        if ok {
            covered += 1;
        }
    }
    covered
}

fn run_once(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    if source_revision.trim().is_empty() {
        return Err("FIELDLAB_SOURCE_REVISION must be non-empty".into());
    }
    let panel = panel_records();
    if panel.len() != 9 {
        return Err("FL-B3 panel size drift".into());
    }
    let b1 = b1_bank()?;
    let rich = rich_bank()?;
    if b1.len() != 16 || rich.len() != 32 {
        return Err("FL-B3 predicate bank size drift".into());
    }

    let mut cases = Vec::with_capacity(9);
    let mut max_error = 0.0_f64;
    let mut b1_keys = Vec::with_capacity(9);
    let mut rich_keys = Vec::with_capacity(9);
    for (construction_index, (signs, label)) in panel.iter().enumerate() {
        let state = state_from_signs(signs)?;
        let error = max_norm_squared_error(&state);
        if !error.is_finite() || error > VALIDITY_NORM_SQUARED_TOLERANCE {
            return Err(
                format!("FL-B3 unit-state validity failed at case {construction_index}").into(),
            );
        }
        max_error = max_error.max(error);
        let b1_code = evaluate_predicates(&state, &b1)?;
        let rich_code = evaluate_predicates(&state, &rich)?;
        b1_keys.push((b1_code.clone(), *label));
        rich_keys.push((rich_code.clone(), *label));
        cases.push(CaseRecord {
            case_id: format!("flb3|k{construction_index}"),
            construction_index,
            signs: signs.clone(),
            label: *label,
            b1_code,
            rich_code,
        });
    }

    let sign_groups = sign_group_records(&panel)?;
    let b1_groups = group_records_from_keys(&b1_keys);
    let rich_groups = group_records_from_keys(&rich_keys);
    let sign_ceiling = ceiling(&sign_groups);
    let b1_ceiling = ceiling(&b1_groups);
    let rich_ceiling = ceiling(&rich_groups);
    let sign_conflict_group_count = conflict_groups(&sign_groups).len();
    let b1_collision_group_count = b1_groups
        .iter()
        .filter(|group| group.member_construction_indices.len() > 1)
        .count();
    let rich_collision_group_count = rich_groups
        .iter()
        .filter(|group| group.member_construction_indices.len() > 1)
        .count();
    let covered_sign_conflict_groups_b1 = boolean_covers_sign_conflicts(&sign_groups, &b1_groups);
    let covered_sign_conflict_groups_rich =
        boolean_covers_sign_conflicts(&sign_groups, &rich_groups);

    let hb3_0 = cases.len() == 9
        && b1.len() == 16
        && rich.len() == 32
        && max_error <= VALIDITY_NORM_SQUARED_TOLERANCE
        && sign_ceiling == 7
        && sign_conflict_group_count == 2;
    let hb3_1 = b1_collision_group_count >= 1;
    let hb3_2 = covered_sign_conflict_groups_b1 == sign_conflict_group_count
        && sign_conflict_group_count > 0;
    let hb3_3 = b1_ceiling == sign_ceiling;
    let hb3_4 = rich_ceiling <= 7 && covered_sign_conflict_groups_rich == sign_conflict_group_count;

    Ok(Report {
        schema: REPORT_SCHEMA,
        predicate_schema: BOOLEAN_FIELD_PREDICATE_SCHEMA,
        source_revision: source_revision.to_owned(),
        case_count: cases.len(),
        b1_predicates: predicate_records(&b1),
        rich_predicates: predicate_records(&rich),
        cases,
        sign_groups,
        b1_groups,
        rich_groups,
        sign_ceiling,
        b1_ceiling,
        rich_ceiling,
        sign_conflict_group_count,
        b1_collision_group_count,
        rich_collision_group_count,
        covered_sign_conflict_groups_b1,
        covered_sign_conflict_groups_rich,
        max_norm_squared_error: max_error,
        replay_exact: false,
        hypotheses: HypothesisOutcomes {
            hb3_0_protocol_valid: hb3_0,
            hb3_1_b1_collisions_exist: hb3_1,
            hb3_2_conflict_coverage: hb3_2,
            hb3_3_ceiling_match: hb3_3,
            hb3_4_richer_bank_does_not_invent_identity: hb3_4,
        },
    })
}

fn run_replayed(source_revision: &str) -> Result<Report, Box<dyn Error>> {
    let first = run_once(source_revision)?;
    let second = run_once(source_revision)?;
    if first != second {
        return Err("FL-B3 semantic replay mismatch".into());
    }
    let mut report = first;
    report.replay_exact = true;
    report.hypotheses.hb3_0_protocol_valid =
        report.hypotheses.hb3_0_protocol_valid && report.replay_exact;
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_revision = std::env::var("FIELDLAB_SOURCE_REVISION")
        .map_err(|_| "FIELDLAB_SOURCE_REVISION must identify the executed repository revision")?;
    let report = run_replayed(&source_revision)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.hypotheses.hb3_0_protocol_valid {
        return Err("FL-B3 execution gate failed; retain output for diagnosis".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{b1_bank, panel_records, rich_bank, run_replayed};

    #[test]
    fn frozen_protocol_has_declared_shape() {
        assert_eq!(b1_bank().expect("b1").len(), 16);
        assert_eq!(rich_bank().expect("rich").len(), 32);
        assert_eq!(panel_records().len(), 9);
    }

    #[test]
    fn campaign_shape_and_replay_are_valid_without_asserting_scientific_outcomes() {
        let report = run_replayed("test-revision").expect("protocol-valid campaign");
        assert_eq!(report.case_count, 9);
        assert_eq!(report.cases.len(), 9);
        assert!(report.replay_exact);
        assert!(report.hypotheses.hb3_0_protocol_valid);
        assert_eq!(report.sign_ceiling, 7);
        assert_eq!(report.sign_conflict_group_count, 2);
    }
}
