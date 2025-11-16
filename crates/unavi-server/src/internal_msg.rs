use std::collections::HashMap;

use anyhow::bail;
use dwn::{
    Actor,
    core::message::{descriptor::Descriptor, mime::APPLICATION_JSON},
};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};
use unavi_constants::{WP_VERSION, protocols::SPACE_HOST_PROTOCOL, schemas::ServerInfo};

use crate::session::DEFAULT_MAX_PLAYERS_PER_SPACE;

pub enum InternalMessage {
    SetActor(Actor),
    SetPlayerCount { record_id: String, count: usize },
}

pub async fn internal_message_handler(msg_rx: &mut Receiver<InternalMessage>) {
    let mut handler = InternalMessageHandler::default();

    while let Some(msg) = msg_rx.recv().await {
        match msg {
            InternalMessage::SetActor(a) => {
                handler.actor = Some(a);
            }
            InternalMessage::SetPlayerCount { record_id, count } => {
                if let Err(e) = handler.set_player_count(record_id, count).await {
                    error!("Failed to set player count: {e:?}");
                }
            }
        }
    }
}

#[derive(Default)]
struct InternalMessageHandler {
    actor: Option<Actor>,
    cached_info_ids: HashMap<String, String>,
}

impl InternalMessageHandler {
    async fn set_player_count(&mut self, record_id: String, count: usize) -> anyhow::Result<()> {
        let Some(actor) = &self.actor else {
            bail!("actor not set")
        };

        let Some(found) = actor.read(record_id.clone()).send_remote().await? else {
            bail!("space record not found");
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
            .protocol_path("space/server-info".to_string())
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
                "space/server-info".to_string(),
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

        info!("Set player count: {record_id}/{info_id} {count}");

        if count == 0 {
            self.cached_info_ids.remove(&record_id);
            if self.cached_info_ids.capacity() > 3 * self.cached_info_ids.len() {
                self.cached_info_ids.shrink_to_fit();
            }
        } else {
            self.cached_info_ids.insert(record_id, info_id);
        };

        Ok(())
    }
}
