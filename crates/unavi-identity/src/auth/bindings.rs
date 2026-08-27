use std::collections::HashMap;

use iroh::EndpointId;
use parking_lot::RwLock;
use xdid::core::did::Did;

/// Maps a peer's endpoint id to the DID it has proven it controls.
///
/// Only a completed `wired/auth` handshake over that peer's own connection may
/// write here. A DID announced elsewhere is a claim anyone can make about
/// anyone, so an unbound peer is indistinguishable from an anonymous one.
#[derive(Debug, Default)]
pub struct Bindings(RwLock<HashMap<EndpointId, Did>>);

impl Bindings {
    pub fn bind(&self, peer: EndpointId, did: Did) {
        self.0.write().insert(peer, did);
    }

    pub fn unbind(&self, peer: EndpointId) {
        self.0.write().remove(&peer);
    }

    #[must_use]
    pub fn did_of(&self, peer: EndpointId) -> Option<Did> {
        self.0.read().get(&peer).cloned()
    }

    #[must_use]
    pub fn is_bound(&self, peer: EndpointId) -> bool {
        self.0.read().contains_key(&peer)
    }
}
