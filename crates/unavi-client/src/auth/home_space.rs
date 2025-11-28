use std::str::FromStr;

use anyhow::Context;
use bevy::{ecs::world::CommandQueue, prelude::*};
use dwn::{
    Actor,
    core::message::mime::{APPLICATION_JSON, TEXT_PLAIN},
};
use unavi_constants::{SPACE_HOST_DID, WP_VERSION};
use wired_protocol::{
    HOME_SPACE_DEFINITION, HOME_SPACE_PROTOCOL, HOST_PROTOCOL, HOSTED_SPACE_SCHEMA,
    SPACE_DEFINITION, SPACE_PROTOCOL,
};
use xdid::core::{did::Did, did_url::DidUrl};

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{
        Space,
        record_ref_url::{new_record_ref_url, parse_record_ref_url},
    },
};

#[allow(clippy::too_many_lines)]
pub async fn join_home_space(actor: &Actor) -> anyhow::Result<()> {
    // Init space protocol.
    let space_definition = serde_json::from_slice(SPACE_DEFINITION)?;

    actor
        .configure_protocol(WP_VERSION, space_definition)
        .process()
        .await?;

    // Init home protocol.
    let home_definition = serde_json::from_slice(HOME_SPACE_DEFINITION)?;

    actor
        .configure_protocol(WP_VERSION, home_definition)
        .process()
        .await?;

    // Fetch or create home.
    let found_homes = actor
        .query()
        .protocol(HOME_SPACE_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("home".to_string())
        .process()
        .await
        .context("query homes")?;

    let mut space_url = None;

    for home in found_homes {
        let home_id = home.entry().record_id.clone();
        let data = if let Some(d) = home.into_data() {
            d
        } else {
            let Some(read) = actor.read(home_id).process().await? else {
                continue;
            };
            let Some(data) = read.into_data() else {
                continue;
            };
            data
        };

        space_url = Some(DidUrl::from_str(&String::from_utf8(data)?)?);
        break;
    }

    let space_url = if let Some(space_url) = space_url {
        space_url
    } else {
        // Create space.
        let data = serde_json::to_vec(&wired_protocol::Space {
            name: Some(format!("{}'s Home", actor.did)),
            ..Default::default()
        })?;

        let space_id = actor
            .write()
            .protocol(SPACE_PROTOCOL.to_string(), WP_VERSION, "space".to_string())
            .data(APPLICATION_JSON, data)
            .process()
            .await
            .context("write space")?;

        let space_url = new_record_ref_url(actor.did.clone(), &space_id);

        // Create home.
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
    };

    // Assume same DID as actor.
    // TODO: generalize
    let space_id = parse_record_ref_url(&space_url)?;

    // Fetch or create hosted instance of home space.
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    let found_hosts = actor
        .query()
        .protocol(SPACE_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("space/host".to_string())
        .parent_id(space_id.to_string())
        .process()
        .await
        .context("query hosted spaces")?;

    let mut hosted_url = None;

    for host in found_hosts {
        let host_id = host.entry().record_id.clone();
        let data = if let Some(d) = host.into_data() {
            d
        } else {
            let Some(read) = actor.read(host_id).process().await? else {
                continue;
            };
            let Some(data) = read.into_data() else {
                continue;
            };
            data
        };

        hosted_url = Some(DidUrl::from_str(&String::from_utf8(data)?)?);
        break;
    }

    let hosted_url = if let Some(url) = hosted_url {
        url
    } else {
        // TODO: Fetch from space host DID
        let host_dwn = actor.remote.clone().expect("value expected");

        // Register space with host.
        let hosted_space_id = actor
            .write()
            .protocol(HOST_PROTOCOL.to_string(), WP_VERSION, "space".to_string())
            .schema(HOSTED_SPACE_SCHEMA.to_string())
            .data(APPLICATION_JSON, space_url.to_string().into_bytes())
            .published(true)
            .target(&space_host)
            .send(&host_dwn)
            .await
            .context("write hosted space")?;

        let hosted_url = new_record_ref_url(space_host, &hosted_space_id);

        // Link to host.
        actor
            .write()
            .protocol(
                SPACE_PROTOCOL.to_string(),
                WP_VERSION,
                "space/host".to_string(),
            )
            .context_id(space_id.to_string())
            .data(TEXT_PLAIN, hosted_url.to_string().into_bytes())
            .process()
            .await
            .context("link hosted space")?;

        hosted_url
    };

    // Connect to hosted space.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([Space::new(
        hosted_url,
    )]));
    ASYNC_COMMAND_QUEUE.0.send(commands)?;

    Ok(())
}
