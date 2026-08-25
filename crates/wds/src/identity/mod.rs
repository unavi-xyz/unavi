use iroh::SecretKey;
use iroh_docs::Author;
use xdid::{
    core::did::Did,
    methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

pub mod labels;
pub mod store;

use store::DeviceSeed;

/// User identity for WDS operations: the DID and signing key used to
/// authenticate with WDS hosts.
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

/// The two secrets a node persists, and where it keeps them.
///
/// The P-256 key is the person. A `did:key` names it directly today; a
/// `did:web` document can later name it as an authorized key without any key
/// material changing. It signs, and nothing derives from it.
///
/// The device seed is the machine. The endpoint key and the docs author come
/// from it and never from the identity, because both name *where* rather than
/// *who*: one person's devices must be separately addressable, and an entry's
/// author should say which of them wrote it.
pub struct RootIdentity {
    device:      DeviceSeed,
    did:         Did,
    signing_key: P256KeyPair,
    storage:     store::KeyStorage,
}

impl RootIdentity {
    pub fn new(
        signing_key: P256KeyPair,
        device: DeviceSeed,
        storage: store::KeyStorage,
    ) -> anyhow::Result<Self> {
        let did = signing_key.public().to_did();

        Ok(Self {
            device,
            did,
            signing_key,
            storage,
        })
    }

    /// Loads the node's identity key and device seed from `storage`,
    /// generating and saving either if absent.
    pub fn load(storage: &store::KeyStorage) -> anyhow::Result<Self> {
        Self::new(
            store::load_or_create(storage)?,
            store::load_or_create_seed(storage)?,
            storage.clone(),
        )
    }

    /// Where this node's local state lives, for the namespace ids it minted.
    #[must_use]
    pub const fn storage(&self) -> &store::KeyStorage {
        &self.storage
    }

    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }

    /// The key the DID names, used to sign anything attributed to this node.
    #[must_use]
    pub const fn signing_key(&self) -> &P256KeyPair {
        &self.signing_key
    }

    /// The iroh endpoint key. Its public half is this device's `EndpointId`,
    /// and equals [`Self::author`]'s id.
    #[must_use]
    pub fn endpoint_key(&self) -> SecretKey {
        SecretKey::from_bytes(self.device.as_bytes())
    }

    /// The docs author this device writes entries under.
    ///
    /// One seed backs both this and the endpoint key, so an entry's author
    /// names the endpoint that wrote it.
    #[must_use]
    pub fn author(&self) -> Author {
        Author::from_bytes(self.device.as_bytes())
    }

    /// The control-plane identity, for authenticating to WDS hosts.
    #[must_use]
    pub fn identity(&self) -> Identity {
        Identity::new(self.did.clone(), self.signing_key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(signing_key: P256KeyPair, seed: DeviceSeed) -> RootIdentity {
        RootIdentity::new(signing_key, seed, store::KeyStorage::Ephemeral).expect("load")
    }

    fn same_key_as(pair: &P256KeyPair) -> P256KeyPair {
        let pem = pair.to_pkcs8_pem().expect("encode");
        P256KeyPair::from_pkcs8_pem(&pem).expect("decode")
    }

    #[test]
    fn a_device_keeps_its_keys_across_loads() {
        let pair = P256KeyPair::generate();
        let seed = DeviceSeed::generate();

        let first = device(same_key_as(&pair), seed.clone());
        let second = device(pair, seed);

        assert_eq!(first.did(), second.did());
        assert_eq!(
            first.endpoint_key().public(),
            second.endpoint_key().public()
        );
        assert_eq!(first.author().id(), second.author().id());
    }

    #[test]
    fn author_id_is_the_endpoint_id() {
        let identity = device(P256KeyPair::generate(), DeviceSeed::generate());

        assert_eq!(
            identity.author().id().as_bytes(),
            identity.endpoint_key().public().as_bytes(),
            "one device seed must back both, so an entry's author names the endpoint that wrote it"
        );
    }

    #[test]
    fn one_identity_on_two_devices_is_two_endpoints() {
        let pair = P256KeyPair::generate();

        let phone = device(same_key_as(&pair), DeviceSeed::generate());
        let headset = device(pair, DeviceSeed::generate());

        assert_eq!(phone.did(), headset.did(), "one person, one DID");
        assert_ne!(
            phone.endpoint_key().public(),
            headset.endpoint_key().public(),
            "discovery maps an endpoint id to addresses, so two live devices \
             sharing one would publish a merged address set"
        );
        assert_ne!(phone.author().id(), headset.author().id());
    }

    #[test]
    fn a_rotated_seed_leaves_the_did_alone() {
        let pair = P256KeyPair::generate();

        let before = device(same_key_as(&pair), DeviceSeed::generate());
        let after = device(pair, DeviceSeed::generate());

        assert_eq!(before.did(), after.did());
        assert_ne!(
            before.endpoint_key().public(),
            after.endpoint_key().public()
        );
    }

    #[test]
    fn separate_keys_are_separate_identities() {
        let a = device(P256KeyPair::generate(), DeviceSeed::generate());
        let b = device(P256KeyPair::generate(), DeviceSeed::generate());

        assert_ne!(a.did(), b.did());
        assert_ne!(a.endpoint_key().public(), b.endpoint_key().public());
    }
}
