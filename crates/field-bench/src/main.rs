#![forbid(unsafe_code)]

use field_core::{dot, Coupling, CouplingGraph, EnergyModel, FieldState, NodeState};
use field_dynamics::{run_steps, IntegratorConfig};

#[derive(Clone, Copy, Debug)]
struct Fl0Config {
    dt: f64,
    mobility: f64,
    steps: usize,
}

#[derive(Debug)]
struct CaseResult {
    weight: f64,
    initial_energy: f64,
    final_energy: f64,
    final_dot: f64,
    max_norm_error: f64,
    replay_equal: bool,
    passed: bool,
}

fn main() {
    let config = Fl0Config {
        dt: 0.01,
        mobility: 1.0,
        steps: 100,
    };

    let attractive = run_case(config, 1.0, 0.9, true);
    let repulsive = run_case(config, -1.0, -0.9, false);
    let manifest = format!(
        "fl0|dt={:.17}|mobility={:.17}|steps={}|cases=attractive,repulsive|integrator=heun",
        config.dt, config.mobility, config.steps
    );
    let provenance_fingerprint = fnv1a64(manifest.as_bytes());
    let passed = attractive.passed && repulsive.passed;

    println!(
        concat!(
            "{{\n",
            "  \"experiment\": \"FL-0\",\n",
            "  \"integrator\": \"heun\",\n",
            "  \"dt\": {:.17},\n",
            "  \"mobility\": {:.17},\n",
            "  \"steps\": {},\n",
            "  \"provenance_fingerprint\": \"fnv1a64:{:016x}\",\n",
            "  \"attractive\": {{\"weight\": {:.17}, \"initial_energy\": {:.17}, \"final_energy\": {:.17}, \"final_dot\": {:.17}, \"max_norm_error\": {:.17}, \"replay_equal\": {}, \"passed\": {}}},\n",
            "  \"repulsive\": {{\"weight\": {:.17}, \"initial_energy\": {:.17}, \"final_energy\": {:.17}, \"final_dot\": {:.17}, \"max_norm_error\": {:.17}, \"replay_equal\": {}, \"passed\": {}}},\n",
            "  \"passed\": {}\n",
            "}}"
        ),
        config.dt,
        config.mobility,
        config.steps,
        provenance_fingerprint,
        attractive.weight,
        attractive.initial_energy,
        attractive.final_energy,
        attractive.final_dot,
        attractive.max_norm_error,
        attractive.replay_equal,
        attractive.passed,
        repulsive.weight,
        repulsive.initial_energy,
        repulsive.final_energy,
        repulsive.final_dot,
        repulsive.max_norm_error,
        repulsive.replay_equal,
        repulsive.passed,
        passed
    );

    if !passed {
        std::process::exit(1);
    }
}

fn run_case(config: Fl0Config, weight: f64, dot_threshold: f64, attractive: bool) -> CaseResult {
    let initial = initial_state();
    let model = model(weight);
    let integrator = IntegratorConfig {
        dt: config.dt,
        mobility: config.mobility,
    };
    let initial_energy = model.energy(&initial).expect("valid FL-0 initial state");
    let first = run_steps(initial.clone(), &model, integrator, config.steps)
        .expect("FL-0 integration must succeed");
    let second = run_steps(initial, &model, integrator, config.steps)
        .expect("FL-0 replay integration must succeed");
    let final_energy = model.energy(&first).expect("valid FL-0 final state");
    let final_dot = dot(
        first.node(0).expect("node 0").values(),
        first.node(1).expect("node 1").values(),
    );
    let max_norm_error = first
        .nodes()
        .iter()
        .map(|node| (dot(node.values(), node.values()) - 1.0).abs())
        .fold(0.0_f64, f64::max);
    let orientation_pass = if attractive {
        final_dot > dot_threshold
    } else {
        final_dot < dot_threshold
    };
    let replay_equal = first == second;
    let passed = final_energy < initial_energy
        && orientation_pass
        && max_norm_error <= 1.0e-12
        && replay_equal;

    CaseResult {
        weight,
        initial_energy,
        final_energy,
        final_dot,
        max_norm_error,
        replay_equal,
        passed,
    }
}

fn initial_state() -> FieldState {
    FieldState::new(vec![
        NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).expect("unit node 0"),
        NodeState::try_unit(vec![0.0, 1.0], 1.0e-12).expect("unit node 1"),
    ])
    .expect("valid FL-0 state")
}

fn model(weight: f64) -> EnergyModel {
    let graph = CouplingGraph::new(
        2,
        vec![Coupling {
            source: 0,
            target: 1,
            weight,
        }],
    )
    .expect("valid FL-0 graph");
    EnergyModel::new(graph, vec![vec![0.0, 0.0], vec![0.0, 0.0]]).expect("valid FL-0 energy model")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_fingerprint_is_stable() {
        assert_eq!(fnv1a64(b"FieldLab"), 0x454b_7e1f_3cb9_5cb6);
    }

    #[test]
    fn fl0_reference_cases_pass() {
        let config = Fl0Config {
            dt: 0.01,
            mobility: 1.0,
            steps: 100,
        };
        assert!(run_case(config, 1.0, 0.9, true).passed);
        assert!(run_case(config, -1.0, -0.9, false).passed);
    }
}
