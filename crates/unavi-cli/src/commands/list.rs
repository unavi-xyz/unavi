use std::str::FromStr;

use anyhow::Result;
use dwn::Actor;
use serde_json::Value;
use tracing::info;
use unavi_constants::{SPACE_HOST_DID, protocols::SPACE_HOST_PROTOCOL};
use xdid::core::did::Did;

pub async fn list_spaces(actor: &Actor) -> Result<()> {
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    let found = actor
        .query()
        .protocol(SPACE_HOST_PROTOCOL.to_string())
        .protocol_path("space".to_string())
        .target(&space_host)
        .send_remote()
        .await?;

    if found.is_empty() {
        info!("No spaces found");
        return Ok(());
    }

    info!("Found {} spaces:", found.len());

    for record in found {
        let id = record.entry().record_id.clone();

        let Some(full) = actor
            .read(id.clone())
            .target(&space_host)
            .send_remote()
            .await?
        else {
            continue;
        };

        let Some(data) = full.data() else {
            continue;
        };

        let value = serde_json::from_slice::<Value>(data)?;
        info!("{id}: {value:#?}");
    }

    Ok(())
}
