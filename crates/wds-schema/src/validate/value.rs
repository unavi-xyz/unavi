//! Type validation for [`LoroValue`] against [`Field`] layouts.

use std::collections::BTreeMap;

use loro::LoroValue;
use smol_str::SmolStr;

use super::ValidationError;
use crate::Field;

/// Validate a [`LoroValue`] against a [`Field`] layout.
///
/// Performs type-only validation. For Restricted field authorization,
/// use [`crate::Validator`].
pub fn validate_value(value: &LoroValue, field: &Field, path: &str) -> Result<(), ValidationError> {
    match field {
        Field::Any => Ok(()),
        Field::Bool => match value {
            LoroValue::Bool(_) => Ok(()),
            _ => Err(type_mismatch(path, "bool")),
        },
        Field::F64 => match value {
            LoroValue::Double(_) => Ok(()),
            _ => Err(type_mismatch(path, "f64")),
        },
        Field::I64 => match value {
            LoroValue::I64(_) => Ok(()),
            _ => Err(type_mismatch(path, "i64")),
        },
        Field::String => match value {
            LoroValue::String(_) => Ok(()),
            _ => Err(type_mismatch(path, "string")),
        },
        Field::Binary => match value {
            LoroValue::Binary(_) => Ok(()),
            _ => Err(type_mismatch(path, "binary")),
        },
        Field::BlobId => match value {
            LoroValue::Binary(bytes) if bytes.len() == 32 => Ok(()),
            _ => Err(type_mismatch(path, "blob_id (32-byte binary)")),
        },
        Field::RecordId => match value {
            LoroValue::Binary(bytes) if bytes.len() == 32 => Ok(()),
            _ => Err(type_mismatch(path, "record_id (32-byte binary)")),
        },
        Field::Optional(inner) => match value {
            LoroValue::Null => Ok(()),
            _ => validate_value(value, inner, path),
        },
        Field::List(inner) | Field::MovableList(inner) => validate_list(value, inner, path),
        Field::Map(inner) => validate_map(value, inner, path),
        Field::Struct(fields) => validate_struct(value, fields, path),
        Field::Tree(inner) => validate_tree(value, inner, path),
        Field::Enum(variants) => validate_enum(value, variants, path),
        Field::Restricted { value: inner, .. } => validate_value(value, inner, path),
    }
}

fn type_mismatch(path: &str, expected: &'static str) -> ValidationError {
    ValidationError::TypeMismatch {
        path: path.to_string(),
        expected,
    }
}

