use bevy::prelude::*;

pub mod runtime;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<runtime::AsyncRuntime>()
            .add_systems(Startup, open_connection);
    }
}

fn open_connection(runtime: Res<runtime::AsyncRuntime>) {
    runtime.0.spawn(async move {
        #[cfg(not(target_family = "wasm"))]
        let endpoint = xwt::current::Endpoint(
            wtransport::Endpoint::client(
                wtransport::ClientConfig::builder()
                    .with_bind_address("0.0.0.0:0".parse().unwrap())
                    .with_native_certs()
                    .build(),
            )
            .expect("should be able to create client endpoint"),
        );

        #[cfg(target_family = "wasm")]
        let endpoint = xwt::current::Endpoint::default();

        let connection = match endpoint.0.connect("https://localhost:3000").await {
            Ok(connection) => connection,
            Err(e) => {
                error!("Failed to connect: {}", e);
                return;
            }
        };

        let bi = match connection.open_bi().await {
            Ok(bi) => bi,
            Err(e) => {
                error!("Failed to open bi: {}", e);
                return;
            }
        };
    });
}
