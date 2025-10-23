use std::{collections::HashSet, sync::Arc};

use dwn::{Actor, core::message::descriptor::Descriptor};
use tokio::sync::Mutex;
use tracing::debug;
use unavi_constants::protocols::WORLD_HOST_PROTOCOL;
use unavi_server_service::{ControlService, Player, RpcResult};

#[derive(Clone)]
pub struct ControlServer {
    actor: Actor,
    worlds: Arc<Mutex<HashSet<String>>>,
}

impl ControlServer {
    pub fn new(actor: Actor) -> Self {
        Self {
            actor,
            worlds: Default::default(),
        }
    }
}

const MAX_WORLDS: usize = 16;
const MAX_WORLD_ID_LEN: usize = 128;

impl ControlService for ControlServer {
    async fn join_world(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_WORLD_ID_LEN {
            return Err("id too long".to_string());
        }

        // Validate world exists in DWN.
        debug!("Validating world {id}");

        let found = self
            .actor
            .read(id.clone())
            .send_remote()
            .await
            .map_err(|e| format!("error reading world record: {e}"))?
            .ok_or_else(|| "world not found".to_string())?;

        let Descriptor::RecordsWrite(found_write) = &found.entry().descriptor else {
            debug!("World not a write record");
            return Err("world not found".to_string());
        };

        if found_write.published != Some(true)
            || found_write.protocol.as_deref() != Some(WORLD_HOST_PROTOCOL)
            || found_write.protocol_path.as_deref() != Some("world")
        {
            debug!("Invalid world write record");
            return Err("world not found".to_string());
        }

        let mut worlds = self.worlds.lock().await;
        if worlds.len() > MAX_WORLDS {
            return Err("joined too many worlds".to_string());
        }
        worlds.insert(id);

        Ok(())
    }
    async fn leave_world(self, _: tarpc::context::Context, id: String) -> RpcResult<()> {
        if id.len() > MAX_WORLD_ID_LEN {
            return Err("id too long".to_string());
        }

        let mut worlds = self.worlds.lock().await;
        worlds.remove(&id);

        Ok(())
    }
    async fn worlds(self, _: tarpc::context::Context) -> Vec<String> {
        self.worlds.lock().await.iter().cloned().collect()
    }

    async fn players(self, _: tarpc::context::Context) -> Vec<Player> {
        Vec::new()
    }
}
