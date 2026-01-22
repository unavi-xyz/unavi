use loro::{LoroValue, TreeID};
use loro_surgeon::{Hydrate, HydrateError};
use smol_str::SmolStr;

/// A node from a Loro tree with hierarchy info preserved.
#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    pub id: TreeID,
    pub parent: Option<TreeID>,
    pub meta: T,
}

/// Hydrate a tree, preserving hierarchy info alongside meta data.
///
/// # Errors
///
/// Returns an error if the value is not a valid tree structure.
pub fn hydrate<T: Hydrate>(value: &LoroValue) -> Result<Vec<TreeNode<T>>, HydrateError> {
    let LoroValue::List(roots) = value else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "list".into(),
            actual: format!("{value:?}").into(),
        });
    };

    let mut result = Vec::new();
    for root in roots.iter() {
        hydrate_node_recursive(root, &mut result)?;
    }
    Ok(result)
}

fn hydrate_node_recursive<T: Hydrate>(
    node: &LoroValue,
    out: &mut Vec<TreeNode<T>>,
) -> Result<(), HydrateError> {
    let LoroValue::Map(node_map) = node else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "map".into(),
            actual: format!("{node:?}").into(),
        });
    };

    // Extract id field.
    let Some(id) = node_map.get("id") else {
        return Err(HydrateError::MissingField("id".into()));
    };
    let LoroValue::String(id) = id else {
        return Err(HydrateError::TypeMismatch {
            path: "id".into(),
            expected: "string".into(),
            actual: format!("{id:?}").into(),
        });
    };

    let id = TreeID::try_from(id.as_str()).map_err(|_| HydrateError::TypeMismatch {
        path: "id".into(),
        expected: "TreeID".into(),
        actual: id.as_str().into(),
    })?;

    // Extract parent field (may be null or string).
    let parent = node_map.get("parent").and_then(|p| match p {
        LoroValue::String(s) => Some(TreeID::try_from(s.as_str()).ok()?),
        _ => None,
    });

    // Extract and hydrate meta field.
    let Some(meta) = node_map.get("meta") else {
        return Err(HydrateError::MissingField("meta".into()));
    };
    let meta = T::hydrate(meta)?;

    out.push(TreeNode { id, parent, meta });

    // Recursively hydrate children.
    if let Some(LoroValue::List(children)) = node_map.get("children") {
        for child in children.iter() {
            hydrate_node_recursive(child, out)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;
    use loro_surgeon::Hydrate;
    use rstest::rstest;

    use super::*;

    /// Simple metadata struct for testing.
    #[derive(Debug, Clone, PartialEq, Hydrate)]
    struct NodeMeta {
        name: String,
    }

    #[rstest]
    fn hydrate_single_root() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");

        // Create a root node.
        let root_id = tree.create(None).expect("create root");
        let meta = tree.get_meta(root_id).expect("get meta");
        meta.insert("name", "root").expect("insert name");

        // Hydrate the tree.
        let value = tree.get_value_with_meta();
        let nodes: Vec<TreeNode<NodeMeta>> = hydrate(&value).expect("hydrate");

        assert_eq!(nodes.len(), 1);
        assert!(nodes[0].parent.is_none());
        assert_eq!(nodes[0].meta.name, "root");
    }

    #[rstest]
    fn hydrate_parent_child() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");

        // Create root and child.
        let root_id = tree.create(None).expect("create root");
        let root_meta = tree.get_meta(root_id).expect("get root meta");
        root_meta.insert("name", "root").expect("insert name");

        let child_id = tree.create(root_id).expect("create child");
        let child_meta = tree.get_meta(child_id).expect("get child meta");
        child_meta.insert("name", "child").expect("insert name");

        // Hydrate the tree.
        let value = tree.get_value_with_meta();
        let nodes: Vec<TreeNode<NodeMeta>> = hydrate(&value).expect("hydrate");

        assert_eq!(nodes.len(), 2);

        // Find root and child.
        let root = nodes
            .iter()
            .find(|n| n.meta.name == "root")
            .expect("find root");
        let child = nodes
            .iter()
            .find(|n| n.meta.name == "child")
            .expect("find child");

        // Verify hierarchy.
        assert!(root.parent.is_none());
        assert_eq!(child.parent, Some(root.id));
    }

    #[rstest]
    fn hydrate_multi_level_hierarchy() {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");

        // Create: root -> child -> grandchild.
        let root_id = tree.create(None).expect("create root");
        tree.get_meta(root_id)
            .expect("meta")
            .insert("name", "root")
            .expect("insert");

        let child_id = tree.create(root_id).expect("create child");
        tree.get_meta(child_id)
            .expect("meta")
            .insert("name", "child")
            .expect("insert");

        let grandchild_id = tree.create(child_id).expect("create grandchild");
        tree.get_meta(grandchild_id)
            .expect("meta")
            .insert("name", "grandchild")
            .expect("insert");

        // Hydrate the tree.
        let value = tree.get_value_with_meta();
        let nodes: Vec<TreeNode<NodeMeta>> = hydrate(&value).expect("hydrate");

        assert_eq!(nodes.len(), 3);

        let root = nodes
            .iter()
            .find(|n| n.meta.name == "root")
            .expect("find root");
        let child = nodes
            .iter()
            .find(|n| n.meta.name == "child")
            .expect("find child");
        let grandchild = nodes
            .iter()
            .find(|n| n.meta.name == "grandchild")
            .expect("find grandchild");

        // Verify hierarchy chain.
        assert!(root.parent.is_none());
        assert_eq!(child.parent, Some(root.id));
        assert_eq!(grandchild.parent, Some(child.id));
    }
}
