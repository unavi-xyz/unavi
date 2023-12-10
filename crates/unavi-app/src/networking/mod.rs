use bevy::prelude::*;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, open_connection);
    }
}

fn open_connection() {
    tracing::span!(tracing::Level::INFO, "open_connection");

    #[cfg(not(target_family = "wasm"))]
    let endpoint = match wtransport::Endpoint::client(
        wtransport::ClientConfig::builder()
            .with_bind_address("0.0.0.0:0".parse().unwrap())
            .with_native_certs()
            .build(),
    ) {
        Ok(endpoint) => xwt::current::Endpoint(endpoint),
        Err(e) => {
            error!("Failed to create endpoint: {}", e);
            return;
        }
    };

    #[cfg(target_family = "wasm")]
    let endpoint = xwt::current::Endpoint::default();
}
