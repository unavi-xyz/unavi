use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::LazyLock,
    time::Duration,
};

use axum::{Json, Router};
use directories::ProjectDirs;
use iroh::{Endpoint, EndpointId};
use tower_http::cors::CorsLayer;
use tracing::info;
use wds::DataStore;
use xdid::{
    core::{
        did::{Did, MethodId, MethodName},
        did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
        document::{Document, ServiceEndpoint, VerificationMethod, VerificationMethodMap},
    },
    methods::key::{DidKeyPair, PublicKey},
};

mod key_pair;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub struct ServerOptions {
    pub in_memory: bool,
    pub port: u16,
}

/// Run the UNAVI server.
///
/// # Errors
///
/// Returns an error if server initialization or startup fails.
pub async fn run_server(opts: ServerOptions) -> anyhow::Result<()> {
    let port = opts.port;

    let (did, _domain) = create_did(port);
    let vc = key_pair::get_or_create_key(opts.in_memory)?;
    info!("Running server as {did}");

    let endpoint = Endpoint::builder().bind().await?;

    let store = {
        let path = DIRS.data_local_dir().join("wds");
        DataStore::builder(endpoint)
            .storage_path(path)
            .gc_timer(Duration::from_mins(15))
            .build()
            .await?
    };

    let app = create_did_document_route(did, &vc, store.endpoint_id());

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
    info!("HTTP listening on port {port}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    store.shutdown().await?;

    Ok(())
}

fn create_did(port: u16) -> (Did, String) {
    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));
    let domain_encoded = domain.replace(':', "%3A");
    let did = Did {
        method_name: MethodName("web".into()),
        method_id: MethodId(domain_encoded),
    };
    (did, domain)
}

const KEY_FRAGMENT: &str = "key";

fn create_did_document_route(did: Did, vc: &impl DidKeyPair, endpoint_id: EndpointId) -> Router {
    let vc_public = vc.public().to_jwk();

    Router::new()
        .route(
            "/.well-known/did.json",
            axum::routing::get(move || async move {
                let doc = Document {
                    id: did.clone(),
                    also_known_as: None,
                    assertion_method: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                        path: RelativeDidUrlPath::Empty,
                        query: None,
                        fragment: Some(KEY_FRAGMENT.into()),
                    })]),
                    authentication: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                        path: RelativeDidUrlPath::Empty,
                        query: None,
                        fragment: Some(KEY_FRAGMENT.into()),
                    })]),
                    capability_delegation: None,
                    capability_invocation: None,
                    controller: None,
                    key_agreement: None,
                    service: Some(vec![ServiceEndpoint {
                        id: "wds".into(),
                        typ: vec![endpoint_id.to_string()],
                    }]),
                    verification_method: Some(vec![VerificationMethodMap {
                        id: DidUrl {
                            did: did.clone(),
                            fragment: Some(KEY_FRAGMENT.into()),
                            query: None,
                            path_abempty: None,
                        },
                        controller: did.clone(),
                        typ: "JsonWebKey2020".into(),
                        public_key_multibase: None,
                        public_key_jwk: Some(vc_public.clone()),
                    }]),
                };

                Json(doc)
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET]),
        )
}
