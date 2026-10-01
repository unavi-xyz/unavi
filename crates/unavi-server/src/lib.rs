//! A unavi node: hosts documents and files over iroh, serves its `did:web`
//! document over HTTP, and optionally runs a registry.

use std::{
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
    },
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    body::Bytes,
    http::header,
};
use iroh::{
    Endpoint,
    endpoint::presets::N0,
};
use tower_http::cors::CorsLayer;
use tracing::{
    info,
    warn,
};
use unavi_identity::{
    auth::{
        self,
        EndpointAuth,
    },
    did_document,
    identity::NodeIdentity,
    resolver::Resolver,
};
use unavi_local::DeviceStorage;
use unavi_registry::server::{
    Config as RegistryConfig,
    Registry,
};
use unavi_store::StoreBuilder;
use xdid::core::did::Did;

pub mod config;
mod files;

/// Bytes of documents the server holds before the retention sweep evicts
/// read-only ones, least recently joined first.
const DOC_BUDGET: u64 = 16 * 1024 * 1024 * 1024;

pub struct ServerOptions {
    /// The domain this node's `did:web` resolves from.
    pub domain:   String,
    /// Where keys, documents and hosted files live. `None` keeps keys and
    /// documents in memory and hosts no files.
    pub data_dir: Option<PathBuf>,
    pub port:     u16,
    /// Serves discovery (catalog, curated views and live presence) under this
    /// policy. One DID, endpoint and data directory serve every role the node
    /// takes on; document sync and file hosting are always on.
    pub registry: Option<RegistryConfig>,
}

/// Runs the node until `shutdown` resolves, then flushes the stores.
pub async fn run_server(
    opts: ServerOptions,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let did = did_web(&opts.domain)?;
    let storage = opts
        .data_dir
        .as_ref()
        .map_or_else(DeviceStorage::memory, |dir| DeviceStorage::at(dir.clone()));

    // The node proves the DID it publishes, so a client that followed it binds
    // this endpoint to that DID.
    let node = Arc::new(NodeIdentity::load_as(&storage, did.clone())?);
    info!(%did, "Running server");

    let resolver = Arc::new(Resolver::new()?);
    let auth = Arc::new(EndpointAuth::new(
        Arc::clone(node.user()),
        Arc::clone(&resolver),
    ));

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
    let store = StoreBuilder::new(endpoint.clone(), node.author())
        .sweep_interval(Duration::from_mins(15))
        .doc_budget(DOC_BUDGET)
        .storage(storage.clone())
        .build()
        .await?;

    if let Some(data_dir) = &opts.data_dir {
        if let Err(err) = files::init_files_dir(data_dir) {
            warn!(?err, "failed to init files dir");
        }
        match files::host_files(store.blob_store(), data_dir).await {
            Ok(hosted) => files::log_manifest(&hosted, data_dir),
            Err(err) => warn!(?err, "failed to host files"),
        }
    }

    let mut rb = iroh::protocol::Router::builder(endpoint);
    rb = store.accept(rb).accept(auth::ALPN, auth_protocol);

    let _registry = if let Some(config) = opts.registry {
        let (registry, protocol) = Registry::create(
            &store,
            config,
            Arc::clone(auth.bindings()),
            Arc::clone(&resolver),
        )
        .await?;

        info!(recent = %registry.views().recent, "Serving registry");
        rb = rb.accept(unavi_registry::rpc::ALPN, protocol);
        Some(registry)
    } else {
        None
    };

    let router = rb.spawn();

    let document = did_document::node_document(&did, node.user().signing_key(), endpoint_id)?;
    let app = did_document_route(Bytes::from(serde_json::to_vec(&document)?));

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, opts.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = opts.port, "HTTP listening");

    let served = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await;

    info!("Shutting down");
    router.shutdown().await?;
    served?;

    Ok(())
}

/// The `did:web` for `domain`. A port's colon is percent-encoded, as the
/// method requires.
fn did_web(domain: &str) -> anyhow::Result<Did> {
    let domain = domain.replace(':', "%3A");
    Ok(Did::from_str(&format!("did:web:{domain}"))?)
}

fn did_document_route(body: Bytes) -> axum::Router {
    axum::Router::new()
        .route(
            "/.well-known/did.json",
            axum::routing::get(move || {
                let body = body.clone();
                async move { ([(header::CONTENT_TYPE, "application/did+json")], body) }
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET]),
        )
}
