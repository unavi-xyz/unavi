use tokio::sync::mpsc::UnboundedSender;

use crate::update_loop::IncomingEvent;

pub struct GlobalContext {
    pub sender: UnboundedSender<IncomingEvent>,
    pub world_host_did: String,
}
