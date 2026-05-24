use lorosurgeon::{Hydrate, MaybeMissing, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorVec(pub Vec<f64>);

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct MaterialAttr {
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub alpha_cutoff: MaybeMissing<f64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub alpha_mode: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub base_color: MaybeMissing<ColorVec>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub base_color_texture: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub double_sided: MaybeMissing<bool>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub emissive: MaybeMissing<ColorVec>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub emissive_texture: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub metallic: MaybeMissing<f64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub metallic_roughness_texture: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub normal_texture: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub occlusion_texture: MaybeMissing<String>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub roughness: MaybeMissing<f64>,
}

impl MaterialAttr {
    pub(crate) fn resolve_refs(&mut self, mut resolve: impl FnMut(&str) -> String) {
        let mut apply = |slot: &mut MaybeMissing<String>| {
            if let MaybeMissing::Present(v) = slot {
                *v = resolve(v.as_str());
            }
        };
        apply(&mut self.base_color_texture);
        apply(&mut self.emissive_texture);
        apply(&mut self.metallic_roughness_texture);
        apply(&mut self.normal_texture);
        apply(&mut self.occlusion_texture);
    }
}

impl Attribute for MaterialAttr {
    const KEY: &str = "material";
}
