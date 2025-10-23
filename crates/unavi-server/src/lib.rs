use std::{net::SocketAddr, sync::LazyLock, time::Duration};

use anyhow::Context;
use axum::{Json, Router};
use directories::ProjectDirs;
use p256::pkcs8::EncodePrivateKey;
use spki::der::Encode;
use tracing::{error, info};
use xdid::{
    core::{
        did::{Did, MethodId, MethodName},
        did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
        document::{Document, VerificationMethod, VerificationMethodMap},
    },
    methods::key::{DidKeyPair, PublicKey},
};
use xwt_wtransport::wtransport;

use crate::{
    cert::CertRes,
    wt_server::{KEY_FRAGMENT, WtServer},
};

mod cert;
mod key_pair;
mod wt_server;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.config_dir()).expect("config dir");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
    let domain = std::env::var("DOMAIN")
        .unwrap_or_else(|_| addr.to_string())
        .replace("127.0.0.1", "localhost");
    let domain_encoded = domain.replace(":", "%3A");
    let did = Did {
        method_name: MethodName("web".to_string()),
        method_id: MethodId(domain_encoded),
    };
    info!("Running server as {did}");

    let CertRes { cert, key } = cert::get_or_generate_cert().await.context("get cert")?;

    let private_key =
        wtransport::tls::PrivateKey::from_der_pkcs8(key.to_pkcs8_der()?.to_bytes().to_vec());
    let cert =
        wtransport::tls::Certificate::from_der(cert.to_der()?).context("create tls certificate")?;
    let cert_chain = wtransport::tls::CertificateChain::single(cert);
    let identity = wtransport::Identity::new(cert_chain, private_key);

    let cfg = wtransport::ServerConfig::builder()
        .with_bind_address(addr)
        .with_identity(identity)
        .build();
    let endpoint = wtransport::Endpoint::server(cfg).context("create wtranspart endpoint")?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let vc = key_pair::get_or_create_key()?;

    {
        let did = did.clone();
        let vc = vc.clone();
        tokio::spawn(async move {
            // Wait for did:web route to come online.
            tokio::time::sleep(Duration::from_secs(5)).await;

            loop {
                let server = match WtServer::new(did.clone(), vc.clone(), domain.clone()).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to crate sever handler: {e:?}");
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        continue;
                    }
                };

                if let Err(e) = server.init_world_host().await {
                    error!("Failed to init world host: {e:?}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                };

                info!("WebTransport listening on {addr}");
                loop {
                    let incoming = endpoint.accept().await;
                    let svr = server.clone();
                    tokio::spawn(async move {
                        if let Err(e) = svr.handle(incoming).await {
                            error!("Handling error: {e:?}");
                        }
                    });
                }
            }
        });
    }

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
                        path_abempty: String::new(),
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

    info!("HTTP listening on {addr}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
