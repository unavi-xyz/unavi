use anyhow::Result;
use bevy::{prelude::*, utils::tracing::Instrument};
use bevy_async_task::AsyncTaskPool;
use unavi_world::InstanceServer;
use xwt_core::base::Session;

mod handler;
#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod web;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, connect_to_instances);
    }
}

#[derive(Component)]
pub struct InstanceSession;

pub fn connect_to_instances(
    mut commands: Commands,
    mut pool: AsyncTaskPool<()>,
    to_open: Query<(Entity, &InstanceServer), Without<InstanceSession>>,
) {
    for (entity, server) in to_open.iter() {
        let address = server.0.clone();
        let span = info_span!("Connection", address);

        pool.spawn(
            async move {
                if let Err(e) = connection_thread(&address).await {
                    error!("{}", e)
                }
            }
            .instrument(span),
        );

        commands.entity(entity).insert(InstanceSession);
    }
}

const LOCALHOST: &str = "https://localhost:";

async fn connection_thread(addr: &str) -> Result<()> {
    let addr = if addr.starts_with(LOCALHOST) {
        addr.replace(LOCALHOST, "https://127.0.0.1:")
    } else {
        addr.to_string()
    };

    let session = connect(&addr).await?;

    handler::handle_session(session).await?;

    Ok(())
}

async fn connect(addr: &str) -> Result<impl Session> {
    info!("Beginning connection process.");

    #[cfg(not(target_family = "wasm"))]
    let conn = crate::native::connect(addr).await?;

    #[cfg(target_family = "wasm")]
    let conn = crate::web::connect(addr)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(conn)
}
