//! Container-level schema validation for Loro documents.
//!
//! This module provides type validation for individual containers. For document-level
//! validation with Restricted field authorization, use [`crate::Validator`].

use loro::{
    LoroValue, ValueOrContainer,
    event::{Diff, ListDiffItem, MapDelta},
};
use smol_str::SmolStr;
use thiserror::Error;

use crate::schema::{Field, Schema};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("missing field: {0}")]
    MissingField(SmolStr),
    #[error("type mismatch at {path}: expected {expected}")]
    TypeMismatch {
        path: String,
        expected: &'static str,
    },
    #[error("invalid element at {path}[{index}]")]
    InvalidElement {
        path: String,
        index: usize,
        #[source]
        source: Box<Self>,
    },
    #[error("invalid field {path}.{key}")]
    InvalidField {
        path: String,
        key: SmolStr,
        #[source]
        source: Box<Self>,
    },
    #[error("access denied at {path}: {action} requires authorization")]
    AccessDenied { path: String, action: &'static str },
}

/// Validate a [`LoroValue`] against a [`Field`] layout.
///
/// This performs type-only validation. For Restricted field authorization,
/// use [`crate::Validator`].
///
/// # Errors
///
/// Returns [`ValidationError`] if the value doesn't match the field type.
#[expect(clippy::too_many_lines)]
pub fn validate_value(value: &LoroValue, field: &Field, path: &str) -> Result<(), ValidationError> {
    match field {
        Field::Any => Ok(()),
        Field::Bool => match value {
            LoroValue::Bool(_) => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "bool",
            }),
        },
        Field::F64 => match value {
            LoroValue::Double(_) => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "f64",
            }),
        },
        Field::I64 => match value {
            LoroValue::I64(_) => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "i64",
            }),
        },
        Field::String => match value {
            LoroValue::String(_) => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "string",
            }),
        },
        Field::Binary => match value {
            LoroValue::Binary(_) => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "binary",
            }),
        },
        Field::BlobRef => match value {
            LoroValue::Binary(bytes) if bytes.len() == 32 => Ok(()),
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "blob_ref (32-byte binary)",
            }),
        },
        Field::Optional(inner) => match value {
            LoroValue::Null => Ok(()),
            _ => validate_value(value, inner, path),
        },
        Field::List(inner) | Field::MovableList(inner) => match value {
            LoroValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    validate_value(item, inner, &format!("{path}[{i}]")).map_err(|e| {
                        ValidationError::InvalidElement {
                            path: path.to_string(),
                            index: i,
                            source: Box::new(e),
                        }
                    })?;
                }
                Ok(())
            }
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "list",
            }),
        },
        Field::Map(inner) => match value {
            LoroValue::Map(map) => {
                for (key, val) in map.iter() {
                    validate_value(val, inner, &format!("{path}.{key}")).map_err(|e| {
                        ValidationError::InvalidField {
                            path: path.to_string(),
                            key: key.into(),
                            source: Box::new(e),
                        }
                    })?;
                }
                Ok(())
            }
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "map",
            }),
        },
        Field::Struct(fields) => match value {
            LoroValue::Map(map) => {
                for (key, inner_field) in fields {
                    let val = map
                        .get(key.as_str())
                        .ok_or_else(|| ValidationError::MissingField(key.clone()))?;
                    validate_value(val, inner_field, &format!("{path}.{key}")).map_err(|e| {
                        ValidationError::InvalidField {
                            path: path.to_string(),
                            key: key.clone(),
                            source: Box::new(e),
                        }
                    })?;
                }
                Ok(())
            }
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "struct",
            }),
        },
        Field::Tree(inner) => match value {
            // Tree's deep value is a list of node objects.
            // Each node has: id, parent, fractional_index, index, meta.
            // We validate the "meta" field which contains user data.
            LoroValue::List(nodes) => {
                for (i, node) in nodes.iter().enumerate() {
                    if let LoroValue::Map(node_map) = node
                        && let Some(meta) = node_map.get("meta")
                    {
                        validate_value(meta, inner, &format!("{path}[{i}].meta")).map_err(|e| {
                            ValidationError::InvalidElement {
                                path: path.to_string(),
                                index: i,
                                source: Box::new(e),
                            }
                        })?;
                    }
                }
                Ok(())
            }
            _ => Err(ValidationError::TypeMismatch {
                path: path.to_string(),
                expected: "tree",
            }),
        },
        // For type validation, Restricted just validates the inner type.
        // Authorization is handled by Validator.
        Field::Restricted { value: inner, .. } => validate_value(value, inner, path),
    }
}

