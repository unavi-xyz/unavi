use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

/// User identity for WDS operations.
///
/// Contains the DID and signing key used to authenticate with WDS hosts.
/// Shared across multiple [`Actor`](crate::actor::Actor) instances via `Arc`.
#[derive(Clone)]
pub struct Identity {
    did: Did,
    signing_key: P256KeyPair,
}

impl Identity {
    #[must_use]
    pub const fn new(did: Did, signing_key: P256KeyPair) -> Self {
        Self { did, signing_key }
    }

    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    #[must_use]
    pub const fn signing_key(&self) -> &P256KeyPair {
        &self.signing_key
    }
}
