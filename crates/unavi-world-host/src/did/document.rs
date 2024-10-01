use std::sync::Arc;

use didkit::{
    ssi::{
        did::{
            RelativeDIDURL, Service, ServiceEndpoint, VerificationMethod, VerificationMethodMap,
        },
        vc::OneOrMany,
    },
    Document,
};
use dwn::actor::Actor;

use super::KEY_FRAGMENT;

pub fn create_document(actor: &Actor, dwn_url: String) -> Arc<Document> {
    let mut document = Document::new(&actor.did);

    document.service = Some(vec![Service {
        id: format!("{}#dwn", actor.did),
        type_: OneOrMany::One("DWN".to_string()),
        property_set: None,
        service_endpoint: Some(OneOrMany::One(ServiceEndpoint::URI(dwn_url))),
    }]);

    document.verification_method = Some(vec![VerificationMethod::Map(VerificationMethodMap {
        controller: actor.did.clone(),
        id: format!("{}#{}", &actor.did, KEY_FRAGMENT),
        public_key_jwk: Some(actor.authorization.jwk.clone().to_public()),
        type_: "JsonWebKey2020".to_string(),
        ..Default::default()
    })]);

    document.assertion_method = Some(vec![VerificationMethod::RelativeDIDURL(RelativeDIDURL {
        fragment: Some(KEY_FRAGMENT.to_string()),
        ..Default::default()
    })]);

    document.authentication = Some(vec![VerificationMethod::RelativeDIDURL(RelativeDIDURL {
        fragment: Some(KEY_FRAGMENT.to_string()),
        ..Default::default()
    })]);

    Arc::new(document)
}
