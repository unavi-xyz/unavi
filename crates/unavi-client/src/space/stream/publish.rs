use std::time::Duration;

use bevy::{prelude::*, tasks::TaskPool};
use unavi_player::LocalPlayer;

use crate::space::Space;

/// The interval at which data should be published to the space's server.
#[derive(Component)]
pub struct PublishInterval {
    pub last_tick: Duration,
    pub tickrate: Duration,
}

pub fn publish_transform_data(
    time: Res<Time>,
    player: Query<Entity, With<LocalPlayer>>,
    mut spaces: Query<(Entity, &Space, &mut PublishInterval)>,
) {
    let Ok(_player) = player.single() else {
        return;
    };

    let now = time.elapsed();

    for (_entity, _, mut interval) in spaces.iter_mut() {
        if now - interval.last_tick < interval.tickrate {
            continue;
        }

        interval.last_tick = now;

        let pool = TaskPool::get_thread_executor();
        pool.spawn(async move {
            // TODO
        })
        .detach();
    }
}
