use tokio::sync::mpsc::UnboundedSender;

use crate::commands::SessionMessage;

pub struct GlobalContext {
    pub sender: UnboundedSender<SessionMessage>,
    pub world_host_did: String,
}
