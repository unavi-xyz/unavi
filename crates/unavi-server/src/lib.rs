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
use tracing::info;
use wds::{
    DataStore,
    WDS_SERVICE_TYPE,
};
use wired_registry::{
    Registry,
    config::Config as RegistryConfig,
    views::ViewIds,
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

mod key_pair;

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
/// discovery.
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

    let (did, _domain) = create_did(port)?;
    let vc = key_pair::get_or_create_key(opts.in_memory)?;
    info!("Running server as {did}");

    let endpoint = Endpoint::builder(N0).bind().await?;

    let path = DIRS.data_local_dir().join("wds");
    let (store, f) = DataStore::builder(endpoint.clone())
        .storage_path(path)
        .gc_timer(Duration::from_mins(15))
        .build()
        .await?;
    let store = Arc::new(store);

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
        rb = rb.accept(wired_registry::control::ALPN, protocol);
        (Some(registry), Some(views))
    } else {
        (None, None)
    };

    let router = rb.spawn();

    let app = create_did_document_route(did, &vc, store.endpoint_id(), views)?;

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
    info!("HTTP listening on port {port}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    router.shutdown().await?;

    Ok(())
}

fn create_did(port: u16) -> anyhow::Result<(Did, String)> {
    let domain = std::env::var("UNAVI_DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));
    let domain_encoded = domain.replace(':', "%3A");
    let did = Did::from_str(&format!("did:web:{domain_encoded}"))?;
    Ok((did, domain))
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
