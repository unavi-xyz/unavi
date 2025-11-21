use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use dwn::{Actor, core::message::mime::APPLICATION_JSON};
use tracing::info;
use unavi_constants::{
    SPACE_HOST_DID, WP_VERSION, protocols::SPACE_HOST_PROTOCOL, schemas::SPACE_SCHEMA,
};
use xdid::core::did::Did;

pub async fn edit_space(actor: &Actor, id: String, data_path: PathBuf) -> Result<()> {
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    // Read JSON data from file.
    let data = fs::read(data_path)?;

    // Validate JSON.
    let _: serde_json::Value = serde_json::from_slice(&data)?;

    actor
        .write()
        .record_id(id.clone())
        .protocol(
            SPACE_HOST_PROTOCOL.to_string(),
            WP_VERSION,
            "space".to_string(),
        )
        .schema(SPACE_SCHEMA.to_string())
        .data(APPLICATION_JSON, data)
        .published(true)
        .target(&space_host)
        .send_remote()
        .await?;

    info!("Updated space: {id}");

    Ok(())
}
