use blake3::Hash;
use loro_surgeon::Hydrate;

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
    pub attrs: Attrs,
    pub node: i64,
}

#[derive(Debug, Clone, Hydrate)]
pub struct Attrs {
    #[loro(rename = "mesh/colors", with = "wired_schemas::conv::hash::optional")]
    mesh_colors: Option<Hash>,
    #[loro(rename = "xform/parent")]
    xform_parent: Option<i64>,
    #[loro(
        rename = "xform/pos",
        with = "wired_schemas::conv::float_slice::optional"
    )]
    xform_pos: Option<[f64; 3]>,
    #[loro(
        rename = "xform/rot",
        with = "wired_schemas::conv::float_slice::optional"
    )]
    xform_rot: Option<[f64; 4]>,
    #[loro(
        rename = "xform/scale",
        with = "wired_schemas::conv::float_slice::optional"
    )]
    xform_scale: Option<[f64; 3]>,
}
