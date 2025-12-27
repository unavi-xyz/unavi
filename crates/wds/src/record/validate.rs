//! Schema validation for Loro documents.

use blake3::Hash;
use iroh_blobs::store::fs::FsStore;
use loro::{
    Frontiers, LoroDoc, LoroValue,
    event::{Diff, DiffBatch, ListDiffItem, MapDelta},
};
use smol_str::SmolStr;
use thiserror::Error;
use xdid::core::did::Did;

use super::schema::{Action, Can, Field, SCHEMA_RECORD, Schema, Who};

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
    #[error("schema not found: {0}")]
    SchemaNotFound(Hash),
    #[error("failed to parse schema")]
    ParseError,
    #[error("access denied at {path}: {action} requires authorization")]
    AccessDenied { path: String, action: &'static str },
}

/// Fetch a schema from the blob store by its hash.
pub async fn fetch_schema(blobs: &FsStore, hash: &Hash) -> Result<Schema, ValidationError> {
    let iroh_hash: iroh_blobs::Hash = (*hash).into();
    let bytes = blobs
        .get_bytes(iroh_hash)
        .await
        .map_err(|_| ValidationError::SchemaNotFound(*hash))?;
    ron::de::from_bytes(&bytes).map_err(|_| ValidationError::ParseError)
}

/// Resolve a path like "acl:write" to a list of DID strings.
///
/// Path format: "container:field" where field can be nested with dots.
fn resolve_acl_path(doc: &LoroDoc, path: &str) -> Option<Vec<String>> {
    let (container, field) = path.split_once(':')?;
    let map = doc.get_map(container);
    let value = map.get_deep_value();

    // Navigate through nested fields.
    let mut current = &value;
    for part in field.split('.') {
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

/// Get the action name for error reporting.
const fn action_name(can: &Can) -> &'static str {
    match can {
        Can::Create => "create",
        Can::Delete => "delete",
        Can::Update => "update",
    }
}

/// Validate a [`LoroValue`] against a [`Field`] layout.
pub fn validate_value(value: &LoroValue, field: &Field, path: &str) -> Result<(), ValidationError> {
    validate_value_inner(value, field, path, None, None)
}

/// Internal validation with optional ACL context.
fn validate_value_inner(
    value: &LoroValue,
    field: &Field,
    path: &str,
    doc: Option<&LoroDoc>,
    author: Option<&Did>,
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
        Field::List(inner) => match value {
            LoroValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    validate_value_inner(item, inner, &format!("{path}[{i}]"), doc, author)
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
        Field::Map(fields) => match value {
            LoroValue::Map(map) => {
                for (key, inner_field) in fields {
                    let val = map
                        .get(key.as_str())
                        .ok_or_else(|| ValidationError::MissingField(key.clone()))?;
                    validate_value_inner(val, inner_field, &format!("{path}.{key}"), doc, author)
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
                expected: "map",
            }),
        },
        Field::Restricted {
            value: inner,
            actions,
        } => {
            // Check ACL if doc and author are provided.
            if let (Some(doc), Some(author)) = (doc, author) {
                for action in actions {
                    for can in &action.can {
                        if !is_authorized(doc, &action.who, author) {
                            return Err(ValidationError::AccessDenied {
                                path: path.to_string(),
                                action: action_name(can),
                            });
                        }
                    }
                }
            }
            validate_value_inner(value, inner, path, doc, author)
        }
    }
}

/// Validate a container in a [`LoroDoc`] against a [`Schema`].
fn validate_container(doc: &LoroDoc, schema: &Schema) -> Result<(), ValidationError> {
    let container = doc.get_map(schema.container());
    let value = container.get_deep_value();
    validate_value(&value, schema.layout(), schema.container())
}

/// Validate that a document contains a valid record structure.
pub async fn validate_record(blobs: &FsStore, doc: &LoroDoc) -> Result<(), ValidationError> {
    let schema = fetch_schema(blobs, &SCHEMA_RECORD).await?;
    validate_container(doc, &schema)
}

/// Validate changes between two document states against schema restrictions.
pub fn validate_diff(
    doc: &LoroDoc,
    old_frontiers: &Frontiers,
    new_frontiers: &Frontiers,
    schema: &Schema,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let diff_batch = doc
        .diff(old_frontiers, new_frontiers)
        .map_err(|_| ValidationError::ParseError)?;

    validate_diff_batch(doc, &diff_batch, schema, author, is_first_envelope)
}

/// Validate a [`DiffBatch`] against schema restrictions.
pub fn validate_diff_batch(
    doc: &LoroDoc,
    diff: &DiffBatch,
    schema: &Schema,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let container_name = schema.container();

    for (container_id, container_diff) in diff.iter() {
        // Only validate diffs for this schema's container.
        if !container_id.is_root() {
            continue;
        }
        if container_id.name().as_str() != container_name {
            continue;
        }

        validate_container_diff(doc, container_diff, schema, author, is_first_envelope)?;
    }

    Ok(())
}

/// Validate a single container's diff.
fn validate_container_diff(
    doc: &LoroDoc,
    diff: &Diff<'_>,
    schema: &Schema,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    let container_name = schema.container();

    match diff {
        Diff::Map(map_delta) => validate_map_diff(
            doc,
            map_delta,
            schema.layout(),
            container_name,
            author,
            is_first_envelope,
        ),
        Diff::List(items) => validate_list_diff(
            doc,
            items,
            schema.layout(),
            container_name,
            author,
            is_first_envelope,
        ),
        _ => Ok(()),
    }
}

/// Validate map field changes.
fn validate_map_diff(
    doc: &LoroDoc,
    map_delta: &MapDelta<'_>,
    field: &Field,
    path: &str,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    for (key, new_value) in &map_delta.updated {
        let change_type = if new_value.is_some() {
            ChangeType::Update // Could be Create, but we treat as Update for simplicity.
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

/// Validate list changes.
fn validate_list_diff(
    doc: &LoroDoc,
    items: &[ListDiffItem],
    field: &Field,
    path: &str,
    author: &Did,
    is_first_envelope: bool,
) -> Result<(), ValidationError> {
    for item in items {
        let change_type = match item {
            ListDiffItem::Insert { .. } => ChangeType::Create,
            ListDiffItem::Delete { .. } => ChangeType::Delete,
            ListDiffItem::Retain { .. } => continue,
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
                    return Err(ValidationError::AccessDenied {
                        path: path.to_string(),
                        action: change_type_name(change_type),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Find all Restricted wrappers along a path in the schema.
fn find_restrictions_for_path<'a>(
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
        Field::Map(fields) => {
            // Parse next path segment (e.g., "container.key.subkey" → "key", "subkey").
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
        Field::List(inner) => {
            // For lists, recurse into inner field.
            find_restrictions_recursive(inner, remaining_path, out);
        }
        _ => {}
    }
}

const fn change_type_name(ct: ChangeType) -> &'static str {
    match ct {
        ChangeType::Create => "create",
        ChangeType::Update => "update",
        ChangeType::Delete => "delete",
    }
}

#[cfg(test)]
mod tests {
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
}
