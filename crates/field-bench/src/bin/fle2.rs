#![forbid(unsafe_code)]

use field_core::{
    dot, Coupling, CouplingGraph, EnergyModel, FieldState, NodeState, OperatorCoupling,
    OperatorEnergyModel,
};
use field_dynamics::{run_steps, IntegratorConfig};
use std::error::Error;

const DT: f64 = 0.01;
const MOBILITY: f64 = 1.0;
const DIRECT_STEPS: usize = 1024;
const COMPOSITION_STEPS: usize = 2048;
const DIRECT_ANCHOR: f64 = 4.0;
const COMPOSITION_ANCHOR: f64 = 8.0;
const DIRECT_TARGET_ANGLE: f64 = 17.0;
const COMPOSITION_MIDDLE_ANGLE: f64 = 17.0;
const COMPOSITION_TARGET_ANGLE: f64 = 83.0;
const ENERGY_TOLERANCE: f64 = 1.0e-10;
const SCALAR_GRID: [f64; 6] = [-2.0, -1.0, -0.5, 0.5, 1.0, 2.0];

type Matrix2 = [[f64; 2]; 2];

#[derive(Clone, Copy, Debug)]
struct Relation {
    name: &'static str,
    matrix: Matrix2,
}

#[derive(Clone, Copy, Debug)]
struct CompositionFixture {
    name: &'static str,
    first: Matrix2,
    second: Matrix2,
}

#[derive(Clone, Copy, Debug)]
struct DirectCase {
    target_cosine: f64,
    source_fidelity: f64,
    energy_nonincreasing: bool,
    replay_equal: bool,
    finite: bool,
}

#[derive(Clone, Copy, Debug)]
struct DirectResult {
    relation: &'static str,
    e1_mean_target_cosine: f64,
    e1_min_target_cosine: f64,
    e1_mean_source_fidelity: f64,
    e1_min_source_fidelity: f64,
    best_e0_mean_target_cosine: f64,
    best_e0_coupling: f64,
    energy_nonincreasing: bool,
    replay_equal: bool,
    finite: bool,
    protocol_valid: bool,
}

#[derive(Clone, Copy, Debug)]
struct CompositionCase {
    middle_cosine: f64,
    target_cosine: f64,
    source_fidelity: f64,
    replay_equal: bool,
    finite: bool,
}

#[derive(Clone, Copy, Debug)]
struct CompositionResult {
    fixture: &'static str,
    e1_mean_middle_cosine: f64,
    e1_min_middle_cosine: f64,
    e1_mean_target_cosine: f64,
    e1_min_target_cosine: f64,
    e1_min_source_fidelity: f64,
    best_e0_mean_target_cosine: f64,
    best_e0_first_coupling: f64,
    best_e0_second_coupling: f64,
    replay_equal: bool,
    finite: bool,
    protocol_valid: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let relations = relations();
    let direct = relations
        .iter()
        .map(run_direct_relation)
        .collect::<Result<Vec<_>, _>>()?;
    let fixtures = composition_fixtures();
    let composition = fixtures
        .iter()
        .map(run_composition_fixture)
        .collect::<Result<Vec<_>, _>>()?;

    let direct_protocol_valid = direct.iter().all(|result| result.protocol_valid);
    let composition_protocol_valid = composition.iter().all(|result| result.protocol_valid);
    let protocol_valid = direct_protocol_valid && composition_protocol_valid;

    let h_a1 = direct
        .iter()
        .take(2)
        .all(|result| result.best_e0_mean_target_cosine >= 0.999);
    let h_a2 = direct
        .iter()
        .skip(2)
        .all(|result| result.e1_mean_target_cosine >= 0.999);
    let h_a3 = direct
        .iter()
        .skip(2)
        .all(|result| result.best_e0_mean_target_cosine <= 0.05);
    let h_b1 = composition.iter().all(|result| result.protocol_valid);
    let h_b2 = composition
        .iter()
        .all(|result| result.best_e0_mean_target_cosine <= 0.05);
    let fingerprint = fnv1a64(manifest().as_bytes());