fn validate_list(value: &LoroValue, inner: &Field, path: &str) -> Result<(), ValidationError> {
    let LoroValue::List(items) = value else {
        return Err(type_mismatch(path, "list"));
    };
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

fn validate_map(value: &LoroValue, inner: &Field, path: &str) -> Result<(), ValidationError> {
    let LoroValue::Map(map) = value else {
        return Err(type_mismatch(path, "map"));
    };
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

fn validate_struct(
    value: &LoroValue,
    fields: &BTreeMap<SmolStr, Box<Field>>,
    path: &str,
) -> Result<(), ValidationError> {
    let LoroValue::Map(map) = value else {
        return Err(type_mismatch(path, "struct"));
    };
    for (key, inner_field) in fields {
        match map.get(key.as_str()) {
            Some(val) => {
                validate_value(val, inner_field, &format!("{path}.{key}")).map_err(|e| {
                    ValidationError::InvalidField {
                        path: path.to_string(),
                        key: key.clone(),
                        source: Box::new(e),
                    }
                })?;
            }
            None => {
                if !matches!(inner_field.as_ref(), Field::Optional(_)) {
                    return Err(ValidationError::MissingField(key.clone()));
                }
            }
        }
    }
    Ok(())
}

fn validate_tree(value: &LoroValue, inner: &Field, path: &str) -> Result<(), ValidationError> {
    let LoroValue::List(nodes) = value else {
        return Err(type_mismatch(path, "tree"));
    };
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

fn validate_enum(
    value: &LoroValue,
    variants: &BTreeMap<SmolStr, Option<Box<Field>>>,
    path: &str,
) -> Result<(), ValidationError> {
    let LoroValue::Map(map) = value else {
        return Err(type_mismatch(path, "enum (map with tag)"));
    };
    let tag_value = map
        .get("tag")
        .ok_or_else(|| ValidationError::MissingField("tag".into()))?;
    let LoroValue::String(tag) = tag_value else {
        return Err(ValidationError::TypeMismatch {
            path: format!("{path}.tag"),
            expected: "string",
        });
    };
    let tag: SmolStr = tag.as_str().into();
    let Some(variant) = variants.get(&tag) else {
        return Err(ValidationError::UnknownVariant(tag));
    };
    if let Some(inner_field) = variant {
        let data = map
            .get(tag.as_str())
            .ok_or_else(|| ValidationError::MissingField(tag.as_str().into()))?;
        validate_value(data, inner_field, &format!("{path}.{tag}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use loro::{LoroTree, LoroValue};

    use super::*;
    use crate::Field;

    #[test]
    fn validate_bool() {
        let value = LoroValue::Bool(true);
        assert!(validate_value(&value, &Field::Bool, "test").is_ok());
        assert!(validate_value(&value, &Field::I64, "test").is_err());
    }

    #[test]
    fn validate_any() {
        let value = LoroValue::I64(42);
        assert!(validate_value(&value, &Field::Any, "test").is_ok());
    }

    #[test]
    fn validate_f64() {
        let value = LoroValue::Double(3.14);
        assert!(validate_value(&value, &Field::F64, "test").is_ok());
        assert!(validate_value(&value, &Field::I64, "test").is_err());
    }

    #[test]
    fn validate_string() {
        let value = LoroValue::String("hello".into());
        assert!(validate_value(&value, &Field::String, "test").is_ok());
        assert!(validate_value(&value, &Field::I64, "test").is_err());
    }

    #[test]
    fn validate_binary() {
        let value = LoroValue::Binary(vec![1, 2, 3].into());
        assert!(validate_value(&value, &Field::Binary, "test").is_ok());
        assert!(validate_value(&value, &Field::String, "test").is_err());
    }

    #[test]
    fn validate_list() {
        let items = vec![LoroValue::String("a".into()), LoroValue::String("b".into())];
        let value = LoroValue::List(items.into());

        assert!(validate_value(&value, &Field::List(Box::new(Field::String)), "test",).is_ok());
        assert!(validate_value(&value, &Field::List(Box::new(Field::I64)), "test",).is_err());
    }

    #[test]
    fn validate_movable_list() {
        let items = vec![LoroValue::I64(1), LoroValue::I64(2), LoroValue::I64(3)];
        let value = LoroValue::List(items.into());

        assert!(validate_value(&value, &Field::MovableList(Box::new(Field::I64)), "test",).is_ok());
        assert!(
            validate_value(&value, &Field::MovableList(Box::new(Field::String)), "test",).is_err()
        );
    }

    #[test]
    fn validate_struct_valid() {
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), LoroValue::String("test".into()));
        map.insert("age".to_string(), LoroValue::I64(42));
        let value = LoroValue::Map(map.into());

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));
        fields.insert("age".into(), Box::new(Field::I64));

        assert!(validate_value(&value, &Field::Struct(fields), "test",).is_ok());
    }

    #[test]
    fn validate_struct_wrong_type() {
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), LoroValue::String("test".into()));
        map.insert("age".to_string(), LoroValue::I64(42));
        let value = LoroValue::Map(map.into());

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));
        fields.insert("age".into(), Box::new(Field::String));
        assert!(validate_value(&value, &Field::Struct(fields), "test",).is_err());
    }

    #[test]
    fn validate_map_homogeneous() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1".to_string(), LoroValue::String("value1".into()));
        map.insert("key2".to_string(), LoroValue::String("value2".into()));
        let value = LoroValue::Map(map.into());

        assert!(validate_value(&value, &Field::Map(Box::new(Field::String)), "test",).is_ok());
        assert!(validate_value(&value, &Field::Map(Box::new(Field::I64)), "test",).is_err());
    }

    #[test]
    fn validate_tree_single_node() {
        let tree = LoroTree::default();
        let leaf_id = tree.create(None).expect("leaf");

        let meta = tree.get_meta(leaf_id).expect("meta");
        meta.insert("key1", LoroValue::String("value1".into()))
            .expect("insert");

        let value = tree.get_value_with_meta();

        let mut fields = BTreeMap::new();
        fields.insert("key1".into(), Box::new(Field::String));

        validate_value(
            &value,
            &Field::Tree(Box::new(Field::Struct(fields))),
            "test",
        )
        .expect("valid");
    }

    #[test]
    fn validate_tree_multiple_nodes() {
        let tree = LoroTree::default();
        let root = tree.create(None).expect("root");
        let child = tree.create(root).expect("child");

        let root_meta = tree.get_meta(root).expect("meta");
        root_meta
            .insert("name", LoroValue::String("root".into()))
            .expect("insert");

        let child_meta = tree.get_meta(child).expect("meta");
        child_meta
            .insert("name", LoroValue::String("child".into()))
            .expect("insert");

        let value = tree.get_value_with_meta();

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));

        validate_value(
            &value,
            &Field::Tree(Box::new(Field::Struct(fields))),
            "test",
        )
        .expect("valid");
    }

    #[test]
    fn validate_tree_invalid_meta_type() {
        let tree = LoroTree::default();
        let node = tree.create(None).expect("node");

        let meta = tree.get_meta(node).expect("meta");
        meta.insert("count", LoroValue::I64(42)).expect("insert");

        let value = tree.get_value_with_meta();

        let mut fields = BTreeMap::new();
        fields.insert("count".into(), Box::new(Field::String));

        assert!(
            validate_value(
                &value,
                &Field::Tree(Box::new(Field::Struct(fields))),
                "test",
            )
            .is_err()
        );
    }

    #[test]
    fn validate_tree_missing_required_field() {
        let tree = LoroTree::default();
        let node = tree.create(None).expect("node");

        let meta = tree.get_meta(node).expect("meta");
        meta.insert("a", LoroValue::String("hello".into()))
            .expect("insert");

        let value = tree.get_value_with_meta();

        let mut fields = BTreeMap::new();
        fields.insert("a".into(), Box::new(Field::String));
        fields.insert("b".into(), Box::new(Field::String));

        assert!(matches!(
            validate_value(
                &value,
                &Field::Tree(Box::new(Field::Struct(fields))),
                "test",
            ),
            Err(ValidationError::InvalidElement { .. })
        ));
    }

    #[test]
    fn validate_tree_empty() {
        let value = LoroValue::List(vec![].into());

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));

        validate_value(
            &value,
            &Field::Tree(Box::new(Field::Struct(fields))),
            "test",
        )
        .expect("empty tree is valid");
    }

    #[test]
    fn validate_tree_nested_meta() {
        let tree = LoroTree::default();
        let node = tree.create(None).expect("node");

        let meta = tree.get_meta(node).expect("meta");
        meta.insert("name", LoroValue::String("node".into()))
            .expect("insert");
        meta.insert(
            "tags",
            LoroValue::List(
                vec![LoroValue::String("a".into()), LoroValue::String("b".into())].into(),
            ),
        )
        .expect("insert");

        let value = tree.get_value_with_meta();

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), Box::new(Field::String));
        fields.insert(
            "tags".into(),
            Box::new(Field::List(Box::new(Field::String))),
        );

        validate_value(
            &value,
            &Field::Tree(Box::new(Field::Struct(fields))),
            "test",
        )
        .expect("valid");
    }

    #[test]
    fn validate_tree_wrong_value_type() {
        let value = LoroValue::String("not a tree".into());
        let field = Field::Tree(Box::new(Field::Struct(BTreeMap::new())));
        assert!(validate_value(&value, &field, "test").is_err());
    }

    #[test]
    fn validate_optional_present() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::String("hello".into());
        assert!(validate_value(&value, &field, "test").is_ok());
    }

    #[test]
    fn validate_optional_null() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::Null;
        assert!(validate_value(&value, &field, "test").is_ok());
    }

    #[test]
    fn validate_optional_wrong_type() {
        let field = Field::Optional(Box::new(Field::String));
        let value = LoroValue::I64(42);
        assert!(validate_value(&value, &field, "test").is_err());
    }

    #[test]
    fn validate_optional_absent_in_struct() {
        let map = std::collections::HashMap::new();
        let value = LoroValue::Map(map.into());

        let mut fields = BTreeMap::new();
        fields.insert(
            "opt".into(),
            Box::new(Field::Optional(Box::new(Field::String))),
        );

        assert!(validate_value(&value, &Field::Struct(fields), "test").is_ok());
    }

    #[test]
    fn validate_required_absent_in_struct() {
        let map = std::collections::HashMap::new();
        let value = LoroValue::Map(map.into());

        let mut fields = BTreeMap::new();
        fields.insert("required".into(), Box::new(Field::String));

        assert!(matches!(
            validate_value(&value, &Field::Struct(fields), "test"),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn validate_blob_id_valid() {
        let value = LoroValue::Binary(vec![0xaf; 32].into());
        assert!(validate_value(&value, &Field::BlobId, "test").is_ok());
    }

    #[test]
    fn validate_blob_id_too_short() {
        let value = LoroValue::Binary(vec![1, 2, 3].into());
        assert!(validate_value(&value, &Field::BlobId, "test").is_err());
    }

    #[test]
    fn validate_blob_id_too_long() {
        let value = LoroValue::Binary(vec![0; 33].into());
        assert!(validate_value(&value, &Field::BlobId, "test").is_err());
    }

    #[test]
    fn validate_blob_id_wrong_type() {
        let value = LoroValue::String("not binary".into());
        assert!(validate_value(&value, &Field::BlobId, "test").is_err());
    }

    #[test]
    fn validate_record_id_valid() {
        let value = LoroValue::Binary(vec![0xaf; 32].into());
        assert!(validate_value(&value, &Field::RecordId, "test").is_ok());
    }

    #[test]
    fn validate_record_id_too_short() {
        let value = LoroValue::Binary(vec![1, 2, 3].into());
        assert!(validate_value(&value, &Field::RecordId, "test").is_err());
    }

    #[test]
    fn validate_record_id_too_long() {
        let value = LoroValue::Binary(vec![0; 33].into());
        assert!(validate_value(&value, &Field::RecordId, "test").is_err());
    }

    #[test]
    fn validate_record_id_wrong_type() {
        let value = LoroValue::String("not binary".into());
        assert!(validate_value(&value, &Field::RecordId, "test").is_err());
    }

    #[test]
    fn validate_enum_unit_variant() {
        let mut map = std::collections::HashMap::new();
        map.insert("tag".to_string(), LoroValue::String("Fixed".into()));
        let value = LoroValue::Map(map.into());

        let mut variants = BTreeMap::new();
        variants.insert("Fixed".into(), None);
        variants.insert("Dynamic".into(), None);

        assert!(validate_value(&value, &Field::Enum(variants), "test").is_ok());
    }

    #[test]
    fn validate_enum_struct_variant() {
        let mut inner = std::collections::HashMap::new();
        inner.insert("radius".to_string(), LoroValue::Double(1.5));

        let mut map = std::collections::HashMap::new();
        map.insert("tag".to_string(), LoroValue::String("Sphere".into()));
        map.insert("Sphere".to_string(), LoroValue::Map(inner.into()));
        let value = LoroValue::Map(map.into());

        let mut sphere_fields = BTreeMap::new();
        sphere_fields.insert("radius".into(), Box::new(Field::F64));

        let mut variants = BTreeMap::new();
        variants.insert(
            "Sphere".into(),
            Some(Box::new(Field::Struct(sphere_fields))),
        );
        variants.insert("Box".into(), None);

        assert!(validate_value(&value, &Field::Enum(variants), "test").is_ok());
    }

    #[test]
    fn validate_enum_unknown_variant() {
        let mut map = std::collections::HashMap::new();
        map.insert("tag".to_string(), LoroValue::String("Unknown".into()));
        let value = LoroValue::Map(map.into());

        let mut variants = BTreeMap::new();
        variants.insert("Fixed".into(), None);

        assert!(matches!(
            validate_value(&value, &Field::Enum(variants), "test"),
            Err(ValidationError::UnknownVariant(_))
        ));
    }

    #[test]
    fn validate_enum_missing_tag() {
        let map = std::collections::HashMap::new();
        let value = LoroValue::Map(map.into());

        let mut variants = BTreeMap::new();
        variants.insert("Fixed".into(), None);

        assert!(matches!(
            validate_value(&value, &Field::Enum(variants), "test"),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn validate_enum_missing_variant_data() {
        let mut map = std::collections::HashMap::new();
        map.insert("tag".to_string(), LoroValue::String("Sphere".into()));
        // "Sphere" data key missing
        let value = LoroValue::Map(map.into());

        let mut sphere_fields = BTreeMap::new();
        sphere_fields.insert("radius".into(), Box::new(Field::F64));

        let mut variants = BTreeMap::new();
        variants.insert(
            "Sphere".into(),
            Some(Box::new(Field::Struct(sphere_fields))),
        );

        assert!(matches!(
            validate_value(&value, &Field::Enum(variants), "test"),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn validate_restricted_delegates_to_inner() {
        let field = Field::Restricted {
            actions: vec![],
            value: Box::new(Field::String),
        };
        let good = LoroValue::String("ok".into());
        let bad = LoroValue::I64(42);
        assert!(validate_value(&good, &field, "test").is_ok());
        assert!(validate_value(&bad, &field, "test").is_err());
    }
}
