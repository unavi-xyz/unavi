use anyhow::Result;
use dwn::Actor;
use serde_json::Value;
use tracing::info;
use wired_protocol::SPACE_PROTOCOL;

pub async fn list_spaces(actor: &Actor) -> Result<()> {
    let found = actor
        .query()
        .protocol(SPACE_PROTOCOL.to_string())
        .protocol_path("space".to_string())
        .send_remote()
        .await?;

    if found.is_empty() {
        info!("No spaces found");
        return Ok(());
    }

    info!("Found {} spaces:", found.len());

    for record in found {
        let id = record.entry().record_id.clone();

        let Some(full) = actor.read(id.clone()).send_remote().await? else {
            continue;
        };

        let Some(data) = full.data() else {
            continue;
        };

        let value = serde_json::from_slice::<Value>(data)?;
        info!("{id}: {}", serde_json::to_string_pretty(&value)?);
    }

    Ok(())
}
