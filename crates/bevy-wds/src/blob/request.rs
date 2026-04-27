use std::sync::Arc;

use bevy::prelude::*;
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::{Mutex, mpsc::Receiver, oneshot};

use crate::blob::get::GetBlob;

#[derive(Component)]
pub struct BlobRequest(pub Hash);

#[derive(Component)]
pub struct BlobPending {
    rx: Arc<Mutex<Receiver<Bytes>>>,
    _cancel: oneshot::Sender<()>,
}

#[derive(Component)]
pub struct BlobResponse(pub Option<Bytes>);

pub(crate) fn on_blob_request_remove(trigger: On<Remove, BlobRequest>, mut commands: Commands) {
    // Removing BlobPending drops the oneshot::Sender, signalling the task to cancel.
    commands.entity(trigger.entity).remove::<BlobPending>();
}

pub(crate) fn on_blob_request_add(
    trigger: On<Add, BlobRequest>,
    requests: Query<&BlobRequest>,
    mut commands: Commands,
) {
    let req = requests.get(trigger.entity).expect("blob request");

    let (cancel_tx, cancel_rx) = oneshot::channel();
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    commands.trigger(GetBlob {
        hash: req.0,
        cancel: Some(cancel_rx),
        tx,
    });

    commands.entity(trigger.entity).insert(BlobPending {
        rx: Arc::new(Mutex::new(rx)),
        _cancel: cancel_tx,
    });
}

pub(crate) fn recv_blob_responses(mut commands: Commands, loading: Query<(Entity, &BlobPending)>) {
    for (entity, load) in loading {
        let Ok(mut rx) = load.rx.try_lock() else {
            continue;
        };

        let Ok(bytes) = rx.try_recv() else {
            continue;
        };

        commands
            .entity(entity)
            .remove::<BlobPending>()
            .insert(BlobResponse(Some(bytes)));
    }
}
