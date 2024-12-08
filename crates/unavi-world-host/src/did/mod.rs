use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::Storage;

pub mod document;

const KEY_FRAGMENT: &str = "key-0";

#[derive(Deserialize, Serialize)]
struct WorldHostIdentity {
    did: String,
    vc_key: VcKey,
}

#[derive(Clone, Deserialize, Serialize)]
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

pub struct ActorOptions {
    pub domain: String,
    pub dwn: DWN,
    pub storage: Storage,
}

pub fn create_actor(opts: ActorOptions) -> Actor {
    let did = format!("did:web:{}", opts.domain.clone().replace(':', "%3A"));

    let actor = match &opts.storage {
        Storage::Path(path) => {
            let identity_path = path.join(Path::new("identity.json"));

            if let Ok(identity) = std::fs::read_to_string(&identity_path) {
                let identity: WorldHostIdentity =
                    serde_json::from_str(&identity).expect("Failed to parse world host identity");

                if identity.did == did {
                    Actor {
                        attestation: identity.vc_key.clone().into(),
                        authorization: identity.vc_key.into(),
                        did: identity.did,
                        dwn: opts.dwn.clone(),
                        remotes: Vec::new(),
                    }
                } else {
                    warn!("World Host DID mismatch. Overwriting identity file.");
                    std::fs::remove_file(&identity_path).unwrap();
                    create_identity(did, opts.dwn, Some(&identity_path))
                }
            } else {
                create_identity(did, opts.dwn, Some(&identity_path))
            }
        }
        Storage::Memory => create_identity(did, opts.dwn, None),
    };

    info!("World Host DID: {}", actor.did);

    actor
}

fn create_identity(did: String, dwn: DWN, path: Option<&Path>) -> Actor {
    // Create a did:key and convert it to a did:web.
    let mut actor = Actor::new_did_key(dwn).unwrap();
    let key_id = format!("{}#{}", did, KEY_FRAGMENT);
    actor.attestation.key_id.clone_from(&key_id);
    actor.authorization.key_id = key_id;
    actor.did = did;

    let identity = WorldHostIdentity {
        did: actor.did.clone(),
        vc_key: actor.authorization.clone().into(),
    };
    let identity = serde_json::to_string(&identity).unwrap();

    if let Some(path) = path {
        std::fs::write(path, identity).unwrap();
    }

    actor
}
