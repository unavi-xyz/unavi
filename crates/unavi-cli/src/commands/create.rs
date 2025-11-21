use std::str::FromStr;

use anyhow::Result;
use dwn::{Actor, core::message::mime::APPLICATION_JSON};
use serde_json::json;
use tracing::info;
use unavi_constants::{
    SPACE_HOST_DID, WP_VERSION, protocols::SPACE_HOST_PROTOCOL, schemas::SPACE_SCHEMA,
};
use xdid::core::did::Did;

pub async fn create_space(actor: &Actor) -> Result<()> {
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    let id = actor
        .write()
        .protocol(
            SPACE_HOST_PROTOCOL.to_string(),
            WP_VERSION,
            "space".to_string(),
        )
        .schema(SPACE_SCHEMA.to_string())
        .data(
            APPLICATION_JSON,
            json!({
                "name": "New Space"
            })
            .to_string()
            .into_bytes(),
        )
        .published(true)
        .target(&space_host)
        .send_remote()
        .await?;

    info!("Created new space: {id}");

    Ok(())
}
