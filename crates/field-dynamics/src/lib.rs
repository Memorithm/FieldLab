#![forbid(unsafe_code)]

use field_core::{dot, EnergyModel, FieldState, NodeState, ValidationError};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Deterministic integration parameters for the FL-0 dynamics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntegratorConfig {
    pub dt: f64,
    pub mobility: f64,
}

impl IntegratorConfig {
    /// Validates the integration parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when `dt` is not finite and positive or when mobility is non-finite or negative.
    pub fn validate(self) -> Result<Self, DynamicsError> {
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(DynamicsError::InvalidTimeStep);
        }
        if !self.mobility.is_finite() || self.mobility < 0.0 {
            return Err(DynamicsError::InvalidMobility);
        }
        Ok(self)
    }
}

/// Advances the projected dissipative field by one explicit Euler step.
///
/// # Errors
///
/// Returns an error for invalid integration parameters or an incompatible/invalid field state.
pub fn euler_step(
    state: &FieldState,
    model: &EnergyModel,
    config: IntegratorConfig,
) -> Result<FieldState, DynamicsError> {
    let config = config.validate()?;
    let fields = model.effective_fields(state)?;
    let next = state
        .nodes()
        .iter()
        .zip(fields)
        .map(|(node, field)| {
            let velocity = tangent_velocity(node.values(), &field, config.mobility);
            let candidate = node
                .values()
                .iter()
                .zip(velocity)
                .map(|(value, delta)| value + config.dt * delta)
                .collect();
            NodeState::from_normalized(candidate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(next)?)
}

/// Advances the projected dissipative field by one Heun predictor-corrector step.
///
/// # Errors
///
/// Returns an error for invalid integration parameters or an incompatible/invalid field state.
pub fn heun_step(
    state: &FieldState,
    model: &EnergyModel,
    config: IntegratorConfig,
) -> Result<FieldState, DynamicsError> {
    let config = config.validate()?;
    let first_fields = model.effective_fields(state)?;
    let first_velocity = state
        .nodes()
        .iter()
        .zip(&first_fields)
        .map(|(node, field)| tangent_velocity(node.values(), field, config.mobility))
        .collect::<Vec<_>>();

    let predictor_nodes = state
        .nodes()
        .iter()
        .zip(&first_velocity)
        .map(|(node, velocity)| {
            let candidate = node
                .values()
                .iter()
                .zip(velocity)
                .map(|(value, delta)| value + config.dt * delta)
                .collect();
            NodeState::from_normalized(candidate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let predictor = FieldState::new(predictor_nodes)?;
    let second_fields = model.effective_fields(&predictor)?;
    let second_velocity = predictor
        .nodes()
        .iter()
        .zip(&second_fields)
        .map(|(node, field)| tangent_velocity(node.values(), field, config.mobility))
        .collect::<Vec<_>>();

    let corrected = state
        .nodes()
        .iter()
        .zip(first_velocity.iter().zip(second_velocity))
        .map(|(node, (first, second))| {
            let candidate = node
                .values()
                .iter()
                .zip(first.iter().zip(second))
                .map(|(value, (k1, k2))| value + config.dt * 0.5 * (k1 + k2))
                .collect();
            NodeState::from_normalized(candidate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FieldState::new(corrected)?)
}

/// Executes a fixed number of deterministic Heun steps.
///
/// # Errors
///
/// Returns the first integration/state validation error encountered.
pub fn run_steps(
    mut state: FieldState,
    model: &EnergyModel,
    config: IntegratorConfig,
    steps: usize,
) -> Result<FieldState, DynamicsError> {
    for _ in 0..steps {
        state = heun_step(&state, model, config)?;
    }
    Ok(state)
}

fn tangent_velocity(state: &[f64], field: &[f64], mobility: f64) -> Vec<f64> {
    let radial = dot(state, field);
    state
        .iter()
        .zip(field)
        .map(|(component, force)| mobility * (force - radial * component))
        .collect()
}

/// Errors surfaced by deterministic field integration.
#[derive(Debug)]
pub enum DynamicsError {
    InvalidTimeStep,
    InvalidMobility,
    InvalidState(ValidationError),
}

impl Display for DynamicsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTimeStep => write!(formatter, "time step must be finite and positive"),
            Self::InvalidMobility => write!(formatter, "mobility must be finite and non-negative"),
            Self::InvalidState(error) => write!(formatter, "invalid field state: {error}"),
        }
    }
}

impl Error for DynamicsError {}

impl From<ValidationError> for DynamicsError {
    fn from(error: ValidationError) -> Self {
        Self::InvalidState(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use field_core::{Coupling, CouplingGraph, EnergyModel};

    fn orthogonal_pair(weight: f64) -> (FieldState, EnergyModel) {
        let state = FieldState::new(vec![
            NodeState::try_unit(vec![1.0, 0.0], 1.0e-12).unwrap(),
            NodeState::try_unit(vec![0.0, 1.0], 1.0e-12).unwrap(),
        ])
        .unwrap();
        let graph = CouplingGraph::new(
            2,
            vec![Coupling {
                source: 0,
                target: 1,
                weight,
            }],
        )
        .unwrap();
        let model = EnergyModel::new(graph, vec![vec![0.0, 0.0], vec![0.0, 0.0]]).unwrap();
        (state, model)
    }

    #[test]
    fn attractive_gradient_flow_decreases_energy() {
        let (state, model) = orthogonal_pair(1.0);
        let initial = model.energy(&state).unwrap();
        let final_state = run_steps(
            state,
            &model,
            IntegratorConfig {
                dt: 0.01,
                mobility: 1.0,
            },
            100,
        )
        .unwrap();
        let final_energy = model.energy(&final_state).unwrap();
        assert!(final_energy < initial);
        assert!(
            field_core::dot(
                final_state.node(0).unwrap().values(),
                final_state.node(1).unwrap().values()
            ) > 0.9
        );
    }

    #[test]
    fn repulsive_gradient_flow_prefers_antialignment() {
        let (state, model) = orthogonal_pair(-1.0);
        let final_state = run_steps(
            state,
            &model,
            IntegratorConfig {
                dt: 0.01,
                mobility: 1.0,
            },
            100,
        )
        .unwrap();
        assert!(
            field_core::dot(
                final_state.node(0).unwrap().values(),
                final_state.node(1).unwrap().values()
            ) < -0.9
        );
    }

    #[test]
    fn replay_is_deterministic() {
        let (state, model) = orthogonal_pair(1.0);
        let config = IntegratorConfig {
            dt: 0.01,
            mobility: 1.0,
        };
        let first = run_steps(state.clone(), &model, config, 64).unwrap();
        let second = run_steps(state, &model, config, 64).unwrap();
        assert_eq!(first, second);
    }
}
