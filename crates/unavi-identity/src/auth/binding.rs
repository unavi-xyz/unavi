use std::{
    collections::HashMap,
    sync::LazyLock,
};

use iroh::EndpointId;
use parking_lot::RwLock;
use xdid::core::did::Did;

/// Maps a peer's endpoint id to the DID it has proven it controls.
///
/// Only a completed `wired/auth` handshake over that peer's own connection may
/// write here. A DID announced elsewhere is a claim anyone can make about
/// anyone, so an unbound peer is indistinguishable from an anonymous one.
static BINDINGS: LazyLock<RwLock<HashMap<[u8; 32], Did>>> = LazyLock::new(RwLock::default);

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

#[must_use]
pub fn did_of_endpoint(peer: EndpointId) -> Option<Did> {
    did_of(*peer.as_bytes())
}

#[must_use]
pub fn is_bound(peer: EndpointId) -> bool {
    BINDINGS.read().contains_key(peer.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn a_peer_is_anonymous_until_it_proves_a_did() {
        let peer = [9u8; 32];
        assert!(did_of(peer).is_none());

        let did = Did::from_str("did:web:example.com").expect("did");
        bind(peer, did.clone());
        assert_eq!(did_of(peer), Some(did));

        unbind(peer);
        assert!(
            did_of(peer).is_none(),
            "a dropped connection proves nothing"
        );
    }
}
