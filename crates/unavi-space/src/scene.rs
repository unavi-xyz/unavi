use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::record::read::ReadRecord;
use loro::LoroDoc;
use tokio::sync::{Notify, mpsc::Receiver};

use crate::Space;

const SPACE_TTL: Duration = Duration::from_hours(24 * 7);

#[derive(Component)]
pub struct PendingScene {
    rx: Receiver<LoroDoc>,
    cancel: Arc<Notify>,
}

pub fn spawn_space_scene(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
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
    mut pending: Query<(Entity, &Space, &mut PendingScene)>,
    mut commands: Commands,
) {
    for (entity, space, mut pending) in &mut pending {
        let Ok(doc) = pending.rx.try_recv() else {
            continue;
        };

        info!(space = %space.0, "Instantiating scene");
        commands
            .entity(entity)
            .insert((HsdDoc(Arc::new(doc)), HsdRecordId(space.0)))
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
