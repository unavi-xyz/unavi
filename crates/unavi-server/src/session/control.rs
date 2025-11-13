use std::{collections::HashSet, sync::Arc};

use dwn::{Actor, core::message::descriptor::Descriptor};
use tokio::sync::Mutex;
use tracing::debug;
use unavi_constants::protocols::SPACE_HOST_PROTOCOL;
use unavi_server_service::{ControlService, Player, RpcResult};

#[derive(Clone)]
pub struct ControlServer {
    actor: Actor,
    player_id: u64,
    spaces: Arc<Mutex<HashSet<String>>>,
}

impl ControlServer {
    pub fn new(actor: Actor, player_id: u64) -> Self {
        Self {
            actor,
            player_id,
            spaces: Default::default(),
        }
    }
}

const MAX_SPACES_PER_PLAYER: usize = 16;
const MAX_SPACE_ID_LEN: usize = 128;

impl ControlService for ControlServer {
    async fn join_space(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_SPACE_ID_LEN {
            return Err("id too long".to_string());
        }

        // Validate space exists in DWN.
        debug!("Validating space {id}");

        let found = self
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

        let mut spaces = self.spaces.lock().await;
        if spaces.len() > MAX_SPACES_PER_PLAYER {
            return Err("joined too many spaces".to_string());
        }
        spaces.insert(id);

        Ok(())
    }
    async fn leave_space(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_SPACE_ID_LEN {
            return Err("id too long".to_string());
        }

        let mut spaces = self.spaces.lock().await;
        spaces.remove(&id);

        Ok(())
    }
    async fn spaces(self, _: tarpc::context::Context) -> Vec<String> {
        self.spaces.lock().await.iter().cloned().collect()
    }

    async fn players(self, _: tarpc::context::Context) -> Vec<Player> {
        Vec::new()
    }
}
