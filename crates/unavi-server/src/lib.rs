use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::LazyLock,
    time::Duration,
};

use anyhow::Context;
use axum::{Json, Router};
use directories::ProjectDirs;
use tracing::{error, info};
use wtransport::{Endpoint, Identity, ServerConfig};
use xdid::{
    core::{
        did::{Did, MethodId, MethodName},
        did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
        document::{Document, VerificationMethod, VerificationMethodMap},
    },
    methods::{
        key::{DidKeyPair, PublicKey},
        web::reqwest::Url,
    },
};

use crate::session::{KEY_FRAGMENT, SessionSpawner, SpawnerOptions};

mod key_pair;
mod session;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub struct ServerOptions {
    pub remote_dwn: Url,
    pub in_memory: bool,
    pub port: u16,
}

pub async fn run_server(opts: ServerOptions) -> anyhow::Result<()> {
    let port = opts.port;

    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));
    let domain_encoded = domain.replace(":", "%3A");
    let did = Did {
        method_name: MethodName("web".to_string()),
        method_id: MethodId(domain_encoded),
    };
    info!("Running server as {did}");

    let identity = Identity::self_signed_builder()
        .subject_alt_names([domain.clone()])
        .from_now_utc()
        .validity_days(14)
        .build()?;
    let cert_hash = identity.certificate_chain().as_slice()[0].hash();

    let cfg = ServerConfig::builder()
        .with_bind_default(port)
        .with_identity(identity)
        .max_idle_timeout(Some(Duration::from_mins(2)))?
        .build();
    let endpoint = Endpoint::server(cfg).context("create wtranspart endpoint")?;

    let vc = key_pair::get_or_create_key(opts.in_memory)?;

    let spawner_opts = SpawnerOptions {
        did: did.clone(),
        domain,
        in_memory: opts.in_memory,
        remote: opts.remote_dwn,
        vc: vc.clone(),
    };

    tokio::spawn(async move {
        // Wait for did:web route to come online.
        tokio::time::sleep(Duration::from_secs(1)).await;

        loop {
            let spawner = match SessionSpawner::new(spawner_opts.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create session spawner: {e:?}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            if let Err(e) = spawner.init_space_host(cert_hash.to_string()).await {
                error!("Failed to init space host: {e:?}");
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            };

            info!("WebTransport listening on port {port}");
            loop {
                let incoming = endpoint.accept().await;
                info!("Incoming session");
                let spawner = spawner.clone();
                tokio::spawn(async move {
                    if let Err(e) = spawner.handle_session(incoming).await {
                        error!("Handling error: {e:?}");
                    }
                });
            }
        }
    });

    let app = Router::new().route(
        "/.well-known/did.json",
        axum::routing::get(|| async move {
            let doc = Document {
                id: did.clone(),
                also_known_as: None,
                assertion_method: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                    path: RelativeDidUrlPath::Empty,
                    query: None,
                    fragment: Some(KEY_FRAGMENT.to_string()),
                })]),
                authentication: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
                    path: RelativeDidUrlPath::Empty,
                    query: None,
                    fragment: Some(KEY_FRAGMENT.to_string()),
                })]),
                capability_delegation: None,
                capability_invocation: None,
                controller: None,
                key_agreement: None,
                service: None,
                verification_method: Some(vec![VerificationMethodMap {
                    id: DidUrl {
                        did: did.clone(),
                        fragment: Some(KEY_FRAGMENT.to_string()),
                        query: None,
                        path_abempty: None,
                    },
                    controller: did,
                    typ: "JsonWebKey2020".to_string(),
                    public_key_multibase: None,
                    public_key_jwk: Some(vc.public().to_jwk()),
                }]),
            };

            Json(doc)
        }),
    );

    info!("HTTP listening on port {port}");

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
