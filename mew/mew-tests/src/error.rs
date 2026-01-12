//! Error types for the examples framework.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for example operations.
pub type ExampleResult<T> = Result<T, ExampleError>;

/// Errors that can occur when running examples.
#[derive(Debug, Error)]
pub enum ExampleError {
    /// Failed to read a file.
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse operations file.
    #[error("failed to parse operations file '{path}': {message}")]
    OperationsParse { path: PathBuf, message: String },

    /// Failed to compile ontology.
    #[error("failed to compile ontology '{path}': {message}")]
    OntologyCompile { path: PathBuf, message: String },

    /// Failed to execute a step.
    #[error("step '{step}' failed: {message}")]
    StepExecution { step: String, message: String },

    /// Assertion failed.
    #[error("assertion failed for step '{step}': {message}")]
    AssertionFailed { step: String, message: String },

    /// Step not found in operations file.
    #[error("step '{step}' not found in operations file")]
    StepNotFound { step: String },

    /// Missing ontology.
    #[error("ontology not specified for scenario '{scenario}'")]
    MissingOntology { scenario: String },

    /// Missing operations file.
    #[error("operations file not found: {path}")]
    MissingOperations { path: PathBuf },

    /// Session error.
    #[error("session error: {0}")]
    Session(#[from] mew_session::SessionError),
}

impl ExampleError {
    pub fn file_read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::FileRead {
            path: path.into(),
            source,
        }
    }

    pub fn operations_parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::OperationsParse {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn ontology_compile(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::OntologyCompile {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn step_execution(step: impl Into<String>, message: impl Into<String>) -> Self {
        Self::StepExecution {
            step: step.into(),
            message: message.into(),
        }
    }

    pub fn assertion_failed(step: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AssertionFailed {
            step: step.into(),
            message: message.into(),
        }
    }

    pub fn step_not_found(step: impl Into<String>) -> Self {
        Self::StepNotFound { step: step.into() }
    }

    pub fn missing_ontology(scenario: impl Into<String>) -> Self {
        Self::MissingOntology {
            scenario: scenario.into(),
        }
    }

    pub fn missing_operations(path: impl Into<PathBuf>) -> Self {
        Self::MissingOperations { path: path.into() }
    }
}
