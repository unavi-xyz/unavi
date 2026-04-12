//! WDS schema validation for Loro CRDT documents.
//!
//! This crate provides schema types and validation for WDS records,
//! including both type validation and Restricted field authorization.

pub mod schema;
#[cfg(feature = "validation")]
pub mod validate;
