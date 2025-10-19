use bevy::prelude::*;
use dwn::{
    Actor,
    core::message::{Version, mime::TEXT_PLAIN},
};

const WP_PREFIX: &str = "https://wired-protocol.org/v0/";
const WP_VERSION: Version = Version::new(0, 1, 0);

const PROTOCOL_PREFIX: &str = constcat::concat!(WP_PREFIX, "protocols/");
const SCHEMA_PREFIX: &str = constcat::concat!(WP_PREFIX, "schemas/");

const HOME_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "home-world.json");
const WORLD_HOST_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "world-host.json");

const SERVER_INFO_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "server-info.json");
const WORLD_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "world.json");

const HOME_DEFINITION: &[u8] = include_bytes!("../../../../protocol/dwn/protocols/home-world.json");

pub async fn join_home_world(actor: Actor) -> anyhow::Result<()> {
    let home_definition = serde_json::from_slice(HOME_DEFINITION)?;
    actor
        .configure_protocol(WP_VERSION, home_definition)
        .process()
        .await?;

    let found_homes = actor
        .query()
        .protocol(HOME_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("uri".to_string())
        .process()
        .await?;
    info!("Found {} homes", found_homes.len());

    let mut home_uri = None;

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
        let data = String::from_utf8(data)?;
        home_uri = Some(data);
        break;
    }

    let home_uri = match home_uri {
        Some(h) => h,
        None => {
            let uri = "todo".to_string();

            actor
                .write()
                .protocol(HOME_PROTOCOL.to_string(), WP_VERSION, "uri".to_string())
                .data(TEXT_PLAIN, uri.into_bytes())
                .process()
                .await?;

            info!("Created new home");

            todo!()
        }
    };

    info!("Home URI: {home_uri}");

    Ok(())
}
