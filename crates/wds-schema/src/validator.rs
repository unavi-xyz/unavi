//! Document-level schema validation with Restricted field authorization.

use std::collections::BTreeMap;

use loro::{
    LoroDoc, LoroValue, TreeExternalDiff,
    event::{Diff, DiffBatch, ListDiffItem, TreeDiff},
};
use smol_str::SmolStr;

use crate::{
    Can, ChangeType, Field, Schema, ValidationError, Who, change_type_name,
    find_restrictions_for_path, unwrap_restricted, validate_container_diff,
};

/// Document-level schema validator.
pub struct Validator<'a> {
    schemas: &'a BTreeMap<SmolStr, Schema>,
    author: &'a str,
}

impl<'a> Validator<'a> {
    #[must_use]
    pub const fn new(schemas: &'a BTreeMap<SmolStr, Schema>, author: &'a str) -> Self {
        Self { schemas, author }
    }

    /// # Errors
    ///
    /// Returns [`ValidationError`] if any value doesn't match its schema
    /// or if the author lacks permission for a Restricted field change.
    pub fn validate_diff_batch(
        &self,
        doc: &LoroDoc,
        diff: &DiffBatch,
    ) -> Result<(), ValidationError> {
        for (container_id, container_diff) in diff.iter() {
            if !container_id.is_root() {
                continue;
            }

            let container_name = container_id.name();

            if let Some(schema) = self.schemas.get(container_name.as_str()) {
                validate_container_diff(container_diff, schema, container_name.as_str())?;

                self.validate_restrictions(
                    doc,
                    container_diff,
                    schema.layout(),
                    container_name.as_str(),
                )?;
            }
        }

        Ok(())
    }

    fn validate_restrictions(
        &self,
        doc: &LoroDoc,
        diff: &Diff<'_>,
        field: &Field,
        path: &str,
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
                    self.validate_field_change(doc, field, &field_path, change_type)?;
                }
                Ok(())
            }
            Diff::List(items) => self.validate_list_restrictions(doc, items, field, path),
            Diff::Tree(tree_diff) => self.validate_tree_restrictions(doc, tree_diff, field, path),
            _ => Ok(()),
        }
    }

    fn validate_list_restrictions(
        &self,
        doc: &LoroDoc,
        items: &[ListDiffItem],
        field: &Field,
        path: &str,
    ) -> Result<(), ValidationError> {
        let is_movable = matches!(unwrap_restricted(field), Field::MovableList(_));

        for item in items {
            match item {
                ListDiffItem::Insert { is_move, .. } => {
                    let change_type = if *is_move && is_movable {
                        ChangeType::Update
                    } else {
                        ChangeType::Create
                    };
                    self.validate_field_change(doc, field, path, change_type)?;
                }
                ListDiffItem::Delete { .. } => {
                    self.validate_field_change(doc, field, path, ChangeType::Delete)?;
                }
                ListDiffItem::Retain { .. } => {}
            }
        }
        Ok(())
    }

    fn validate_tree_restrictions(
        &self,
        doc: &LoroDoc,
        tree_diff: &TreeDiff,
        field: &Field,
        path: &str,
    ) -> Result<(), ValidationError> {
        for item in &tree_diff.diff {
            let change_type = match &item.action {
                TreeExternalDiff::Create { .. } => ChangeType::Create,
                TreeExternalDiff::Move { .. } => ChangeType::Update,
                TreeExternalDiff::Delete { .. } => ChangeType::Delete,
            };
            self.validate_field_change(doc, field, path, change_type)?;
        }
        Ok(())
    }

    fn validate_field_change(
        &self,
        doc: &LoroDoc,
        field: &Field,
        path: &str,
        change_type: ChangeType,
    ) -> Result<(), ValidationError> {
        let restrictions = find_restrictions_for_path(field, path);

        for (actions, _) in restrictions {
            for action in actions {
                let covers_change = action.can.iter().any(|c| {
                    matches!(
                        (c, change_type),
                        (Can::Create, ChangeType::Create)
                            | (Can::Update, ChangeType::Update)
                            | (Can::Delete, ChangeType::Delete)
                    )
                });

                if covers_change && !self.is_authorized(doc, &action.who) {
                    return Err(ValidationError::AccessDenied {
                        path: path.to_string(),
                        action: change_type_name(change_type),
                    });
                }
            }
        }

        Ok(())
    }

    /// If path doesn't exist, no restriction applies (allowed).
    fn is_authorized(&self, doc: &LoroDoc, who: &Who) -> bool {
        match who {
            Who::Anyone => true,
            Who::Path(path) => {
                resolve_path(doc, path).is_none_or(|ids| ids.contains(&self.author.to_string()))
            }
        }
    }
}

/// Returns `None` if the path doesn't exist.
fn resolve_path(doc: &LoroDoc, path: &str) -> Option<Vec<String>> {
    let mut parts = path.split('.');
    let container = parts.next()?;
    let map = doc.get_map(container);
    let value = map.get_deep_value();

    let mut current = &value;
    for part in parts {
        let LoroValue::Map(m) = current else {
            return None;
        };
        current = m.get(part)?;
    }

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
