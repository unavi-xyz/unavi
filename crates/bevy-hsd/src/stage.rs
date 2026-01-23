//! Stage hydration types for parsing Loro documents.

use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError};
use smol_str::SmolStr;
use wired_schemas::conv::tree::TreeNode;

use crate::attributes::{material::Material, mesh::Mesh, xform::Xform};

/// Top-level stage data.
#[derive(Debug, Clone, Hydrate)]
pub struct StageData {
    pub layers: Vec<LayerData>,
}

/// Layer within a stage.
#[derive(Debug, Clone, Hydrate)]
pub struct LayerData {
    #[loro(with = "wired_schemas::conv::tree")]
    pub nodes: Vec<TreeNode<NodeData>>,
    pub opinions: Vec<OpinionData>,
}

/// Node metadata within a tree.
#[derive(Debug, Clone, Hydrate)]
pub struct NodeData {
    pub id: SmolStr,
}

/// Opinion with parsed attributes.
#[derive(Debug, Clone, Hydrate)]
pub struct OpinionData {
    pub id: SmolStr,
    #[loro(hydrate_with = "hydrate_attrs")]
    pub attrs: Attrs,
}

/// Known attributes parsed from the attrs map.
#[derive(Debug, Clone, Default)]
pub struct Attrs {
    pub material: Option<Material>,
    pub mesh: Option<Mesh>,
    pub xform: Option<Xform>,
}

fn hydrate_attrs(value: &LoroValue) -> Result<Attrs, HydrateError> {
    let LoroValue::Map(map) = value else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "map".into(),
            actual: format!("{value:?}").into(),
        });
    };

    let material = map.get("material").map(Material::hydrate).transpose()?;
    let mesh = map.get("mesh").map(Mesh::hydrate).transpose()?;
    let xform = map.get("xform").map(Xform::hydrate).transpose()?;

    Ok(Attrs {
        material,
        mesh,
        xform,
    })
}
