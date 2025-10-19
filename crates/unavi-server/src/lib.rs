use std::{net::SocketAddr, sync::LazyLock};

use anyhow::Context;
use directories::ProjectDirs;
use tracing::{error, info};
use xwt_wtransport::wtransport;

use crate::{cert::CertRes, server::Server};

mod cert;
mod server;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-server").expect("project dirs");
    std::fs::create_dir_all(dirs.config_dir()).expect("config dir");
    dirs
});

pub async fn run_server(addr: SocketAddr) -> anyhow::Result<()> {
    let CertRes { cert, private_key } = cert::get_or_generate_cert().await.context("get cert")?;

    let private_key = wtransport::tls::PrivateKey::from_der_pkcs8(private_key);
    let cert = wtransport::tls::Certificate::from_der(cert).context("create tls certificate")?;
    let cert_chain = wtransport::tls::CertificateChain::single(cert);
    let identity = wtransport::Identity::new(cert_chain, private_key);

    let cfg = wtransport::ServerConfig::builder()
        .with_bind_address(addr)
        .with_identity(identity)
        .build();
    let endpoint = wtransport::Endpoint::server(cfg).context("create wtranspart endpoint")?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let svr = Server::new().await?;

    info!("UNAVI server running on {addr}");

    loop {
        let incoming = endpoint.accept().await;

        let svr = svr.clone();
        tokio::spawn(async move {
            if let Err(e) = svr.handle(incoming).await {
                error!("Handling error: {e:?}");
            }
        });
    }
}
