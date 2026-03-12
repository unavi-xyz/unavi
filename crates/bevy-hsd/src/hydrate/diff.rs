use loro::{ContainerID, Index, TreeExternalDiff, TreeParentId};
use smol_str::SmolStr;

use super::events::RawHsdChange;

pub(super) fn extract_changes_from_diff(e: &loro::event::DiffEvent, queue: &mut Vec<RawHsdChange>) {
    for cd in &e.events {
        let path = cd.path;

        match &cd.diff {
            loro::event::Diff::Tree(tree_diff) => {
                for item in &tree_diff.diff {
                    let tree_id = tree_id_str(item.target);
                    let change = match &item.action {
                        TreeExternalDiff::Create { parent, .. } => {
                            let parent_id = node_parent_str(parent);
                            RawHsdChange::NodeAdded { tree_id, parent_id }
                        }
                        TreeExternalDiff::Delete { .. } => RawHsdChange::NodeRemoved { tree_id },
                        TreeExternalDiff::Move { .. } => RawHsdChange::NodeChanged { tree_id },
                    };
                    queue.push(change);
                }
            }

            loro::event::Diff::Map(map_delta) => {
                if let Some(tree_id) = node_tree_id_in_path(path) {
                    queue.push(RawHsdChange::NodeChanged { tree_id });
                } else if let Some(id) = map_id_for_key(path, "meshes") {
                    queue.push(RawHsdChange::MeshChanged { id });
                } else if let Some(id) = map_id_for_key(path, "materials") {
                    queue.push(RawHsdChange::MaterialChanged { id });
                } else if path_ends_with_key(path, "meshes") {
                    for (key, val) in &map_delta.updated {
                        let id = SmolStr::from(key.as_ref());
                        let change = if val.is_some() {
                            RawHsdChange::MeshAdded { id }
                        } else {
                            RawHsdChange::MeshRemoved { id }
                        };
                        queue.push(change);
                    }
                } else if path_ends_with_key(path, "materials") {
                    for (key, val) in &map_delta.updated {
                        let id = SmolStr::from(key.as_ref());
                        let change = if val.is_some() {
                            RawHsdChange::MaterialAdded { id }
                        } else {
                            RawHsdChange::MaterialRemoved { id }
                        };
                        queue.push(change);
                    }
                }
            }

            _ => {}
        }
    }
}

fn map_id_for_key(path: &[(ContainerID, Index)], key: &str) -> Option<SmolStr> {
    let mut found = false;
    for (_, idx) in path {
        if found {
            return if let Index::Key(k) = idx {
                Some(k.as_str().into())
            } else {
                None
            };
        }
        if matches!(idx, Index::Key(k) if k.as_str() == key) {
            found = true;
        }
    }
    None
}

fn path_ends_with_key(path: &[(ContainerID, Index)], key: &str) -> bool {
    path.last()
        .is_some_and(|(_, idx)| matches!(idx, Index::Key(k) if k.as_str() == key))
}

fn tree_id_str(id: loro::TreeID) -> SmolStr {
    format!("{}@{}", id.counter, id.peer).into()
}

fn node_parent_str(parent: &TreeParentId) -> Option<SmolStr> {
    match parent {
        TreeParentId::Node(pid) => Some(tree_id_str(*pid)),
        _ => None,
    }
}

fn node_tree_id_in_path(path: &[(ContainerID, Index)]) -> Option<SmolStr> {
    for (_, idx) in path {
        if let Index::Node(tid) = idx {
            return Some(tree_id_str(*tid));
        }
    }
    None
}
