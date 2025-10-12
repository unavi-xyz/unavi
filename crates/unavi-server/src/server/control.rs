use tarpc::context::Context;
use unavi_server_service::{ControlService, Version};

#[derive(Clone)]
pub struct ControlServer;

impl ControlService for ControlServer {
    async fn version(self, _: Context) -> Version {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        Version::parse(VERSION).expect("parse version")
    }
}