    print_report(
        &direct,
        &composition,
        [h_a1, h_a2, h_a3, h_b1, h_b2],
        protocol_valid,
        fingerprint,
    );

    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn run_direct_relation(relation: &Relation) -> Result<DirectResult, Box<dyn Error>> {
    let mut target_sum = 0.0;
    let mut source_sum = 0.0;
    let mut min_target = f64::INFINITY;
    let mut min_source = f64::INFINITY;
    let mut energy_nonincreasing = true;
    let mut replay_equal = true;
    let mut finite = true;

    for angle in source_angles() {
        let case = run_e1_direct_case(*relation, angle)?;
        target_sum += case.target_cosine;
        source_sum += case.source_fidelity;
        min_target = min_target.min(case.target_cosine);
        min_source = min_source.min(case.source_fidelity);
        energy_nonincreasing &= case.energy_nonincreasing;
        replay_equal &= case.replay_equal;
        finite &= case.finite;
    }

    let count = usize_to_f64(source_angles().len());
    let mean_target = target_sum / count;
    let mean_source = source_sum / count;
    let (best_e0_coupling, best_e0_mean_target_cosine) = best_e0_direct(*relation)?;
    let protocol_valid = finite
        && replay_equal
        && energy_nonincreasing
        && mean_target >= 0.999
        && min_target >= 0.99
        && min_source >= 0.995;

    Ok(DirectResult {
        relation: relation.name,
        e1_mean_target_cosine: mean_target,
        e1_min_target_cosine: min_target,
        e1_mean_source_fidelity: mean_source,
        e1_min_source_fidelity: min_source,
        best_e0_mean_target_cosine,
        best_e0_coupling,
        energy_nonincreasing,
        replay_equal,
        finite,
        protocol_valid,
    })
}

fn run_e1_direct_case(relation: Relation, angle: f64) -> Result<DirectCase, Box<dyn Error>> {
    let source = unit_vector(angle);
    let expected = matrix_vector(relation.matrix, source);
    let initial = FieldState::new(vec![node(angle)?, node(DIRECT_TARGET_ANGLE)?])?;
    let model = OperatorEnergyModel::new(
        vec![scaled_vector(source, DIRECT_ANCHOR), vec![0.0, 0.0]],
        vec![OperatorCoupling {
            source: 0,
            target: 1,
            operator: matrix_to_vec(transpose(relation.matrix)),
        }],
        Vec::new(),
    )?;
    let config = integrator();
    let initial_energy = model.energy(&initial)?;
    let first = run_steps(initial.clone(), &model, config, DIRECT_STEPS)?;
    let second = run_steps(initial, &model, config, DIRECT_STEPS)?;
    let final_energy = model.energy(&first)?;
    let final_source = first.node(0).ok_or("missing direct source")?.values();
    let final_target = first.node(1).ok_or("missing direct target")?.values();
    let target_cosine = dot(final_target, &expected);
    let source_fidelity = dot(final_source, &source);
    let finite = target_cosine.is_finite()
        && source_fidelity.is_finite()
        && initial_energy.is_finite()
        && final_energy.is_finite();

    Ok(DirectCase {
        target_cosine,
        source_fidelity,
        energy_nonincreasing: final_energy <= initial_energy + ENERGY_TOLERANCE,
        replay_equal: first == second,
        finite,
    })
}

fn best_e0_direct(relation: Relation) -> Result<(f64, f64), Box<dyn Error>> {
    let mut best_coupling = SCALAR_GRID[0];
    let mut best_score = f64::NEG_INFINITY;
    for coupling in SCALAR_GRID {
        let mut score_sum = 0.0;
        for angle in source_angles() {
            score_sum += run_e0_direct_case(relation, angle, coupling)?;
        }
        let mean = score_sum / usize_to_f64(source_angles().len());
        if mean > best_score {
            best_score = mean;
            best_coupling = coupling;
        }
    }
    Ok((best_coupling, best_score))
}

fn run_e0_direct_case(
    relation: Relation,
    angle: f64,
    coupling: f64,
) -> Result<f64, Box<dyn Error>> {
    let source = unit_vector(angle);
    let expected = matrix_vector(relation.matrix, source);
    let initial = FieldState::new(vec![node(angle)?, node(DIRECT_TARGET_ANGLE)?])?;
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
        vec![scaled_vector(source, DIRECT_ANCHOR), vec![0.0, 0.0]],
    )?;
    let final_state = run_steps(initial, &model, integrator(), DIRECT_STEPS)?;
    let target = final_state.node(1).ok_or("missing E0 direct target")?;
    Ok(dot(target.values(), &expected))
}

