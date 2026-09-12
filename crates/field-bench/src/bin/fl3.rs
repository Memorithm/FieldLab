#![forbid(unsafe_code)]

use field_core::{CouplingGraph, EnergyModel, FieldState, NodeState};
use field_dynamics::{run_steps, IntegratorConfig};
use field_hysteresis::{Relay, RelayState};
use std::error::Error;

const SEGMENT_LEN: usize = 20;
const SEGMENT_COUNT: usize = 4;
const OBSERVATION_COUNT: usize = SEGMENT_LEN * SEGMENT_COUNT;
const TRANSITION_RAMP_LEN: usize = 6;
const TRUE_TRANSITIONS: usize = 3;
const CONTRADICTION_COUNT: usize = 8;
const FIELD_STEPS: usize = 32;
const FIELD_DT: f64 = 0.05;
const FIELD_MOBILITY: f64 = 1.0;
const HYSTERESIS_GAIN: f64 = 0.35;
const LOCK_IN_LIMIT: usize = 3;
const LOOP_STEP: f64 = 0.05;
const LOOP_TOLERANCE: f64 = 1.0e-12;
const THRESHOLDS: [f64; 6] = [0.10, 0.20, 0.30, 0.40, 0.50, 0.60];
const RAMP_MAGNITUDES: [f64; TRANSITION_RAMP_LEN] = [0.10, 0.20, 0.30, 0.40, 0.55, 0.70];
const CONTRADICTION_OFFSETS: [usize; 2] = [9, 14];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Observation {
    truth: i8,
    evidence: f64,
    in_transition: bool,
    contradiction: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RelayLoop {
    threshold: f64,
    down_switch: f64,
    up_switch: f64,
    width: f64,
    valid: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ConditionMetrics {
    name: String,
    threshold: Option<f64>,
    total_errors: usize,
    contradiction_errors: usize,
    transition_errors: usize,
    false_context_changes: usize,
    switch_latencies: [usize; TRUE_TRANSITIONS],
    mean_switch_latency: f64,
    lock_in_events: usize,
    finite: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = context_fixture();
    let fixture_valid = validate_fixture(&fixture);

    let loops = THRESHOLDS
        .iter()
        .copied()
        .map(measure_relay_loop)
        .collect::<Result<Vec<_>, _>>()?;
    let loop_reference_valid = loops.iter().all(|loop_result| loop_result.valid);
    let loop_replay_equal = loops == THRESHOLDS
        .iter()
        .copied()
        .map(measure_relay_loop)
        .collect::<Result<Vec<_>, _>>()?;

    let baseline = evaluate_memoryless(&fixture)?;
    let conditions = THRESHOLDS
        .iter()
        .copied()
        .map(|threshold| evaluate_hysteretic(&fixture, threshold))
        .collect::<Result<Vec<_>, _>>()?;
    let replay_conditions = THRESHOLDS
        .iter()
        .copied()
        .map(|threshold| evaluate_hysteretic(&fixture, threshold))
        .collect::<Result<Vec<_>, _>>()?;
    let replay_equal = baseline == evaluate_memoryless(&fixture)? && conditions == replay_conditions;
    let finite = baseline.finite && conditions.iter().all(|condition| condition.finite);

    let h3_a1 = conditions
        .iter()
        .any(|condition| condition.total_errors < baseline.total_errors);
    let h3_a2 = conditions
        .iter()
        .any(|condition| condition.contradiction_errors == 0);
    let h3_a3 = conditions
        .iter()
        .filter(|condition| condition.contradiction_errors == 0)
        .all(|condition| condition.mean_switch_latency > 0.0);
    let best_lower = conditions
        .iter()
        .filter(|condition| condition.threshold.is_some_and(|threshold| threshold <= 0.30))
        .min_by_key(|condition| condition.total_errors)
        .ok_or("missing lower-threshold condition")?;
    let h3_a4 = conditions.iter().any(|condition| {
        condition.threshold.is_some_and(|threshold| threshold > 0.30)
            && condition.lock_in_events > best_lower.lock_in_events
    });

    let protocol_valid = fixture_valid
        && loop_reference_valid
        && loop_replay_equal
        && replay_equal
        && finite;

    let manifest = format!(
        "fl3|segments={SEGMENT_COUNT}|segment_len={SEGMENT_LEN}|ramp=0.10,0.20,0.30,0.40,0.55,0.70|contradiction_offsets=9,14|contradiction_mag=0.25|stable_mag=0.55|thresholds=0.10,0.20,0.30,0.40,0.50,0.60|gain={HYSTERESIS_GAIN:.17}|field_steps={FIELD_STEPS}|dt={FIELD_DT:.17}|mobility={FIELD_MOBILITY:.17}|lock_in_limit={LOCK_IN_LIMIT}"
    );
    let fingerprint = fnv1a64(manifest.as_bytes());

    println!("{{");
    println!("  \"experiment\": \"FL-3\",");
    println!("  \"protocol\": \"hysteresis-context-switch-v1\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    println!("  \"fixture\": {{\"observations\": {OBSERVATION_COUNT}, \"true_transitions\": {TRUE_TRANSITIONS}, \"contradictory_pulses\": {CONTRADICTION_COUNT}}},");
    println!("  \"relay_loops\": [");
    for (index, loop_result) in loops.iter().enumerate() {
        let suffix = if index + 1 == loops.len() { "" } else { "," };
        println!(
            "    {{\"threshold\": {:.2}, \"down_switch\": {:.6}, \"up_switch\": {:.6}, \"width\": {:.6}, \"valid\": {}}}{suffix}",
            loop_result.threshold,
            loop_result.down_switch,
            loop_result.up_switch,
            loop_result.width,
            loop_result.valid,
        );
    }
    println!("  ],");
    print_condition("baseline", &baseline, true);
    println!("  \"hysteretic_conditions\": [");
    for (index, condition) in conditions.iter().enumerate() {
        let suffix = if index + 1 == conditions.len() { "" } else { "," };
        print_condition_entry(condition, suffix);
    }
    println!("  ],");
    println!("  \"hypotheses\": {{");
    println!("    \"H3_A1_useful_retention\": {h3_a1},");
    println!("    \"H3_A2_disturbance_rejection\": {h3_a2},");
    println!("    \"H3_A3_switching_cost\": {h3_a3},");
    println!("    \"H3_A4_lock_in_frontier\": {h3_a4}");
    println!("  }},");
    println!("  \"fixture_valid\": {fixture_valid},");
    println!("  \"relay_reference_valid\": {loop_reference_valid},");
    println!("  \"replay_equal\": {},", replay_equal && loop_replay_equal);
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");

    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn context_fixture() -> Vec<Observation> {
    let truths = [1_i8, -1, 1, -1];
    let mut fixture = Vec::with_capacity(OBSERVATION_COUNT);
    for (segment_index, truth) in truths.into_iter().enumerate() {
        for offset in 0..SEGMENT_LEN {
            let in_transition = segment_index > 0 && offset < TRANSITION_RAMP_LEN;
            let contradiction = CONTRADICTION_OFFSETS.contains(&offset);
            let evidence = if in_transition {
                f64::from(truth) * RAMP_MAGNITUDES[offset]
            } else if contradiction {
                -f64::from(truth) * 0.25
            } else {
                f64::from(truth) * 0.55
            };
            fixture.push(Observation {
                truth,
                evidence,
                in_transition,
                contradiction,
            });
        }
    }
    fixture
}

fn validate_fixture(fixture: &[Observation]) -> bool {
    if fixture.len() != OBSERVATION_COUNT {
        return false;
    }
    let contradictions = fixture.iter().filter(|item| item.contradiction).count();
    let transition_observations = fixture.iter().filter(|item| item.in_transition).count();
    let transitions = fixture
        .windows(2)
        .filter(|window| window[0].truth != window[1].truth)
        .count();
    contradictions == CONTRADICTION_COUNT
        && transition_observations == TRUE_TRANSITIONS * TRANSITION_RAMP_LEN
        && transitions == TRUE_TRANSITIONS
        && fixture.iter().all(|item| item.evidence.is_finite())
}

fn measure_relay_loop(threshold: f64) -> Result<RelayLoop, Box<dyn Error>> {
    let mut relay = Relay::symmetric(threshold, RelayState::Positive)?;
    let mut down_switch = None;
    for step in (-20_i32..=20).rev() {
        let input = f64::from(step) / 20.0;
        if relay.update(input)? == RelayState::Negative && down_switch.is_none() {
            down_switch = Some(input);
        }
    }
    let mut up_switch = None;
    for step in -20_i32..=20 {
        let input = f64::from(step) / 20.0;
        if relay.update(input)? == RelayState::Positive && up_switch.is_none() {
            up_switch = Some(input);
        }
    }
    let down_switch = down_switch.ok_or("relay never switched negative")?;
    let up_switch = up_switch.ok_or("relay never switched positive")?;
    let width = up_switch - down_switch;
    let valid = down_switch <= -threshold + LOOP_TOLERANCE
        && down_switch >= -threshold - LOOP_STEP - LOOP_TOLERANCE
        && up_switch >= threshold - LOOP_TOLERANCE
        && up_switch <= threshold + LOOP_STEP + LOOP_TOLERANCE
        && width > 0.0;
    Ok(RelayLoop {
        threshold,
        down_switch,
        up_switch,
        width,
        valid,
    })
}

fn evaluate_memoryless(fixture: &[Observation]) -> Result<ConditionMetrics, Box<dyn Error>> {
    let predictions = fixture
        .iter()
        .map(|item| field_response(item.evidence))
        .collect::<Result<Vec<_>, _>>()?;
    metrics("memoryless", None, fixture, &predictions)
}

fn evaluate_hysteretic(
    fixture: &[Observation],
    threshold: f64,
) -> Result<ConditionMetrics, Box<dyn Error>> {
    let mut relay = Relay::symmetric(threshold, RelayState::Positive)?;
    let mut predictions = Vec::with_capacity(fixture.len());
    for item in fixture {
        relay.update(item.evidence)?;
        let effective = item.evidence + relay.signed_bias(HYSTERESIS_GAIN)?;
        predictions.push(field_response(effective)?);
    }
    metrics(
        &format!("relay-{threshold:.2}"),
        Some(threshold),
        fixture,
        &predictions,
    )
}

fn field_response(effective_evidence: f64) -> Result<i8, Box<dyn Error>> {
    if !effective_evidence.is_finite() || effective_evidence == 0.0 {
        return Err("effective evidence must be finite and non-zero".into());
    }
    let initial = FieldState::new(vec![NodeState::try_unit(vec![0.0, 1.0], 1.0e-12)?])?;
    let graph = CouplingGraph::new(1, Vec::new())?;
    let model = EnergyModel::new(graph, vec![vec![effective_evidence, 0.0]])?;
    let final_state = run_steps(
        initial,
        &model,
        IntegratorConfig {
            dt: FIELD_DT,
            mobility: FIELD_MOBILITY,
        },
        FIELD_STEPS,
    )?;
    let first_component = final_state
        .node(0)
        .ok_or("missing FL-3 field node")?
        .values()[0];
    if !first_component.is_finite() || first_component == 0.0 {
        return Err("invalid FL-3 field response".into());
    }
    Ok(if first_component > 0.0 { 1 } else { -1 })
}

fn metrics(
    name: &str,
    threshold: Option<f64>,
    fixture: &[Observation],
    predictions: &[i8],
) -> Result<ConditionMetrics, Box<dyn Error>> {
    if fixture.len() != predictions.len() {
        return Err("fixture/prediction length mismatch".into());
    }
    let total_errors = fixture
        .iter()
        .zip(predictions)
        .filter(|(item, prediction)| item.truth != **prediction)
        .count();
    let contradiction_errors = fixture
        .iter()
        .zip(predictions)
        .filter(|(item, prediction)| item.contradiction && item.truth != **prediction)
        .count();
    let transition_errors = fixture
        .iter()
        .zip(predictions)
        .filter(|(item, prediction)| item.in_transition && item.truth != **prediction)
        .count();
    let false_context_changes = (1..fixture.len())
        .filter(|&index| {
            !fixture[index].in_transition
                && fixture[index].truth == fixture[index - 1].truth
                && predictions[index] != predictions[index - 1]
        })
        .count();

    let mut switch_latencies = [0_usize; TRUE_TRANSITIONS];
    let mut lock_in_events = 0_usize;
    for (transition_number, start) in [SEGMENT_LEN, 2 * SEGMENT_LEN, 3 * SEGMENT_LEN]
        .into_iter()
        .enumerate()
    {
        let truth = fixture[start].truth;
        let latency = (0..TRANSITION_RAMP_LEN)
            .find(|offset| predictions[start + offset] == truth)
            .unwrap_or(TRANSITION_RAMP_LEN);
        switch_latencies[transition_number] = latency;
        lock_in_events += usize::from(latency > LOCK_IN_LIMIT);
    }
    let latency_sum = switch_latencies.iter().sum::<usize>();
    let mean_switch_latency = usize_to_f64(latency_sum) / usize_to_f64(TRUE_TRANSITIONS);

    Ok(ConditionMetrics {
        name: name.to_owned(),
        threshold,
        total_errors,
        contradiction_errors,
        transition_errors,
        false_context_changes,
        switch_latencies,
        mean_switch_latency,
        lock_in_events,
        finite: mean_switch_latency.is_finite(),
    })
}

fn print_condition(key: &str, condition: &ConditionMetrics, comma: bool) {
    let suffix = if comma { "," } else { "" };
    println!(
        "  \"{key}\": {{\"total_errors\": {}, \"contradiction_errors\": {}, \"transition_errors\": {}, \"false_context_changes\": {}, \"switch_latencies\": [{}, {}, {}], \"mean_switch_latency\": {:.9}, \"lock_in_events\": {}, \"finite\": {}}}{suffix}",
        condition.total_errors,
        condition.contradiction_errors,
        condition.transition_errors,
        condition.false_context_changes,
        condition.switch_latencies[0],
        condition.switch_latencies[1],
        condition.switch_latencies[2],
        condition.mean_switch_latency,
        condition.lock_in_events,
        condition.finite,
    );
}

fn print_condition_entry(condition: &ConditionMetrics, suffix: &str) {
    let threshold = condition.threshold.unwrap_or(0.0);
    println!(
        "    {{\"name\": \"{}\", \"threshold\": {:.2}, \"total_errors\": {}, \"contradiction_errors\": {}, \"transition_errors\": {}, \"false_context_changes\": {}, \"switch_latencies\": [{}, {}, {}], \"mean_switch_latency\": {:.9}, \"lock_in_events\": {}, \"finite\": {}}}{suffix}",
        condition.name,
        threshold,
        condition.total_errors,
        condition.contradiction_errors,
        condition.transition_errors,
        condition.false_context_changes,
        condition.switch_latencies[0],
        condition.switch_latencies[1],
        condition.switch_latencies[2],
        condition.mean_switch_latency,
        condition.lock_in_events,
        condition.finite,
    );
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
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
    fn fixture_has_declared_structure() {
        assert!(validate_fixture(&context_fixture()));
    }

    #[test]
    fn symmetric_relay_loops_match_grid_bounds() {
        for threshold in THRESHOLDS {
            assert!(measure_relay_loop(threshold).unwrap().valid);
        }
    }

    #[test]
    fn memoryless_fixture_has_zero_switch_latency() {
        let fixture = context_fixture();
        let metrics = evaluate_memoryless(&fixture).unwrap();
        assert_eq!(metrics.switch_latencies, [0, 0, 0]);
    }
}
