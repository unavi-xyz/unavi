use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

/// The collider shape, without its buffers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderKind {
    Capsule { height: f64, radius: f64 },
    ConvexHull,
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
    Trimesh,
}

/// A collider: the shape and the buffers it may read.
///
/// The buffers are payload fields, not sibling keys; `ConvexHull` reads
/// `vertices` and `Trimesh` reads both. They stay beside the kind because the
/// script API sets the kind and the buffers in separate calls. See
/// `docs/designs/hsd-attribute-fields.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColliderAttr {
    pub kind:     ColliderKind,
    pub vertices: Option<Vec<u8>>,
    pub indices:  Option<Vec<u8>>,
}

impl Attribute for ColliderAttr {
    const KEY: &'static str = "collider";
}
