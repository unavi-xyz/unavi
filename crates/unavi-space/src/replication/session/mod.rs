//! Session opinions: one layer per writer per document, stacked above the
//! document's own layers and masked by the viewer.
//!
//! A writer is one device speaking for one DID. [`layers::SessionLayers`]
//! keeps each writer's [`cells::Cells`], the signed statement of them, and
//! the peers that carry them, and charges them to the writer. [`compose`]
//! stacks them into the documents they speak for. A layer's mask, not
//! admission, decides what it may contribute, so every peer holds every
//! present writer's layer and resolves them the same way.

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
use xdid::core::did::Did;

pub mod cells;
pub mod compose;
pub mod layers;
#[cfg(test)] mod tests;

/// Which key of which prim a session opinion is about: the same
/// `(prim, property)` key space the document uses.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SessionKey {
    pub prim: PrimId,
    pub name: PropName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session property name is invalid or too long")]
    BadName,
    #[error("session tick carries too many writes or bytes")]
    TooLarge,
    #[error("session write to a document present in another space")]
    WrongSpace,
    #[error("session state is no newer than the one held")]
    Stale,
    #[error("session write exceeds quota")]
    QuotaExceeded,
    #[error("session state could not be signed")]
    Unsigned,
    /// The world the replica lives in is gone.
    #[error("session write failed: replica unavailable")]
    Unavailable,
}

/// A session layer's writer: the device writing it, and the DID signing it.
///
/// A relayed layer's device is only the signer's claim, so a forged one is a
/// layer of the forger's own and never displaces the device's real one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct WriterId {
    pub device: EndpointId,
    pub did:    DidHash,
}

impl WriterId {
    #[must_use]
    pub fn new(device: EndpointId, did: &Did) -> Self {
        Self {
            device,
            did: DidHash::of(did),
        }
    }
}

/// A DID's blake3 hash, a fixed-size key for one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DidHash([u8; 32]);

impl DidHash {
    #[must_use]
    pub fn of(did: &Did) -> Self {
        Self(*blake3::hash(did.to_string().as_bytes()).as_bytes())
    }
}

/// Every session layer's id starts with this.
pub const LAYER_PREFIX: &str = "session/";

/// The stack layer holding `writer`'s session opinions.
#[must_use]
pub fn layer_id(writer: WriterId) -> LayerId {
    let did = blake3::Hash::from_bytes(writer.did.0).to_hex();
    LayerId::new(format!("{LAYER_PREFIX}{}/{}", writer.device, &did[..32]))
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
    pub writer:      WriterId,
    pub is_author:   bool,
    pub trust:       Trust,
    /// What the rules grant at `trust`, so a rules change re-masks too.
    pub permissions: Permissions,
}

/// Orders session layers weakest first: other writers by rung, then the
/// author's devices, each group by writer.
///
/// Every viewer holding the same facts picks the same winner, and no clock
/// outvotes the author.
pub fn order(layers: &mut [Standing]) {
    layers.sort_unstable_by_key(|layer| (layer.is_author, layer.trust, layer.writer));
}
