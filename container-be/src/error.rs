//! Provide a custom error struct
//!
//! Allow derriving MyError from other Error types from dependant packages.

use std::io;

use hamsrs::hamserror::HamsError;
use thiserror::Error;

/// Error type for handling errors on Sample
#[derive(Error, Debug)]
pub enum MyError {
    #[error("General error `{0}`")]
    Message(&'static str),
    #[error("Service Cancelled")]
    Cancelled,
    #[error("Serdes error `{0}`")]
    Serde(#[from] serde_json::Error),
    #[error("data store disconnected")]
    Io(#[from] io::Error),
    /// Error with JSON validation
    #[error("JsonValidation error `{0:?}`")]
    JsonValidation(Vec<String>),
    /// Validation error
    /// Improve this to carry an object to describe the details of the validation failure
    #[error("Validation error")]
    ValidationError(),
    #[error("Parquet error `{0}`")]
    ParquetError(#[from] parquet::errors::ParquetError),
    #[error("Figment error `{0}`")]
    FigmentError(#[from] figment::error::Error),
    #[error("Sqlx error `{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("SQLX Migrate error `{0}`")]
    SqlxMigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("PreFlight error")]
    PreflightCheck,
    /// Failed to complete shutdown check
    #[error("Shutdown error")]
    ShutdownCheck,
    #[error("Prometheus error `{0}`")]
    PrometheusError(#[from] prometheus::Error),
    #[error("HaMs error `{0}`")]
    HamsError(#[from] HamsError),
}