fn run_composition_fixture(
    fixture: &CompositionFixture,
) -> Result<CompositionResult, Box<dyn Error>> {
    let mut middle_sum = 0.0;
    let mut target_sum = 0.0;
    let mut min_middle = f64::INFINITY;
    let mut min_target = f64::INFINITY;
    let mut min_source = f64::INFINITY;
    let mut replay_equal = true;
    let mut finite = true;

    for angle in source_angles() {
        let case = run_e1_composition_case(*fixture, angle)?;
        middle_sum += case.middle_cosine;
        target_sum += case.target_cosine;
        min_middle = min_middle.min(case.middle_cosine);
        min_target = min_target.min(case.target_cosine);
        min_source = min_source.min(case.source_fidelity);
        replay_equal &= case.replay_equal;
        finite &= case.finite;
    }

    let count = usize_to_f64(source_angles().len());
    let mean_middle = middle_sum / count;
    let mean_target = target_sum / count;
    let (best_first, best_second, best_e0) = best_e0_composition(*fixture)?;
    let protocol_valid = finite
        && replay_equal
        && mean_middle >= 0.999
        && mean_target >= 0.999
        && min_target >= 0.99
        && min_source >= 0.995;

    Ok(CompositionResult {
        fixture: fixture.name,
        e1_mean_middle_cosine: mean_middle,
        e1_min_middle_cosine: min_middle,
        e1_mean_target_cosine: mean_target,
        e1_min_target_cosine: min_target,
        e1_min_source_fidelity: min_source,
        best_e0_mean_target_cosine: best_e0,
        best_e0_first_coupling: best_first,
        best_e0_second_coupling: best_second,
        replay_equal,
        finite,
        protocol_valid,
    })
}

fn run_e1_composition_case(
    fixture: CompositionFixture,
    angle: f64,
) -> Result<CompositionCase, Box<dyn Error>> {
    let source = unit_vector(angle);
    let expected_middle = matrix_vector(fixture.first, source);
    let expected_target = matrix_vector(fixture.second, expected_middle);
    let initial = FieldState::new(vec![
        node(angle)?,
        node(COMPOSITION_MIDDLE_ANGLE)?,
        node(COMPOSITION_TARGET_ANGLE)?,
    ])?;
    let model = OperatorEnergyModel::new(
        vec![
            scaled_vector(source, COMPOSITION_ANCHOR),
            vec![0.0, 0.0],
            vec![0.0, 0.0],
        ],
        vec![
            OperatorCoupling {
                source: 0,
                target: 1,
                operator: matrix_to_vec(transpose(fixture.first)),
            },
            OperatorCoupling {
                source: 1,
                target: 2,
                operator: matrix_to_vec(transpose(fixture.second)),
            },
        ],
        Vec::new(),
    )?;
    let config = integrator();
    let first = run_steps(initial.clone(), &model, config, COMPOSITION_STEPS)?;
    let second = run_steps(initial, &model, config, COMPOSITION_STEPS)?;
    let final_source = first.node(0).ok_or("missing composition source")?.values();
    let final_middle = first.node(1).ok_or("missing composition middle")?.values();
    let final_target = first.node(2).ok_or("missing composition target")?.values();
    let middle_cosine = dot(final_middle, &expected_middle);
    let target_cosine = dot(final_target, &expected_target);
    let source_fidelity = dot(final_source, &source);
    let finite = middle_cosine.is_finite()
        && target_cosine.is_finite()
        && source_fidelity.is_finite();

    Ok(CompositionCase {
        middle_cosine,
        target_cosine,
        source_fidelity,
        replay_equal: first == second,
        finite,
    })
}

fn best_e0_composition(
    fixture: CompositionFixture,
) -> Result<(f64, f64, f64), Box<dyn Error>> {
    let mut best_first = SCALAR_GRID[0];
    let mut best_second = SCALAR_GRID[0];
    let mut best_score = f64::NEG_INFINITY;
    for first in SCALAR_GRID {
        for second in SCALAR_GRID {
            let mut score_sum = 0.0;
            for angle in source_angles() {
                score_sum += run_e0_composition_case(fixture, angle, first, second)?;
            }
            let mean = score_sum / usize_to_f64(source_angles().len());
            if mean > best_score {
                best_score = mean;
                best_first = first;
                best_second = second;
            }
        }
    }
    Ok((best_first, best_second, best_score))
}

