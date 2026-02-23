//! Document-level schema validation for WDS.
//!
//! Delegates to [`wds_schema::Validator`] for type and Restricted field checking.

use std::collections::BTreeMap;

use blake3::Hash;
use iroh_blobs::api::Store;
use loro::{Frontiers, LoroDoc};
use smol_str::SmolStr;
use thiserror::Error;
use wds_schema::{Schema, Validator};
use xdid::core::did::Did;

/// Validation errors for WDS documents.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("schema not found: {0}")]
    SchemaNotFound(Hash),
    #[error("failed to parse schema")]
    ParseError,
    #[error(transparent)]
    Schema(#[from] wds_schema::ValidationError),
}

/// Fetch a schema from the blob store by its hash.
///
/// # Errors
///
/// Returns error if schema not found or fails to parse.
pub async fn fetch_schema(blobs: &Store, hash: &Hash) -> Result<Schema, ValidationError> {
    let iroh_hash: iroh_blobs::Hash = (*hash).into();
    let bytes = blobs
        .get_bytes(iroh_hash)
        .await
        .map_err(|_| ValidationError::SchemaNotFound(*hash))?;
    Schema::from_bytes(&bytes).map_err(|_| ValidationError::ParseError)
}

/// Validate changes between two document states against schema restrictions.
///
/// `old_doc` is used for authorization checks (the state before the change).
/// `new_doc` is used for computing the diff (should have the new envelope applied).
/// `is_first_envelope` skips restriction checks (allows ACL bootstrap).
///
/// # Errors
///
/// Returns error if validation fails.
pub fn validate_diff(
    old_doc: &LoroDoc,
    new_doc: &LoroDoc,
    old_frontiers: &Frontiers,
    new_frontiers: &Frontiers,
    schemas: &BTreeMap<SmolStr, Schema>,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let diff_batch = new_doc
        .diff(old_frontiers, new_frontiers)
        .map_err(|_| ValidationError::ParseError)?;

    let author_str = author.to_string();
    let mut validator = Validator::new(schemas, &author_str);
    if is_first_envelope {
        validator = validator.skip_restrictions();
    }
    validator.validate_diff_batch(old_doc, &diff_batch)?;

    Ok(())
}
