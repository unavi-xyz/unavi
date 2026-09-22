use std::collections::BTreeMap;

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

/// Topology plus the vertex streams and index buffer, all in one payload.
///
/// Streams are raw little-endian `f32` bytes keyed by name (`POSITION`,
/// `NORMAL`, `UV_0`, …); the index buffer is raw little-endian `u32`. A
/// `BTreeMap`, not a `HashMap`, so identical meshes encode byte-identically.
/// See `docs/designs/hsd-attribute-fields.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAttr {
    pub topology: Topology,
    pub streams:  BTreeMap<String, Vec<u8>>,
    pub indices:  Option<Vec<u8>>,
}

impl Attribute for MeshAttr {
    const KEY: &'static str = "mesh";
}
