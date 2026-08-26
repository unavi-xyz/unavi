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

    /// For callers holding a peer id that was never parsed into an
    /// [`EndpointId`]. Parsing one decompresses a curve point, which this
    /// avoids on paths that run per write.
    #[must_use]
    pub fn did_of_bytes(&self, peer: &[u8; 32]) -> Option<Did> {
        self.0.read().get(peer).cloned()
    }

    #[must_use]
    pub fn is_bound(&self, peer: EndpointId) -> bool {
        self.0.read().contains_key(&peer)
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    #[test]
    fn a_raw_lookup_finds_a_typed_key() {
        let peer = SecretKey::generate().public();
        let did = "did:web:example.com".parse::<Did>().expect("did");

        let bindings = Bindings::default();
        bindings.bind(peer, did.clone());

        assert_eq!(
            bindings.did_of_bytes(peer.as_bytes()),
            Some(did),
            "`EndpointId` must hash identically to its bytes, or `did_of_bytes` \
             silently finds nothing"
        );
    }
}