fn run_e0_composition_case(
    fixture: CompositionFixture,
    angle: f64,
    first: f64,
    second: f64,
) -> Result<f64, Box<dyn Error>> {
    let source = unit_vector(angle);
    let expected_middle = matrix_vector(fixture.first, source);
    let expected_target = matrix_vector(fixture.second, expected_middle);
    let initial = FieldState::new(vec![
        node(angle)?,
        node(COMPOSITION_MIDDLE_ANGLE)?,
        node(COMPOSITION_TARGET_ANGLE)?,
    ])?;
    let graph = CouplingGraph::new(
        3,
        vec![
            Coupling {
                source: 0,
                target: 1,
                weight: first,
            },
            Coupling {
                source: 1,
                target: 2,
                weight: second,
            },
        ],
    )?;
    let model = EnergyModel::new(
        graph,
        vec![
            scaled_vector(source, COMPOSITION_ANCHOR),
            vec![0.0, 0.0],
            vec![0.0, 0.0],
        ],
    )?;
    let final_state = run_steps(initial, &model, integrator(), COMPOSITION_STEPS)?;
    let target = final_state.node(2).ok_or("missing E0 composition target")?;
    Ok(dot(target.values(), &expected_target))
}

fn relations() -> Vec<Relation> {
    vec![
        Relation {
            name: "identity",
            matrix: [[1.0, 0.0], [0.0, 1.0]],
        },
        Relation {
            name: "inversion",
            matrix: [[-1.0, 0.0], [0.0, -1.0]],
        },
        Relation {
            name: "rotate_plus_90",
            matrix: rotation(90.0),
        },
        Relation {
            name: "rotate_minus_90",
            matrix: rotation(-90.0),
        },
        Relation {
            name: "reflect_x",
            matrix: [[1.0, 0.0], [0.0, -1.0]],
        },
        Relation {
            name: "swap_xy",
            matrix: [[0.0, 1.0], [1.0, 0.0]],
        },
    ]
}

fn composition_fixtures() -> Vec<CompositionFixture> {
    vec![
        CompositionFixture {
            name: "rotate_plus_90_then_reflect_x",
            first: rotation(90.0),
            second: [[1.0, 0.0], [0.0, -1.0]],
        },
        CompositionFixture {
            name: "reflect_x_then_rotate_plus_90",
            first: [[1.0, 0.0], [0.0, -1.0]],
            second: rotation(90.0),
        },
        CompositionFixture {
            name: "rotate_plus_45_then_rotate_plus_90",
            first: rotation(45.0),
            second: rotation(90.0),
        },
        CompositionFixture {
            name: "swap_then_rotate_minus_45",
            first: [[0.0, 1.0], [1.0, 0.0]],
            second: rotation(-45.0),
        },
    ]
}

fn source_angles() -> [f64; 16] {
    std::array::from_fn(|index| usize_to_f64(index) * 22.5)
}

fn rotation(degrees: f64) -> Matrix2 {
    let radians = degrees.to_radians();
    let cosine = radians.cos();
    let sine = radians.sin();
    [[cosine, -sine], [sine, cosine]]
}

fn transpose(matrix: Matrix2) -> Matrix2 {
    [
        [matrix[0][0], matrix[1][0]],
        [matrix[0][1], matrix[1][1]],
    ]
}

fn matrix_vector(matrix: Matrix2, vector: [f64; 2]) -> [f64; 2] {
    [
        matrix[0][0] * vector[0] + matrix[0][1] * vector[1],
        matrix[1][0] * vector[0] + matrix[1][1] * vector[1],
    ]
}

fn matrix_to_vec(matrix: Matrix2) -> Vec<Vec<f64>> {
    matrix.into_iter().map(<[f64; 2]>::to_vec).collect()
}

fn node(degrees: f64) -> Result<NodeState, Box<dyn Error>> {
    let values = unit_vector(degrees);
    Ok(NodeState::try_unit(values.to_vec(), 1.0e-12)?)
}

fn unit_vector(degrees: f64) -> [f64; 2] {
    let radians = degrees.to_radians();
    [radians.cos(), radians.sin()]
}

fn scaled_vector(vector: [f64; 2], scale: f64) -> Vec<f64> {
    vec![vector[0] * scale, vector[1] * scale]
}

