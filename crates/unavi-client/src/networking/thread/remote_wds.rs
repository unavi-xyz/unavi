use std::str::FromStr;

use anyhow::Context;
use iroh::EndpointId;
use log::{info, warn};
use xdid::{core::did::Did, resolver::DidResolver};

pub async fn fetch_remote_host() -> anyhow::Result<Option<EndpointId>> {
    let remote_wds_did = Did::from_str(
        std::env::var("REMOTE_WDS")
            .as_deref()
            .unwrap_or("did:web:localhost%3A5000"),
    )
    .context("parse remote wds did")?;

    info!("Resolving remote WDS: {remote_wds_did}");
    let values = fetch_wds_service(&remote_wds_did).await?;

    for v in values {
        match EndpointId::from_str(&v) {
            Ok(id) => {
                info!("Got remote WDS: {id}");
                return Ok(Some(id));
            }
            Err(e) => {
                warn!("invalid wds service value {v}: {e:?}");
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
