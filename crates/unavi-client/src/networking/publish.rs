use std::{collections::HashSet, sync::Arc, task::Poll};

use bevy::prelude::*;
use bevy_async_task::TaskPool;
use tokio::{sync::Mutex, time::Interval};
use unavi_player::LocalPlayer;

use crate::space::Space;

/// The interval at which data should be published to the space's server.
#[derive(Component)]
pub struct PublishInterval(pub Arc<Mutex<Interval>>);

pub fn publish_transform_data(
    player: Query<Entity, With<LocalPlayer>>,
    mut spaces: Query<(Entity, &Space, &mut PublishInterval)>,
    mut pool: TaskPool<Entity>,
    mut to_publish: Local<HashSet<Entity>>,
    mut polling: Local<HashSet<Entity>>,
) {
    for status in pool.iter_poll() {
        if let Poll::Ready(entity) = status {
            to_publish.insert(entity);
            polling.remove(&entity);
        }
    }

    for (entity, _, interval) in spaces.iter_mut() {
        if polling.contains(&entity) {
            continue;
        };

        let interval = interval.0.clone();

        pool.spawn(async move {
            interval.lock().await.tick().await;
            entity
        });
    }

    if to_publish.is_empty() {
        return;
    }

    let Ok(_player) = player.single() else {
        return;
    };

    for _space_ent in to_publish.iter() {
        tokio::spawn(async move {
            // send
        });
    }

    to_publish.clear();
}
