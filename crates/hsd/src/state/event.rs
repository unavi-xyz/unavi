use smol_str::SmolStr;

use crate::{
    id::PrimId,
    property::Property,
};

/// Describes the *realized* scene only.
///
/// A prim whose parent has not arrived produces nothing until it does, at
/// which point `Realized` is followed by one event per property and slot it
/// already holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneEvent {
    /// `parent` is `None` for a root of the document.
    Realized {
        prim:   PrimId,
        parent: Option<PrimId>,
    },
    Reparented {
        prim:   PrimId,
        parent: Option<PrimId>,
    },
    Unrealized {
        prim: PrimId,
    },
    Property {
        prim:  PrimId,
        name:  SmolStr,
        value: Option<Property>,
    },
    Slot {
        prim:  PrimId,
        name:  SmolStr,
        value: Option<Vec<u8>>,
    },
}
