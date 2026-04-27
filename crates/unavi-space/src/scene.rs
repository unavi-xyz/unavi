use std::sync::Arc;
use std::time::Duration;

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::record::read::ReadRecord;
use loro::LoroDoc;
use tokio::sync::oneshot;

use crate::Space;

const SPACE_TTL: Duration = Duration::from_hours(24 * 7);

#[derive(Component)]
pub struct PendingScene {
    rx: Receiver<LoroDoc>,
    _cancel: oneshot::Sender<()>,
}

pub fn spawn_space_scene(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
    let space = spaces.get(trigger.entity).map(|v| v.0).expect("space");

    let (mut event, rx, cancel) = ReadRecord::new(space);
    event.ttl = Some(SPACE_TTL);
    event.retries = 5;
    commands.trigger(event);

    commands.entity(trigger.entity).insert(PendingScene {
        rx,
        _cancel: cancel,
    });
}

pub fn instantiate_pending_scenes(
    pending: Query<(Entity, &Space, &PendingScene)>,
    mut commands: Commands,
) {
    for (entity, space, pending) in &pending {
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

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to cancel.
    commands
        .entity(trigger.entity)
        .remove::<(PendingScene, HsdDoc)>();
}
