use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Payload,
        Property,
        name::PropName,
        render_bytes,
    },
};

pub const GROUP: &str = "collider";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderKind {
    Capsule {
        height: f64,
        radius: f64,
    },
    /// Reads [`ColliderVertices`].
    ConvexHull,
    Cuboid {
        x: f64,
        y: f64,
        z: f64,
    },
    Cylinder {
        height: f64,
        radius: f64,
    },
    Sphere(f64),
    /// Reads [`ColliderVertices`] and [`ColliderIndices`].
    Trimesh,
}

impl Property for ColliderKind {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/kind"));
}

/// Raw little-endian `f32` positions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColliderVertices(pub Vec<u8>);

impl Property for ColliderVertices {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/vertices"));

    fn render(payload: &[u8]) -> String {
        match Self::decode(payload) {
            Ok(value) => render_bytes(value.0.len()),
            Err(err) => format!("<undecodable: {err}>"),
        }
    }
}

/// Raw little-endian `u32` triangle indices.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColliderIndices(pub Vec<u8>);

impl Property for ColliderIndices {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/indices"));

    fn render(payload: &[u8]) -> String {
        match Self::decode(payload) {
            Ok(value) => render_bytes(value.0.len()),
            Err(err) => format!("<undecodable: {err}>"),
        }
    }
}
