//! Container-level schema validation for Loro CRDT documents.
//!
//! This crate provides generic schema types and validation for Loro containers.
//! It is designed to be reusable across different applications without protocol-
//! specific dependencies.
//!
//! # Overview
//!
//! - [`Schema`] - Defines the structure and validation rules for a container.
//! - [`Field`] - Type definitions (Bool, String, List, Map, Struct, etc.).
//! - [`validate_value`] - Validate a `LoroValue` against a Field.

mod schema;
mod validate;

pub use schema::{Action, Can, Field, Schema, Who};
pub use validate::{
    ChangeType, ValidateContext, ValidationError, change_type_name, find_restrictions_for_path,
    unwrap_restricted, validate_container_diff, validate_value, validate_value_with_context,
};
