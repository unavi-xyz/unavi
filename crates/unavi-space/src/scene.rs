use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_hsd::HsdDoc;
use bevy_wds::{LocalActor, SyncTargets, record::get::ReadRecord};
use loro::LoroDoc;
use tokio::sync::{Notify, mpsc::Receiver};

use crate::Space;

const SPACE_TTL: Duration = Duration::from_hours(24 * 7);

#[derive(Component)]
pub struct PendingScene {
    rx: Receiver<LoroDoc>,
    cancel: Arc<Notify>,
}

pub fn spawn_space_scene(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    actor: Query<(&LocalActor, &SyncTargets)>,
    mut commands: Commands,
) {
    let Ok((_actor, _sync_targets)) = actor.single() else {
        warn!("space scene failed: no actor");
        return;
    };

    let space = spaces.get(trigger.entity).map(|v| v.0).expect("space");

    let (mut event, rx, cancel) = ReadRecord::new(space);
    event.ttl = Some(SPACE_TTL);
    event.retries = 5;

    commands.trigger(event);

    commands
        .entity(trigger.entity)
        .insert(PendingScene { rx, cancel });
}

pub fn instantiate_pending_scenes(
    mut pending: Query<(Entity, &mut PendingScene)>,
    mut commands: Commands,
) {
    for (entity, mut pending) in &mut pending {
        let Ok(doc) = pending.rx.try_recv() else {
            continue;
        };

        commands
            .entity(entity)
            .insert(HsdDoc(Arc::new(doc)))
            .remove::<PendingScene>();
    }
}

pub fn despawn_space_scene(
    trigger: On<Remove, Space>,
    pending: Query<&PendingScene>,
    mut commands: Commands,
) {
    if let Ok(pending) = pending.get(trigger.entity) {
        pending.cancel.notify_one();
    }

    commands
        .entity(trigger.entity)
        .remove::<(PendingScene, HsdDoc)>();
}
