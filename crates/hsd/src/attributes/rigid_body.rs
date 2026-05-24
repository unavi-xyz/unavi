use lorosurgeon::{Hydrate, MaybeMissing, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyKind {
    #[default]
    Dynamic,
    Kinematic,
    Static,
}

#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct RigidBodyAttr {
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub angular_damping: MaybeMissing<f64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub friction: MaybeMissing<f64>,
    pub kind: RigidBodyKind,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub linear_damping: MaybeMissing<f64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub mass: MaybeMissing<f64>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub restitution: MaybeMissing<f64>,
}

impl Attribute for RigidBodyAttr {
    const KEY: &str = "rigid_body";
}
