use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Property,
        name::PropName,
    },
};

pub const GROUP: &str = "rigid_body";

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

impl Property for RigidBodyAttr {
    const NAME: PropName = prop_name!(GROUP);
}
