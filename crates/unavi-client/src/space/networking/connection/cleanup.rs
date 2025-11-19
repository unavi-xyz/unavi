use bevy::{app::AppExit, ecs::world::CommandQueue, log::*, prelude::*, tasks::TaskPool};
use wtransport::VarInt;

use super::lifecycle::HostConnections;
use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{
        Host, HostPlayers, Space, connect_info::ConnectInfo,
        networking::streams::publish::HostTransformStreams,
    },
};

pub fn handle_space_disconnect(
    event: On<Remove, ConnectInfo>,
    connections: Res<HostConnections>,
    transform_streams: Res<HostTransformStreams>,
    spaces: Query<(Entity, &Space, &ConnectInfo), With<Space>>,
) -> Result {
    let Ok((_, _, info)) = spaces.get(event.entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };

    let mut other_spaces_found = 0;
    for (other_entity, _, other_info) in spaces.iter() {
        if other_entity == event.entity {
            continue;
        }
        if other_info.connect_url != info.connect_url {
            continue;
        }
        other_spaces_found += 1;
    }

    let connect_url = info.connect_url.to_string();
    let connections = connections.0.clone();
    let streams_map = transform_streams.0.clone();

    let pool = TaskPool::get_thread_executor();
    pool.spawn(async move {
        if other_spaces_found == 0 {
            let _ = streams_map.remove_async(&connect_url).await;

            if let Some((_, connection)) = connections.remove_async(&connect_url).await {
                connection.connection.close(VarInt::from_u32(200), &[]);
                cleanup_host_entity(connect_url);
            }
        }
    })
    .detach();

    Ok(())
}

fn cleanup_host_entity(connect_url: String) {
    let mut queue = CommandQueue::default();
    queue.push(move |world: &mut World| {
        let mut host_entity = None;
        let mut query = world.query::<(Entity, &Host)>();
        for (entity, host) in query.iter(world) {
            if host.connect_url == connect_url {
                host_entity = Some(entity);
                break;
            }
        }

        if let Some(host_ent) = host_entity {
            if let Some(host_players) = world.get::<HostPlayers>(host_ent) {
                let players = host_players.0.clone();
                for player in players {
                    world.despawn(player);
                }
            }

            world.despawn(host_ent);
        }
    });

    if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
        error!("failed to cleanup host entity: {e:?}");
    }
}

pub fn cleanup_connections_on_exit(
    connections: Res<HostConnections>,
    mut exit_events: MessageReader<AppExit>,
) {
    for _ in exit_events.read() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        connections.0.iter_sync(|_, c| {
            info!("closing connection: {}", c.connection.stable_id());
            c.connection.close(VarInt::from_u32(0), b"shutdown");
            rt.block_on(c.connection.closed());
            false
        });
    }
}
