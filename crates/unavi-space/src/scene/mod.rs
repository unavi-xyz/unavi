use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use bevy_wds::record::read::ReadRecord;
use loro::LoroDoc;
use tokio::sync::oneshot;

use crate::Space;

pub mod pinned_docs;

const SPACE_TTL: Duration = Duration::from_hours(7 * 24);

#[derive(Component)]
pub struct PendingScene {
    rx:      Receiver<LoroDoc>,
    _cancel: oneshot::Sender<()>,
}

pub fn spawn_space_scene(trigger: On<Add, Space>, spaces: Query<&Space>, mut commands: Commands) {
    let id = spaces.get(trigger.entity).map(|v| v.0).expect("space");
    info!(%id, "Reading space");

    let (mut event, rx, cancel) = ReadRecord::new(id);
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
            .insert((Hsd(Arc::new(doc)), HsdRecordId(space.0)))
            .remove::<PendingScene>();
    }
}

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to
    // cancel.
    commands
        .entity(trigger.entity)
        .remove::<(PendingScene, Hsd)>();
}
