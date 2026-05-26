use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyKind {
    Dynamic,
    Kinematic,
    Static,
}

#[serde_with::skip_serializing_none]
#[derive(Hydrate, Reconcile, Debug, Clone, Default, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct RigidBodyAttr {
    pub angular_damping: Option<f64>,
    pub friction: Option<f64>,
    pub kind: Option<RigidBodyKind>,
    pub linear_damping: Option<f64>,
    pub mass: Option<f64>,
    pub restitution: Option<f64>,
}

impl Attribute for RigidBodyAttr {
    const KEY: &str = "rigid_body";
}
