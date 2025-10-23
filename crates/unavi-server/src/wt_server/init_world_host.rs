use dwn::core::message::mime::APPLICATION_JSON;
use serde_json::json;
use tracing::info;
use unavi_constants::{
    WP_VERSION,
    protocols::{WORLD_HOST_DEFINITION, WORLD_HOST_PROTOCOL},
};
use xdid::methods::web::reqwest::Url;

use crate::wt_server::WtServer;

impl WtServer {
    pub async fn init_world_host(&self, cert_hash: String) -> anyhow::Result<()> {
        let host_def = serde_json::from_slice(WORLD_HOST_DEFINITION)?;

        self.actor
            .configure_protocol(WP_VERSION, host_def)
            .process()
            .await?;

        let http = if self.domain.starts_with("localhost:") {
            "http"
        } else {
            "https"
        };
        let connect_url = Url::parse(&format!("{http}://{}", self.domain))?;
        info!("Publishing connect URL: {connect_url}");

        let prev_connect_url = self
            .actor
            .query()
            .protocol(WORLD_HOST_PROTOCOL.to_string())
            .protocol_version(WP_VERSION)
            .protocol_path("connect-url".to_string())
            .process()
            .await?;

        let prev_record_id = prev_connect_url
            .into_iter()
            .next()
            .map(|v| v.entry().record_id.clone());

        let data = json!({
            "url": connect_url,
            "certHash": cert_hash
        });

        let mut builder = self
            .actor
            .write()
            .protocol(
                WORLD_HOST_PROTOCOL.to_string(),
                WP_VERSION,
                "connect-url".to_string(),
            )
            .data(APPLICATION_JSON, data.to_string().into_bytes())
            // .sign(true) TODO sign + verify in client
            .published(true);

        if let Some(prev_record_id) = prev_record_id {
            builder = builder.record_id(prev_record_id);
        }

        builder.process().await?;

        Ok(())
    }
}
