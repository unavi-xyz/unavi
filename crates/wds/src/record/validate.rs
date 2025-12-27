//! Schema validation for Loro documents.

use blake3::Hash;
use iroh_blobs::store::fs::FsStore;
use loro::{LoroDoc, LoroValue};
use smol_str::SmolStr;
use thiserror::Error;

use super::schema::{Field, SCHEMA_RECORD, Schema};

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

/// Validate a [`LoroValue`] against a [`Field`] layout.
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
        Field::List(inner) => match value {
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
        Field::Map(fields) => match value {
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
                expected: "map",
            }),
        },
        Field::Restricted { value: inner, .. } => {
            // TODO validate acl
            validate_value(value, inner, path)
        }
    }
}

/// Validate a container in a [`LoroDoc`] against a [`Schema`].
pub fn validate_container(doc: &LoroDoc, schema: &Schema) -> Result<(), ValidationError> {
    let container = doc.get_map(schema.container());
    let value = container.get_deep_value();
    validate_value(&value, schema.layout(), schema.container())
}

/// Validate that a document contains a valid record structure.
pub async fn validate_record(blobs: &FsStore, doc: &LoroDoc) -> Result<(), ValidationError> {
    let schema = fetch_schema(blobs, &SCHEMA_RECORD).await?;
    validate_container(doc, &schema)
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
