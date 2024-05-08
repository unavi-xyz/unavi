use std::{future::Future, sync::Arc};

use axum::{routing::get, Json, Router};
use didkit::{
    ssi::{
        did::{
            RelativeDIDURL, Service, ServiceEndpoint, VerificationMethod, VerificationMethodMap,
        },
        vc::OneOrMany,
    },
    Document, JWK,
};
use dwn::{
    actor::{Actor, VerifiableCredential},
    message::descriptor::{protocols::ProtocolsFilter, records::RecordsFilter},
    store::{DataStore, MessageStore},
    DWN,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use wired_protocol::protocols::world_host::{
    world_host_definition, world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION,
};

const IDENTITY_PATH: &str = ".unavi/server/social/world-host-identity.json";
const KEY_FRAGMENT: &str = "key-0";

pub async fn router(
    dwn: Arc<DWN<impl DataStore, impl MessageStore>>,
    domain: String,
) -> (Router, impl Future) {
    let domain_name = domain.split(':').next().unwrap();

    let dwn_url = if domain_name == "localhost" {
        format!("http://{}", domain)
    } else {
        format!("https://{}", domain)
    };

    let did = format!("did:web:{}", domain.clone().replace(':', "%3A"));

    let actor = if let Ok(identity) = std::fs::read_to_string(IDENTITY_PATH) {
        let identity: WorldHostIdentity =
            serde_json::from_str(&identity).expect("Failed to parse world host identity");

        if identity.did == did {
            Actor {
                attestation: identity.vc_key.clone().into(),
                authorization: identity.vc_key.into(),
                did: identity.did,
                dwn: dwn.clone(),
                remotes: Vec::new(),
            }
        } else {
            warn!("World Host DID mismatch. Overwriting identity file.");
            std::fs::remove_file(IDENTITY_PATH).unwrap();
            create_identity(did, dwn)
        }
    } else {
        create_identity(did, dwn)
    };

    info!("World Host DID: {}", actor.did);

    let mut document = Document::new(&actor.did);

    document.service = Some(vec![Service {
        id: format!("{}#dwn", actor.did),
        type_: OneOrMany::One("DWN".to_string()),
        property_set: None,
        service_endpoint: Some(OneOrMany::One(ServiceEndpoint::URI(dwn_url.clone()))),
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

    let document = Arc::new(document);

    let router = Router::new().route(
        "/.well-known/did.json",
        get(|| async move { Json(document.clone()) }),
    );

    let create_world_host = async move {
        let mut definition = world_host_definition();
        definition.published = true;

        let protocols = actor
            .query_protocols(ProtocolsFilter {
                protocol: definition.protocol.clone(),
                versions: vec![WORLD_HOST_PROTOCOL_VERSION],
            })
            .process()
            .await
            .unwrap();

        if protocols.entries.is_empty() {
            info!(
                "Initializing world host protocol v{}",
                WORLD_HOST_PROTOCOL_VERSION
            );

            actor
                .register_protocol(definition)
                .protocol_version(WORLD_HOST_PROTOCOL_VERSION)
                .process()
                .await
                .unwrap();
        }

        let connect_records = actor
            .query_records(RecordsFilter {
                data_format: Some("text/plain".to_string()),
                protocol: Some(world_host_protocol_url()),
                protocol_version: Some(WORLD_HOST_PROTOCOL_VERSION),
                ..Default::default()
            })
            .process()
            .await
            .unwrap();

        // TODO: Update connect-url if it doesn't match.

        if connect_records.entries.is_empty() {
            let connect_url = format!("{}", dwn_url);

            info!("Initializing connect-url: {}", connect_url);

            let reply = actor
                .create_record()
                .protocol(
                    world_host_protocol_url(),
                    WORLD_HOST_PROTOCOL_VERSION,
                    "connect-url".to_string(),
                )
                .data_format("text/plain".to_string())
                .data(connect_url.into())
                .process()
                .await
                .unwrap();

            info!("connect-url record: {}", reply.record_id);
        }
    };

    (router, create_world_host)
}
