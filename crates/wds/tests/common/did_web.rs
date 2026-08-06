use std::str::FromStr;

use axum::{
    Json,
    Router,
};
use iroh::EndpointId;
use tokio::{
    net::TcpListener,
    task::JoinHandle,
};
use wds::WDS_SERVICE_TYPE;
use xdid::{
    core::{
        did::Did,
        did_url::{
            relative::{
                RelativeDidUrl,
                RelativeDidUrlPath,
            },
            url::DidUrl,
        },
        document::{
            Document,
            ServiceEndpoint,
            VerificationMethod,
            VerificationMethodMap,
        },
    },
    methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
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

    let did = Did::from_str(&format!("did:web:localhost%3A{port}")).expect("valid did");

    // WDS service allows these endpoints to authenticate on behalf of this DID.
    let wds_service = ServiceEndpoint {
        id:               "wds".into(),
        typ:              vec![WDS_SERVICE_TYPE.into()],
        service_endpoint: wds_endpoints.iter().map(ToString::to_string).collect(),
    };

    let key_ref = VerificationMethod::RelativeUrl(
        RelativeDidUrl::new(RelativeDidUrlPath::Empty, None, Some("key".into()))
            .expect("valid relative url"),
    );

    let doc = Document {
        context:               None,
        id:                    did.clone(),
        also_known_as:         None,
        assertion_method:      Some(vec![key_ref.clone()]),
        authentication:        Some(vec![key_ref]),
        capability_delegation: None,
        capability_invocation: None,
        controller:            None,
        key_agreement:         None,
        service:               Some(vec![wds_service]),
        verification_method:   Some(vec![VerificationMethodMap {
            id:                   DidUrl::new(did.clone(), None, None, Some("key".into()))
                .expect("valid did url"),
            controller:           did.clone(),
            typ:                  "JsonWebKey2020".into(),
            public_key_multibase: None,
            public_key_jwk:       Some(key.public().to_jwk()),
        }]),
    };

    let body = serde_json::to_value(&doc).expect("serialize document");

    let app = Router::new().route(
        "/.well-known/did.json",
        axum::routing::get(move || {
            let body = body.clone();
            async move { Json(body) }
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
