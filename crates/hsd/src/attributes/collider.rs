use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

/// `ConvexHull` and `Trimesh` read their buffers from the `collider:vertices`
/// and `collider:indices` bulk entries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColliderAttr {
    Capsule { height: f64, radius: f64 },
    ConvexHull,
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
    Trimesh,
}

impl Attribute for ColliderAttr {
    const KEY: &'static str = "collider";
}
