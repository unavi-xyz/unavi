use loro_surgeon::Hydrate;

use crate::attributes::Attribute;

#[derive(Debug, Clone, Hydrate)]
pub struct Material {
    #[loro(with = "wired_schemas::conv::float_slice::optional")]
    pub color: Option<[f64; 3]>,
    pub metallic: f64,
    pub roughness: f64,
}

impl Attribute for Material {
    fn merge(mut self, next: &Self) -> Self {
        if let Some(color) = &next.color {
            self.color = Some(*color);
        }
        self
    }
}
