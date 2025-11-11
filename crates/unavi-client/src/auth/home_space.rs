use std::str::FromStr;

use anyhow::{Context, bail};
use bevy::{ecs::world::CommandQueue, prelude::*};
use dwn::{
    Actor,
    core::message::mime::{APPLICATION_JSON, TEXT_PLAIN},
};
use serde::Deserialize;
use serde_json::json;
use unavi_constants::{
    SPACE_HOST_DID, WP_VERSION,
    protocols::{HOME_SPACE_DEFINITION, HOME_SPACE_PROTOCOL, SPACE_HOST_PROTOCOL},
    schemas::SPACE_SCHEMA,
};
use wtransport::tls::Sha256Digest;
use xdid::{core::did::Did, methods::web::reqwest::Url};

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::join::{ConnectInfo, JoinSpace},
};

use super::record_ref_url::parse_record_ref_url;

pub async fn join_home_space(actor: Actor) -> anyhow::Result<()> {
    let home_definition = serde_json::from_slice(HOME_SPACE_DEFINITION)?;
    actor
        .configure_protocol(WP_VERSION, home_definition)
        .process()
        .await?;

    let found_homes = actor
        .query()
        .protocol(HOME_SPACE_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("home".to_string())
        .process()
        .await?;

    let mut connect_info = None;

    for home in found_homes {
        let record_id = home.entry().record_id.clone();
        let data = match home.into_data() {
            Some(d) => d,
            None => {
                let Some(read) = actor.read(record_id).process().await? else {
                    continue;
                };
                let Some(data) = read.into_data() else {
                    continue;
                };
                data
            }
        };

        let record_ref_url = String::from_utf8(data)?;
        let (host_did, space_record_id) = parse_record_ref_url(&record_ref_url)?;

        // TODO: Fetch from did document
        let host_dwn = actor.remote.clone().unwrap();

        let Some((url, cert_hash)) = fetch_connect_url(&actor, &host_did, &host_dwn)
            .await
            .context("fetch connect url")?
        else {
            continue;
        };

        connect_info = Some(ConnectInfo {
            url,
            cert_hash,
            space_id: space_record_id,
        });
        break;
    }

    let space_host =
        Did::from_str(SPACE_HOST_DID).map_err(|_| anyhow::anyhow!("failed to parse space host"))?;

    let connect_info = match connect_info {
        Some(c) => c,
        None => {
            let data = json!({
                "name": format!("{}'s Home", actor.did),
            });

            // TODO: Fetch from did document
            let host_dwn = actor.remote.clone().unwrap();

            let home_record_id = actor
                .write()
                .protocol(
                    SPACE_HOST_PROTOCOL.to_string(),
                    WP_VERSION,
                    "space".to_string(),
                )
                .schema(SPACE_SCHEMA.to_string())
                .data(APPLICATION_JSON, data.to_string().into_bytes())
                .published(true)
                .target(&space_host)
                .send(&host_dwn)
                .await
                .context("write space")?;

            let record_ref_url = format!(
                "{}?service=dwn&relativeRef=/records/{}",
                space_host, home_record_id
            );

            actor
                .write()
                .protocol(
                    HOME_SPACE_PROTOCOL.to_string(),
                    WP_VERSION,
                    "home".to_string(),
                )
                .data(TEXT_PLAIN, record_ref_url.into_bytes())
                .process()
                .await
                .context("write home")?;

            let Some((url, cert_hash)) = fetch_connect_url(&actor, &space_host, &host_dwn)
                .await
                .context("fetch connect url")?
            else {
                bail!("host connect url not found")
            };

            ConnectInfo {
                url,
                cert_hash,
                space_id: home_record_id,
            }
        }
    };

    info!("Got home: {}@{}", connect_info.space_id, connect_info.url);

    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::trigger(JoinSpace(connect_info)));

    ASYNC_COMMAND_QUEUE.0.send(commands)?;

    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedConnectInfo {
    url: String,
    cert_hash: String,
}

async fn fetch_connect_url(
    actor: &Actor,
    host_did: &Did,
    host_dwn: &Url,
) -> anyhow::Result<Option<(Url, Sha256Digest)>> {
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

        let url = Url::parse(&parsed.url)?;
        let cert_hash = Sha256Digest::from_str(&parsed.cert_hash)?;

        return Ok(Some((url, cert_hash)));
    }

    Ok(None)
}
