//! Document-level schema validation for WDS.
//!
//! # Validation Methodology
//!
//! Validation happens during envelope ingestion via [`validate_diff`], which
//! performs two passes over the document diff:
//!
//! 1. **Type validation**: Delegated to [`loro_schema::validate_container_diff`]
//!    which validates new values against schema field types.
//!
//! 2. **ACL restrictions**: Each changed field is checked against `Restricted`
//!    wrappers in the schema based on the change type (create/update/delete).
//!    The `who` field specifies authorized DIDs via path resolution.
//!
//! ## Path Resolution
//!
//! `Who::Path` references use dot notation to navigate the document:
//! - `acl.manage` → resolves to the `manage` field in the `acl` container
//! - `container.field.subfield` → nested field access
//!
//! The resolved value should be a DID string or list of DID strings.
//!
//! ## First Envelope Handling
//!
//! The first envelope (creating the record) bypasses ACL checks since there's
//! no prior state to authorize against. The creator implicitly has permission
//! to set initial values including the ACL itself.

use blake3::Hash;
use iroh_blobs::api::Store;
use loro::{
    Frontiers, LoroDoc, LoroValue, TreeExternalDiff,
    event::{Diff, DiffBatch, ListDiffItem, TreeDiff},
};
use loro_schema::{
    Can, ChangeType, Field, Schema, ValidateContext, Who, change_type_name,
    find_restrictions_for_path,
};
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
pub async fn fetch_schema(blobs: &Store, hash: &Hash) -> Result<Schema, ValidationError> {
    let iroh_hash: iroh_blobs::Hash = (*hash).into();
    let bytes = blobs
        .get_bytes(iroh_hash)
        .await
        .map_err(|_| ValidationError::SchemaNotFound(*hash))?;
    Schema::from_bytes(&bytes).map_err(|_| ValidationError::ParseError)
}

/// Resolve a path like `acl.write` to a list of DID strings.
///
/// Path format: `container.field.subfield` using dot notation throughout.
fn resolve_acl_path(doc: &LoroDoc, path: &str) -> Option<Vec<String>> {
    let mut parts = path.split('.');
    let container = parts.next()?;
    let map = doc.get_map(container);
    let value = map.get_deep_value();

    // Navigate through nested fields.
    let mut current = &value;
    for part in parts {
        let LoroValue::Map(m) = current else {
            return None;
        };
        current = m.get(part)?;
    }

    // Extract strings from the value (single string or list).
    match current {
        LoroValue::String(s) => Some(vec![s.to_string()]),
        LoroValue::List(list) => {
            let mut result = Vec::with_capacity(list.len());
            for item in list.iter() {
                if let LoroValue::String(s) = item {
                    result.push(s.to_string());
                }
            }
            Some(result)
        }
        _ => None,
    }
}

/// Check if an author is authorized according to a [`Who`] rule.
fn is_authorized(doc: &LoroDoc, who: &Who, author: &Did) -> bool {
    match who {
        Who::Anyone => true,
        Who::Path(path) => {
            resolve_acl_path(doc, path).is_some_and(|dids| dids.contains(&author.to_string()))
        }
    }
}

/// Validate changes between two document states against schema restrictions.
///
/// `auth_doc` is used for authorization checks (should be the OLD state).
/// `diff_doc` is used for computing the diff (should have the new envelope applied).
pub fn validate_diff(
    old_doc: &LoroDoc,
    new_doc: &LoroDoc,
    old_frontiers: &Frontiers,
    new_frontiers: &Frontiers,
    schemas: &std::collections::BTreeMap<SmolStr, Schema>,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let diff_batch = new_doc
        .diff(old_frontiers, new_frontiers)
        .map_err(|_| ValidationError::ParseError)?;

    validate_diff_batch(old_doc, &diff_batch, schemas, author, is_first_envelope)
}

/// Validate a [`DiffBatch`] against schema restrictions.
pub fn validate_diff_batch(
    doc: &LoroDoc,
    diff: &DiffBatch,
    schemas: &std::collections::BTreeMap<SmolStr, Schema>,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    // Build path resolver for ACL checks.
    let resolve_path = |path: &str| resolve_acl_path(doc, path);
    let ctx = ValidateContext {
        author: Some(&author.to_string()),
        resolve_path: Some(&resolve_path),
    };

    for (container_id, container_diff) in diff.iter() {
        // Only validate root containers.
        if !container_id.is_root() {
            continue;
        }

        let container_name = container_id.name();

        // Check if this container has a schema.
        if let Some(schema) = schemas.get(container_name.as_str()) {
            // Type validation via loro_schema.
            loro_schema::validate_container_diff(
                container_diff,
                schema,
                container_name.as_str(),
                &ctx,
            )?;

            // ACL validation based on change types.
            validate_acl_for_diff(
                doc,
                container_diff,
                schema.layout(),
                container_name.as_str(),
                author,
                is_first_envelope,
            )?;
        }
    }

    Ok(())
}

