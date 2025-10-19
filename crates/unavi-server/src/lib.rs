use std::{net::SocketAddr, sync::LazyLock};

use anyhow::Context;
use axum::{Json, Router};
use directories::ProjectDirs;
use p256::pkcs8::EncodePrivateKey;
use spki::der::Encode;
use tracing::{error, info};
use xdid::core::{
    did::{Did, MethodId, MethodName},
    did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
    document::{Document, VerificationMethod, VerificationMethodMap},
};
use xwt_wtransport::wtransport;

use crate::{cert::CertRes, server::Server};

mod cert;
mod server;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.config_dir()).expect("config dir");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
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

    let server_handler = Server::new().await?;

    tokio::spawn(async move {
        info!("WebTransport listening on {addr}");
        loop {
            let incoming = endpoint.accept().await;
            let svr = server_handler.clone();
            tokio::spawn(async move {
                if let Err(e) = svr.handle(incoming).await {
                    error!("Handling error: {e:?}");
                }
            });
        }
    });

    let app = Router::new().route("/.well-known/did.json", axum::routing::get(serve_did_json));

    info!("HTTP listening on {addr}");

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn serve_did_json() -> Json<Document> {
    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".into());
    let did = Did {
        method_name: MethodName("web".to_string()),
        method_id: MethodId(domain),
    };

    let doc = Document {
        id: did.clone(),
        also_known_as: None,
        assertion_method: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
            path: RelativeDidUrlPath::Empty,
            query: None,
            fragment: Some("owner".to_string()),
        })]),
        authentication: None,
        capability_delegation: None,
        capability_invocation: None,
        controller: None,
        key_agreement: None,
        service: None,
        verification_method: Some(vec![VerificationMethodMap {
            id: DidUrl {
                did: did.clone(),
                fragment: Some("owner".to_string()),
                query: None,
                path_abempty: String::new(),
            },
            controller: did,
            typ: "JsonWebKey2020".to_string(),
            public_key_multibase: None,
            public_key_jwk: None,
        }]),
    };

    Json(doc)
}
