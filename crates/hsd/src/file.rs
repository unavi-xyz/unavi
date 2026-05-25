use std::collections::{
    BTreeMap,
    HashMap,
};

use anyhow::Result;
use loro::{
    LoroDoc,
    TreeID,
    TreeParentId,
};
use loro_surgeon::{
    Reconcile,
    reconcile::RootReconciler,
};
use ron::extensions::Extensions;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::warn;

use crate::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::Attributes,
};

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFile(pub Vec<HsdFilePrim>);

impl HsdFile {
    pub fn from_ron(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }

    pub fn load_into_doc(&self, doc: &LoroDoc) -> Result<()> {
        let tree = doc.get_tree(&*HSD_CONTAINER_ID);
        let mut id_map: HashMap<String, TreeID> = HashMap::new();
        let mut pairs: Vec<(&HsdFilePrim, TreeID)> = Vec::new();

        create_prims(&tree, TreeParentId::Root, &self.0, &mut id_map, &mut pairs);

        for (prim, tree_id) in &pairs {
            let meta = tree.get_meta(*tree_id)?;

            let mut attributes = prim.attributes.clone();
            if let Some(material) = &mut attributes.material {
                material.resolve_refs(|name| resolve_ref(&id_map, name));
            }

            let mut rels: BTreeMap<String, String> = BTreeMap::new();
            for (key, val) in &prim.relationships {
                rels.insert(key.clone(), resolve_ref(&id_map, val));
            }

            let prim_meta = PrimMeta {
                attributes:    Some(attributes),
                relationships: if rels.is_empty() { None } else { Some(rels) },
            };

            prim_meta
                .reconcile(RootReconciler::new(meta))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }

        doc.commit();
        Ok(())
    }
}

fn resolve_ref(id_map: &HashMap<String, TreeID>, val: &str) -> String {
    id_map.get(val).map_or_else(
        || {
            warn!("hsd reference {val:?} does not match any named prim; keeping literal");
            val.to_owned()
        },
        ToString::to_string,
    )
}

fn create_prims<'a>(
    tree: &loro::LoroTree,
    parent: TreeParentId,
    prims: &'a [HsdFilePrim],
    id_map: &mut HashMap<String, TreeID>,
    out: &mut Vec<(&'a HsdFilePrim, TreeID)>,
) {
    for prim in prims {
        let tree_id = tree.create(parent).expect("create prim");
        if let Some(name) = &prim.attributes.name
            && let Some(prev) = id_map.insert(name.0.clone(), tree_id)
        {
            warn!(
                "hsd: duplicate prim name {:?}; later definition (tree id {tree_id}) shadows {prev}",
                name.0,
            );
        }
        out.push((prim, tree_id));
        create_prims(
            tree,
            TreeParentId::Node(tree_id),
            &prim.children,
            id_map,
            out,
        );
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HsdFilePrim {
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub attributes:    Attributes,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relationships: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children:      Vec<Self>,
}
