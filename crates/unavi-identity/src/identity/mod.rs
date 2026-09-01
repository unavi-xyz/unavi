use std::sync::Arc;

use iroh::SecretKey;
use iroh_docs::Author;
use unavi_store::local::Storage;
use xdid::{
    core::did::Did,
    method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

pub mod store;

#[derive(Clone)]
pub struct Identity {
    did:         Did,
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

/// A user identity paired with the endpoint key of the device it runs on.
///
/// Every device a user owns shares their [`Identity`] but carries its own
/// endpoint key, so discovery maps each device to its own address set.
pub struct NodeIdentity {
    user:     Arc<Identity>,
    endpoint: SecretKey,
}

impl NodeIdentity {
    #[must_use]
    pub fn new(signing_key: P256KeyPair, endpoint: SecretKey) -> Self {
        let did = signing_key.public().to_did();

        Self {
            user: Arc::new(Identity::new(did, signing_key)),
            endpoint,
        }
    }

    pub fn load(storage: &Storage) -> anyhow::Result<Self> {
        let keys = store::load(storage)?;
        Ok(Self::new(keys.identity, keys.endpoint))
    }

    #[must_use]
    pub const fn user(&self) -> &Arc<Identity> {
        &self.user
    }

    #[must_use]
    pub const fn endpoint(&self) -> &SecretKey {
        &self.endpoint
    }

    /// The docs author this device writes entries under. One key backs both
    /// this and [`Self::endpoint`], so an entry's author names the endpoint
    /// that wrote it.
    #[must_use]
    pub fn author(&self) -> Author {
        Author::from_bytes(&self.endpoint.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_id_is_the_endpoint_id() {
        let identity = NodeIdentity::new(P256KeyPair::generate(), SecretKey::generate());

        assert_eq!(
            identity.author().id().as_bytes(),
            identity.endpoint().public().as_bytes(),
            "one key backs both, so an entry's author names the endpoint that wrote it"
        );
    }
}
