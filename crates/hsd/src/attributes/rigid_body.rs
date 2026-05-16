use lorosurgeon::{Hydrate, MaybeMissing, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug, Clone, Default, PartialEq)]
pub enum RigidBodyKind {
    #[default]
    Dynamic,
    Kinematic,
    Static,
}

#[derive(Hydrate, Reconcile, Debug, Clone, Default)]
#[loro(default)]
pub struct RigidBodyAttr {
    pub angular_damping: MaybeMissing<f64>,
    pub friction: MaybeMissing<f64>,
    pub kind: RigidBodyKind,
    pub linear_damping: MaybeMissing<f64>,
    pub mass: MaybeMissing<f64>,
    pub restitution: MaybeMissing<f64>,
}

impl Attribute for RigidBodyAttr {
    const KEY: &str = "rigid_body";
}
