use loro::LoroMapValue;
use loro_surgeon::Hydrate;
use smol_str::SmolStr;
use wired_schemas::HydratedHash;

use crate::hydration::topology::HydratedTopology;

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
    #[loro(rename = "collider/params", default)]
    pub collider_params: Option<Vec<f32>>,
    #[loro(rename = "collider/shape", default)]
    pub collider_shape: Option<SmolStr>,

    #[loro(rename = "material/base_color", default)]
    pub material_base_color: Option<[f32; 3]>,
    #[loro(rename = "material/base_color_texture", default)]
    pub material_base_color_texture: Option<HydratedHash>,
    #[loro(rename = "material/double_sided", default)]
    pub material_double_sided: Option<bool>,
    #[loro(rename = "material/metallic", default)]
    pub material_metallic: Option<f32>,
    #[loro(rename = "material/metallic_roughness_texture", default)]
    pub material_metallic_roughness_texture: Option<HydratedHash>,
    #[loro(rename = "material/normal_texture", default)]
    pub material_normal_texture: Option<HydratedHash>,
    #[loro(rename = "material/occlusion_texture", default)]
    pub material_occlusion_texture: Option<HydratedHash>,
    #[loro(rename = "material/roughness", default)]
    pub material_roughness: Option<f32>,

    #[loro(rename = "mesh/colors", default)]
    pub mesh_colors: Option<HydratedHash>,
    #[loro(rename = "mesh/indices", default)]
    pub mesh_indices: Option<HydratedHash>,
    #[loro(rename = "mesh/normals", default)]
    pub mesh_normals: Option<HydratedHash>,
    #[loro(rename = "mesh/points", default)]
    pub mesh_points: Option<HydratedHash>,
    #[loro(rename = "mesh/tangents", default)]
    pub mesh_tangents: Option<HydratedHash>,
    #[loro(rename = "mesh/topology", default)]
    pub mesh_topology: Option<HydratedTopology>,
    #[loro(rename = "mesh/uv0", default)]
    pub mesh_uv_0: Option<HydratedHash>,
    #[loro(rename = "mesh/uv1", default)]
    pub mesh_uv_1: Option<HydratedHash>,

    #[loro(rename = "rigid_body/friction", default)]
    pub rigid_body_friction: Option<f32>,
    #[loro(rename = "rigid_body/kind", default)]
    pub rigid_body_kind: Option<SmolStr>,
    #[loro(rename = "rigid_body/mass", default)]
    pub rigid_body_mass: Option<f32>,

    #[loro(rename = "xform/parent", default)]
    pub xform_parent: Option<i64>,
    #[loro(rename = "xform/pos", default)]
    pub xform_pos: Option<[f32; 3]>,
    #[loro(rename = "xform/rot", default)]
    pub xform_rot: Option<[f32; 4]>,
    #[loro(rename = "xform/scale", default)]
    pub xform_scale: Option<[f32; 3]>,
}
