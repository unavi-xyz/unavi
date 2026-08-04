use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyKind {
    Dynamic,
    Kinematic,
    #[default]
    Static,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct RigidBodyAttr {
    pub angular_damping: Option<f64>,
    pub friction:        Option<f64>,
    pub kind:            Option<RigidBodyKind>,
    pub linear_damping:  Option<f64>,
    pub mass:            Option<f64>,
    pub restitution:     Option<f64>,
}

impl Attribute for RigidBodyAttr {
    const KEY: &'static str = "rigid_body";
}
