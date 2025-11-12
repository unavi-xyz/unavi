use bevy::{ecs::world::CommandQueue, prelude::*, tasks::TaskPool};
use xdid::core::did_url::DidUrl;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE, auth::LocalActor, space::connect_info::ConnectInfo,
};

pub mod connect;
mod connect_info;
mod record_ref_url;
pub mod runtime;
pub mod transform;

/// Declarative space definition.
/// Upon add, the space will be fetched and joined.
#[derive(Component)]
pub struct Space {
    pub url: DidUrl,
}

pub fn handle_space_add(
    on: On<Add, Space>,
    actor: Res<LocalActor>,
    spaces: Query<&Space>,
) -> Result {
    let pool = TaskPool::get_thread_executor();

    let Some(actor) = actor.0.clone() else {
        Err(anyhow::anyhow!("actor not found"))?
    };

    let entity = on.event().entity;

    let Ok(space) = spaces.get(entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };

    let host_did = space.url.did.clone();

    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            // TODO: Fetch from host DID document
            let host_dwn = actor.remote.clone().unwrap();

            match connect_info::fetch_connect_info(&actor, &host_did, &host_dwn).await? {
                Some(info) => {
                    let mut commands = CommandQueue::default();
                    commands.push(bevy::ecs::system::command::trigger(ConnectInfoFetched {
                        space: entity,
                        info,
                    }));
                    ASYNC_COMMAND_QUEUE.0.send(commands)?;
                }
                None => {
                    warn!("Connect info not found for space {entity}, unable to join")
                }
            }

            Ok::<_, anyhow::Error>(())
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();

    Ok(())
}

#[derive(Event)]
pub struct ConnectInfoFetched {
    space: Entity,
    info: ConnectInfo,
}

pub fn handle_connect_info_fetched(event: On<ConnectInfoFetched>, mut commands: Commands) {
    commands.entity(event.space).insert(event.info.clone());
}
