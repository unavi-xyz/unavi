//! Restriction and authorization helpers.

use super::ChangeType;
use crate::Field;

/// Unwrap Restricted and Optional wrappers to get the inner
/// field type.
#[must_use]
pub fn unwrap_restricted(field: &Field) -> &Field {
    match field {
        Field::Restricted { value, .. } => unwrap_restricted(value),
        Field::Optional(inner) => unwrap_restricted(inner),
        other => other,
    }
}

#[must_use]
pub const fn change_type_name(ct: ChangeType) -> &'static str {
    match ct {
        ChangeType::Create => "create",
        ChangeType::Delete => "delete",
        ChangeType::Update => "update",
    }
}

/// Find all Restricted actions that apply to a given path.
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
    use crate::{Action, Can, Field, Who};

    #[test]
    fn unwrap_plain_field() {
        assert!(matches!(unwrap_restricted(&Field::Bool), Field::Bool));
    }

    #[test]
    fn unwrap_single_restricted() {
        let field = Field::Restricted {
            actions: vec![],
            value: Box::new(Field::String),
        };
        assert!(matches!(unwrap_restricted(&field), Field::String));
    }

    #[test]
    fn unwrap_nested_restricted() {
        let field = Field::Restricted {
            actions: vec![],
            value: Box::new(Field::Optional(Box::new(Field::Restricted {
                actions: vec![],
                value: Box::new(Field::I64),
            }))),
        };
        assert!(matches!(unwrap_restricted(&field), Field::I64));
    }

    #[test]
    fn restrictions_top_level() {
        let field = Field::Restricted {
            actions: vec![Action {
                who: Who::Anyone,
                can: vec![Can::Update],
            }],
            value: Box::new(Field::String),
        };
        let r = find_restrictions_for_path(&field, "container");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn restrictions_nested_struct() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "inner".into(),
            Box::new(Field::Restricted {
                actions: vec![Action {
                    who: Who::Anyone,
                    can: vec![Can::Create],
                }],
                value: Box::new(Field::String),
            }),
        );
        let field = Field::Struct(fields);
        let r = find_restrictions_for_path(&field, "c.inner");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn restrictions_no_match() {
        let field = Field::String;
        let r = find_restrictions_for_path(&field, "c.missing");
        assert!(r.is_empty());
    }

    #[test]
    fn restrictions_through_tree() {
        let inner = Field::Restricted {
            actions: vec![Action {
                who: Who::Anyone,
                can: vec![Can::Create, Can::Delete],
            }],
            value: Box::new(Field::Struct(BTreeMap::new())),
        };
        let field = Field::Tree(Box::new(inner));
        let r = find_restrictions_for_path(&field, "nodes");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn change_type_names() {
        assert_eq!(change_type_name(ChangeType::Create), "create");
        assert_eq!(change_type_name(ChangeType::Delete), "delete");
        assert_eq!(change_type_name(ChangeType::Update), "update");
    }
}
