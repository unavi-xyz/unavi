//! Document-level schema validation for WDS.
//!
//! Delegates to [`loro_schema::Validator`] for type and Restricted field checking.

use std::collections::BTreeMap;

use blake3::Hash;
use iroh_blobs::api::Store;
use loro::{Frontiers, LoroDoc};
use loro_schema::{Schema, Validator};
use smol_str::SmolStr;
use thiserror::Error;
use xdid::core::did::Did;

/// Validation errors for WDS documents.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("schema not found: {0}")]
    SchemaNotFound(Hash),
    #[error("failed to parse schema")]
    ParseError,
    #[error(transparent)]
    Schema(#[from] loro_schema::ValidationError),
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
) -> Result<(), ValidationError> {
    let diff_batch = new_doc
        .diff(old_frontiers, new_frontiers)
        .map_err(|_| ValidationError::ParseError)?;

    Validator::new(schemas, &author.to_string()).validate_diff_batch(old_doc, &diff_batch)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use loro::LoroValue;
    use loro_schema::{Field, validate_value};

    #[test]
    fn test_validate_bool() {
        let value = LoroValue::Bool(true);
        assert!(validate_value(&value, &Field::Bool, "test").is_ok());
        assert!(validate_value(&value, &Field::I64, "test").is_err());
    }

    #[test]
    fn test_validate_any() {
        let value = LoroValue::I64(42);
        assert!(validate_value(&value, &Field::Any, "test").is_ok());
    }

    #[test]
    fn test_validate_list() {
        let items = vec![LoroValue::String("a".into()), LoroValue::String("b".into())];
        let value = LoroValue::List(items.into());

        assert!(validate_value(&value, &Field::List(Box::new(Field::String)), "test").is_ok());
        assert!(validate_value(&value, &Field::List(Box::new(Field::I64)), "test").is_err());
    }

    #[test]
    fn test_validate_movable_list() {
        let items = vec![LoroValue::I64(1), LoroValue::I64(2), LoroValue::I64(3)];
        let value = LoroValue::List(items.into());

        assert!(validate_value(&value, &Field::MovableList(Box::new(Field::I64)), "test").is_ok());
        assert!(
            validate_value(&value, &Field::MovableList(Box::new(Field::String)), "test").is_err()
        );
    }

    #[test]
    fn test_validate_struct() {
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), LoroValue::String("test".into()));
        map.insert("age".to_string(), LoroValue::I64(42));
        let value = LoroValue::Map(map.into());

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));
        fields.insert("age".into(), Box::new(Field::I64));

        assert!(validate_value(&value, &Field::Struct(fields.clone()), "test").is_ok());

        // Wrong type for age field.
        let mut bad_fields = BTreeMap::new();
        bad_fields.insert("name".into(), Box::new(Field::String));
        bad_fields.insert("age".into(), Box::new(Field::String));
        assert!(validate_value(&value, &Field::Struct(bad_fields), "test").is_err());
    }

    #[test]
    fn test_validate_map_homogeneous() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1".to_string(), LoroValue::String("value1".into()));
        map.insert("key2".to_string(), LoroValue::String("value2".into()));
        let value = LoroValue::Map(map.into());

        assert!(validate_value(&value, &Field::Map(Box::new(Field::String)), "test").is_ok());
        assert!(validate_value(&value, &Field::Map(Box::new(Field::I64)), "test").is_err());
    }

    #[test]
    fn test_validate_tree() {
        // Tree's deep value is a list of node objects with meta field.
        let mut node_map = std::collections::HashMap::new();
        let mut meta_map = std::collections::HashMap::new();
        meta_map.insert("name".to_string(), LoroValue::String("node1".into()));
        node_map.insert("meta".to_string(), LoroValue::Map(meta_map.into()));
        node_map.insert("id".to_string(), LoroValue::String("123".into()));

        let nodes = vec![LoroValue::Map(node_map.into())];
        let value = LoroValue::List(nodes.into());

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));

        assert!(
            validate_value(
                &value,
                &Field::Tree(Box::new(Field::Struct(fields))),
                "test"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_optional_present() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::String("hello".into());
        assert!(validate_value(&value, &field, "test").is_ok());
    }

    #[test]
    fn test_validate_optional_null() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::Null;
        assert!(validate_value(&value, &field, "test").is_ok());
    }

    #[test]
    fn test_validate_optional_wrong_type() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::I64(42);
        assert!(validate_value(&value, &field, "test").is_err());
    }
}
