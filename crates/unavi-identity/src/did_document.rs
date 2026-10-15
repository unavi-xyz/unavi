//! The DID document a node publishes, and the endpoint it names.

use std::str::FromStr;

use iroh::EndpointId;
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

/// The endpoint `doc` names, if any. Matched by type, with the id written as
/// a bare name, a fragment, or a full DID URL.
#[must_use]
pub fn endpoint_of(doc: &Document) -> Option<EndpointId> {
    doc.service
        .as_deref()?
        .iter()
        .filter(|service| service.typ.iter().any(|typ| typ == ENDPOINT_TYPE))
        .filter(|service| {
            service.id == ENDPOINT_FRAGMENT
                || service
                    .id
                    .rsplit_once('#')
                    .is_some_and(|(_, fragment)| fragment == ENDPOINT_FRAGMENT)
        })
        .find_map(|service| {
            service
                .service_endpoint
                .first()
                .and_then(|id| EndpointId::from_str(id).ok())
        })
}

/// The document a node serves for `did`: `key` authenticates it, and it answers
/// on `endpoint`.
pub fn node_document(
    did: &Did,
    key: &impl DidKeyPair,
    endpoint: EndpointId,
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
        service:               Some(vec![endpoint_service(endpoint)]),
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
    use xdid::method::key::p256::P256KeyPair;

    use super::*;

    #[test]
    fn a_node_document_names_its_endpoint() {
        let did = Did::from_str("did:web:example.com").expect("did");
        let endpoint = SecretKey::generate().public();
        let doc = node_document(&did, &P256KeyPair::generate(), endpoint).expect("document");

        assert_eq!(endpoint_of(&doc), Some(endpoint));
    }

    #[test]
    fn a_service_id_may_be_a_full_did_url() {
        let did = Did::from_str("did:web:example.com").expect("did");
        let endpoint = SecretKey::generate().public();
        let mut doc = node_document(&did, &P256KeyPair::generate(), endpoint).expect("document");
        for service in doc.service.iter_mut().flatten() {
            service.id = format!("{did}#iroh");
        }

        assert_eq!(endpoint_of(&doc), Some(endpoint));
    }
}
