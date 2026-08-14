use wired_math::types::Vec3;

use crate::{
    cast::Cast,
    tree::Mote,
};

/// Where a carried mote was let go, in the frame a document's offset is
/// measured in.
#[derive(Debug, Clone, Copy)]
pub struct Landing {
    pub at:       Vec3,
    pub velocity: Vec3,
}

/// What a surface did, in the terms a consumer thinks in.
pub enum Event {
    /// A container opened; the level inside it is drawn now.
    Opened(Mote),
    /// The way back was taken, to the level now open.
    Closed(Mote),
    /// A leaf fired.
    Activated(Mote),
    /// A cast site opened. Nothing has fired: holding attention on it fills
    /// it, and looking away aborts.
    Casting(Mote),
    /// A cast was held to the end.
    Cast(Mote),
    /// A cast was abandoned before it filled.
    Aborted(Mote),
    /// Carried out and let go in the room. What lands there, if anything, is
    /// the consumer's to place.
    Planted(Mote, Landing),
    /// Carried out and filed into a grid.
    Filed(Mote),
    /// The page turned. `total` is how many motes the level holds in all.
    Paged {
        index: usize,
        count: usize,
        total: usize,
    },
}

/// A carried mote released into the world, for the host to route.
pub struct Released {
    pub mote:    Mote,
    pub landing: Landing,
}

/// What a shape's fixed step resolved, for the host to finish routing.
pub struct FixedUpdate {
    pub events:   Vec<Event>,
    pub released: Option<Released>,
}

/// A consequential action, mid-cast. Shared by every shape that can show one.
pub(crate) struct Casting {
    pub slot: usize,
    pub mote: Mote,
    pub cast: Cast,
}
