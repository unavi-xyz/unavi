use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, SyncSender},
};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::TaskPool};
use xdid::core::did_url::DidUrl;

use unavi_server_service::from_server::ControlMessage;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    auth::LocalActor,
    space::{
        connect_info::ConnectInfo,
        streams::{publish::HostTransformStreams, transform::TransformChannels},
    },
};

pub mod connect;
mod connect_info;
pub mod record_ref_url;
mod streams;
mod tickrate;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<connect::HostConnections>()
            .init_resource::<connect::SpaceSessions>()
            .init_resource::<HostTransformStreams>()
            .add_observer(handle_space_add)
            .add_observer(insert_connect_info)
            .add_observer(connect::handle_space_connect)
            .add_observer(connect::handle_space_disconnect)
            .add_systems(
                FixedUpdate,
                (
                    streams::control::apply_controls,
                    streams::publish::publish_transform_data,
                    streams::transform::apply_player_transforms,
                    tickrate::set_space_tickrates,
                ),
            )
            .add_systems(Last, connect::cleanup_connections_on_exit);
    }
}

/// Host server connection entity.
#[derive(Component)]
#[require(HostPlayers)]
pub struct Host {
    pub connect_url: String,
}

#[derive(Component)]
pub struct HostTransformChannels {
    pub players: TransformChannels,
}

#[derive(Component)]
pub struct HostControlChannel {
    #[allow(dead_code)]
    pub tx: SyncSender<ControlMessage>,
    pub rx: Arc<Mutex<Receiver<ControlMessage>>>,
}

#[derive(Component, Default)]
#[relationship_target(relationship = PlayerHost)]
pub struct HostPlayers(Vec<Entity>);

/// Remote player entity.
#[derive(Component)]
pub struct RemotePlayer {
    pub player_id: u64,
}

#[derive(Component)]
#[relationship(relationship_target = HostPlayers)]
pub struct PlayerHost(Entity);

/// Declarative space definition.
/// Upon add, the space will be fetched and joined.
#[derive(Component)]
pub struct Space {
    pub url: DidUrl,
}

impl Space {
    pub fn new(url: DidUrl) -> Self {
        Self { url }
    }
}

pub fn handle_space_add(
    event: On<Add, Space>,
    actor: Res<LocalActor>,
    spaces: Query<&Space>,
) -> Result {
    let pool = TaskPool::get_thread_executor();

    let Some(actor) = actor.0.clone() else {
        Err(anyhow::anyhow!("actor not found"))?
    };

    let entity = event.entity;

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

pub fn insert_connect_info(event: On<ConnectInfoFetched>, mut commands: Commands) {
    commands.entity(event.space).insert(event.info.clone());
}
