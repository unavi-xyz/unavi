use std::{fs, path::PathBuf};

use anyhow::Result;
use dwn::{Actor, core::message::mime::APPLICATION_JSON};
use tracing::info;
use unavi_constants::WP_VERSION;
use wired_protocol::{SPACE_PROTOCOL, SPACE_SCHEMA};

pub async fn edit_space(actor: &Actor, id: String, data_path: PathBuf) -> Result<()> {
    // Read JSON data from file.
    let data = fs::read(data_path)?;

    // Validate JSON.
    let _: serde_json::Value = serde_json::from_slice(&data)?;

    actor
        .write()
        .record_id(id.clone())
        .protocol(SPACE_PROTOCOL.to_string(), WP_VERSION, "space".to_string())
        .schema(SPACE_SCHEMA.to_string())
        .data(APPLICATION_JSON, data)
        .published(true)
        .send_remote()
        .await?;

    info!("Updated space: {id}");

    Ok(())
}
