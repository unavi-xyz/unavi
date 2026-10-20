//! The DID document a node publishes, and the endpoint it names.

use std::str::FromStr;

use iroh::EndpointId;
use iroh_docs::NamespaceId;
use xdid::{
    core::{
        did::Did,
        did_url::{
            DidUrl,
            relative::{
                RelativeDidUrl,
                RelativeDidUrlPath,
            },
        },
        document::{
            Document,
            ServiceEndpoint,
            VerificationMethod,
            VerificationMethodMap,
        },
    },
    method::key::{
        DidKeyPair,
        PublicKey,
    },
};

/// Fragment of the service naming the iroh endpoint a DID answers on.
const ENDPOINT_FRAGMENT: &str = "iroh";

/// Service type of that service.
const ENDPOINT_TYPE: &str = "IrohEndpoint";
/// Fragment of the service naming the DID's profile namespace.
const PROFILE_FRAGMENT: &str = "profile";
const PROFILE_TYPE: &str = "WiredProfile";

const KEY_FRAGMENT: &str = "key";

/// The service naming `endpoint` as the one a DID answers on.
#[must_use]
pub fn endpoint_service(endpoint: EndpointId) -> ServiceEndpoint {
    ServiceEndpoint {
        id:               format!("#{ENDPOINT_FRAGMENT}"),
        typ:              vec![ENDPOINT_TYPE.into()],
        service_endpoint: vec![endpoint.to_string()],
    }
}

/// The service naming `ns` as the DID's profile document.
#[must_use]
pub fn profile_service(ns: NamespaceId) -> ServiceEndpoint {
    ServiceEndpoint {
        id:               format!("#{PROFILE_FRAGMENT}"),
        typ:              vec![PROFILE_TYPE.into()],
        service_endpoint: vec![ns.to_string()],
    }
}

/// The endpoint `doc` names, if any. Matched by type, with the id written as
/// a bare name, a fragment, or a full DID URL.
#[must_use]
pub fn endpoint_of(doc: &Document) -> Option<EndpointId> {
    service_value(doc, ENDPOINT_TYPE, ENDPOINT_FRAGMENT)
}

/// The first parsable endpoint of the service of type `typ` whose id names
/// `fragment`.
fn service_value<T: FromStr>(doc: &Document, typ: &str, fragment: &str) -> Option<T> {
    doc.service
        .as_deref()?
        .iter()
        .filter(|service| service.typ.iter().any(|t| t == typ))
        .filter(|service| {
            service.id == fragment
                || service
                    .id
                    .rsplit_once('#')
                    .is_some_and(|(_, f)| f == fragment)
        })
        .find_map(|service| {
            service
                .service_endpoint
                .first()
                .and_then(|value| T::from_str(value).ok())
        })
}

/// The profile namespace `doc` names, if any. Matched as [`endpoint_of`]
/// matches.
#[must_use]
pub fn profile_of(doc: &Document) -> Option<NamespaceId> {
    service_value(doc, PROFILE_TYPE, PROFILE_FRAGMENT)
}

/// The document a node serves for `did`: `key` authenticates it, it answers
/// on `endpoint`, and its profile is `profile`.
pub fn node_document(
    did: &Did,
    key: &impl DidKeyPair,
    endpoint: EndpointId,
    profile: NamespaceId,
) -> anyhow::Result<Document> {
    let key_ref = VerificationMethod::RelativeUrl(RelativeDidUrl::new(
        RelativeDidUrlPath::Empty,
        None,
        Some(KEY_FRAGMENT.into()),
    )?);

    Ok(Document {
        context:               None,
        id:                    did.clone(),
        also_known_as:         None,
        assertion_method:      Some(vec![key_ref.clone()]),
        authentication:        Some(vec![key_ref]),
        capability_delegation: None,
        capability_invocation: None,
        controller:            None,
        key_agreement:         None,
        service:               Some(vec![endpoint_service(endpoint), profile_service(profile)]),
        verification_method:   Some(vec![VerificationMethodMap {
            id:                   DidUrl::new(did.clone(), None, None, Some(KEY_FRAGMENT.into()))?,
            controller:           did.clone(),
            typ:                  "JsonWebKey2020".into(),
            public_key_multibase: None,
            public_key_jwk:       Some(key.public().to_jwk()),
        }]),
    })
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;
    use iroh_docs::NamespaceSecret;
    use xdid::method::key::p256::P256KeyPair;

    use super::*;

    fn profile() -> NamespaceId {
        NamespaceSecret::from_bytes(&[7; 32]).id()
    }

    #[test]
    fn a_node_document_names_its_profile() {
        let did = Did::from_str("did:web:example.com").expect("did");
        let endpoint = SecretKey::generate().public();
        let doc =
            node_document(&did, &P256KeyPair::generate(), endpoint, profile()).expect("document");

        assert_eq!(profile_of(&doc), Some(profile()));
    }

    #[test]
    fn a_node_document_names_its_endpoint() {
        let did = Did::from_str("did:web:example.com").expect("did");
        let endpoint = SecretKey::generate().public();
        let doc =
            node_document(&did, &P256KeyPair::generate(), endpoint, profile()).expect("document");

        assert_eq!(endpoint_of(&doc), Some(endpoint));
    }

    #[test]
    fn a_service_id_may_be_a_full_did_url() {
        let did = Did::from_str("did:web:example.com").expect("did");
        let endpoint = SecretKey::generate().public();
        let mut doc =
            node_document(&did, &P256KeyPair::generate(), endpoint, profile()).expect("document");
        for service in doc.service.iter_mut().flatten() {
            service.id = format!("{did}#iroh");
        }

        assert_eq!(endpoint_of(&doc), Some(endpoint));
    }
}
