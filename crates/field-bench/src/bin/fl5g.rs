#![forbid(unsafe_code)]

use field_bench::qualification::{group_observables, AxialHessian, ObservableGroup};
use field_core::{
    dot, CouplingGraph, EnergyModel, FieldModel, FieldState, LocalAnisotropy, NodeState,
    OperatorCoupling, OperatorEnergyModel,
};
use field_dynamics::{heun_step, IntegratorConfig};
use field_memory::PatternBank;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const N: usize = 8;
const DURATION: f64 = 51.2;
const GRIDS: [(f64, usize); 3] = [(0.05, 1024), (0.025, 2048), (0.0125, 4096)];
const MARGIN: f64 = 0.25;
const FD_EPS: f64 = 1e-3;
const FD_TOL: f64 = 1e-5;
const NUMERIC_TOL: f64 = 1e-10;
const PARENT: &str = "9a403aa1bdd16d62f47434a838966ad7699a032d";

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Curvature {
    pattern: usize,
    stiffness: f64,
    row_lower_bound: f64,
    minimum_tested_rayleigh: f64,
    minimum_witness_mask: usize,
    stationary_tangent_residual: f64,
    maximum_second_difference_error: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ExitBracket {
    step: usize,
    lower_time: f64,
    upper_time: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Trajectory {
    model: String,
    pattern: usize,
    perturbation: usize,
    dt: f64,
    steps: usize,
    first_exit: Option<ExitBracket>,
    terminal_label: Option<usize>,
    final_max_axial_angle: f64,
    max_norm_squared_error: f64,
    max_energy_increase: f64,
    terminal_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Observability {
    records: usize,
    distinct_inputs: usize,
    conflicting_groups: usize,
    deterministic_maximum_correct: usize,
    groups: Vec<ObservableGroup<Vec<i8>, usize>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Campaign {
    stiffness: f64,
    zero_lift_max_error: f64,
    curvature: Vec<Curvature>,
    trajectories: Vec<Trajectory>,
    observability: Observability,
}

fn main() -> Result<()> {
    let first = run_campaign()?;
    let replay_equal = first == run_campaign()?;
    let derivative_valid = first.curvature.iter().all(|c| {
        c.stationary_tangent_residual <= 1e-12 && c.maximum_second_difference_error <= FD_TOL
    });
    let numerics_valid = first
        .trajectories
        .iter()
        .all(|t| t.max_norm_squared_error <= NUMERIC_TOL && t.max_energy_increase <= NUMERIC_TOL);
    let reference_valid = reference_128_valid(&first.trajectories);
    let certificate = first
        .curvature
        .iter()
        .filter(|c| c.stiffness > 0.0)
        .all(|c| c.row_lower_bound > 0.0);
    let retained = first
        .trajectories
        .iter()
        .filter(|t| t.model == "E1")
        .all(|t| t.first_exit.is_none() && t.final_max_axial_angle <= 1e-3);
    let protocol_valid = first.trajectories.len() == 180
        && first.curvature.len() == 6
        && first.zero_lift_max_error <= 1e-12
        && derivative_valid
        && numerics_valid
        && reference_valid
        && replay_equal;
    let report = serde_json::json!({
        "experiment": "FL-5G",
        "protocol": "axial-stability-observable-input-v1",
        "parent_commit": PARENT,
        "fieldlab_commit": std::env::var("FIELDLAB_SOURCE_SHA")
            .or_else(|_| std::env::var("GITHUB_SHA"))
            .unwrap_or_else(|_| "local".to_owned()),
        "preregistration_commit": "89d8c1ac02b72d873d96c0c187f98d891425d50a",
        "frozen_constants": {
            "nodes": N, "duration": DURATION, "dt_and_steps": GRIDS,
            "stiffness_margin": MARGIN, "fd_epsilon": FD_EPS, "fd_tolerance": FD_TOL,
            "numeric_tolerance": NUMERIC_TOL, "patterns": patterns(),
            "perturbations": "0=uniform transverse +.15; 1=uniform transverse -.15; 2..9=single angular coordinate +.01",
            "stiffness_rule": "max(0,-min_s row_lower_bound(B_s(0))) + .25",
            "directions": "all 256 sign vectors / sqrt(8), not eigenvectors"
        },
        "hypotheses": {
            "H5G_1_positive_curvature": certificate && derivative_valid,
            "H5G_2_perturbed_clean_retention": retained,
            "H5G_3_timestep_robustness": refinement_valid(&first.trajectories),
            "H5G_4_observable_label_conflict": first.observability.conflicting_groups > 0,
            "H5G_5_replay_and_invariants": replay_equal && numerics_valid
        },
        "validity": {
            "trajectory_count_exact": first.trajectories.len() == 180,
            "derivative_checks": derivative_valid,
            "numerical_invariants": numerics_valid,
            "historical_reference": reference_valid,
            "zero_lift_equivalence": first.zero_lift_max_error <= 1e-12,
            "replay_equal": replay_equal,
            "protocol_valid": protocol_valid
        },
        "campaign": first
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !protocol_valid {
        return Err("FL-5G execution gate failed; retain output for diagnosis".into());
    }
    Ok(())
}

fn patterns() -> Vec<Vec<i8>> {
    vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1, -1, -1],
        vec![1, 1, 1, 1, -1, -1, 1, 1],
    ]
}

fn model_e1(graph: &CouplingGraph, stiffness: f64) -> Result<OperatorEnergyModel> {
    let couplings = graph
        .couplings()
        .iter()
        .map(|e| OperatorCoupling {
            source: e.source,
            target: e.target,
            operator: vec![vec![e.weight, 0.0], vec![0.0, e.weight]],
        })
        .collect();
    let anisotropies = (0..graph.node_count())
        .map(|node| LocalAnisotropy {
            node,
            matrix: vec![vec![stiffness, 0.0], vec![0.0, 0.0]],
        })
        .collect();
    Ok(OperatorEnergyModel::new(
        vec![vec![0.0; 2]; N],
        couplings,
        anisotropies,
    )?)
}

fn angular_state(signs: &[i8], angles: &[f64]) -> Result<FieldState> {
    if signs.len() != angles.len() {
        return Err("angular chart dimension mismatch".into());
    }
    let nodes = signs
        .iter()
        .zip(angles)
        .map(|(s, q)| {
            let (sin, cos) = q.sin_cos();
            NodeState::try_unit(vec![f64::from(*s) * cos, f64::from(*s) * sin], 1e-12)
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(FieldState::new(nodes)?)
}

fn initial_state(signs: &[i8], perturbation: usize) -> Result<FieldState> {
    let mut angles = vec![0.0; N];
    match perturbation {
        0 | 1 => {
            let tilt = if perturbation == 0 { 0.15 } else { -0.15 };
            for (q, s) in angles.iter_mut().zip(signs) {
                *q = tilt * f64::from(*s);
            }
        }
        2..=9 => angles[perturbation - 2] = 0.01,
        _ => return Err("unknown frozen perturbation".into()),
    }
    angular_state(signs, &angles)
}

fn run_campaign() -> Result<Campaign> {
    let bank = PatternBank::new(patterns())?;
    let graph = bank.hebbian_graph()?;
    let e0 = bank.energy_model()?;
    let mut worst_bound = 0.0_f64;
    for signs in bank.patterns() {
        worst_bound = worst_bound.min(AxialHessian::new(&graph, signs, 0.0)?.lower_bound());
    }
    let stiffness = -worst_bound + MARGIN;
    let e1 = model_e1(&graph, stiffness)?;
    let zero = model_e1(&graph, 0.0)?;
    let mut zero_error = 0.0_f64;
    let mut curvature = Vec::new();
    let mut trajectories = Vec::new();
    for (pattern, signs) in bank.patterns().iter().enumerate() {
        for (name, a, model) in [
            ("E0", 0.0, &e0 as &dyn FieldModel),
            ("E1", stiffness, &e1 as &dyn FieldModel),
        ] {
            curvature.push(curvature_check(&graph, signs, pattern, a, model)?);
            for perturbation in 0..10 {
                let initial = initial_state(signs, perturbation)?;
                zero_error = zero_error.max(zero_lift_error(&e0, &zero, &initial)?);
                for (dt, steps) in GRIDS {
                    trajectories.push(trajectory(
                        &bank,
                        model,
                        initial.clone(),
                        name,
                        pattern,
                        perturbation,
                        dt,
                        steps,
                    )?);
                }
            }
        }
    }
    Ok(Campaign {
        stiffness,
        zero_lift_max_error: zero_error,
        curvature,
        trajectories,
        observability: observable_panel()?,
    })
}

fn zero_lift_error(
    e0: &EnergyModel,
    zero: &OperatorEnergyModel,
    state: &FieldState,
) -> Result<f64> {
    let mut error = (e0.energy(state)? - zero.energy(state)?).abs();
    let f0 = e0.effective_fields(state)?;
    let f1 = zero.effective_fields(state)?;
    for (a, b) in f0.iter().flatten().zip(f1.iter().flatten()) {
        error = error.max((a - b).abs());
    }
    Ok(error)
}

fn curvature_check(
    graph: &CouplingGraph,
    signs: &[i8],
    pattern: usize,
    stiffness: f64,
    model: &dyn FieldModel,
) -> Result<Curvature> {
    let h = AxialHessian::new(graph, signs, stiffness)?;
    let origin = angular_state(signs, &[0.0; N])?;
    let energy = model.energy(&origin)?;
    let fields = model.effective_fields(&origin)?;
    let mut residual = 0.0_f64;
    for (node, force) in origin.nodes().iter().zip(&fields) {
        let radial = dot(node.values(), force);
        for (x, f) in node.values().iter().zip(force) {
            residual = residual.max((f - radial * x).abs());
        }
    }
    let mut min_rayleigh = f64::INFINITY;
    let mut witness = 0;
    let mut fd_error = 0.0_f64;
    for mask in 0..(1_usize << N) {
        let v: Vec<f64> = (0..N)
            .map(|i| (if mask & (1 << i) == 0 { -1.0 } else { 1.0 }) / 8.0_f64.sqrt())
            .collect();
        let q = h.quadratic_form(&v)?;
        if q < min_rayleigh {
            min_rayleigh = q;
            witness = mask;
        }
        let plus: Vec<f64> = v.iter().map(|x| FD_EPS * x).collect();
        let minus: Vec<f64> = plus.iter().map(|x| -x).collect();
        let fd = (model.energy(&angular_state(signs, &plus)?)?
            + model.energy(&angular_state(signs, &minus)?)?
            - 2.0 * energy)
            / (FD_EPS * FD_EPS);
        fd_error = fd_error.max((fd - q).abs());
    }
    if ![h.lower_bound(), min_rayleigh, residual, fd_error]
        .iter()
        .all(|x| x.is_finite())
    {
        return Err("non-finite curvature diagnostic".into());
    }
    Ok(Curvature {
        pattern,
        stiffness,
        row_lower_bound: h.lower_bound(),
        minimum_tested_rayleigh: min_rayleigh,
        minimum_witness_mask: witness,
        stationary_tangent_residual: residual,
        maximum_second_difference_error: fd_error,
    })
}

#[allow(clippy::too_many_arguments)] // Frozen trajectory coordinates retained in the record.
fn trajectory(
    bank: &PatternBank,
    model: &dyn FieldModel,
    mut state: FieldState,
    name: &str,
    pattern: usize,
    perturbation: usize,
    dt: f64,
    steps: usize,
) -> Result<Trajectory> {
    let target = &bank.patterns()[pattern];
    let mut first_exit = None;
    let mut norm_error = 0.0_f64;
    let mut energy_increase = 0.0_f64;
    let mut previous_energy = model.energy(&state)?;
    let config = IntegratorConfig { dt, mobility: 1.0 };
    for step in 0..=steps {
        let energy = model.energy(&state)?;
        if !energy.is_finite() {
            return Err("non-finite trajectory energy".into());
        }
        energy_increase = energy_increase.max(energy - previous_energy);
        previous_energy = energy;
        for node in state.nodes() {
            let norm_sq = dot(node.values(), node.values());
            if !norm_sq.is_finite() || node.values().iter().any(|x| !x.is_finite()) {
                return Err("non-finite trajectory state".into());
            }
            norm_error = norm_error.max((norm_sq - 1.0).abs());
        }
        if first_exit.is_none() && bank.decode_state(&state)? != *target {
            first_exit = Some(ExitBracket {
                step,
                lower_time: f64::from(u32::try_from(step.saturating_sub(1))?) * dt,
                upper_time: f64::from(u32::try_from(step)?) * dt,
            });
        }
        if step < steps {
            state = heun_step(&state, model, config)?;
        }
    }
    let final_angle = state
        .nodes()
        .iter()
        .zip(target)
        .map(|(node, sign)| {
            (node.values()[0] * f64::from(*sign))
                .clamp(-1.0, 1.0)
                .acos()
        })
        .fold(0.0_f64, f64::max);
    if !final_angle.is_finite() {
        return Err("non-finite terminal angle".into());
    }
    let decoded = bank.decode_state(&state)?;
    let mut digest = Sha256::new();
    for node in state.nodes() {
        for x in node.values() {
            digest.update(x.to_bits().to_le_bytes());
        }
    }
    Ok(Trajectory {
        model: name.to_owned(),
        pattern,
        perturbation,
        dt,
        steps,
        first_exit,
        terminal_label: bank.patterns().iter().position(|p| *p == decoded),
        final_max_axial_angle: final_angle,
        max_norm_squared_error: norm_error,
        max_energy_increase: energy_increase,
        terminal_sha256: format!("{:x}", digest.finalize()),
    })
}

fn observable_panel() -> Result<Observability> {
    let p = patterns();
    let mut records: Vec<(Vec<i8>, usize)> = p.iter().cloned().zip(0..3).collect();
    records.push((p[1].clone(), 0));
    records.push((p[2].clone(), 0));
    for (target, positions) in [(1, [6, 7]), (2, [4, 5])] {
        for bit in positions {
            let mut cue = p[0].clone();
            cue[bit] = -1;
            records.push((cue, target));
        }
    }
    let groups = group_observables(&records)?;
    Ok(Observability {
        records: records.len(),
        distinct_inputs: groups.len(),
        conflicting_groups: groups.iter().filter(|g| g.conflicts()).count(),
        deterministic_maximum_correct: groups.iter().map(ObservableGroup::maximum_correct).sum(),
        groups,
    })
}

fn reference_128_valid(runs: &[Trajectory]) -> bool {
    (0..3).all(|p| {
        runs.iter()
            .find(|r| r.model == "E0" && r.pattern == p && r.perturbation == 0 && r.steps == 1024)
            .is_some_and(|r| {
                if p == 0 {
                    r.first_exit.is_none()
                } else {
                    r.first_exit.as_ref().is_some_and(|e| e.step == 145)
                }
            })
    })
}

fn refinement_valid(runs: &[Trajectory]) -> bool {
    (0..3).all(|p| {
        let selected: Vec<&Trajectory> = runs
            .iter()
            .filter(|r| r.model == "E0" && r.pattern == p && r.perturbation == 0)
            .collect();
        if selected.len() != 3 {
            return false;
        }
        if p == 0 {
            return selected.iter().all(|r| r.first_exit.is_none());
        }
        let exits: Vec<f64> = selected
            .iter()
            .filter_map(|r| r.first_exit.as_ref().map(|e| e.upper_time))
            .collect();
        exits.len() == 3
            && exits.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                - exits.iter().copied().fold(f64::INFINITY, f64::min)
                <= 0.10
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_horizon_is_matched() {
        for (dt, steps) in GRIDS {
            assert!((dt * f64::from(u32::try_from(steps).unwrap()) - DURATION).abs() < 1e-12);
        }
    }

    #[test]
    fn positive_tilt_matches_historical_encoding() {
        let bank = PatternBank::new(patterns()).unwrap();
        for p in bank.patterns() {
            let old = bank.encode_cue(p, 0.15).unwrap();
            let new = initial_state(p, 0).unwrap();
            for (a, b) in old.nodes().iter().zip(new.nodes()) {
                for (x, y) in a.values().iter().zip(b.values()) {
                    assert!((x - y).abs() < 1e-15);
                }
            }
        }
    }

    #[test]
    fn observable_conflicts_have_no_target_in_key() {
        let panel = observable_panel().unwrap();
        assert_eq!(panel.records, 9);
        assert_eq!(panel.distinct_inputs, 7);
        assert_eq!(panel.conflicting_groups, 2);
        assert_eq!(panel.deterministic_maximum_correct, 7);
    }

    #[test]
    fn angular_chart_rejects_shape_mismatch() {
        assert!(angular_state(&[1], &[]).is_err());
        assert!(initial_state(&patterns()[0], 10).is_err());
    }
}
