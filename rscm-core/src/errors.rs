use crate::timeseries::FloatValue;
use thiserror::Error;

/// Error type for invalid operations.
#[derive(Error, Debug)]
pub enum RSCMError {
    #[error("{0}")]
    Error(String),
    #[error("Extrapolation is not allowed. Target={0}, {1} interpolation range={2}")]
    ExtrapolationNotAllowed(f32, String, f32),
    #[error("Wrong input units. Expected {0}, got {1}")]
    WrongUnits(String, String),
}

/// Convenience type for `Result<T, EosError>`.
pub type RSCMResult<T> = Result<T, RSCMError>;
