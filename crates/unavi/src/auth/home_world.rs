use std::str::FromStr;

use anyhow::{Context, bail};
use bevy::{ecs::world::CommandQueue, prelude::*};
use dwn::{Actor, core::message::mime::APPLICATION_JSON};
use serde::{Deserialize, Serialize};
use serde_json::json;
use unavi_constants::{
    WORLD_HOST_DID, WP_VERSION,
    protocols::{HOME_WORLD_DEFINITION, HOME_WORLD_PROTOCOL, WORLD_HOST_PROTOCOL},
    schemas::{REMOTE_RECORD_SCHEMA, WORLD_SCHEMA},
};
use wtransport::tls::Sha256Digest;
use xdid::{core::did::Did, methods::web::reqwest::Url};

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    join_world::{ConnectInfo, JoinWorld},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteRecord {
    did: Did,
    record_id: String,
}

pub async fn join_home_world(actor: Actor) -> anyhow::Result<()> {
    let home_definition = serde_json::from_slice(HOME_WORLD_DEFINITION)?;
    actor
        .configure_protocol(WP_VERSION, home_definition)
        .process()
        .await?;

    let found_homes = actor
        .query()
        .protocol(HOME_WORLD_PROTOCOL.to_string())
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

        let rr = serde_json::from_slice::<RemoteRecord>(&data)?;

        // TODO: Fetch from did document
        let host_dwn = actor.remote.clone().unwrap();

        let Some((url, cert_hash)) = fetch_connect_url(&actor, &rr.did, &host_dwn)
            .await
            .context("fetch connect url")?
        else {
            continue;
        };

        connect_info = Some(ConnectInfo {
            url,
            cert_hash,
            world_id: rr.record_id,
        });
        break;
    }

    let world_host =
        Did::from_str(WORLD_HOST_DID).map_err(|_| anyhow::anyhow!("failed to parse world host"))?;

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
                    WORLD_HOST_PROTOCOL.to_string(),
                    WP_VERSION,
                    "world".to_string(),
                )
                .schema(WORLD_SCHEMA.to_string())
                .data(APPLICATION_JSON, data.to_string().into_bytes())
                .target(&world_host)
                .send(&host_dwn)
                .await
                .context("write world")?;

            let data = json!({
                "did": world_host,
                "recordId": home_record_id,
            });

            actor
                .write()
                .protocol(
                    HOME_WORLD_PROTOCOL.to_string(),
                    WP_VERSION,
                    "home".to_string(),
                )
                .schema(REMOTE_RECORD_SCHEMA.to_string())
                .data(APPLICATION_JSON, data.to_string().into_bytes())
                .process()
                .await
                .context("write home")?;

            let Some((url, cert_hash)) = fetch_connect_url(&actor, &world_host, &host_dwn)
                .await
                .context("fetch connect url")?
            else {
                bail!("host connect url not found")
            };

            ConnectInfo {
                url,
                cert_hash,
                world_id: home_record_id,
            }
        }
    };

    info!("Got home: {}@{}", connect_info.world_id, connect_info.url);

    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::trigger(JoinWorld(connect_info)));

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
        .protocol(WORLD_HOST_PROTOCOL.to_string())
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
