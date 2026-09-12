#![forbid(unsafe_code)]

use field_core::{
    dot, Coupling, CouplingGraph, EnergyModel, FieldState, LocalAnisotropy, NodeState,
    OperatorCoupling, OperatorEnergyModel, ValidationError,
};
use field_dynamics::{run_steps, IntegratorConfig};
use std::error::Error;

const STEPS: usize = 256;
const DT: f64 = 0.01;
const MOBILITY: f64 = 1.0;
const TOLERANCE: f64 = 1.0e-10;

fn main() -> Result<(), Box<dyn Error>> {
    let duplicate_edge_guard = duplicate_edge_guard();
    let (scalar_energy_error, scalar_field_error) = scalar_equivalence()?;
    let scalar_equivalence_valid =
        scalar_energy_error <= TOLERANCE && scalar_field_error <= TOLERANCE;
    let (e0_motion, e1_motion, e1_energy_drop, replay_equal) = operator_expressivity()?;
    let operator_expressivity_valid =
        e0_motion <= TOLERANCE && e1_motion > 0.5 && e1_energy_drop > 0.9 && replay_equal;
    let (anisotropy_alignment_before, anisotropy_alignment_after, anisotropy_energy_drop) =
        anisotropy_case()?;
    let anisotropy_valid = anisotropy_alignment_after > anisotropy_alignment_before
        && anisotropy_alignment_after > 0.99
        && anisotropy_energy_drop > 0.4;
    let protocol_valid = duplicate_edge_guard
        && scalar_equivalence_valid
        && operator_expressivity_valid
        && anisotropy_valid;

    println!("{{");
    println!("  \"experiment\": \"FL-E1\",");
    println!("  \"protocol\": \"energy-model-extension-v1\",");
    println!("  \"duplicate_edge_guard\": {duplicate_edge_guard},");
    println!("  \"scalar_equivalence\": {{");
    println!("    \"max_energy_error\": {scalar_energy_error:.12e},");
    println!("    \"max_field_error\": {scalar_field_error:.12e},");
    println!("    \"valid\": {scalar_equivalence_valid}");
    println!("  }},");
    println!("  \"operator_expressivity\": {{");
    println!("    \"e0_motion\": {e0_motion:.12},");
    println!("    \"e1_motion\": {e1_motion:.12},");
    println!("    \"e1_energy_drop\": {e1_energy_drop:.12},");
    println!("    \"replay_equal\": {replay_equal},");
    println!("    \"valid\": {operator_expressivity_valid}");
    println!("  }},");
    println!("  \"anisotropy\": {{");
    println!("    \"alignment_before\": {anisotropy_alignment_before:.12},");
    println!("    \"alignment_after\": {anisotropy_alignment_after:.12},");
    println!("    \"energy_drop\": {anisotropy_energy_drop:.12},");
    println!("    \"valid\": {anisotropy_valid}");
    println!("  }},");
    println!("  \"protocol_valid\": {protocol_valid}");
    println!("}}");

    if !protocol_valid {
        std::process::exit(1);
    }
    Ok(())
}

fn duplicate_edge_guard() -> bool {
    matches!(
        CouplingGraph::new(
            2,
            vec![
                Coupling {
                    source: 0,
                    target: 1,
                    weight: 1.0,
                },
                Coupling {
                    source: 1,
                    target: 0,
                    weight: 1.0,
                },
            ],
        ),
        Err(ValidationError::DuplicateCoupling {
            source: 0,
            target: 1
        })
    )
}

fn scalar_equivalence() -> Result<(f64, f64), Box<dyn Error>> {
    let states = [
        pair_from_angles(0.0, 90.0)?,
        pair_from_angles(30.0, -20.0)?,
        pair_from_angles(135.0, 220.0)?,
    ];
    let weights = [-1.25, -0.5, 0.0, 0.75, 1.5];
    let mut max_energy_error = 0.0_f64;
    let mut max_field_error = 0.0_f64;

    for weight in weights {
        let e0 = EnergyModel::new(
            CouplingGraph::new(
                2,
                vec![Coupling {
                    source: 0,
                    target: 1,
                    weight,
                }],
            )?,
            vec![vec![0.2, -0.3], vec![-0.4, 0.1]],
        )?;
        let e1 = OperatorEnergyModel::from_e0(&e0);
        for state in &states {
            max_energy_error = max_energy_error.max((e0.energy(state)? - e1.energy(state)?).abs());
            let fields0 = e0.effective_fields(state)?;
            let fields1 = e1.effective_fields(state)?;
            for (left, right) in fields0.iter().flatten().zip(fields1.iter().flatten()) {
                max_field_error = max_field_error.max((left - right).abs());
            }
        }
    }
    Ok((max_energy_error, max_field_error))
}

