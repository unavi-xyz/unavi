use bevy::prelude::*;

pub mod runtime;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<runtime::AsyncRuntime>()
            .add_systems(Startup, open_connection);
    }
}

const WORLD_ADDRESS: &str = "https://127.0.0.1:3000";

fn open_connection(runtime: Res<runtime::AsyncRuntime>) {
    runtime.0.spawn(async move {
        #[cfg(not(target_family = "wasm"))]
        let endpoint = {
            let config = wtransport::ClientConfig::builder().with_bind_default();

            #[cfg(not(feature = "disable-cert-validation"))]
            let config = config.with_native_certs();

            #[cfg(feature = "disable-cert-validation")]
            let config = {
                warn!("Certificate validation is disabled. This is not recommended for production use.");
                config.with_no_cert_validation()
            };

            xwt::current::Endpoint(
                wtransport::Endpoint::client(config.build())
                    .expect("should be able to create client endpoint"),
            )
        };

        #[cfg(target_family = "wasm")]
        let endpoint = xwt::current::Endpoint::default();

        info!("Connecting to {}", WORLD_ADDRESS);

        let connection = match endpoint.0.connect(WORLD_ADDRESS).await {
            Ok(connection) => connection,
            Err(e) => {
                error!("Failed to connect: {}", e);
                return;
            }
        };

        info!("Connected to server");

        let opening = match connection.open_bi().await {
            Ok(opening) => opening,
            Err(e) => {
                error!("Failed to open bi: {}", e);
                return;
            }
        };

        let (mut send, mut recv) = match opening.await {
            Ok(bi) => bi,
            Err(e) => {
                error!("Failed to accept bi: {}", e);
                return;
            }
        };

        info!("Opened bi");

        send.write_all(b"Hello, world!").await.unwrap();

        let mut buf = [0; 1024];
        let n = recv.read(&mut buf).await.unwrap().unwrap();

        let msg = std::str::from_utf8(&buf[..n]).unwrap();

        info!("Received: {}", msg);
    });
}
