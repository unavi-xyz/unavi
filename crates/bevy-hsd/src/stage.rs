//! Stage hydration types for parsing Loro documents.

use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError};

use crate::attributes::xform::Xform;

/// Top-level stage data.
#[derive(Debug, Clone, Hydrate)]
pub struct StageData {
    pub layers: Vec<LayerData>,
}

/// Layer within a stage.
#[derive(Debug, Clone, Hydrate)]
pub struct LayerData {
    #[loro(with = "wired_schemas::conv::tree")]
    pub nodes: Vec<NodeData>,
    pub opinions: Vec<OpinionData>,
}

/// Node within a tree.
#[derive(Debug, Clone, Hydrate)]
pub struct NodeData {
    pub id: String,
}

/// Opinion with parsed attributes.
#[derive(Debug, Clone, Hydrate)]
pub struct OpinionData {
    pub id: String,
    #[loro(hydrate_with = "hydrate_attrs")]
    pub attrs: Attrs,
}

/// Known attributes parsed from the attrs map.
#[derive(Debug, Clone, Default)]
pub struct Attrs {
    pub xform: Option<Xform>,
}

fn hydrate_attrs(value: &LoroValue) -> Result<Attrs, HydrateError> {
    let LoroValue::Map(map) = value else {
        return Err(HydrateError::TypeMismatch {
            path: String::new(),
            expected: "map".into(),
            actual: format!("{value:?}"),
        });
    };

    let xform = map.get("xform").map(Xform::hydrate).transpose()?;

    Ok(Attrs { xform })
}
