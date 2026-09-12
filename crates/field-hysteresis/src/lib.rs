#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Binary internal state of a deterministic hysteresis relay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelayState {
    Negative,
    Positive,
}

impl RelayState {
    /// Returns the state as `-1.0` or `+1.0`.
    #[must_use]
    pub const fn signed(self) -> f64 {
        match self {
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }
}

/// Rate-independent two-threshold relay (Schmitt-type hysteron).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Relay {
    lower: f64,
    upper: f64,
    state: RelayState,
}

impl Relay {
    /// Creates a relay whose state is retained while the input lies strictly between the thresholds.
    ///
    /// # Errors
    ///
    /// Returns an error if either threshold is non-finite or `lower >= upper`.
    pub fn new(lower: f64, upper: f64, initial: RelayState) -> Result<Self, HysteresisError> {
        if !lower.is_finite() || !upper.is_finite() {
            return Err(HysteresisError::NonFiniteThreshold);
        }
        if lower >= upper {
            return Err(HysteresisError::InvalidThresholdOrder);
        }
        Ok(Self {
            lower,
            upper,
            state: initial,
        })
    }

    /// Creates a relay with symmetric thresholds `[-theta, +theta]`.
    ///
    /// # Errors
    ///
    /// Returns an error if `theta` is non-finite or not strictly positive.
    pub fn symmetric(theta: f64, initial: RelayState) -> Result<Self, HysteresisError> {
        if !theta.is_finite() {
            return Err(HysteresisError::NonFiniteThreshold);
        }
        if theta <= 0.0 {
            return Err(HysteresisError::InvalidThresholdOrder);
        }
        Self::new(-theta, theta, initial)
    }

    /// Updates and returns the internal state for one scalar input.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-finite input.
    pub fn update(&mut self, input: f64) -> Result<RelayState, HysteresisError> {
        if !input.is_finite() {
            return Err(HysteresisError::NonFiniteInput);
        }
        if input <= self.lower {
            self.state = RelayState::Negative;
        } else if input >= self.upper {
            self.state = RelayState::Positive;
        }
        Ok(self.state)
    }

    /// Returns the current relay state without changing it.
    #[must_use]
    pub const fn state(self) -> RelayState {
        self.state
    }

    /// Returns the lower switching threshold.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Returns the upper switching threshold.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }

    /// Returns a signed scalar bias with magnitude `gain`.
    ///
    /// # Errors
    ///
    /// Returns an error when `gain` is non-finite or negative.
    pub fn signed_bias(self, gain: f64) -> Result<f64, HysteresisError> {
        if !gain.is_finite() {
            return Err(HysteresisError::NonFiniteGain);
        }
        if gain < 0.0 {
            return Err(HysteresisError::NegativeGain);
        }
        Ok(self.state.signed() * gain)
    }
}

/// Validation failures for explicit hysteresis operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HysteresisError {
    NonFiniteThreshold,
    InvalidThresholdOrder,
    NonFiniteInput,
    NonFiniteGain,
    NegativeGain,
}

impl Display for HysteresisError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for HysteresisError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_state_inside_deadband() {
        let mut positive = Relay::symmetric(0.4, RelayState::Positive).unwrap();
        let mut negative = Relay::symmetric(0.4, RelayState::Negative).unwrap();
        assert_eq!(positive.update(0.0).unwrap(), RelayState::Positive);
        assert_eq!(negative.update(0.0).unwrap(), RelayState::Negative);
    }

    #[test]
    fn switches_at_declared_thresholds() {
        let mut relay = Relay::symmetric(0.3, RelayState::Positive).unwrap();
        assert_eq!(relay.update(-0.29).unwrap(), RelayState::Positive);
        assert_eq!(relay.update(-0.3).unwrap(), RelayState::Negative);
        assert_eq!(relay.update(0.29).unwrap(), RelayState::Negative);
        assert_eq!(relay.update(0.3).unwrap(), RelayState::Positive);
    }

    #[test]
    fn rejects_invalid_threshold_order() {
        assert_eq!(
            Relay::new(0.2, 0.2, RelayState::Positive).unwrap_err(),
            HysteresisError::InvalidThresholdOrder
        );
    }
}
