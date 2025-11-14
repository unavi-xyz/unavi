use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::TaskPool};
use unavi_player::{LocalPlayer, PlayerSpawner};
use xdid::core::did_url::DidUrl;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    auth::LocalActor,
    space::{connect_info::ConnectInfo, streams::transform::RecievedTransform},
};

pub mod connect;
mod connect_info;
mod record_ref_url;
mod streams;
mod tickrate;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<connect::HostConnections>()
            .init_resource::<connect::SpaceSessions>()
            .add_observer(handle_space_add)
            .add_observer(insert_connect_info)
            .add_observer(connect::handle_space_connect)
            .add_observer(connect::handle_space_disconnect)
            .add_systems(
                FixedUpdate,
                (
                    streams::publish::publish_transform_data,
                    streams::transform::apply_player_transforms,
                    tickrate::set_space_tickrates,
                ),
            );
    }
}

/// Declarative space definition.
/// Upon add, the space will be fetched and joined.
#[derive(Component)]
pub struct Space {
    pub url: DidUrl,
    transform_tx: Sender<RecievedTransform>,
    transform_rx: Arc<Mutex<Receiver<RecievedTransform>>>,
}

impl Space {
    pub fn new(url: DidUrl) -> Self {
        let (transform_tx, transform_rx) = std::sync::mpsc::channel();
        Self {
            url,
            transform_tx,
            transform_rx: Arc::new(Mutex::new(transform_rx)),
        }
    }
}

pub fn handle_space_add(
    event: On<Add, Space>,
    asset_server: Res<AssetServer>,
    actor: Res<LocalActor>,
    spaces: Query<&Space>,
    local_players: Query<Entity, With<LocalPlayer>>,
    mut commands: Commands,
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

    if local_players.is_empty() {
        PlayerSpawner::default().spawn(&mut commands, &asset_server);
    }

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
