use dwn::core::message::descriptor::Descriptor;
use tracing::debug;
use unavi_constants::protocols::SPACE_HOST_PROTOCOL;
use unavi_server_service::{ControlService, Player, RpcResult};

use crate::session::{ServerContext, TICKRATE};

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
        debug!("Validating space {id}");

        let found = self
            .ctx
            .actor
            .read(id.clone())
            .send_remote()
            .await
            .map_err(|e| format!("error reading space record: {e}"))?
            .ok_or_else(|| "space not found".to_string())?;

        let Descriptor::RecordsWrite(found_write) = &found.entry().descriptor else {
            debug!("Space not a write record");
            return Err("space not found".to_string());
        };

        if found_write.published != Some(true)
            || found_write.protocol.as_deref() != Some(SPACE_HOST_PROTOCOL)
            || found_write.protocol_path.as_deref() != Some("space")
        {
            debug!("Invalid space write record");
            return Err("space not found".to_string());
        }

        // Add player to space.
        {
            let mut players = self.ctx.players.write().await;
            let Some(player) = players.get_mut(&self.player_id) else {
                return Ok(());
            };
            if player.spaces.len() > MAX_SPACES_PER_PLAYER {
                return Err("joined too many spaces".to_string());
            }

            let mut spaces = self.ctx.spaces.write().await;
            let space = spaces.entry(id.clone()).or_default();
            if space.players.len() >= space.max_players {
                return Err("space is full".to_string());
            }

            player.spaces.insert(id);
            space.players.insert(self.player_id);
        }

        Ok(())
    }
    async fn leave_space(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_SPACE_ID_LEN {
            return Err("id too long".to_string());
        }

        {
            let mut players = self.ctx.players.write().await;
            let Some(player) = players.get_mut(&self.player_id) else {
                return Ok(());
            };
            player.spaces.remove(&id);
        }

        {
            let mut spaces = self.ctx.spaces.write().await;
            if let Some(space) = spaces.get_mut(&id) {
                space.players.remove(&self.player_id);
                if space.players.is_empty() {
                    spaces.remove(&id);
                }
            }
        }

        Ok(())
    }
    async fn spaces(self, _: tarpc::context::Context) -> Vec<String> {
        let players = self.ctx.players.read().await;
        let Some(player) = players.get(&self.player_id) else {
            return Vec::new();
        };
        player.spaces.iter().cloned().collect()
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
        // Validate tickrate is at least the server minimum.
        if tickrate_ms < TICKRATE.as_millis() as u64 {
            return Err(format!(
                "tickrate must be at least {} ms",
                TICKRATE.as_millis()
            ));
        }

        let tickrate = std::time::Duration::from_millis(tickrate_ms);

        let mut player_tickrates = self.ctx.player_tickrates.write().await;
        player_tickrates
            .entry(self.player_id)
            .or_default()
            .insert(player_id, tickrate);

        Ok(())
    }
}
