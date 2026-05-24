use loro_surgeon::{bytes::ByteArray, {Hydrate, Reconcile}};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Serialize, Deserialize)]
pub enum ColliderAttr {
    Capsule {
        height: f64,
        radius: f64,
    },
    ConvexHull(ByteArray<32>),
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
    Trimesh {
        indices: ByteArray<32>,
        vertices: ByteArray<32>,
    },
}

impl Attribute for ColliderAttr {
    const KEY: &str = "collider";
}
