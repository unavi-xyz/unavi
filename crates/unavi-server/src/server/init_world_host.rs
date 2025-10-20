use dwn::core::message::mime::TEXT_PLAIN;
use unavi_constants::{
    WP_VERSION,
    protocols::{WORLD_HOST_DEFINITION, WORLD_HOST_PROTOCOL},
};

use crate::server::Server;

impl Server {
    pub async fn init_world_host(&self) -> anyhow::Result<()> {
        let host_def = serde_json::from_slice(WORLD_HOST_DEFINITION)?;

        self.actor
            .configure_protocol(WP_VERSION, host_def)
            .process()
            .await?;

        self.actor
            .write()
            .protocol(
                WORLD_HOST_PROTOCOL.to_string(),
                WP_VERSION,
                "connect-url".to_string(),
            )
            .data(TEXT_PLAIN, self.domain.clone().into_bytes())
            .process()
            .await?;

        Ok(())
    }
}
