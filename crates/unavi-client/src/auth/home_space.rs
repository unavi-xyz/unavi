use std::str::FromStr;

use anyhow::Context;
use bevy::{ecs::world::CommandQueue, prelude::*};
use dwn::{
    Actor,
    core::message::mime::{APPLICATION_JSON, TEXT_PLAIN},
};
use serde_json::json;
use unavi_constants::{
    SPACE_HOST_DID, WP_VERSION,
    protocols::{HOME_SPACE_DEFINITION, HOME_SPACE_PROTOCOL, SPACE_HOST_PROTOCOL},
    schemas::SPACE_SCHEMA,
};
use xdid::core::{did::Did, did_url::DidUrl};

use crate::{async_commands::ASYNC_COMMAND_QUEUE, space::Space};

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

    let mut space_url = None;

    for home in found_homes {
        let home_id = home.entry().record_id.clone();
        let data = match home.into_data() {
            Some(d) => d,
            None => {
                let Some(read) = actor.read(home_id).process().await? else {
                    continue;
                };
                let Some(data) = read.into_data() else {
                    continue;
                };
                data
            }
        };

        space_url = Some(DidUrl::from_str(&String::from_utf8(data)?)?);
        break;
    }

    let space_host =
        Did::from_str(SPACE_HOST_DID).map_err(|_| anyhow::anyhow!("failed to parse space host"))?;

    let space_url = match space_url {
        Some(c) => c,
        None => {
            let data = json!({
                "name": format!("{}'s Home", actor.did),
            });

            // TODO: Fetch from did document
            let host_dwn = actor.remote.clone().unwrap();

            let space_id = actor
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

            let space_url = DidUrl {
                did: space_host,
                query: Some(format!("service=dwn&relativeRef=/records/{space_id}")),
                fragment: None,
                path_abempty: None,
            };

            actor
                .write()
                .protocol(
                    HOME_SPACE_PROTOCOL.to_string(),
                    WP_VERSION,
                    "home".to_string(),
                )
                .data(TEXT_PLAIN, space_url.to_string().into_bytes())
                .process()
                .await
                .context("write home")?;

            space_url
        }
    };

    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([Space {
        url: space_url,
    }]));
    ASYNC_COMMAND_QUEUE.0.send(commands)?;

    Ok(())
}
