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
    config::Config as RegistryConfig,
    views::ViewIds,
};
use wds::{
    DataStore,
    WDS_SERVICE_TYPE,
    identity::{
        RootIdentity,
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

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

/// Which services this node runs.
///
/// One DID, one endpoint, one storage directory; the roles are toggles rather
/// than separate processes, so a registry-only deployment still has a
/// resolvable identity and a storage node still shares its endpoint with
/// discovery. File hosting is always on: the files directory is cheap and
/// inert when empty.
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
    let identity = Arc::new(RootIdentity::load(&key_storage(opts.in_memory))?);
    info!("Running server as {did}");

    // Fixes the endpoint id across restarts. The served DID document names it,
    // and `signed_by_wds_service` has peers verify a challenge against it.
    let endpoint = Endpoint::builder(N0)
        .secret_key(identity.endpoint_key())
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
        let (registry, protocol) =
            Registry::create(Arc::clone(&store), RegistryConfig::default()).await?;
        let views = registry.views();
        info!(recent = %views.recent, "Serving registry");
        rb = rb.accept(unavi_registry::control::ALPN, protocol);
        (Some(registry), Some(views))
    } else {
        (None, None)
    };

    let router = rb.spawn();

    let app = create_did_document_route(did, identity.signing_key(), store.endpoint_id(), views)?;

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
