use std::str::FromStr;

use anyhow::Context;
use bevy::log::{info, warn};
use iroh::{EndpointAddr, EndpointId};
use xdid::{core::did::Did, resolver::DidResolver};

/// The default WDS DID, compiled at build time.
/// Set via REMOTE_WDS environment variable during build.
const DEFAULT_REMOTE_WDS: &str = match option_env!("REMOTE_WDS") {
    Some(did) => did,
    None => "did:web:localhost%3A5000",
};

pub async fn fetch_remote_host() -> anyhow::Result<Option<EndpointAddr>> {
    let remote_wds_did = Did::from_str(DEFAULT_REMOTE_WDS).context("parse remote wds did")?;

    info!(did = %remote_wds_did, "Resolving remote WDS");
    let values = match fetch_wds_service(&remote_wds_did).await {
        Ok(v) => v,
        Err(err) => {
            warn!(?err, "failed to resolve remote host");
            return Ok(None);
        }
    };

    for v in values {
        match EndpointId::from_str(&v) {
            Ok(id) => {
                info!(%id, "Got remote WDS");
                return Ok(Some(EndpointAddr::new(id).with_relay_url(
                    iroh::defaults::prod::default_na_east_relay().url,
                )));
            }
            Err(e) => {
                warn!(value = %v, err = ?e, "invalid wds service value");
            }
        }
    }

    Ok(None)
}

async fn fetch_wds_service(did: &Did) -> anyhow::Result<Vec<String>> {
    let resolver = DidResolver::new()?;
    let remote_doc = resolver.resolve(did).await?;

    for service in remote_doc.service.unwrap_or_default() {
        if service.id == "wds" {
            return Ok(service.typ);
        }
    }

    Ok(Vec::new())
}
