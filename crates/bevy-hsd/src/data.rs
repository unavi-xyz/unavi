use std::collections::BTreeMap;

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use loro::{LoroMap, LoroValue};
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError};
use smol_str::SmolStr;
use wired_schemas::HydratedHash;

#[derive(Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdMaterial {
    #[loro(default)]
    pub alpha_cutoff: Option<f64>,
    #[loro(default)]
    pub alpha_mode: Option<SmolStr>,
    #[loro(default)]
    pub base_color: Option<Vec<f64>>,
    #[loro(default)]
    pub base_color_texture: Option<HydratedHash>,
    #[loro(default)]
    pub double_sided: Option<bool>,
    #[loro(default)]
    pub emissive: Option<Vec<f64>>,
    #[loro(default)]
    pub emissive_texture: Option<HydratedHash>,
    #[loro(default)]
    pub metallic: Option<f64>,
    #[loro(default)]
    pub metallic_roughness_texture: Option<HydratedHash>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
    pub normal_texture: Option<HydratedHash>,
    #[loro(default)]
    pub occlusion_texture: Option<HydratedHash>,
    #[loro(default)]
    pub roughness: Option<f64>,
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdMesh {
    #[loro(default)]
    pub attributes: BTreeMap<SmolStr, HydratedHash>,
    #[loro(default)]
    pub indices: Option<HydratedHash>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    pub topology: HydratedTopology,
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdNodeData {
    #[loro(default)]
    pub collider: Option<HsdCollider>,
    #[loro(default)]
    pub material: Option<SmolStr>,
    #[loro(default)]
    pub mesh: Option<SmolStr>,
    #[loro(default)]
    pub name: Option<SmolStr>,
    #[loro(default)]
    pub rigid_body: Option<HsdRigidBody>,
    #[loro(default)]
    pub rotation: Option<Vec<f64>>,
    #[loro(default)]
    pub scale: Option<Vec<f64>>,
    #[loro(default)]
    pub scripts: Option<Vec<HydratedHash>>,
    #[loro(default)]
    pub translation: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Hydrate, Reconcile)]
pub enum HsdCollider {
    Capsule {
        height: f64,
        radius: f64,
    },
    ConvexHull(HydratedHash),
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
        indices: HydratedHash,
        vertices: HydratedHash,
    },
}

#[derive(Debug, Clone, Default, Hydrate, Reconcile)]
pub struct HsdRigidBody {
    #[loro(default)]
    pub angular_damping: Option<f64>,
    #[loro(default)]
    pub friction: Option<f64>,
    pub kind: SmolStr,
    #[loro(default)]
    pub linear_damping: Option<f64>,
    #[loro(default)]
    pub mass: Option<f64>,
    #[loro(default)]
    pub restitution: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HydratedTopology(pub PrimitiveTopology);

impl Default for HydratedTopology {
    fn default() -> Self {
        Self(PrimitiveTopology::TriangleList)
    }
}

impl Hydrate for HydratedTopology {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        let LoroValue::I64(num) = value else {
            return Err(HydrateError::TypeMismatch {
                expected: "i64".into(),
                actual: format!("{value:?}").into(),
            });
        };

        let inner = match num {
            0 => PrimitiveTopology::PointList,
            1 => PrimitiveTopology::LineList,
            2 => PrimitiveTopology::LineStrip,
            3 => PrimitiveTopology::TriangleList,
            4 => PrimitiveTopology::TriangleStrip,
            other => {
                return Err(HydrateError::TypeMismatch {
                    expected: "valid topology".into(),
                    actual: format!("{other}").into(),
                });
            }
        };

        Ok(Self(inner))
    }
}

impl Reconcile for HydratedTopology {
    fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
        Err(ReconcileError::Custom(
            "HydratedTopology cannot be reconciled as a root container".into(),
        ))
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        map.insert(key, num)?;
        Ok(())
    }

    fn to_loro_value(&self) -> Option<LoroValue> {
        let num: i64 = match self.0 {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        Some(LoroValue::I64(num))
    }
}
