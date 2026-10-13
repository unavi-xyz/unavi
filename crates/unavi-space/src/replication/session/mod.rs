//! Session opinions: one layer per peer per document, stacked above the
//! document's own layers and masked by the viewer.
//!
//! [`layers::SessionLayers`] keeps each peer's opinions and charges them to
//! that peer. [`compose`] stacks them into the documents they speak for. A
//! layer's mask, not admission, decides what it may contribute, so every
//! peer holds every present peer's layer and resolves them the same way.

use hsd::{
    attributes::portal::PortalAttr,
    id::PrimId,
    property::{
        Payload,
        Property,
        name::PropName,
        value::Value,
    },
    state::stack::{
        LayerId,
        Mask,
        MaskKey,
    },
};
use iroh::EndpointId;
use unavi_policy::{
    permissions::{
        HostApi,
        Permissions,
    },
    trust::Trust,
};

pub mod compose;
pub mod layers;
#[cfg(test)] mod tests;

/// Which key of which prim a session opinion is about: the same
/// `(prim, property)` key space the document uses.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct SessionKey {
    pub(crate) prim: PrimId,
    pub(crate) name: PropName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session property name is invalid or too long")]
    BadName,
    #[error("session write to a document present in another space")]
    WrongSpace,
    #[error("session write exceeds quota")]
    QuotaExceeded,
    /// The world the replica lives in is gone.
    #[error("session write failed: replica unavailable")]
    Unavailable,
}

/// Bytes one opinion costs its writer.
pub(crate) fn cell_bytes(key: &SessionKey, value: Option<&[u8]>) -> u64 {
    (key.name.as_str().len() + value.map_or(0, <[u8]>::len)) as u64
}

/// The stack layer holding `peer`'s session opinions.
#[must_use]
pub fn layer_id(peer: EndpointId) -> LayerId {
    LayerId::new(format!("session/{peer}"))
}

/// The group guest scripts keep their own session state under. It carries no
/// meaning to the engine.
const SCRIPT_STATE_GROUP: &str = "state";

/// Largest side of a portal a non-author's layer may state, in metres.
const MAX_PAIRED_PORTAL_SIDE: f64 = 16.0;

/// Whether a layer that is not the author's may contribute `value` at `name`.
///
/// Every other property is applied as if authored, so a non-author's layer
/// stating one would decide what the author's document runs, references or
/// simulates. A portal is admitted only as the far half of a pairing, a
/// linked destination of bounded size and never a tombstone, and only from a
/// writer whose rung opens portals.
fn admits_non_author(name: &PropName, value: Option<&Value>, pairs: bool) -> bool {
    if name.group() == SCRIPT_STATE_GROUP {
        return true;
    }
    if !pairs || *name != PortalAttr::NAME {
        return false;
    }
    let Some(Ok(portal)) = value
        .and_then(Value::as_attribute)
        .map(|payload| <PortalAttr as Payload>::decode(payload))
    else {
        return false;
    };
    let bounded = |side: f64| side.is_finite() && side > 0.0 && side <= MAX_PAIRED_PORTAL_SIDE;
    portal.destination.is_some_and(|dest| dest.link.is_some())
        && bounded(portal.size_x)
        && bounded(portal.size_y)
}

/// What a peer's layer may contribute: everything when it is the author's,
/// nothing from a peer granted no host API, and the open class otherwise.
#[must_use]
pub fn mask(is_author: bool, permissions: Permissions) -> Mask {
    if is_author {
        return Mask::all();
    }
    if permissions == Permissions::none() {
        return Mask::new(|_, _| false);
    }
    let pairs = permissions.require(HostApi::Portal).is_ok();
    Mask::new(move |_, key| match key {
        MaskKey::Parent(_) => false,
        MaskKey::Property(name, value) => admits_non_author(name, value, pairs),
    })
}

/// One peer's layer as the stack builder sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub peer:      EndpointId,
    pub is_author: bool,
    pub trust:     Trust,
}

/// Orders session layers weakest first: other peers by rung, then the
/// author's devices, each group by peer id. Every viewer holding the same
/// facts picks the same winner, and no clock outvotes the author.
pub fn order(layers: &mut [Standing]) {
    layers.sort_unstable_by_key(|layer| (layer.is_author, layer.trust, layer.peer));
}
