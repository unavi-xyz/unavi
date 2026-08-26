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
use unavi_registry::{
    Registry,
    config::{
        Config as RegistryConfig,
        RegistryDocs,
    },
    views::ViewIds,
};
use wds::{
    DataStore,
    WDS_SERVICE_TYPE,
    identity::{
        WdsIdentity,
        store::KeyStorage,
    },
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

const REGISTRY_DOCS_FILE: &str = "registry-docs.json";

/// The registry docs this node minted before, if any. An unreadable record is
/// reported and replaced: minting fresh abandons the old namespaces, but a
/// registry that cannot start at all serves no one.
fn load_registry_docs() -> Option<RegistryDocs> {
    let text = std::fs::read_to_string(DIRS.data_local_dir().join(REGISTRY_DOCS_FILE)).ok()?;
    match serde_json::from_str(&text) {
        Ok(docs) => Some(docs),
        Err(err) => {
            warn!(?err, "unreadable registry docs record; minting fresh");
            None
        }
    }
}

fn save_registry_docs(docs: &RegistryDocs) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(docs)?;
    std::fs::write(DIRS.data_local_dir().join(REGISTRY_DOCS_FILE), json)?;
    Ok(())
}

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

/// Which services this node runs. One DID, endpoint, and storage directory
/// serve both roles; file hosting is always on.
pub struct Features {
    pub registry: bool,
    pub wds:      bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            registry: true,
            wds:      true,
        }
    }
}

pub struct ServerOptions {
    pub features:  Features,
    pub in_memory: bool,
    pub port:      u16,
}

pub async fn run_server(opts: ServerOptions) -> anyhow::Result<()> {
    let port = opts.port;

    let did = create_did(&secrets::Secrets::load().unavi_domain)?;
    let identity = Arc::new(WdsIdentity::load(&key_storage(opts.in_memory))?);
    info!("Running server as {did}");

    // Fixes the endpoint id across restarts. The served DID document names it,
    // and `signed_by_wds_service` has peers verify a challenge against it.
    let endpoint = Endpoint::builder(N0)
        .secret_key(identity.endpoint().clone())
        .bind()
        .await?;

    let builder = DataStore::builder(endpoint.clone(), Arc::clone(&identity))
        .gc_timer(Duration::from_mins(15))
        .serve_control();
    let builder = if opts.in_memory {
        builder
    } else {
        builder.storage_path(DIRS.data_local_dir().join("wds"))
    };

    let (store, f) = builder.build().await?;
    let store = Arc::new(store);

    if let Err(err) = files::init_files_dir() {
        warn!(?err, "failed to init files dir");
    }
    match files::host_files(&store).await {
        Ok(hosted) => files::log_manifest(&hosted),
        Err(err) => warn!(?err, "failed to host files"),
    }

    let mut rb = iroh::protocol::Router::builder(endpoint);
    if opts.features.wds {
        info!("Serving WDS storage");
        rb = f(rb);
    }

    let (_registry, views) = if opts.features.registry {
        let mut config = RegistryConfig::default();
        config.docs = load_registry_docs();

        let (registry, protocol) = Registry::create(Arc::clone(&store), config).await?;
        if let Err(err) = save_registry_docs(&registry.docs()) {
            warn!(?err, "could not persist registry doc ids");
        }

        let views = registry.views();
        info!(recent = %views.recent, "Serving registry");
        rb = rb.accept(unavi_registry::control::ALPN, protocol);
        (Some(registry), Some(views))
    } else {
        (None, None)
    };

    let router = rb.spawn();

    let app = create_did_document_route(
        did,
        identity.user().signing_key(),
        store.endpoint_id(),
        views,
    )?;

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
    info!("HTTP listening on port {port}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    router.shutdown().await?;

    Ok(())
}

fn key_storage(in_memory: bool) -> KeyStorage {
    if in_memory {
        KeyStorage::Ephemeral
    } else {
        KeyStorage::Path(DIRS.data_local_dir().to_path_buf())
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
    views: Option<ViewIds>,
) -> anyhow::Result<axum::Router> {
    // Advertise the endpoint always, and the registry's entry view only when
    // this node runs one, so resolvers learn which roles it serves.
    let mut service_endpoint = vec![endpoint_id.to_string()];
    if let Some(views) = views {
        service_endpoint.push(format!("registry:{}", views.recent));
    }

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
            id: "wds".into(),
            typ: vec![WDS_SERVICE_TYPE.into()],
            service_endpoint,
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
