#![forbid(unsafe_code)]

use field_core::{dot, Coupling, CouplingGraph, EnergyModel, FieldState, NodeState};
use field_dynamics::{run_steps, IntegratorConfig};
use std::error::Error;

const COMPETITION_STEPS: usize = 256;
const TRIANGLE_STEPS: usize = 512;
const DT: f64 = 0.02;
const MOBILITY: f64 = 1.0;
const CASES_PER_CONDITION: usize = 84;
const TRIANGLE_TOLERANCE: f64 = 1.0e-6;
const TIE_TOLERANCE: f64 = 1.0e-12;
const ENERGY_TOLERANCE: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug)]
struct CompetitionCondition {
    name: &'static str,
    coupling: f64,
}

#[derive(Clone, Copy, Debug)]
struct CompetitionResult {
    cases: usize,
    non_tie_cases: usize,
    correct_winners: usize,
    mean_useful_separation: f64,
    min_useful_separation: f64,
    max_tie_polarization: f64,
    monotonic_separation: bool,
    energy_nonincreasing: bool,
    finite: bool,
}

#[derive(Clone, Debug)]
struct TriangleResult {
    final_energy: f64,
    pairwise_dots: [f64; 3],
    max_dot_error: f64,
    energy_error: f64,
    replay_equal: bool,
    finite: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let integrator = IntegratorConfig {
        dt: DT,
        mobility: MOBILITY,
    };
    let conditions = [
        CompetitionCondition {
            name: "repulsive",
            coupling: -0.5,
        },
        CompetitionCondition {
            name: "uncoupled",
            coupling: 0.0,
        },
        CompetitionCondition {
            name: "attractive",
            coupling: 0.5,
        },
    ];

    let mut competition = Vec::new();
    for condition in conditions {
        competition.push((condition, run_competition(condition, integrator)?));
    }

    let repulsive_triangle = run_triangle(-1.0, -0.5, -1.5, integrator)?;
    let attractive_triangle = run_triangle(1.0, 1.0, -3.0, integrator)?;

    let repulsive = competition[0].1;
    let uncoupled = competition[1].1;
    let attractive = competition[2].1;

    let h2_a1 = repulsive.correct_winners == repulsive.non_tie_cases;
    let h2_a2 = repulsive.mean_useful_separation > uncoupled.mean_useful_separation;
    let h2_a3 = repulsive.mean_useful_separation > attractive.mean_useful_separation;
    let h2_a4 = repulsive.max_tie_polarization <= TIE_TOLERANCE;

    let case_counts_valid = competition
        .iter()
        .all(|(_, result)| result.cases == CASES_PER_CONDITION);
    let competition_finite = competition.iter().all(|(_, result)| result.finite);
    let triangles_valid = repulsive_triangle.max_dot_error <= TRIANGLE_TOLERANCE
        && repulsive_triangle.energy_error <= TRIANGLE_TOLERANCE
        && attractive_triangle.max_dot_error <= TRIANGLE_TOLERANCE
        && attractive_triangle.energy_error <= TRIANGLE_TOLERANCE;
    let replay_equal = competition_replay(integrator)?
        && repulsive_triangle.replay_equal
        && attractive_triangle.replay_equal;
    let protocol_valid = case_counts_valid
        && competition_finite
        && repulsive_triangle.finite
        && attractive_triangle.finite
        && triangles_valid
        && replay_equal;

    let manifest = format!(
        "fl2|competition_steps={COMPETITION_STEPS}|triangle_steps={TRIANGLE_STEPS}|dt={DT:.17}|mobility={MOBILITY:.17}|c=0,0.25,0.5,1|d=-1:0.1:1|J=-0.5,0,0.5|triangle_J=-1,1|triangle_angles=0,70,210"
    );
    let fingerprint = fnv1a64(manifest.as_bytes());

