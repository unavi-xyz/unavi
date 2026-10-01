use crate::{
    id::PrimId,
    property::{
        name::PropName,
        value::Value,
    },
};

/// A change to the scene. `Added` is followed by one `Property`
/// event per property the prim already holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneEvent {
    /// `parent` is `None` for a root.
    Added {
        prim:   PrimId,
        parent: Option<PrimId>,
    },
    Reparented {
        prim:   PrimId,
        parent: Option<PrimId>,
    },
    Removed {
        prim: PrimId,
    },
    Property {
        prim:  PrimId,
        name:  PropName,
        value: Option<Value>,
    },
}
