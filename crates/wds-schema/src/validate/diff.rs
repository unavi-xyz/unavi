//! Diff-level validation for Loro container changes.

use loro::{
    Container, ValueOrContainer,
    event::{Diff, ListDiffItem, MapDelta, TreeDiff},
};

use super::{ValidationError, restriction::unwrap_restricted, value::validate_value};
use crate::{Field, Schema};

/// Validate a container diff against its schema.
pub fn validate_container_diff(
    diff: &Diff<'_>,
    schema: &Schema,
    container_name: &str,
) -> Result<(), ValidationError> {
    match diff {
        Diff::List(items) => validate_list_diff(items, schema.layout(), container_name),
        Diff::Map(map_delta) => validate_map_diff(map_delta, schema.layout(), container_name),
        Diff::Tree(tree_diff) => validate_tree_diff(tree_diff, schema.layout(), container_name),
        _ => Err(ValidationError::TypeMismatch {
            path: container_name.to_string(),
            expected: "map, list, or tree",
        }),
    }
}

fn validate_map_diff(
    map_delta: &MapDelta<'_>,
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let inner = unwrap_restricted(field);

    let (struct_fields, map_inner, enum_variants) = match inner {
        Field::Enum(variants) => (None, None, Some(variants)),
        Field::Map(inner) => (None, Some(inner.as_ref()), None),
        Field::Struct(fields) => (Some(fields), None, None),
        _ => (None, None, None),
    };

    for (key, new_value) in &map_delta.updated {
        let field_path = format!("{path}.{key}");

        if let Some(value) = new_value {
            if let Some(fields) = struct_fields {
                let key_smol: smol_str::SmolStr = key.to_string().into();
                if let Some(expected_field) = fields.get(&key_smol) {
                    validate_value_or_container(value, expected_field, &field_path)?;
                }
            } else if let Some(expected) = map_inner {
                validate_value_or_container(value, expected, &field_path)?;
            } else if let Some(variants) = enum_variants {
                if key == "tag" {
                    validate_value_or_container(value, &Field::String, &field_path)?;
                } else {
                    let key_smol: smol_str::SmolStr = key.to_string().into();
                    if let Some(Some(inner_field)) = variants.get(&key_smol) {
                        validate_value_or_container(value, inner_field, &field_path)?;
                    }
                }
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

fn validate_tree_diff(
    _tree_diff: &TreeDiff,
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let inner = unwrap_restricted(field);
    match inner {
        Field::Tree(_) => {
            // Tree diffs carry structural changes
            // (create/move/delete) without metadata values.
            // Metadata type validation happens via
            // validate_value on the resolved tree state.
            Ok(())
        }
        _ => Err(ValidationError::TypeMismatch {
            path: path.to_string(),
            expected: "tree",
        }),
    }
}

/// Resolve a [`ValueOrContainer`] to a [`LoroValue`] and
/// validate. For tree containers, uses `get_value_with_meta()`
/// to include node metadata.
fn validate_value_or_container(
    value: &ValueOrContainer,
    field: &Field,
    path: &str,
) -> Result<(), ValidationError> {
    let loro_value = match value {
        ValueOrContainer::Container(Container::Tree(tree)) => tree.get_value_with_meta(),
        other => other.get_deep_value(),
    };
    validate_value(&loro_value, field, path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use loro::{Frontiers, LoroDoc, LoroTree, LoroValue};

    use super::*;
    use crate::{Field, Schema};

    fn make_schema(layout: Field) -> Schema {
        let ron_str = format!(
            "(id: \"test\", version: 0, layout: {})",
            ron::to_string(&layout).expect("serialize layout")
        );
        ron::from_str(&ron_str).expect("parse schema")
    }

    #[test]
    fn map_diff_struct_valid() {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert("name", "hello").expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::Struct(BTreeMap::from([(
            "name".into(),
            Box::new(Field::String),
        )])));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        for (cid, d) in diff.iter() {
            if cid.is_root() {
                validate_container_diff(d, &schema, "root").expect("valid");
            }
        }
    }

    #[test]
    fn map_diff_struct_type_mismatch() {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert("name", 42i64).expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::Struct(BTreeMap::from([(
            "name".into(),
            Box::new(Field::String),
        )])));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        let mut found_error = false;
        for (cid, d) in diff.iter() {
            if cid.is_root() && validate_container_diff(d, &schema, "root").is_err() {
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[test]
    fn map_diff_homogeneous_valid() {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert("a", "hello").expect("insert");
        map.insert("b", "world").expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::Map(Box::new(Field::String)));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        for (cid, d) in diff.iter() {
            if cid.is_root() {
                validate_container_diff(d, &schema, "root").expect("valid");
            }
        }
    }

    #[test]
    fn list_diff_valid_inserts() {
        let doc = LoroDoc::new();
        let list = doc.get_list("root");
        list.insert(0, "a").expect("insert");
        list.insert(1, "b").expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::List(Box::new(Field::String)));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        for (cid, d) in diff.iter() {
            if cid.is_root() {
                validate_container_diff(d, &schema, "root").expect("valid");
            }
        }
    }

    #[test]
    fn list_diff_wrong_element_type() {
        let doc = LoroDoc::new();
        let list = doc.get_list("root");
        list.insert(0, 42i64).expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::List(Box::new(Field::String)));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        let mut found_error = false;
        for (cid, d) in diff.iter() {
            if cid.is_root() && validate_container_diff(d, &schema, "root").is_err() {
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[test]
    fn tree_diff_accepted_for_tree_schema() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("root");
        tree.create(None).expect("create");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::Tree(Box::new(Field::Struct(BTreeMap::new()))));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        for (cid, d) in diff.iter() {
            if cid.is_root() {
                validate_container_diff(d, &schema, "root").expect("tree diff should be accepted");
            }
        }
    }

    #[test]
    fn tree_diff_rejected_for_non_tree_schema() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("root");
        tree.create(None).expect("create");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::Struct(BTreeMap::new()));

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        let mut found_error = false;
        for (cid, d) in diff.iter() {
            if cid.is_root() && validate_container_diff(d, &schema, "root").is_err() {
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[test]
    fn text_diff_rejected() {
        let doc = LoroDoc::new();
        let text = doc.get_text("root");
        text.insert(0, "hello").expect("insert");
        let frontiers_after = doc.oplog_frontiers();

        let schema = make_schema(Field::String);

        let diff = doc
            .diff(&Frontiers::default(), &frontiers_after)
            .expect("diff");
        let mut found_error = false;
        for (cid, d) in diff.iter() {
            if cid.is_root() && validate_container_diff(d, &schema, "root").is_err() {
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[test]
    fn value_or_container_tree_resolves_meta() {
        let tree = LoroTree::default();
        let id = tree.create(None).expect("create");
        let meta = tree.get_meta(id).expect("meta");
        meta.insert("name", LoroValue::String("test_node".into()))
            .expect("insert");

        let voc = ValueOrContainer::Container(Container::Tree(tree));
        let field = Field::Tree(Box::new(Field::Struct(BTreeMap::from([(
            "name".into(),
            Box::new(Field::String),
        )]))));
        validate_value_or_container(&voc, &field, "test").expect("should resolve tree metadata");
    }

    #[test]
    fn value_or_container_tree_catches_invalid_meta() {
        let tree = LoroTree::default();
        let id = tree.create(None).expect("create");
        let meta = tree.get_meta(id).expect("meta");
        meta.insert("name", LoroValue::I64(42)).expect("insert");

        let voc = ValueOrContainer::Container(Container::Tree(tree));
        let field = Field::Tree(Box::new(Field::Struct(BTreeMap::from([(
            "name".into(),
            Box::new(Field::String),
        )]))));
        assert!(validate_value_or_container(&voc, &field, "test").is_err());
    }
}
