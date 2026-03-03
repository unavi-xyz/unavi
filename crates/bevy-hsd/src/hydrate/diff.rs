use loro::{ContainerID, Index, TreeExternalDiff, TreeParentId};
use smol_str::SmolStr;

use crate::HsdChange;

pub(super) fn extract_changes_from_diff(e: &loro::event::DiffEvent, queue: &mut Vec<HsdChange>) {
    for cd in &e.events {
        let path = cd.path;

        match &cd.diff {
            loro::event::Diff::Tree(tree_diff) => {
                for item in &tree_diff.diff {
                    let tree_id = tree_id_str(item.target);
                    match &item.action {
                        TreeExternalDiff::Create { parent, .. } => {
                            let parent_id = node_parent_str(parent);
                            queue.push(HsdChange::NodeAdded { tree_id, parent_id });
                        }
                        TreeExternalDiff::Delete { .. } => {
                            queue.push(HsdChange::NodeRemoved { tree_id });
                        }
                        TreeExternalDiff::Move { .. } => {
                            queue.push(HsdChange::NodeMetaChanged { tree_id });
                        }
                    }
                }
            }

            loro::event::Diff::Map(_) => {
                if let Some(tid) = node_tree_id_in_path(path) {
                    queue.push(HsdChange::NodeMetaChanged { tree_id: tid });
                } else if let Some(idx) = list_index_for_key(path, "meshes") {
                    queue.push(HsdChange::MeshChanged { index: idx });
                } else if let Some(idx) = list_index_for_key(path, "materials") {
                    queue.push(HsdChange::MaterialChanged { index: idx });
                }
            }

            loro::event::Diff::List(items) => {
                let last_key = path.last().and_then(|(_, idx)| match idx {
                    Index::Key(k) => Some(k.as_str()),
                    _ => None,
                });
                match last_key {
                    Some("meshes") => process_list_diff(items, queue, true),
                    Some("materials") => process_list_diff(items, queue, false),
                    _ => {}
                }
            }

            _ => {}
        }
    }
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

fn list_index_for_key(path: &[(ContainerID, Index)], key: &str) -> Option<usize> {
    let mut found = false;
    for (_, idx) in path {
        if found {
            return if let Index::Seq(i) = idx {
                Some(*i)
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

fn process_list_diff(
    items: &[loro::event::ListDiffItem],
    queue: &mut Vec<HsdChange>,
    is_mesh: bool,
) {
    use loro::event::ListDiffItem;
    let mut pos: usize = 0;
    for item in items {
        match item {
            ListDiffItem::Retain { retain } => pos += retain,
            ListDiffItem::Insert { insert, .. } => {
                for _ in insert {
                    queue.push(if is_mesh {
                        HsdChange::MeshAdded
                    } else {
                        HsdChange::MaterialAdded
                    });
                    pos += 1;
                }
            }
            ListDiffItem::Delete { delete } => {
                for _ in 0..*delete {
                    queue.push(if is_mesh {
                        HsdChange::MeshRemoved { index: pos }
                    } else {
                        HsdChange::MaterialRemoved { index: pos }
                    });
                    pos += 1;
                }
            }
        }
    }
}
