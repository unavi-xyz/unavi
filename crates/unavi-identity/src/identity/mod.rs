use std::sync::Arc;

use iroh::SecretKey;
use iroh_docs::Author;
use parking_lot::RwLock;
use unavi_store::local::Storage;
use xdid::{
    core::did::Did,
    methods::key::keys::{
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

/// The identity this process acts as, readable from any task.
///
/// Answering an identity proof happens on a background task with no path back
/// to whatever loaded the key, and the DID is the ego node every trust score is
/// measured from.
static LOCAL: RwLock<Option<Arc<Identity>>> = RwLock::new(None);

pub fn set_local(identity: Arc<Identity>) {
    *LOCAL.write() = Some(identity);
}

#[must_use]
pub fn local() -> Option<Arc<Identity>> {
    LOCAL.read().clone()
}

#[must_use]
pub fn local_did() -> Option<Did> {
    LOCAL.read().as_ref().map(|i| i.did().clone())
}

/// One node: a user identity plus the iroh endpoint and local state of the
/// device it runs on.
///
/// A user's several devices share their [`Identity`] but each carries its own
/// endpoint key, so discovery maps every device to its own address set.
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

    /// Loads the user key and endpoint key from `storage`, generating and
    /// saving either if absent.
    pub fn load(storage: &Storage) -> anyhow::Result<Self> {
        Ok(Self::new(
            store::load_or_create(storage)?,
            store::load_or_create_endpoint(storage)?,
        ))
    }

    /// The user this node acts as, for anything attributed to the DID.
    #[must_use]
    pub const fn user(&self) -> &Arc<Identity> {
        &self.user
    }

    /// The iroh endpoint key. Its public half is this device's `EndpointId`,
    /// and equals [`Self::author`]'s id.
    #[must_use]
    pub const fn endpoint(&self) -> &SecretKey {
        &self.endpoint
    }

    /// The docs author this device writes entries under.
    ///
    /// One key backs both this and [`Self::endpoint`], so an entry's author
    /// names the endpoint that wrote it.
    #[must_use]
    pub fn author(&self) -> Author {
        Author::from_bytes(&self.endpoint.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(signing_key: P256KeyPair, endpoint: SecretKey) -> NodeIdentity {
        NodeIdentity::new(signing_key, endpoint)
    }

    fn same_key_as(pair: &P256KeyPair) -> P256KeyPair {
        let pem = pair.to_pkcs8_pem().expect("encode");
        P256KeyPair::from_pkcs8_pem(&pem).expect("decode")
    }

    #[test]
    fn a_device_keeps_its_keys_across_loads() {
        let pair = P256KeyPair::generate();
        let endpoint = SecretKey::generate();

        let first = node(same_key_as(&pair), endpoint.clone());
        let second = node(pair, endpoint);

        assert_eq!(first.user().did(), second.user().did());
        assert_eq!(first.endpoint().public(), second.endpoint().public());
        assert_eq!(first.author().id(), second.author().id());
    }

    #[test]
    fn author_id_is_the_endpoint_id() {
        let identity = node(P256KeyPair::generate(), SecretKey::generate());

        assert_eq!(
            identity.author().id().as_bytes(),
            identity.endpoint().public().as_bytes(),
            "one key backs both, so an entry's author names the endpoint that wrote it"
        );
    }

    #[test]
    fn one_identity_on_two_devices_is_two_endpoints() {
        let pair = P256KeyPair::generate();

        let phone = node(same_key_as(&pair), SecretKey::generate());
        let headset = node(pair, SecretKey::generate());

        assert_eq!(
            phone.user().did(),
            headset.user().did(),
            "one person, one DID"
        );
        assert_ne!(
            phone.endpoint().public(),
            headset.endpoint().public(),
            "discovery maps an endpoint id to addresses, so two live devices \
             sharing one would publish a merged address set"
        );
        assert_ne!(phone.author().id(), headset.author().id());
    }

    #[test]
    fn a_rotated_endpoint_leaves_the_did_alone() {
        let pair = P256KeyPair::generate();

        let before = node(same_key_as(&pair), SecretKey::generate());
        let after = node(pair, SecretKey::generate());

        assert_eq!(before.user().did(), after.user().did());
        assert_ne!(before.endpoint().public(), after.endpoint().public());
    }

    #[test]
    fn separate_keys_are_separate_identities() {
        let a = node(P256KeyPair::generate(), SecretKey::generate());
        let b = node(P256KeyPair::generate(), SecretKey::generate());

        assert_ne!(a.user().did(), b.user().did());
        assert_ne!(a.endpoint().public(), b.endpoint().public());
    }
}
