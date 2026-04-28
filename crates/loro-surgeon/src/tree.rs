use loro::{LoroMap, LoroTree, LoroValue, TreeID, TreeParentId};
use serde::{Deserialize, Serialize};

use crate::{Hydrate, HydrateError, Reconcile, ReconcileError};

/// A node in a Loro tree with typed metadata, supporting arbitrary depth.
///
/// The `id` field is populated during hydration and ignored during reconcile.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TreeNode<T>
where
    T: Hydrate + Reconcile,
{
    /// Loro tree ID; set when hydrated from Loro, `None` when constructing new nodes.
    #[serde(skip)]
    pub id: Option<TreeID>,
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub children: Vec<Self>,
}

impl<T: Hydrate + Reconcile> TreeNode<T> {
    pub fn insert_into(
        &self,
        tree: &LoroTree,
        parent: impl Into<TreeParentId>,
    ) -> Result<(), ReconcileError> {
        let node = tree.create(parent)?;
        if let Some(data) = &self.data {
            let meta = tree.get_meta(node)?;
            data.reconcile(&meta)?;
        }
        for child in &self.children {
            child.insert_into(tree, node)?;
        }
        Ok(())
    }
}

impl<T: Hydrate + Reconcile> Reconcile for TreeNode<T> {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "cannot reconcile TreeNode as root container".into(),
        ))
    }

    /// Inserts this node as the sole root of the tree at `key`, clearing existing roots first.
    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let tree = map.get_or_create_container(key, LoroTree::new())?;
        for id in tree.roots() {
            tree.delete(id)?;
        }
        self.insert_into(&tree, TreeParentId::Root)?;
        Ok(())
    }
}

impl<T: Hydrate + Reconcile + Default> Hydrate for TreeNode<T> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::Map(map) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "Map (tree node)".into(),
                actual: format!("{value:?}").into(),
            });
        };

        let id = match map.get("id") {
            Some(LoroValue::String(s)) => Some(
                TreeID::try_from(s.as_str())
                    .map_err(|e| HydrateError::Custom(format!("invalid tree id: {e}").into()))?,
            ),
            _ => None,
        };

        let data = match map.get("meta") {
            Some(LoroValue::Null) | None => None,
            Some(meta) => Some(T::hydrate(meta)?),
        };

        let children = match map.get("children") {
            Some(LoroValue::List(list)) => {
                list.iter().map(Self::hydrate).collect::<Result<_, _>>()?
            }
            _ => vec![],
        };

        Ok(Self { id, data, children })
    }
}
