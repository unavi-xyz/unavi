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

pub const GROUP: &str = "portal";

/// Shared by the two halves of a two-way portal.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct LinkId(pub [u8; 16]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalDestination {
    pub space: [u8; 32],
    /// Pairs this portal with the one in `space` bearing the same link. A
    /// destination without one is a one-way gateway.
    pub link:  Option<LinkId>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PortalAttr {
    pub destination: Option<PortalDestination>,
    pub size_x:      f64,
    pub size_y:      f64,
}

impl Property for PortalAttr {
    const NAME: PropName = prop_name!(GROUP);
}
