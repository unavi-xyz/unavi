use anyhow::Context;
use iroh::SecretKey;
use iroh_docs::{
    Author,
    NamespaceSecret,
};
use p256::pkcs8::DecodePrivateKey;
use xdid::{
    core::did::Did,
    methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};
use zeroize::Zeroizing;

pub mod derive;
pub mod labels;
pub mod store;

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

/// The one secret a node persists, and every key derived from it.
///
/// The P-256 key is the root. A `did:key` names it directly today; a `did:web`
/// document can later name it as an authorized key without any key material
/// changing. The iroh endpoint key, the docs author and every well-known
/// namespace derive from it, so reloading the key reloads the node's whole
/// addressable identity rather than just its name.
pub struct RootIdentity {
    did:            Did,
    ed25519:        Zeroizing<[u8; 32]>,
    namespace_root: Zeroizing<[u8; 32]>,
    signing_key:    P256KeyPair,
}

impl RootIdentity {
    /// Derives every subordinate key from `signing_key`.
    pub fn new(signing_key: P256KeyPair) -> anyhow::Result<Self> {
        let did = signing_key.public().to_did();
        let scalar = scalar_bytes(&signing_key)?;

        Ok(Self {
            did,
            ed25519: derive::ed25519_seed(scalar.as_slice()),
            namespace_root: derive::namespace_root(scalar.as_slice()),
            signing_key,
        })
    }

    /// Loads the node's key from `storage`, generating and saving one if
    /// absent.
    pub fn load(storage: &store::KeyStorage) -> anyhow::Result<Self> {
        Self::new(store::load_or_create(storage)?)
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

    /// The iroh endpoint key. Its public half is this node's `EndpointId`, and
    /// equals [`Self::author`]'s id.
    #[must_use]
    pub fn endpoint_key(&self) -> SecretKey {
        SecretKey::from_bytes(&self.ed25519)
    }

    /// The docs author this node writes entries under.
    #[must_use]
    pub fn author(&self) -> Author {
        Author::from_bytes(&self.ed25519)
    }

    /// The write capability for the well-known namespace named by `label`.
    ///
    /// Labels must come from [`labels`], never from a peer: a namespace derived
    /// from an attacker-chosen label is a namespace they can predict.
    #[must_use]
    pub fn namespace(&self, label: &str) -> NamespaceSecret {
        derive::namespace(&self.namespace_root, label)
    }

    /// The control-plane identity, for authenticating to WDS hosts.
    #[must_use]
    pub fn identity(&self) -> Identity {
        Identity::new(self.did.clone(), self.signing_key.clone())
    }
}

/// The raw secret scalar, read back through PKCS#8 because [`P256KeyPair`]
/// exposes no accessor for it.
///
/// Deriving from the scalar rather than from its encoding keeps every derived
/// key stable if the PKCS#8 writer ever changes what it emits.
fn scalar_bytes(pair: &P256KeyPair) -> anyhow::Result<Zeroizing<p256::FieldBytes>> {
    let pem = pair.to_pkcs8_pem().context("encode identity key")?;
    let secret = p256::SecretKey::from_pkcs8_pem(&pem).context("decode identity key")?;
    Ok(Zeroizing::new(secret.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subordinate_keys_are_stable_across_loads() {
        let pair = P256KeyPair::generate();
        let pem = pair.to_pkcs8_pem().expect("encode");

        let first = RootIdentity::new(pair).expect("derive");
        let second =
            RootIdentity::new(P256KeyPair::from_pkcs8_pem(&pem).expect("decode")).expect("derive");

        assert_eq!(first.did(), second.did());
        assert_eq!(
            first.endpoint_key().public(),
            second.endpoint_key().public()
        );
        assert_eq!(first.author().id(), second.author().id());
        assert_eq!(
            first.namespace(labels::ROOT_DOC).to_bytes(),
            second.namespace(labels::ROOT_DOC).to_bytes()
        );
    }

    #[test]
    fn author_id_is_the_endpoint_id() {
        let identity = RootIdentity::new(P256KeyPair::generate()).expect("derive");

        assert_eq!(
            identity.author().id().as_bytes(),
            identity.endpoint_key().public().as_bytes(),
            "one ed25519 seed must back both, so a peer's author names its endpoint"
        );
    }

    #[test]
    fn separate_keys_derive_separate_identities() {
        let a = RootIdentity::new(P256KeyPair::generate()).expect("derive");
        let b = RootIdentity::new(P256KeyPair::generate()).expect("derive");

        assert_ne!(a.did(), b.did());
        assert_ne!(a.endpoint_key().public(), b.endpoint_key().public());
        assert_ne!(
            a.namespace(labels::ROOT_DOC).to_bytes(),
            b.namespace(labels::ROOT_DOC).to_bytes()
        );
    }
}
