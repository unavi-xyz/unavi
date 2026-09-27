use crate::{
    id::PrimId,
    property::{
        name::PropName,
        value::Value,
    },
};

/// A change to the realized scene. `Realized` is followed by one `Property`
/// event per property the prim already holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneEvent {
    /// `parent` is `None` for a root.
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
        name:  PropName,
        value: Option<Value>,
    },
}
