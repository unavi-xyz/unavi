use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
};

#[cfg(target_family = "wasm")]
use xwt_core::traits::EndpointConnect;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, open_connection)
            .add_systems(Update, handle_tasks);
    }
}

const WORLD_ADDRESS: &str = "https://127.0.0.1:3000";

#[derive(Component)]
pub struct OpenConnection(Task<String>);

fn open_connection(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();

    #[cfg(not(target_family = "wasm"))]
    let task = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        thread_pool.spawn(async move { rt.block_on(test_connection()).unwrap() })
    };

    #[cfg(target_family = "wasm")]
    let task = thread_pool.spawn(async move { test_connection().await.unwrap() });

    commands.spawn(OpenConnection(task));
}

async fn test_connection() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(not(target_family = "wasm"))]
    let endpoint = {
        let config = wtransport::ClientConfig::builder().with_bind_default();

        #[cfg(not(feature = "disable-cert-validation"))]
        let config = config.with_native_certs();

        #[cfg(feature = "disable-cert-validation")]
        let config = {
            warn!(
                "Certificate validation is disabled. This is not recommended for production use."
            );
            config.with_no_cert_validation()
        };

        xwt::current::Endpoint(wtransport::Endpoint::client(config.build()).unwrap()).0
    };

    #[cfg(target_family = "wasm")]
    let endpoint = xwt::current::Endpoint::default();

    info!("Connecting to {}", WORLD_ADDRESS);

    let connection = endpoint.connect(WORLD_ADDRESS).await?;
    let opening = connection.open_bi().await?;
    let (mut send, mut recv) = opening.await?;

    info!("Opened bi stream");

    send.write_all(b"Hello, world!").await.unwrap();

    let mut buf = [0; 1024];
    let n = recv.read(&mut buf).await.unwrap().unwrap();

    let msg = std::str::from_utf8(&buf[..n]).unwrap();

    Ok(String::from(msg))
}

fn handle_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut OpenConnection)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<OpenConnection>();
            info!("Received message: {:?}", result);
        }
    }
}
