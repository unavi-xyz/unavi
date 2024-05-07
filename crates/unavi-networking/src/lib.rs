use anyhow::Result;
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use unavi_world::InstanceServer;
use xwt_core::base::Session;

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod web;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, poll_connections);
    }
}

#[derive(Component)]
pub struct InstanceSession;

pub fn poll_connections(
    mut commands: Commands,
    to_open: Query<(Entity, &InstanceServer), Without<InstanceSession>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, server) in to_open.iter() {
        let server = server.0.clone();

        pool.spawn(async move {
            if let Err(e) = connection_thread(&server).await {
                error!("Instance connection error: {}", e)
            }
        })
        .detach();

        commands.entity(entity).insert(InstanceSession);
    }
}

async fn connection_thread(addr: &str) -> Result<()> {
    let _session = connect(addr).await?;

    Ok(())
}

async fn connect(addr: &str) -> Result<impl Session> {
    info!("Beginning connection process with {}", addr);

    #[cfg(not(target_family = "wasm"))]
    let conn = crate::native::connect(addr).await?;

    #[cfg(target_family = "wasm")]
    let conn = crate::web::connect(addr)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(conn)
}
