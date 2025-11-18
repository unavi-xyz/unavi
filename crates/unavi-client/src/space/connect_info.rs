use std::str::FromStr;

use anyhow::Context;
use bevy::prelude::*;
use dwn::Actor;
use serde::Deserialize;
use unavi_constants::{WP_VERSION, protocols::SPACE_HOST_PROTOCOL};
use wtransport::tls::Sha256Digest;
use xdid::{core::did::Did, methods::web::reqwest::Url};

use crate::space::connect::state::{ConnectionAttempt, ConnectionState, ConnectionTasks};

#[derive(Component, Clone)]
#[require(ConnectionState, ConnectionAttempt, ConnectionTasks)]
pub struct ConnectInfo {
    pub connect_url: Url,
    pub cert_hash: Sha256Digest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedConnectInfo {
    url: String,
    cert_hash: String,
}

pub async fn fetch_connect_info(
    actor: &Actor,
    host_did: &Did,
    host_dwn: &Url,
) -> anyhow::Result<Option<ConnectInfo>> {
    let found_urls = actor
        .query()
        .protocol(SPACE_HOST_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("connect-url".to_string())
        .target(host_did)
        .send(host_dwn)
        .await
        .context("query connect url")?;

    for found in found_urls {
        let record_id = found.entry().record_id.clone();

        let data = match found.into_data() {
            Some(d) => d,
            None => {
                let Some(read) = actor
                    .read(record_id)
                    .target(host_did)
                    .send(host_dwn)
                    .await?
                else {
                    warn!("connect url record not found");
                    continue;
                };
                let Some(data) = read.into_data() else {
                    warn!("connect url data not found");
                    continue;
                };
                data
            }
        };

        let parsed: ParsedConnectInfo = serde_json::from_slice(&data)?;

        let connect_url = Url::parse(&parsed.url)?;
        let cert_hash = Sha256Digest::from_str(&parsed.cert_hash)?;

        return Ok(Some(ConnectInfo {
            connect_url,
            cert_hash,
        }));
    }

    Ok(None)
}
