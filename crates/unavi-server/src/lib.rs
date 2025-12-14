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

use crate::{
    internal_msg::InternalMessage,
    session::{KEY_FRAGMENT, SessionSpawner, SpawnerOptions},
};

const MAX_IDLE_TIMEOUT: Duration = Duration::from_mins(2);
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

const INITIAL_WEB_TRANSPORT_DELAY: Duration = Duration::from_secs(2);
const SPACE_RETRY_DELAY: Duration = Duration::from_secs(30);

mod internal_msg;
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

fn create_did(port: u16) -> (Did, String) {
    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));
    let domain_encoded = domain.replace(':', "%3A");
    let did = Did {
        method_name: MethodName("web".into()),
        method_id: MethodId(domain_encoded),
    };
    (did, domain)
}

fn create_server_identity(domain: &str) -> anyhow::Result<(Identity, String)> {
    let identity = Identity::self_signed_builder()
        .subject_alt_names([domain])
        .from_now_utc()
        .validity_days(14)
        .build()?;
    let cert_hash = identity.certificate_chain().as_slice()[0]
        .hash()
        .to_string();
    Ok((identity, cert_hash))
}

fn create_server_config(
    identity: Identity,
    port: u16,
) -> anyhow::Result<Endpoint<wtransport::endpoint::endpoint_side::Server>> {
    let cfg = ServerConfig::builder()
        .with_bind_default(port)
        .with_identity(identity)
        .max_idle_timeout(Some(MAX_IDLE_TIMEOUT))?
        .keep_alive_interval(Some(KEEP_ALIVE_INTERVAL))
        .build();
    Endpoint::server(cfg).context("create wtransport endpoint")
}

fn create_did_document_route(did: Did, vc: &impl DidKeyPair) -> Router {
    let vc_public = vc.public().to_jwk();
    Router::new().route(
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
                service: None,
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
}

/// Run the UNAVI server.
///
/// # Errors
///
/// Returns an error if server initialization or startup fails.
pub async fn run_server(opts: ServerOptions) -> anyhow::Result<()> {
    let port = opts.port;

    let (did, domain) = create_did(port);
    info!("Running server as {did}");

    let (identity, cert_hash) = create_server_identity(&domain)?;
    let endpoint = create_server_config(identity, port)?;

    let vc = key_pair::get_or_create_key(opts.in_memory)?;

    let (msg_tx, msg_rx) = flume::bounded(16);

    let spawner_opts = SpawnerOptions {
        did: did.clone(),
        domain,
        in_memory: opts.in_memory,
        msg_tx: msg_tx.clone(),
        remote: opts.remote_dwn,
        vc: vc.clone(),
    };

    // Internal message handler.
    tokio::spawn(async move {
        internal_msg::internal_message_handler(msg_rx).await;
    });

    // WebTransport handler.
    tokio::spawn(async move {
        // Wait for did:web route to come online.
        tokio::time::sleep(INITIAL_WEB_TRANSPORT_DELAY).await;

        loop {
            let spawner = match SessionSpawner::new(spawner_opts.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create session spawner: {e:?}");
                    tokio::time::sleep(SPACE_RETRY_DELAY).await;
                    continue;
                }
            };

            if let Err(e) = msg_tx
                .send_async(InternalMessage::SetActor(spawner.ctx.actor.clone()))
                .await
            {
                error!("Failed to set actor: {e:?}");
                tokio::time::sleep(SPACE_RETRY_DELAY).await;
                continue;
            }

            if let Err(e) = spawner.init_space_host(cert_hash.clone()).await {
                error!("Failed to init space host: {e:?}");
                tokio::time::sleep(SPACE_RETRY_DELAY).await;
                continue;
            }

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

    let app = create_did_document_route(did, &vc);

    info!("HTTP listening on port {port}");

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