fn operator_expressivity() -> Result<(f64, f64, f64, bool), Box<dyn Error>> {
    let initial = pair_from_angles(0.0, 0.0)?;
    let config = IntegratorConfig {
        dt: DT,
        mobility: MOBILITY,
    };

    let e0 = EnergyModel::new(
        CouplingGraph::new(
            2,
            vec![Coupling {
                source: 0,
                target: 1,
                weight: 1.0,
            }],
        )?,
        vec![vec![0.0, 0.0]; 2],
    )?;
    let e0_final = run_steps(initial.clone(), &e0, config, STEPS)?;

    let e1 = OperatorEnergyModel::new(
        vec![vec![0.0, 0.0]; 2],
        vec![OperatorCoupling {
            source: 0,
            target: 1,
            operator: vec![vec![0.0, -1.0], vec![1.0, 0.0]],
        }],
        Vec::new(),
    )?;
    let initial_energy = e1.energy(&initial)?;
    let first = run_steps(initial.clone(), &e1, config, STEPS)?;
    let second = run_steps(initial.clone(), &e1, config, STEPS)?;
    let final_energy = e1.energy(&first)?;

    let e0_motion = state_distance(&initial, &e0_final)?;
    let e1_motion = state_distance(&initial, &first)?;
    Ok((
        e0_motion,
        e1_motion,
        initial_energy - final_energy,
        first == second,
    ))
}

fn anisotropy_case() -> Result<(f64, f64, f64), Box<dyn Error>> {
    let initial = FieldState::new(vec![node_from_angle(45.0)?])?;
    let model = OperatorEnergyModel::new(
        vec![vec![0.0, 0.0]],
        Vec::new(),
        vec![LocalAnisotropy {
            node: 0,
            matrix: vec![vec![2.0, 0.0], vec![0.0, 0.0]],
        }],
    )?;
    let initial_energy = model.energy(&initial)?;
    let final_state = run_steps(
        initial.clone(),
        &model,
        IntegratorConfig {
            dt: DT,
            mobility: MOBILITY,
        },
        STEPS,
    )?;
    let final_energy = model.energy(&final_state)?;
    let axis = [1.0, 0.0];
    let before = dot(
        initial.node(0).ok_or("missing initial node")?.values(),
        &axis,
    )
    .abs();
    let after = dot(
        final_state.node(0).ok_or("missing final node")?.values(),
        &axis,
    )
    .abs();
    Ok((before, after, initial_energy - final_energy))
}

fn pair_from_angles(first: f64, second: f64) -> Result<FieldState, Box<dyn Error>> {
    Ok(FieldState::new(vec![
        node_from_angle(first)?,
        node_from_angle(second)?,
    ])?)
}

fn node_from_angle(degrees: f64) -> Result<NodeState, Box<dyn Error>> {
    let radians = degrees.to_radians();
    Ok(NodeState::try_unit(
        vec![radians.cos(), radians.sin()],
        1.0e-12,
    )?)
}

fn state_distance(left: &FieldState, right: &FieldState) -> Result<f64, Box<dyn Error>> {
    if left.node_count() != right.node_count() || left.dimension() != right.dimension() {
        return Err("state shape mismatch".into());
    }
    let squared = left
        .nodes()
        .iter()
        .zip(right.nodes())
        .flat_map(|(left_node, right_node)| left_node.values().iter().zip(right_node.values()))
        .map(|(left_value, right_value)| {
            let difference = left_value - right_value;
            difference * difference
        })
        .sum::<f64>();
    Ok(squared.sqrt())
}
