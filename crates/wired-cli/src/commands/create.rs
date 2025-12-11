use anyhow::Result;
use dwn::{Actor, core::message::mime::APPLICATION_JSON};
use tracing::info;
use unavi_constants::WP_VERSION;
use wired_protocol::{SPACE_PROTOCOL, SPACE_SCHEMA, Space};

pub async fn create_space(actor: &Actor) -> Result<()> {
    let space_id = actor
        .write()
        .protocol(SPACE_PROTOCOL.to_string(), WP_VERSION, "space".to_string())
        .schema(SPACE_SCHEMA.to_string())
        .data(
            APPLICATION_JSON,
            serde_json::to_vec(&Space {
                name: Some("New Space".to_string()),
                ..Default::default()
            })?,
        )
        .published(true)
        .send_remote()
        .await?;

    info!("Created new space: {space_id}");

    Ok(())
}
