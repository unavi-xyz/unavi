//! Schema validation for Loro documents.
//!
//! This module provides type validation for containers and values.
//! For document-level validation with Restricted field authorization,
//! use [`crate::Validator`].

pub mod diff;
pub mod restriction;
pub mod validator;
pub mod value;

use smol_str::SmolStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Create,
    Delete,
    Update,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("access denied at {path}: {action} requires authorization")]
    AccessDenied {
        path:   String,
        action: &'static str,
    },
    #[error("invalid element at {path}[{index}]")]
    InvalidElement {
        path:   String,
        index:  usize,
        #[source]
        source: Box<Self>,
    },
    #[error("invalid field {path}.{key}")]
    InvalidField {
        path:   String,
        key:    SmolStr,
        #[source]
        source: Box<Self>,
    },
    #[error("missing field: {0}")]
    MissingField(SmolStr),
    #[error("unknown variant: {0}")]
    UnknownVariant(SmolStr),
    #[error("type mismatch at {path}: expected {expected}")]
    TypeMismatch {
        path:     String,
        expected: &'static str,
    },
}
