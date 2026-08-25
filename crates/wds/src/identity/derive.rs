use blake3::{
    derive_key,
    keyed_hash,
};
use iroh_docs::NamespaceSecret;
use zeroize::Zeroizing;

/// Fixed for the life of the format: changing a context changes every key
/// derived under it, and with it the node's endpoint id, author id and every
/// well-known namespace.
const ED25519_CONTEXT: &str = "unavi.xyz 2026-08-25 identity ed25519 v1";
const NAMESPACE_CONTEXT: &str = "unavi.xyz 2026-08-25 identity namespace root v1";

/// The ed25519 seed backing both the iroh endpoint key and the docs author.
///
/// One seed serves both because [`iroh_docs::Author`] wraps [`iroh::SecretKey`]
/// and each reads the 32 bytes as an ed25519 signing key, making the author id
/// and the endpoint id the same bytes.
#[must_use]
pub fn ed25519_seed(scalar: &[u8]) -> Zeroizing<[u8; 32]> {
    Zeroizing::new(derive_key(ED25519_CONTEXT, scalar))
}

#[must_use]
pub fn namespace_root(scalar: &[u8]) -> Zeroizing<[u8; 32]> {
    Zeroizing::new(derive_key(NAMESPACE_CONTEXT, scalar))
}

/// The namespace secret for `label` under a node's namespace root.
///
/// The label is keyed material rather than KDF context because blake3 requires
/// a hardcoded context string, which a per-label one could not be.
#[must_use]
pub fn namespace(root: &[u8; 32], label: &str) -> NamespaceSecret {
    NamespaceSecret::from_bytes(keyed_hash(root, label.as_bytes()).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALAR: &[u8] = b"a 32 byte test scalar stand-in..";

    #[test]
    fn derivation_is_stable() {
        assert_eq!(*ed25519_seed(SCALAR), *ed25519_seed(SCALAR));
        assert_eq!(*namespace_root(SCALAR), *namespace_root(SCALAR));
    }

    #[test]
    fn contexts_are_separated() {
        assert_ne!(
            *ed25519_seed(SCALAR),
            *namespace_root(SCALAR),
            "one scalar under two contexts must not yield the same key"
        );
    }

    #[test]
    fn distinct_scalars_derive_distinct_seeds() {
        assert_ne!(*ed25519_seed(SCALAR), *ed25519_seed(b"a different scalar"));
    }

    #[test]
    fn labels_yield_distinct_namespaces() {
        let root = namespace_root(SCALAR);

        assert_eq!(
            namespace(&root, "home").to_bytes(),
            namespace(&root, "home").to_bytes(),
            "a label must name the same namespace on every load"
        );
        assert_ne!(
            namespace(&root, "home").to_bytes(),
            namespace(&root, "away").to_bytes()
        );
    }
}
