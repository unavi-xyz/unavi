use std::sync::Arc;

use axum::{Json, Router};
use iroh::EndpointId;
use rusqlite::params;
use tokio::{net::TcpListener, task::JoinHandle};
use wds::{DataStore, actor::Actor, identity::Identity};
use xdid::{
    core::{
        did::{Did, MethodId, MethodName},
        did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
        document::{Document, ServiceEndpoint, VerificationMethod, VerificationMethodMap},
    },
    methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair},
};

pub struct DidWebServer {
    pub did: Did,
    _handle: JoinHandle<()>,
}

pub async fn spawn_did_web_server(
    key: &P256KeyPair,
    wds_endpoints: Vec<EndpointId>,
) -> DidWebServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    let domain = format!("localhost:{port}");
    let domain_encoded = domain.replace(':', "%3A");
    let did = Did {
        method_name: MethodName("web".into()),
        method_id: MethodId(domain_encoded),
    };

    let jwk = key.public().to_jwk();
    let did_clone = did.clone();

    // WDS service allows these endpoints to authenticate on behalf of this DID.
    let wds_service = ServiceEndpoint {
        id: "wds".into(),
        typ: wds_endpoints.iter().map(ToString::to_string).collect(),
    };

    let app = Router::new().route(
        "/.well-known/did.json",
        axum::routing::get(move || {
            let did = did_clone.clone();
            let jwk = jwk.clone();
            let service = wds_service.clone();
            async move {
                Json(Document {
                    id: did.clone(),
                    also_known_as: None,
                    assertion_method: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                        path: RelativeDidUrlPath::Empty,
                        query: None,
                        fragment: Some("key".into()),
                    })]),
                    authentication: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                        path: RelativeDidUrlPath::Empty,
                        query: None,
                        fragment: Some("key".into()),
                    })]),
                    capability_delegation: None,
                    capability_invocation: None,
                    controller: None,
                    key_agreement: None,
                    service: Some(vec![service]),
                    verification_method: Some(vec![VerificationMethodMap {
                        id: DidUrl {
                            did: did.clone(),
                            fragment: Some("key".into()),
                            query: None,
                            path_abempty: None,
                        },
                        controller: did,
                        typ: "JsonWebKey2020".into(),
                        public_key_multibase: None,
                        public_key_jwk: Some(jwk),
                    }]),
                })
            }
        }),
    );

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    DidWebServer {
        did,
        _handle: handle,
    }
}

pub struct ActorWithServer {
    pub actor: Actor,
    pub server: DidWebServer,
}

pub async fn generate_actor_web(
    store: &DataStore,
    wds_endpoints: Vec<EndpointId>,
) -> ActorWithServer {
    let key = P256KeyPair::generate();
    let server = spawn_did_web_server(&key, wds_endpoints).await;

    let identity = Arc::new(Identity::new(server.did.clone(), key));
    let actor = store.local_actor(identity);

    let did_str = server.did.to_string();
    store
        .db()
        .async_call(move |conn| {
            conn.execute(
                "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
                params![&did_str],
            )?;
            Ok(())
        })
        .await
        .expect("create quota");

    ActorWithServer { actor, server }
}