    println!("{{");
    println!("  \"experiment\": \"FL-2\",");
    println!("  \"protocol\": \"signed-competition-frustration-v1\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    println!("  \"competition\": [");
    for (index, (condition, result)) in competition.iter().enumerate() {
        let suffix = if index + 1 == competition.len() {
            ""
        } else {
            ","
        };
        println!(
            "    {{\"condition\": \"{}\", \"coupling\": {:.6}, \"cases\": {}, \"non_tie_cases\": {}, \"correct_winners\": {}, \"winner_rate\": {:.9}, \"mean_useful_separation\": {:.12}, \"min_useful_separation\": {:.12}, \"max_tie_polarization\": {:.12}, \"monotonic_separation\": {}, \"energy_nonincreasing\": {}, \"finite\": {}}}{suffix}",
            condition.name,
            condition.coupling,
            result.cases,
            result.non_tie_cases,
            result.correct_winners,
            ratio(result.correct_winners, result.non_tie_cases),
            result.mean_useful_separation,
            result.min_useful_separation,
            result.max_tie_polarization,
            result.monotonic_separation,
            result.energy_nonincreasing,
            result.finite,
        );
    }
    println!("  ],");
    print_triangle("repulsive_triangle", &repulsive_triangle);
    println!(",");
    print_triangle("attractive_triangle", &attractive_triangle);
    println!(",");
    println!("  \"hypotheses\": {{");
    println!("    \"H2_A1_repulsive_winner_correctness\": {h2_a1},");
    println!("    \"H2_A2_repulsive_gt_uncoupled_separation\": {h2_a2},");
    println!("    \"H2_A3_repulsive_gt_attractive_separation\": {h2_a3},");
    println!("    \"H2_A4_no_false_tie_polarization\": {h2_a4}");
    println!("  }},");
    println!("  \"replay_equal\": {replay_equal},");
    println!("  \"triangle_reference_valid\": {triangles_valid},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");

    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn run_competition(
    condition: CompetitionCondition,
    integrator: IntegratorConfig,
) -> Result<CompetitionResult, Box<dyn Error>> {
    let common_modes = [0.0, 0.25, 0.5, 1.0];
    let mut cases = 0_usize;
    let mut non_tie_cases = 0_usize;
    let mut correct_winners = 0_usize;
    let mut useful_sum = 0.0_f64;
    let mut min_useful = f64::INFINITY;
    let mut max_tie = 0.0_f64;
    let mut monotonic = true;
    let mut energy_nonincreasing = true;
    let mut finite = true;

    for common in common_modes {
        let mut positive = [0.0_f64; 10];
        let mut negative = [0.0_f64; 10];
        for differential_step in -10_i32..=10 {
            let differential = f64::from(differential_step) / 10.0;
            let (margin, initial_energy, final_energy) =
                run_competition_case(common, differential, condition.coupling, integrator)?;
            cases += 1;
            finite &= margin.is_finite() && initial_energy.is_finite() && final_energy.is_finite();
            energy_nonincreasing &= final_energy <= initial_energy + ENERGY_TOLERANCE;

            if differential_step == 0 {
                max_tie = max_tie.max(margin.abs());
                continue;
            }

            non_tie_cases += 1;
            let useful = differential.signum() * margin;
            useful_sum += useful;
            min_useful = min_useful.min(useful);
            correct_winners += usize::from(useful > 0.0);

            let index = usize::try_from(differential_step.unsigned_abs() - 1)
                .expect("differential index fits usize");
            if differential_step > 0 {
                positive[index] = useful;
            } else {
                negative[index] = useful;
            }
        }
        monotonic &= non_decreasing(&positive) && non_decreasing(&negative);
    }

    Ok(CompetitionResult {
        cases,
        non_tie_cases,
        correct_winners,
        mean_useful_separation: useful_sum / usize_to_f64(non_tie_cases),
        min_useful_separation: min_useful,
        max_tie_polarization: max_tie,
        monotonic_separation: monotonic,
        energy_nonincreasing,
        finite,
    })
}

fn run_competition_case(
    common: f64,
    differential: f64,
    coupling: f64,
    integrator: IntegratorConfig,
) -> Result<(f64, f64, f64), Box<dyn Error>> {
    let initial = neutral_pair()?;
    let graph = CouplingGraph::new(
        2,
        vec![Coupling {
            source: 0,
            target: 1,
            weight: coupling,
        }],
    )?;
    let model = EnergyModel::new(
        graph,
        vec![
            vec![common + differential, 0.0],
            vec![common - differential, 0.0],
        ],
    )?;
    let initial_energy = model.energy(&initial)?;
    let final_state = run_steps(initial, &model, integrator, COMPETITION_STEPS)?;
    let final_energy = model.energy(&final_state)?;
    let margin = (final_state.node(0).ok_or("missing node A")?.values()[0]
        - final_state.node(1).ok_or("missing node B")?.values()[0])
        / 2.0;
    Ok((margin, initial_energy, final_energy))
}

fn run_triangle(
    coupling: f64,
    expected_dot: f64,
    expected_energy: f64,
    integrator: IntegratorConfig,
) -> Result<TriangleResult, Box<dyn Error>> {
    let initial = triangle_initial()?;
    let model = triangle_model(coupling)?;
    let first = run_steps(initial.clone(), &model, integrator, TRIANGLE_STEPS)?;
    let second = run_steps(initial, &model, integrator, TRIANGLE_STEPS)?;
    let pairwise_dots = triangle_dots(&first)?;
    let final_energy = model.energy(&first)?;
    let max_dot_error = pairwise_dots
        .iter()
        .map(|value| (value - expected_dot).abs())
        .fold(0.0_f64, f64::max);
    let energy_error = (final_energy - expected_energy).abs();
    let finite = final_energy.is_finite()
        && max_dot_error.is_finite()
        && energy_error.is_finite()
        && pairwise_dots.iter().all(|value| value.is_finite());

    Ok(TriangleResult {
        final_energy,
        pairwise_dots,
        max_dot_error,
        energy_error,
        replay_equal: first == second,
        finite,
    })
}

fn competition_replay(integrator: IntegratorConfig) -> Result<bool, Box<dyn Error>> {
    let initial = neutral_pair()?;
    let graph = CouplingGraph::new(
        2,
        vec![Coupling {
            source: 0,
            target: 1,
            weight: -0.5,
        }],
    )?;
    let model = EnergyModel::new(graph, vec![vec![0.75, 0.0], vec![0.25, 0.0]])?;
    let first = run_steps(initial.clone(), &model, integrator, COMPETITION_STEPS)?;
    let second = run_steps(initial, &model, integrator, COMPETITION_STEPS)?;
    Ok(first == second)
}

fn neutral_pair() -> Result<FieldState, Box<dyn Error>> {
    Ok(FieldState::new(vec![
        NodeState::try_unit(vec![0.0, 1.0], 1.0e-12)?,
        NodeState::try_unit(vec![0.0, 1.0], 1.0e-12)?,
    ])?)
}

fn triangle_initial() -> Result<FieldState, Box<dyn Error>> {
    let angles = [0.0_f64, 70.0, 210.0];
    let nodes = angles
        .into_iter()
        .map(|degrees| {
            let radians = degrees.to_radians();
            NodeState::try_unit(vec![radians.cos(), radians.sin()], 1.0e-12)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
}

fn triangle_model(coupling: f64) -> Result<EnergyModel, Box<dyn Error>> {
    let graph = CouplingGraph::new(
        3,
        vec![
            Coupling {
                source: 0,
                target: 1,
                weight: coupling,
            },
            Coupling {
                source: 1,
                target: 2,
                weight: coupling,
            },
            Coupling {
                source: 2,
                target: 0,
                weight: coupling,
            },
        ],
    )?;
    Ok(EnergyModel::new(graph, vec![vec![0.0, 0.0]; 3])?)
}

fn triangle_dots(state: &FieldState) -> Result<[f64; 3], Box<dyn Error>> {
    let a = state.node(0).ok_or("missing triangle node 0")?;
    let b = state.node(1).ok_or("missing triangle node 1")?;
    let c = state.node(2).ok_or("missing triangle node 2")?;
    Ok([
        dot(a.values(), b.values()),
        dot(b.values(), c.values()),
        dot(c.values(), a.values()),
    ])
}

fn non_decreasing(values: &[f64]) -> bool {
    values
        .windows(2)
        .all(|window| window[1] + 1.0e-12 >= window[0])
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    usize_to_f64(numerator) / usize_to_f64(denominator)
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("FL-2 bounded count fits u32"))
}

fn print_triangle(name: &str, result: &TriangleResult) {
    println!(
        "  \"{name}\": {{\"final_energy\": {:.12}, \"pairwise_dots\": [{:.12}, {:.12}, {:.12}], \"max_dot_error\": {:.12}, \"energy_error\": {:.12}, \"replay_equal\": {}, \"finite\": {}}}",
        result.final_energy,
        result.pairwise_dots[0],
        result.pairwise_dots[1],
        result.pairwise_dots[2],
        result.max_dot_error,
        result.energy_error,
        result.replay_equal,
        result.finite,
    );
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
    fn declared_grid_has_eighty_four_cases_per_condition() {
        let count = 4 * (-10_i32..=10).count();
        assert_eq!(count, CASES_PER_CONDITION);
    }

    #[test]
    fn triangle_initial_state_is_valid() {
        let state = triangle_initial().unwrap();
        assert_eq!(state.node_count(), 3);
        assert_eq!(state.dimension(), 2);
    }

    #[test]
    fn monotonic_check_accepts_equal_or_increasing_values() {
        assert!(non_decreasing(&[0.1, 0.1, 0.2, 0.3]));
        assert!(!non_decreasing(&[0.1, 0.3, 0.2]));
    }
}
