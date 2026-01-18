//! Container-level schema validation for Loro documents.
//!
//! This module provides type validation for individual containers. Document-level
//! validation (which schemas apply to which containers, ACL enforcement across
//! containers) is handled by the consumer (e.g., WDS).

use loro::{
    LoroValue, ValueOrContainer,
    event::{Diff, ListDiffItem, MapDelta},
};
use smol_str::SmolStr;
use thiserror::Error;

use crate::schema::{Action, Can, Field, Schema, Who};

/// Type of change detected in a diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
}

/// Validation errors.
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

/// Context for ACL-aware validation.
///
/// Provides author information and a path resolver for checking Restricted fields.
#[derive(Default)]
pub struct ValidateContext<'a> {
    /// Author identifier as a string (for Restricted field checks).
    pub author: Option<&'a str>,
    /// Path resolver for `Who::Path` references.
    /// Returns a list of authorized identifiers for a given path.
    pub resolve_path: Option<&'a dyn Fn(&str) -> Option<Vec<String>>>,
}

/// Validate a [`LoroValue`] against a [`Field`] layout.
///
/// This performs type-only validation without ACL checks.
///
/// # Errors
///
/// Returns [`ValidationError`] if the value doesn't match the field type.
pub fn validate_value(value: &LoroValue, field: &Field, path: &str) -> Result<(), ValidationError> {
    validate_value_with_context(value, field, path, &ValidateContext::default())
}

/// Validate a [`LoroValue`] against a [`Field`] layout with ACL context.
///
/// When `ctx.author` and `ctx.resolve_path` are provided, Restricted fields
/// will be checked for authorization.
///
/// # Errors
///
/// Returns [`ValidationError`] if the value doesn't match the field type
/// or if access is denied for restricted fields.
#[expect(clippy::too_many_lines)]
pub fn validate_value_with_context(
    value: &LoroValue,
    field: &Field,
    path: &str,
    ctx: &ValidateContext<'_>,
) -> Result<(), ValidationError> {
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
        Field::Optional(inner) => match value {
            LoroValue::Null => Ok(()),
            _ => validate_value_with_context(value, inner, path, ctx),
        },
        Field::List(inner) | Field::MovableList(inner) => match value {
            LoroValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    validate_value_with_context(item, inner, &format!("{path}[{i}]"), ctx)
                        .map_err(|e| ValidationError::InvalidElement {
                            path: path.to_string(),
                            index: i,
                            source: Box::new(e),
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
                    validate_value_with_context(val, inner, &format!("{path}.{key}"), ctx)
                        .map_err(|e| ValidationError::InvalidField {
                            path: path.to_string(),
                            key: key.into(),
                            source: Box::new(e),
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
                    validate_value_with_context(val, inner_field, &format!("{path}.{key}"), ctx)
                        .map_err(|e| ValidationError::InvalidField {
                            path: path.to_string(),
                            key: key.clone(),
                            source: Box::new(e),
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
                        validate_value_with_context(meta, inner, &format!("{path}[{i}].meta"), ctx)
                            .map_err(|e| ValidationError::InvalidElement {
                                path: path.to_string(),
                                index: i,
                                source: Box::new(e),
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
        Field::Restricted {
            value: inner,
            actions,
        } => {
            // Check ACL if author and resolver are provided.
            if let (Some(author), Some(resolve)) = (ctx.author, ctx.resolve_path) {
                for action in actions {
                    for can in &action.can {
                        if !is_authorized(&action.who, author, resolve) {
                            return Err(ValidationError::AccessDenied {
                                path: path.to_string(),
                                action: action_name(can),
                            });
                        }
                    }
                }
            }
            validate_value_with_context(value, inner, path, ctx)
        }
    }
}

/// Check if an author is authorized according to a [`Who`] rule.
fn is_authorized(who: &Who, author: &str, resolve: &dyn Fn(&str) -> Option<Vec<String>>) -> bool {
    match who {
        Who::Anyone => true,
        Who::Path(path) => resolve(path).is_some_and(|dids| dids.contains(&author.to_string())),
    }
}

/// Get the action name for error reporting.
const fn action_name(can: &Can) -> &'static str {
    match can {
        Can::Create => "create",
        Can::Delete => "delete",
        Can::Update => "update",
    }
}

/// Validate a single container's diff against a schema.
///
/// # Errors
///
/// Returns [`ValidationError`] if any value in the diff doesn't match the schema.
pub fn validate_container_diff(
    diff: &Diff<'_>,
    schema: &Schema,
    container_name: &str,
    ctx: &ValidateContext<'_>,
) -> Result<(), ValidationError> {
    match diff {
        Diff::Map(map_delta) => validate_map_diff(map_delta, schema.layout(), container_name, ctx),
        Diff::List(items) => validate_list_diff(items, schema.layout(), container_name, ctx),
        // Tree metadata is validated via separate Map diffs for each node.
        _ => Ok(()),
    }
}

/// Validate map field changes.
fn validate_map_diff(
    map_delta: &MapDelta<'_>,
    field: &Field,
    path: &str,
    ctx: &ValidateContext<'_>,
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
                    validate_value_or_container(value, expected_field, &field_path, ctx)?;
                }
            } else if let Some(expected) = map_inner {
                validate_value_or_container(value, expected, &field_path, ctx)?;
            }
        }
    }
    Ok(())
}

/// Validate list changes.
fn validate_list_diff(
    items: &[ListDiffItem],
    field: &Field,
    path: &str,
    ctx: &ValidateContext<'_>,
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
                validate_value_or_container(value, expected, &elem_path, ctx)?;
            }
        }
    }
    Ok(())
}

/// Unwrap Restricted wrappers to get the inner field type.
#[must_use]
pub fn unwrap_restricted(field: &Field) -> &Field {
    match field {
        Field::Restricted { value, .. } => unwrap_restricted(value),
        Field::Optional(inner) => unwrap_restricted(inner),
        other => other,
    }
}

/// Validate a [`ValueOrContainer`] against a [`Field`] layout.
fn validate_value_or_container(
    value: &ValueOrContainer,
    field: &Field,
    path: &str,
    ctx: &ValidateContext<'_>,
) -> Result<(), ValidationError> {
    let loro_value = value.get_deep_value();
    validate_value_with_context(&loro_value, field, path, ctx)
}

/// Get the change type name for error reporting.
#[must_use]
pub const fn change_type_name(ct: ChangeType) -> &'static str {
    match ct {
        ChangeType::Create => "create",
        ChangeType::Update => "update",
        ChangeType::Delete => "delete",
    }
}

/// Find all Restricted wrappers along a path in the schema.
#[must_use]
pub fn find_restrictions_for_path<'a>(
    field: &'a Field,
    path: &str,
) -> Vec<(&'a Vec<Action>, &'a Field)> {
    let mut restrictions = Vec::new();
    find_restrictions_recursive(field, path, &mut restrictions);
    restrictions
}

/// Recursively find restrictions along a path.
fn find_restrictions_recursive<'a>(
    field: &'a Field,
    remaining_path: &str,
    out: &mut Vec<(&'a Vec<Action>, &'a Field)>,
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
                        find_restrictions_recursive(inner, after, out);
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
}
