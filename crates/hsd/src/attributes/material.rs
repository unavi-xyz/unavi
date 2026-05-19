use lorosurgeon::{Hydrate, MaybeMissing, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default)]
pub struct ColorVec(pub Vec<f64>);

#[derive(Hydrate, Reconcile, Debug, Clone, Default)]
#[loro(default)]
pub struct MaterialAttr {
    pub alpha_cutoff: MaybeMissing<f64>,
    pub alpha_mode: MaybeMissing<String>,
    pub base_color: MaybeMissing<ColorVec>,
    pub base_color_texture: MaybeMissing<String>,
    pub double_sided: MaybeMissing<bool>,
    pub emissive: MaybeMissing<ColorVec>,
    pub emissive_texture: MaybeMissing<String>,
    pub metallic: MaybeMissing<f64>,
    pub metallic_roughness_texture: MaybeMissing<String>,
    pub normal_texture: MaybeMissing<String>,
    pub occlusion_texture: MaybeMissing<String>,
    pub roughness: MaybeMissing<f64>,
}

impl Attribute for MaterialAttr {
    const KEY: &str = "material";
}
