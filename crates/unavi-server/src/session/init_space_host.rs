use dwn::core::message::mime::APPLICATION_JSON;
use tracing::info;
use unavi_constants::WP_VERSION;
use wired_protocol::{HOST_DEFINITION, HOST_PROTOCOL, Server};
use xdid::methods::web::reqwest::Url;

use crate::session::SessionSpawner;

impl SessionSpawner {
    pub async fn init_space_host(&self, cert_hash: String) -> anyhow::Result<()> {
        let host_def = serde_json::from_slice(HOST_DEFINITION)?;

        self.ctx
            .actor
            .configure_protocol(WP_VERSION, host_def)
            .process()
            .await?;

        let http = if self.domain.starts_with("localhost:") {
            "http"
        } else {
            "https"
        };
        let connect_url = Url::parse(&format!("{http}://{}", self.domain))?;
        info!("Publishing server URL: {connect_url}");

        let prev_connect_url = self
            .ctx
            .actor
            .query()
            .protocol(HOST_PROTOCOL.to_string())
            .protocol_version(WP_VERSION)
            .protocol_path("server".to_string())
            .process()
            .await?;

        let prev_record_id = prev_connect_url
            .into_iter()
            .next()
            .map(|v| v.entry().record_id.clone());

        let data = serde_json::to_vec(&Server {
            url: connect_url.to_string(),
            cert_hash,
        })?;

        let mut builder = self
            .ctx
            .actor
            .write()
            .protocol(HOST_PROTOCOL.to_string(), WP_VERSION, "server".to_string())
            .data(APPLICATION_JSON, data)
            // .sign(true) TODO sign + verify in client
            .published(true);

        if let Some(prev_record_id) = prev_record_id {
            builder = builder.record_id(prev_record_id);
        }

        builder.process().await?;

        Ok(())
    }
}