/// # Errors
///
/// Returns [`ValidationError`] if any value in the diff doesn't match the schema.
pub fn validate_container_diff(
    diff: &Diff<'_>,
    schema: &Schema,
    container_name: &str,
) -> Result<(), ValidationError> {
    match diff {
        Diff::Map(map_delta) => validate_map_diff(map_delta, schema.layout(), container_name),
        Diff::List(items) => validate_list_diff(items, schema.layout(), container_name),
        _ => Ok(()),
    }
}

fn validate_map_diff(
    map_delta: &MapDelta<'_>,
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let inner = unwrap_restricted(field);

    let (struct_fields, map_inner) = match inner {
        Field::Struct(fields) => (Some(fields), None),
        Field::Map(inner) => (None, Some(inner.as_ref())),
        _ => (None, None),
    };

    for (key, new_value) in &map_delta.updated {
        let field_path = format!("{path}.{key}");

        if let Some(value) = new_value {
            if let Some(fields) = struct_fields {
                let key_smol: SmolStr = key.to_string().into();
                if let Some(expected_field) = fields.get(&key_smol) {
                    validate_value_or_container(value, expected_field, &field_path)?;
                }
            } else if let Some(expected) = map_inner {
                validate_value_or_container(value, expected, &field_path)?;
            }
        }
    }
    Ok(())
}

fn validate_list_diff(
    items: &[ListDiffItem],
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let inner = unwrap_restricted(field);
    let item_field = match inner {
        Field::List(inner) | Field::MovableList(inner) => Some(inner.as_ref()),
        _ => None,
    };

    for item in items {
        if let ListDiffItem::Insert { insert, .. } = item
            && let Some(expected) = item_field
        {
            for (i, value) in insert.iter().enumerate() {
                let elem_path = format!("{path}[{i}]");
                validate_value_or_container(value, expected, &elem_path)?;
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn unwrap_restricted(field: &Field) -> &Field {
    match field {
        Field::Restricted { value, .. } => unwrap_restricted(value),
        Field::Optional(inner) => unwrap_restricted(inner),
        other => other,
    }
}

fn validate_value_or_container(
    value: &ValueOrContainer,
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let loro_value = value.get_deep_value();
    validate_value(&loro_value, field, path)
}

#[must_use]
pub const fn change_type_name(ct: ChangeType) -> &'static str {
    match ct {
        ChangeType::Create => "create",
        ChangeType::Update => "update",
        ChangeType::Delete => "delete",
    }
}

#[must_use]
pub fn find_restrictions_for_path<'a>(
    field: &'a Field,
    path: &str,
) -> Vec<(&'a Vec<crate::Action>, &'a Field)> {
    let mut restrictions = Vec::new();
    // Strip container name prefix (e.g., "acl.manage" -> ".manage").
    let relative_path = path.find('.').map_or("", |dot_pos| &path[dot_pos..]);
    find_restrictions_recursive(field, relative_path, &mut restrictions);
    restrictions
}

fn find_restrictions_recursive<'a>(
    field: &'a Field,
    remaining_path: &str,
    out: &mut Vec<(&'a Vec<crate::Action>, &'a Field)>,
) {
    match field {
        Field::Restricted { actions, value } => {
            out.push((actions, value));
            find_restrictions_recursive(value, remaining_path, out);
        }
        Field::Struct(fields) => {
            if let Some(rest) = remaining_path.strip_prefix('.') {
                if let Some((key, after)) = rest.split_once('.') {
                    if let Some(inner) = fields.get(key) {
                        find_restrictions_recursive(inner, &format!(".{after}"), out);
                    }
                } else if let Some(inner) = fields.get(rest) {
                    find_restrictions_recursive(inner, "", out);
                }
            }
        }
        Field::Map(inner) | Field::List(inner) | Field::MovableList(inner) | Field::Tree(inner) => {
            find_restrictions_recursive(inner, remaining_path, out);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

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

    #[test]
    fn test_validate_blob_ref_valid() {
        // 32 bytes = valid blake3 hash.
        let hash_bytes = vec![0xaf; 32];
        let value = LoroValue::Binary(hash_bytes.into());
        assert!(validate_value(&value, &Field::BlobRef, "test").is_ok());
    }

    #[test]
    fn test_validate_blob_ref_invalid_length_short() {
        let value = LoroValue::Binary(vec![1, 2, 3].into());
        assert!(validate_value(&value, &Field::BlobRef, "test").is_err());
    }

    #[test]
    fn test_validate_blob_ref_invalid_length_long() {
        let value = LoroValue::Binary(vec![0; 33].into());
        assert!(validate_value(&value, &Field::BlobRef, "test").is_err());
    }

    #[test]
    fn test_validate_blob_ref_wrong_type_string() {
        let value = LoroValue::String("not a binary".into());
        assert!(validate_value(&value, &Field::BlobRef, "test").is_err());
    }
}
