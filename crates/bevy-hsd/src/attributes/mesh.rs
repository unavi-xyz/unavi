use blake3::Hash;
use loro_surgeon::Hydrate;

use crate::attributes::Attribute;

#[derive(Debug, Clone, Hydrate)]
pub struct Mesh {
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub colors: Option<Hash>,
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub indices: Option<Hash>,
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub normals: Option<Hash>,
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub points: Option<Hash>,
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub tangents: Option<Hash>,
    #[loro(with = "wired_schemas::conv::hash::optional")]
    pub uvs: Option<Hash>,
}

impl Attribute for Mesh {
    fn merge(mut self, next: &Self) -> Self {
        if let Some(colors) = &next.colors {
            self.colors = Some(*colors);
        }
        if let Some(indices) = &next.indices {
            self.indices = Some(*indices);
        }
        if let Some(normals) = &next.normals {
            self.normals = Some(*normals);
        }
        if let Some(points) = &next.points {
            self.points = Some(*points);
        }
        if let Some(tangents) = &next.tangents {
            self.tangents = Some(*tangents);
        }
        if let Some(uvs) = &next.uvs {
            self.uvs = Some(*uvs);
        }
        self
    }
}
