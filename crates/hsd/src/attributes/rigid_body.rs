use lorosurgeon::{Hydrate, MaybeMissing, Reconcile};

use crate::attributes::Attribute;

/// kind: 0 = Dynamic, 1 = Static, 2 = Kinematic
#[derive(Hydrate, Reconcile, Debug, Clone, Default)]
#[loro(default)]
pub struct RigidBodyAttr {
    pub angular_damping: MaybeMissing<f64>,
    pub friction: MaybeMissing<f64>,
    pub kind: i64,
    pub linear_damping: MaybeMissing<f64>,
    pub mass: MaybeMissing<f64>,
    pub restitution: MaybeMissing<f64>,
}

impl Attribute for RigidBodyAttr {
    const KEY: &str = "rigid_body";
}
