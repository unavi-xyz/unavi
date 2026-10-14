//! Session opinions: one layer per writer per document, stacked above the
//! document's own layers and masked by the viewer.
//!
//! [`layers::SessionLayers`] keeps each writer's opinions, the signed batches
//! that state them and the peers that carry them, and charges them to the
//! writer. [`compose`] stacks them into the documents they speak for. A
//! layer's mask, not admission, decides what it may contribute, so every
//! peer holds every present writer's layer and resolves them the same way.

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
use tracing::debug;
use unavi_policy::{
    permissions::{
        HostApi,
        Permissions,
    },
    trust::Trust,
};

use crate::replication::message::SESSION_NAME_MAX_BYTES;

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

impl SessionKey {
    /// A session name is a property name, so it answers to the same
    /// key-layout rule every document property does, plus a length the store
    /// will accept.
    pub(crate) fn new(prim: PrimId, name: &str) -> Result<Self, SessionError> {
        if name.len() > SESSION_NAME_MAX_BYTES {
            return Err(SessionError::BadName);
        }
        let name = name.parse().map_err(|_| SessionError::BadName)?;
        Ok(Self { prim, name })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session property name is invalid or too long")]
    BadName,
    #[error("session batch carries too many writes")]
    TooLarge,
    #[error("session write to a document present in another space")]
    WrongSpace,
    #[error("session layer is held under another DID")]
    WrongDid,
    #[error("session write exceeds quota")]
    QuotaExceeded,
    #[error("session batch could not be signed")]
    Unsigned,
    /// The world the replica lives in is gone.
    #[error("session write failed: replica unavailable")]
    Unavailable,
}

/// Bytes one opinion costs its writer.
pub(crate) fn cell_bytes(key: &SessionKey, value: Option<&[u8]>) -> u64 {
    (key.name.as_str().len() + value.map_or(0, <[u8]>::len)) as u64
}

/// Every session layer's id starts with this.
pub const LAYER_PREFIX: &str = "session/";

/// The stack layer holding `writer`'s session opinions.
#[must_use]
pub fn layer_id(writer: EndpointId) -> LayerId {
    LayerId::new(format!("{LAYER_PREFIX}{writer}"))
}

/// The group guest scripts keep their own session state under. It carries no
/// meaning to the engine.
const SCRIPT_STATE_GROUP: &str = "state";

fn decode_portal(value: Option<&Value>) -> Option<PortalAttr> {
    value
        .and_then(Value::as_attribute)
        .and_then(|payload| <PortalAttr as Payload>::decode(payload).ok())
}

/// Whether a layer that is not the author's may contribute `value` at `name`,
/// over `beneath`, what the layers below it resolve the key to.
///
/// Every other property is applied as if authored, so a non-author's layer
/// stating one would decide what the author's document runs, references or
/// simulates. A portal is admitted only as the far half of a pairing: a linked
/// destination over a portal the layers beneath leave with none, at the same
/// size, from a writer whose rung opens portals. It never redirects, resizes
/// or adds a portal.
fn admits_non_author(
    name: &PropName,
    value: Option<&Value>,
    beneath: Option<&Value>,
    pairs: bool,
) -> bool {
    if name.group() == SCRIPT_STATE_GROUP && name.field().is_some() {
        return true;
    }
    if *name != PortalAttr::NAME {
        return false;
    }
    if !pairs {
        debug!("session portal refused: the writer's rung opens no portals");
        return false;
    }
    let (Some(pairing), Some(frame)) = (decode_portal(value), decode_portal(beneath)) else {
        debug!("session portal refused: no pairing over an authored frame");
        return false;
    };
    let admitted = frame.destination.is_none()
        && pairing.destination.is_some_and(|dest| dest.link.is_some())
        && pairing.size_x.to_bits() == frame.size_x.to_bits()
        && pairing.size_y.to_bits() == frame.size_y.to_bits();
    if !admitted {
        debug!(
            frame_leads = frame.destination.is_some(),
            paired = pairing.destination.is_some(),
            pairing_size = ?(pairing.size_x, pairing.size_y),
            frame_size = ?(frame.size_x, frame.size_y),
            "session portal refused by the pairing rule"
        );
    }
    admitted
}

/// What a peer's layer may contribute: nothing from a peer granted no host
/// API, everything when it is the author's, and the open class otherwise.
#[must_use]
pub fn mask(is_author: bool, permissions: Permissions) -> Mask {
    if permissions == Permissions::none() {
        return Mask::new(|_, _| false);
    }
    if is_author {
        return Mask::all();
    }
    let pairs = permissions.require(HostApi::Portal).is_ok();
    Mask::new(move |_, key| match key {
        MaskKey::Parent(_) => false,
        MaskKey::Property {
            name,
            value,
            beneath,
        } => admits_non_author(name, value, beneath, pairs),
    })
}

/// One writer's layer as the stack builder sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub peer:        EndpointId,
    pub is_author:   bool,
    pub trust:       Trust,
    /// What the rules grant at `trust`, so a rules change re-masks too.
    pub permissions: Permissions,
}

/// Orders session layers weakest first: other peers by rung, then the
/// author's devices, each group by peer id. Every viewer holding the same
/// facts picks the same winner, and no clock outvotes the author.
pub fn order(layers: &mut [Standing]) {
    layers.sort_unstable_by_key(|layer| (layer.is_author, layer.trust, layer.peer));
}
