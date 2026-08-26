use std::{
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
    },
    str::FromStr,
    sync::{
        Arc,
        LazyLock,
    },
    time::Duration,
};

use axum::Json;
use directories::ProjectDirs;
use iroh::{
    Endpoint,
    EndpointId,
    endpoint::presets::N0,
};
use tower_http::cors::CorsLayer;
use tracing::{
    info,
    warn,
};
use unavi_identity::{
    ENDPOINT_SERVICE_ID,
    ENDPOINT_SERVICE_TYPE,
    auth::{
        self,
        EndpointAuth,
    },
    identity::{
        self,
        NodeIdentity,
    },
};
use unavi_registry::{
    Registry,
    config::Config as RegistryConfig,
};
use unavi_store::{
    builder::{
        Builder as StoreBuilder,
        Store,
    },
    local::Storage,
};
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
    },
};

mod files;
pub mod secrets;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub struct ServerOptions {
    pub in_memory: bool,
    pub port:      u16,
    /// Whether this node serves discovery: catalog, curated views, and live
    /// presence. One DID, endpoint, and storage directory serve every role it
    /// takes on; document sync and file hosting are always on.
    pub registry:  bool,
}

pub async fn run_server(opts: ServerOptions) -> anyhow::Result<()> {
    let port = opts.port;

    let did = create_did(&secrets::Secrets::load().unavi_domain)?;
    let storage = key_storage(opts.in_memory);
    let node = Arc::new(NodeIdentity::load(&storage)?);
    info!("Running server as {did}");

    // Published before the endpoint binds: a peer can dial the instant it is
    // listening, and answering its proof needs the key.
    identity::set_local(Arc::clone(node.user()));

    let auth = EndpointAuth::new();

    // Fixes the endpoint id across restarts. The served DID document names it,
    // so a client that resolved this DID can still reach the node.
    let endpoint = auth
        .install(Endpoint::builder(N0).secret_key(node.endpoint().clone()))
        .bind()
        .await?;

    let (auth_protocol, _auth_task) = auth
        .serve(endpoint.clone())
        .ok_or_else(|| anyhow::anyhow!("identity handshake already served"))?;

    let endpoint_id = endpoint.id();
    let builder = StoreBuilder::new(endpoint.clone(), node.author())
        .gc_timer(Duration::from_mins(15))
        .storage(storage.clone());

    let Store {
        blobs,
        docs,
        router,
        guard: _guard,
        ..
    } = builder.build().await?;

    if let Err(err) = files::init_files_dir() {
        warn!(?err, "failed to init files dir");
    }
    match files::host_files(&blobs).await {
        Ok(hosted) => files::log_manifest(&hosted),
        Err(err) => warn!(?err, "failed to host files"),
    }

    let mut rb = iroh::protocol::Router::builder(endpoint);
    rb = router(rb).accept(auth::ALPN, auth_protocol);

    let _registry = if opts.registry {
        let config = RegistryConfig::default();
        let (registry, protocol) = Registry::create(&docs, blobs.blobs(), config, &storage).await?;

        info!(recent = %registry.views().recent, "Serving registry");
        rb = rb.accept(unavi_registry::control::ALPN, protocol);
        Some(registry)
    } else {
        None
    };

    let router = rb.spawn();

    let app = create_did_document_route(did, node.user().signing_key(), endpoint_id)?;

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
    info!("HTTP listening on port {port}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    router.shutdown().await?;

    Ok(())
}

fn key_storage(in_memory: bool) -> Storage {
    if in_memory {
        Storage::Ephemeral
    } else {
        Storage::Path(DIRS.data_local_dir().to_path_buf())
    }
}

fn create_did(domain: &str) -> anyhow::Result<Did> {
    let domain_encoded = domain.replace(':', "%3A");
    Did::from_str(&format!("did:web:{domain_encoded}"))
}

const KEY_FRAGMENT: &str = "key";

fn create_did_document_route(
    did: Did,
    vc: &impl DidKeyPair,
    endpoint_id: EndpointId,
) -> anyhow::Result<axum::Router> {
    let key_ref = VerificationMethod::RelativeUrl(RelativeDidUrl::new(
        RelativeDidUrlPath::Empty,
        None,
        Some(KEY_FRAGMENT.into()),
    )?);

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
        service:               Some(vec![ServiceEndpoint {
            id:               ENDPOINT_SERVICE_ID.into(),
            typ:              vec![ENDPOINT_SERVICE_TYPE.into()],
            service_endpoint: vec![endpoint_id.to_string()],
        }]),
        verification_method:   Some(vec![VerificationMethodMap {
            id:                   DidUrl::new(did.clone(), None, None, Some(KEY_FRAGMENT.into()))?,
            controller:           did,
            typ:                  "JsonWebKey2020".into(),
            public_key_multibase: None,
            public_key_jwk:       Some(vc.public().to_jwk()),
        }]),
    };

    let body = serde_json::to_value(&doc)?;

    Ok(axum::Router::new()
        .route(
            "/.well-known/did.json",
            axum::routing::get(move || {
                let body = body.clone();
                async move { Json(body) }
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET]),
        ))
}
