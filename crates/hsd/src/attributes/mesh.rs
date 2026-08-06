use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

/// Vertex buffers are not named here. Each is its own `mesh:<NAME>` slot,
/// so nothing in the payload has to be kept consistent with the blob store.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAttr {
    pub topology: Topology,
}

impl Attribute for MeshAttr {
    const KEY: &'static str = "mesh";
}
