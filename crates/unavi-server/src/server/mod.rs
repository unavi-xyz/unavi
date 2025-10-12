use std::sync::Arc;

use xwt_wtransport::wtransport::endpoint::IncomingSession;

pub struct Server {}

pub async fn handle(server: Arc<Server>, incoming: IncomingSession) -> anyhow::Result<()> {
    Ok(())
}
