//! A user identity and the device it runs on.

use std::sync::Arc;

use iroh::SecretKey;
use iroh_docs::Author;
use unavi_local::DeviceStorage;
use xdid::{
    core::did::Did,
    method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

use crate::identity::keys::NodeKeys;

pub mod keys;
pub mod root_document;

/// A DID and the key that signs for it.
#[derive(Clone)]
pub struct Identity {
    did:         Did,
    signing_key: P256KeyPair,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("did", &self.did)
            .finish_non_exhaustive()
    }
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
    pub fn new(user: Identity, endpoint: SecretKey) -> Self {
        Self {
            user: Arc::new(user),
            endpoint,
        }
    }

    /// Loads this device's keys, answering as the `did:key` of its identity
    /// key.
    pub fn load(storage: &DeviceStorage) -> anyhow::Result<Self> {
        let keys = NodeKeys::load(storage)?;
        let did = keys.identity.public().to_did();
        Ok(Self::new(Identity::new(did, keys.identity), keys.endpoint))
    }

    /// Loads this device's keys, answering as `did`. Peers verify against
    /// `did`'s document, so it must list the identity key.
    pub fn load_as(storage: &DeviceStorage, did: Did) -> anyhow::Result<Self> {
        let keys = NodeKeys::load(storage)?;
        Ok(Self::new(Identity::new(did, keys.identity), keys.endpoint))
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
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        let user = Identity::new(did, key);
        let identity = NodeIdentity::new(user, SecretKey::generate());

        assert_eq!(
            identity.author().id().as_bytes(),
            identity.endpoint().public().as_bytes(),
            "one key backs both, so an entry's author names the endpoint that wrote it"
        );
    }
}
