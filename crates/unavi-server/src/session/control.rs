use std::sync::Arc;

use dwn::core::message::descriptor::Descriptor;
use tracing::info;
use unavi_constants::protocols::SPACE_HOST_PROTOCOL;
use unavi_server_service::{ControlService, Player, RpcResult};

use crate::{
    internal_msg::InternalMessage,
    session::{ServerContext, TICKRATE},
};

#[derive(Clone)]
pub struct ControlServer {
    ctx: ServerContext,
    player_id: u64,
}

impl ControlServer {
    pub fn new(ctx: ServerContext, player_id: u64) -> Self {
        Self { ctx, player_id }
    }
}

const MAX_SPACES_PER_PLAYER: usize = 16;
const MAX_SPACE_ID_LEN: usize = 128;

impl ControlService for ControlServer {
    async fn tickrate_ms(self, _: tarpc::context::Context) -> u64 {
        TICKRATE.as_millis() as u64
    }
    async fn join_space(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_SPACE_ID_LEN {
            return Err("id too long".to_string());
        }

        // Validate space exists in DWN.
        info!("Validating space {id}");

        let found = self
            .ctx
            .actor
            .read(id.clone())
            .send_remote()
            .await
            .map_err(|e| format!("error reading space record: {e}"))?
            .ok_or_else(|| "space not found".to_string())?;

        let Descriptor::RecordsWrite(found_write) = &found.entry().descriptor else {
            info!("Space not a write record");
            return Err("space not found".to_string());
        };

        if found_write.published != Some(true)
            || found_write.protocol.as_deref() != Some(SPACE_HOST_PROTOCOL)
            || found_write.protocol_path.as_deref() != Some("space")
        {
            info!("Invalid space write record");
            return Err("space not found".to_string());
        }

        // Add player to space.
        let result = self
            .ctx
            .players
            .update_async(&self.player_id, |_, player| {
                if player.spaces.len() > MAX_SPACES_PER_PLAYER {
                    return Err("joined too many spaces".to_string());
                }
                player.spaces.insert(id.clone());
                Ok(())
            })
            .await;

        if let Some(Err(e)) = result {
            return Err(e);
        }
        if result.is_none() {
            return Ok(());
        }

        // Ensure space exists.
        let _ = self
            .ctx
            .spaces
            .entry_async(id.clone())
            .await
            .or_insert_with(Default::default);

        // Try to add player to space.
        let count_result = self
            .ctx
            .spaces
            .update_async(&id, |_, space| {
                if space.players.len() >= space.max_players {
                    return Err("space is full".to_string());
                }
                let changed = space.players.insert(self.player_id);
                Ok((changed, space.players.len()))
            })
            .await;

        match count_result {
            Some(Ok((true, count))) => {
                let _ = self
                    .ctx
                    .msg_tx
                    .send_async(InternalMessage::SetPlayerCount {
                        record_id: id,
                        count,
                    })
                    .await;
            }
            Some(Err(e)) => {
                // Rollback player spaces change.
                let _ = self
                    .ctx
                    .players
                    .update_async(&self.player_id, |_, player| {
                        player.spaces.remove(&id);
                    })
                    .await;
                return Err(e);
            }
            _ => {}
        }

        Ok(())
    }
    async fn leave_space(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_SPACE_ID_LEN {
            return Err("id too long".to_string());
        }

        let player_removed = self
            .ctx
            .players
            .update_async(&self.player_id, |_, player| player.spaces.remove(&id))
            .await;

        if player_removed.is_none() {
            return Ok(());
        }

        let count_result = self
            .ctx
            .spaces
            .update_async(&id, |_, space| {
                let count_changed = space.players.remove(&self.player_id);
                if count_changed {
                    let count = space.players.len();
                    (true, count == 0, count)
                } else {
                    (false, false, 0)
                }
            })
            .await;

        if let Some((true, should_remove, count)) = count_result {
            if should_remove {
                let _ = self.ctx.spaces.remove_async(&id).await;
            }

            self.ctx.update_space_player_count(id, count).await;
        }

        Ok(())
    }
    async fn spaces(self, _: tarpc::context::Context) -> Vec<String> {
        self.ctx
            .players
            .read_async(&self.player_id, |_, player| {
                player.spaces.iter().cloned().collect()
            })
            .await
            .unwrap_or_default()
    }

    async fn players(self, _: tarpc::context::Context) -> Vec<Player> {
        Vec::new()
    }

    async fn set_player_tickrate(
        self,
        _: tarpc::context::Context,
        player_id: u64,
        tickrate_ms: u64,
    ) -> RpcResult<()> {
        use scc::HashMap as SccHashMap;

        // Validate tickrate is at least the server minimum.
        if tickrate_ms < TICKRATE.as_millis() as u64 {
            return Err(format!(
                "tickrate must be at least {} ms",
                TICKRATE.as_millis()
            ));
        }

        let tickrate = std::time::Duration::from_millis(tickrate_ms);

        // Ensure the map exists for this player.
        let _ = self
            .ctx
            .player_tickrates
            .entry_async(self.player_id)
            .await
            .or_insert_with(|| Arc::new(SccHashMap::new()));

        // Now insert the tickrate.
        let rates = self
            .ctx
            .player_tickrates
            .read_async(&self.player_id, |_, rates| rates.clone())
            .await;

        if let Some(rates) = rates {
            let _ = rates.insert_async(player_id, tickrate).await;
        } else {
            return Err("player not found".to_string());
        }

        Ok(())
    }
}
