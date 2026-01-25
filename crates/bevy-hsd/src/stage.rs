use loro::LoroMapValue;
use loro_surgeon::Hydrate;
use wired_schemas::HydratedHash;

#[derive(Debug, Clone, Hydrate)]
pub struct StageData {
    pub layers: Vec<LayerData>,
}

#[derive(Debug, Clone, Hydrate)]
pub struct LayerData {
    pub enabled: bool,
    pub opinions: Vec<OpinionData>,
}

#[derive(Debug, Clone, Hydrate)]
pub struct OpinionData {
    pub attrs: LoroMapValue,
    pub node: i64,
}

/// Typed attributes, hydrated from merged [`LoroMapValue`] during compilation.
#[derive(Debug, Clone, Default, Hydrate)]
pub struct Attrs {
    #[loro(rename = "mesh/colors")]
    pub mesh_colors: Option<HydratedHash>,
    #[loro(rename = "xform/parent")]
    pub xform_parent: Option<i64>,
    #[loro(rename = "xform/pos")]
    pub xform_pos: Option<[f64; 3]>,
    #[loro(rename = "xform/rot")]
    pub xform_rot: Option<[f64; 4]>,
    #[loro(rename = "xform/scale")]
    pub xform_scale: Option<[f64; 3]>,
}
