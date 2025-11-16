use std::collections::HashMap;

use anyhow::bail;
use dwn::{
    Actor,
    core::message::{descriptor::Descriptor, mime::APPLICATION_JSON},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use unavi_constants::{WP_VERSION, protocols::SPACE_HOST_PROTOCOL};

use crate::session::DEFAULT_MAX_PLAYERS_PER_SPACE;

pub enum InternalMessage {
    SetActor(Actor),
    SetPlayerCount { record_id: String, count: usize },
}

pub async fn internal_message_handler(
    msg_rx: &mut Receiver<InternalMessage>,
) -> anyhow::Result<()> {
    let mut cached_info_ids = HashMap::new();

    while let Some(msg) = msg_rx.recv().await {
        let mut actor = None;

        match msg {
            InternalMessage::SetActor(a) => {
                actor = Some(a);
            }
            InternalMessage::SetPlayerCount { record_id, count } => {
                let Some(actor) = actor else {
                    continue;
                };

                let Some(found) = actor.read(record_id.clone()).process().await? else {
                    continue;
                };
                let Descriptor::RecordsWrite(found_write) = &found.entry().descriptor else {
                    bail!("player count record not RecordsWrite")
                };
                if found_write.protocol.as_deref() != Some(SPACE_HOST_PROTOCOL)
                    || found_write.protocol_path.as_deref() != Some("space")
                {
                    bail!("player count record not a space")
                }

                let found_info = actor
                    .query()
                    .protocol(SPACE_HOST_PROTOCOL.to_string())
                    .protocol_path("server-info".to_string())
                    .parent_id(record_id.clone())
                    .process()
                    .await?;

                let mut builder = actor.write();

                if let Some(published) = found_write.published {
                    builder = builder.published(published);
                }

                if let Some(info) = found_info.into_iter().next() {
                    builder = builder.record_id(info.entry().record_id.clone());
                }

                let info_id = builder
                    .protocol(
                        SPACE_HOST_PROTOCOL.to_string(),
                        WP_VERSION,
                        "server-info".to_string(),
                    )
                    .context_id(record_id.clone())
                    .data(
                        APPLICATION_JSON,
                        serde_json::to_vec(&ServerInfo {
                            max_players: DEFAULT_MAX_PLAYERS_PER_SPACE,
                            num_players: count,
                        })?,
                    )
                    .process()
                    .await?;

                if count == 0 {
                    cached_info_ids.remove(&record_id);
                    if cached_info_ids.capacity() > 3 * cached_info_ids.len() {
                        cached_info_ids.shrink_to_fit();
                    }
                } else {
                    cached_info_ids.insert(record_id, info_id);
                };
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerInfo {
    max_players: usize,
    num_players: usize,
}
