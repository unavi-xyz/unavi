//! Schema validation for Loro CRDT documents.
//!
//! This crate provides schema types and validation for Loro containers,
//! including both type validation and Restricted field authorization.
//!
//! # Overview
//!
//! - [`Schema`] - Defines the structure and validation rules for a container.
//! - [`Field`] - Type definitions (Bool, String, List, Map, Struct, etc.).
//! - [`validate_value`] - Validate a `LoroValue` against a Field (type-only).
//! - [`Validator`] - Document-level validation with Restricted field checking.

mod schema;
mod validate;
mod validator;

pub use schema::{Action, Can, Field, Schema, Who};
pub use validate::{
    ChangeType, ValidationError, change_type_name, find_restrictions_for_path, unwrap_restricted,
    validate_container_diff, validate_value,
};
pub use validator::Validator;