/// Validate ACL permissions for changes in a container diff.
///
/// This checks that the author has permission to perform each change type
/// (create/update/delete) according to the Restricted fields in the schema.
fn validate_acl_for_diff(
    doc: &LoroDoc,
    diff: &Diff<'_>,
    field: &Field,
    path: &str,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    match diff {
        Diff::Map(map_delta) => {
            for (key, new_value) in &map_delta.updated {
                let change_type = if new_value.is_some() {
                    ChangeType::Update
                } else {
                    ChangeType::Delete
                };
                let field_path = format!("{path}.{key}");
                validate_field_change(
                    doc,
                    field,
                    &field_path,
                    change_type,
                    author,
                    is_first_envelope,
                )?;
            }
            Ok(())
        }
        Diff::List(items) => validate_list_acl(doc, items, field, path, author, is_first_envelope),
        Diff::Tree(tree_diff) => {
            validate_tree_acl(doc, tree_diff, field, path, author, is_first_envelope)
        }
        _ => Ok(()),
    }
}

/// Validate ACL permissions for list changes.
fn validate_list_acl(
    doc: &LoroDoc,
    items: &[ListDiffItem],
    field: &Field,
    path: &str,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let is_movable = matches!(loro_schema::unwrap_restricted(field), Field::MovableList(_));

    for item in items {
        match item {
            ListDiffItem::Insert { is_move, .. } => {
                // For MovableList, moves are updates (reordering).
                // For regular List, moves are treated as creates.
                let change_type = if *is_move && is_movable {
                    ChangeType::Update
                } else {
                    ChangeType::Create
                };
                validate_field_change(doc, field, path, change_type, author, is_first_envelope)?;
            }
            ListDiffItem::Delete { .. } => {
                validate_field_change(
                    doc,
                    field,
                    path,
                    ChangeType::Delete,
                    author,
                    is_first_envelope,
                )?;
            }
            ListDiffItem::Retain { .. } => {}
        }
    }
    Ok(())
}

/// Validate ACL permissions for tree changes.
fn validate_tree_acl(
    doc: &LoroDoc,
    tree_diff: &TreeDiff,
    field: &Field,
    path: &str,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    for item in &tree_diff.diff {
        let change_type = match &item.action {
            TreeExternalDiff::Create { .. } => ChangeType::Create,
            TreeExternalDiff::Move { .. } => ChangeType::Update,
            TreeExternalDiff::Delete { .. } => ChangeType::Delete,
        };
        validate_field_change(doc, field, path, change_type, author, is_first_envelope)?;
    }
    Ok(())
}

/// Validate a single field change against schema restrictions.
fn validate_field_change(
    doc: &LoroDoc,
    field: &Field,
    path: &str,
    change_type: ChangeType,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    // Find all Restricted wrappers that apply to this path.
    let restrictions = find_restrictions_for_path(field, path);

    for (actions, _) in restrictions {
        for action in actions {
            // Check if this action covers our change type.
            let covers_change = action.can.iter().any(|c| {
                matches!(
                    (c, change_type),
                    (Can::Create, ChangeType::Create)
                        | (Can::Update, ChangeType::Update)
                        | (Can::Delete, ChangeType::Delete)
                )
            });

            if covers_change {
                // First envelope: creator is implicitly authorized.
                if is_first_envelope {
                    continue;
                }

                // Check if author is authorized.
                if !is_authorized(doc, &action.who, author) {
                    return Err(loro_schema::ValidationError::AccessDenied {
                        path: path.to_string(),
                        action: change_type_name(change_type),
                    }
                    .into());
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use loro::LoroValue;
    use loro_schema::validate_value;

    use super::*;

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

        // MovableList validates the same as List for value checking.
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

        // All values are strings - should pass.
        assert!(validate_value(&value, &Field::Map(Box::new(Field::String)), "test").is_ok());

        // Expecting I64 - should fail.
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