const fn integrator() -> IntegratorConfig {
    IntegratorConfig {
        dt: DT,
        mobility: MOBILITY,
    }
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("FL-E2 bounded count fits u32"))
}

fn print_report(
    direct: &[DirectResult],
    composition: &[CompositionResult],
    hypotheses: [bool; 5],
    protocol_valid: bool,
    fingerprint: u64,
) {
    println!("{{");
    println!("  \"experiment\": \"FL-E2\",");
    println!("  \"protocol\": \"relational-transport-composition-v1\",");
    println!("  \"provenance_fingerprint\": \"fnv1a64:{fingerprint:016x}\",");
    print_direct_results(direct);
    print_composition_results(composition);
    println!("  \"hypotheses\": {{");
    println!("    \"H_E2_A1_control_preservation\": {},", hypotheses[0]);
    println!("    \"H_E2_A2_operator_transport\": {},", hypotheses[1]);
    println!("    \"H_E2_A3_scalar_insufficiency\": {},", hypotheses[2]);
    println!("    \"H_E2_B1_composition\": {},", hypotheses[3]);
    println!("    \"H_E2_B2_scalar_chain_insufficiency\": {}", hypotheses[4]);
    println!("  }},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");
}

fn print_direct_results(results: &[DirectResult]) {
    println!("  \"direct\": [");
    for (index, result) in results.iter().enumerate() {
        let suffix = if index + 1 == results.len() { "" } else { "," };
        println!(
            "    {{\"relation\": \"{}\", \"e1_mean_target_cosine\": {:.12}, \"e1_min_target_cosine\": {:.12}, \"e1_mean_source_fidelity\": {:.12}, \"e1_min_source_fidelity\": {:.12}, \"best_e0_mean_target_cosine\": {:.12}, \"best_e0_coupling\": {:.6}, \"energy_nonincreasing\": {}, \"replay_equal\": {}, \"finite\": {}, \"protocol_valid\": {}}}{suffix}",
            result.relation,
            result.e1_mean_target_cosine,
            result.e1_min_target_cosine,
            result.e1_mean_source_fidelity,
            result.e1_min_source_fidelity,
            result.best_e0_mean_target_cosine,
            result.best_e0_coupling,
            result.energy_nonincreasing,
            result.replay_equal,
            result.finite,
            result.protocol_valid,
        );
    }
    println!("  ],");
}

fn print_composition_results(results: &[CompositionResult]) {
    println!("  \"composition\": [");
    for (index, result) in results.iter().enumerate() {
        let suffix = if index + 1 == results.len() { "" } else { "," };
        println!(
            "    {{\"fixture\": \"{}\", \"e1_mean_middle_cosine\": {:.12}, \"e1_min_middle_cosine\": {:.12}, \"e1_mean_target_cosine\": {:.12}, \"e1_min_target_cosine\": {:.12}, \"e1_min_source_fidelity\": {:.12}, \"best_e0_mean_target_cosine\": {:.12}, \"best_e0_first_coupling\": {:.6}, \"best_e0_second_coupling\": {:.6}, \"replay_equal\": {}, \"finite\": {}, \"protocol_valid\": {}}}{suffix}",
            result.fixture,
            result.e1_mean_middle_cosine,
            result.e1_min_middle_cosine,
            result.e1_mean_target_cosine,
            result.e1_min_target_cosine,
            result.e1_min_source_fidelity,
            result.best_e0_mean_target_cosine,
            result.best_e0_first_coupling,
            result.best_e0_second_coupling,
            result.replay_equal,
            result.finite,
            result.protocol_valid,
        );
    }
    println!("  ],");
}

fn manifest() -> String {
    format!(
        "fle2|dt={DT:.17}|mobility={MOBILITY:.17}|angles=0:22.5:337.5|direct_steps={DIRECT_STEPS}|direct_anchor={DIRECT_ANCHOR:.17}|direct_target={DIRECT_TARGET_ANGLE:.17}|composition_steps={COMPOSITION_STEPS}|composition_anchor={COMPOSITION_ANCHOR:.17}|composition_middle={COMPOSITION_MIDDLE_ANGLE:.17}|composition_target={COMPOSITION_TARGET_ANGLE:.17}|scalar_grid=-2,-1,-0.5,0.5,1,2"
    )
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
