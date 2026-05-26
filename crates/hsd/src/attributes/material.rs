use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorVec(pub Vec<f64>);

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct MaterialAttr {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode: Option<String>,
    pub base_color: Option<ColorVec>,
    pub base_color_texture: Option<String>,
    pub double_sided: Option<bool>,
    pub emissive: Option<ColorVec>,
    pub emissive_texture: Option<String>,
    pub metallic: Option<f64>,
    pub metallic_roughness_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub occlusion_texture: Option<String>,
    pub roughness: Option<f64>,
}

impl MaterialAttr {
    pub(crate) fn resolve_refs(&mut self, mut resolve: impl FnMut(&str) -> String) {
        let mut apply = |slot: &mut Option<String>| {
            if let Some(v) = slot {
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
