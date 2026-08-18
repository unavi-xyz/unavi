use std::{
    collections::HashMap,
    sync::LazyLock,
};

use parking_lot::RwLock;
use xdid::core::did::Did;

/// Maps a peer's endpoint id to the DID it has proven it controls.
///
/// Only a completed challenge-response over that peer's own connection may
/// write here. A DID announced elsewhere is a claim anyone can make about
/// anyone, so an unbound peer is indistinguishable from an anonymous one.
static BINDINGS: LazyLock<RwLock<HashMap<[u8; 32], Did>>> = LazyLock::new(RwLock::default);

/// The local user's own DID, which is the ego node every trust score is
/// measured from.
static SELF: RwLock<Option<Did>> = RwLock::new(None);

pub fn set_self(did: Did) {
    *SELF.write() = Some(did);
}

#[must_use]
pub fn self_did() -> Option<Did> {
    SELF.read().clone()
}

pub fn bind(peer: [u8; 32], did: Did) {
    BINDINGS.write().insert(peer, did);
}

pub fn unbind(peer: [u8; 32]) {
    BINDINGS.write().remove(&peer);
}

#[must_use]
pub fn did_of(peer: [u8; 32]) -> Option<Did> {
    BINDINGS.read().get(&peer).cloned()
}
