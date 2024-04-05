use std::sync::Arc;

use didkit::JWK;
use dwn::{
    actor::{Actor, VerifiableCredential},
    message::descriptor::{
        protocols::{ProtocolDefinition, ProtocolsFilter},
        records::Version,
    },
    store::{DataStore, MessageStore},
    DWN,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

const IDENTITY_PATH: &str = ".unavi/registry_identity.json";
const PROTOCOL_DEFINITION: &str =
    include_str!("../../../wired-protocol/social/dwn/protocols/world-registry.json");
const PROTOCOL_VERSION: &str = "0.0.1";

pub async fn create_world_registry(dwn: Arc<DWN<impl DataStore, impl MessageStore>>) {
    let actor = if let Ok(identity) = std::fs::read_to_string(IDENTITY_PATH) {
        let identity: RegistryIdentity =
            serde_json::from_str(&identity).expect("Failed to parse registry identity");
        info!("Found existing registry identity: {}", identity.did);

        Actor {
            attestation: identity.attestation.into(),
            authorization: identity.authorization.into(),
            did: identity.did,
            dwn: dwn.clone(),
            remotes: Vec::new(),
        }
    } else {
        let actor = Actor::new_did_key(dwn).unwrap();
        info!("Created new registry identity: {}", actor.did);

        let identity = RegistryIdentity {
            attestation: actor.attestation.clone().into(),
            authorization: actor.authorization.clone().into(),
            did: actor.did.clone(),
        };
        let identity = serde_json::to_string(&identity).unwrap();

        std::fs::create_dir_all(IDENTITY_PATH).unwrap();
        std::fs::write(IDENTITY_PATH, identity).unwrap();

        actor
    };

    let value: Value = serde_json::from_str(PROTOCOL_DEFINITION).unwrap();
    let protocol = value["protocol"].as_str().unwrap().to_string();
    let version = Version::parse(PROTOCOL_VERSION).unwrap();

    let query = actor
        .query_protocols(ProtocolsFilter {
            protocol,
            versions: vec![version.clone()],
        })
        .process()
        .await
        .unwrap();

    if query.entries.is_empty() {
        info!("Creating world registry v{}", PROTOCOL_VERSION);
        let definition = json_to_defition(value);
        actor
            .register_protocol(definition)
            .protocol_version(version)
            .process()
            .await
            .unwrap();
    }
}

fn json_to_defition(json: Value) -> ProtocolDefinition {
    let protocol = json["protocol"].as_str().unwrap().to_string();

    let structure = json["structure"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::from_value(v.clone()).unwrap()))
        .collect();

    let types = json["types"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::from_value(v.clone()).unwrap()))
        .collect();

    ProtocolDefinition {
        protocol,
        published: true,
        structure,
        types,
    }
}

#[derive(Deserialize, Serialize)]
struct VcKey {
    jwk: JWK,
    key_id: String,
}

impl From<VerifiableCredential> for VcKey {
    fn from(vc: VerifiableCredential) -> Self {
        Self {
            jwk: vc.jwk,
            key_id: vc.key_id,
        }
    }
}

impl From<VcKey> for VerifiableCredential {
    fn from(vc: VcKey) -> Self {
        Self {
            jwk: vc.jwk,
            key_id: vc.key_id,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct RegistryIdentity {
    attestation: VcKey,
    authorization: VcKey,
    did: String,
}
