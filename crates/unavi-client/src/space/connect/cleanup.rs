use bevy::{app::AppExit, ecs::world::CommandQueue, log::*, prelude::*, tasks::TaskPool};
use wtransport::VarInt;

use super::{lifecycle::HostConnections, state::ConnectionTasks};
use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{
        Host, HostPlayers, Space, connect_info::ConnectInfo, streams::publish::HostTransformStreams,
    },
};

/// Observer triggered when ConnectInfo is removed from a Space.
pub fn handle_space_disconnect(
    event: On<Remove, ConnectInfo>,
    connections: Res<HostConnections>,
    transform_streams: Res<HostTransformStreams>,
    spaces: Query<(Entity, &Space, &ConnectInfo), With<Space>>,
    mut tasks: Query<&mut ConnectionTasks>,
) -> Result {
    let Ok((_, _space, info)) = spaces.get(event.entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };

    // Abort background tasks for this space.
    if let Ok(mut connection_tasks) = tasks.get_mut(event.entity) {
        connection_tasks.abort_all();
    }

    // Count other spaces using the same connection.
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
        // Disconnect from host if no other spaces using the connection.
        if other_spaces_found == 0 {
            let _ = streams_map.remove_async(&connect_url).await;

            if let Some((_, connection)) = connections.remove_async(&connect_url).await {
                connection.connection.close(VarInt::from_u32(200), &[]);

                // Despawn Host entity and all remote players.
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
        // Find Host entity by connect_url.
        let mut host_entity = None;
        let mut query = world.query::<(Entity, &Host)>();
        for (entity, host) in query.iter(world) {
            if host.connect_url == connect_url {
                host_entity = Some(entity);
                break;
            }
        }

        if let Some(host_ent) = host_entity {
            // Get all remote players.
            if let Some(host_players) = world.get::<HostPlayers>(host_ent) {
                let players = host_players.0.clone();
                for player in players {
                    world.despawn(player);
                }
            }

            // Despawn host.
            world.despawn(host_ent);
        }
    });

    if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
        error!("Failed to send cleanup commands: {e:?}");
    }
}

/// System to cleanup all connections on app exit.
pub fn cleanup_connections_on_exit(
    connections: Res<HostConnections>,
    mut exit_events: MessageReader<AppExit>,
    mut tasks_query: Query<&mut ConnectionTasks>,
) {
    for _ in exit_events.read() {
        // Abort all background tasks.
        for mut tasks in tasks_query.iter_mut() {
            tasks.abort_all();
        }

        // Close all connections.
        connections.0.iter_sync(|_, c| {
            c.connection.close(VarInt::from_u32(0), b"shutdown");
            false
        });
    }
}
