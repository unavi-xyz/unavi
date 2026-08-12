use bevy::prelude::*;
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::oneshot;

use crate::blob::get::{
    BlobError,
    GetBlob,
};

#[derive(Component)]
pub struct BlobRequest(pub Hash);

#[derive(Component)]
pub struct BlobPending {
    rx:      async_channel::Receiver<Result<Bytes, BlobError>>,
    _cancel: oneshot::Sender<()>,
}

#[derive(Component)]
pub struct BlobResponse(pub Result<Bytes, BlobError>);

pub(crate) fn on_blob_request_remove(trigger: On<Remove, BlobRequest>, mut commands: Commands) {
    // Removing BlobPending drops the oneshot::Sender, signalling the task to
    // cancel.
    commands.entity(trigger.entity).try_remove::<BlobPending>();
}

pub(crate) fn on_blob_request_add(
    trigger: On<Add, BlobRequest>,
    requests: Query<&BlobRequest>,
    mut commands: Commands,
) {
    let req = requests.get(trigger.entity).expect("blob request");

    let (cancel_tx, cancel_rx) = oneshot::channel();
    let (tx, rx) = async_channel::bounded(1);

    commands.trigger(GetBlob {
        hash: req.0,
        cancel: Some(cancel_rx),
        tx,
    });

    commands.entity(trigger.entity).insert(BlobPending {
        rx,
        _cancel: cancel_tx,
    });
}

pub(crate) fn recv_blob_responses(mut commands: Commands, loading: Query<(Entity, &BlobPending)>) {
    for (entity, load) in loading {
        match load.rx.try_recv() {
            Ok(Ok(bytes)) => {
                commands
                    .entity(entity)
                    .try_remove::<BlobPending>()
                    .try_insert(BlobResponse(Ok(bytes)));
            }
            Ok(Err(err)) => {
                commands
                    .entity(entity)
                    .try_remove::<BlobPending>()
                    .try_insert(BlobResponse(Err(err)));
            }
            Err(async_channel::TryRecvError::Empty) => {}
            Err(async_channel::TryRecvError::Closed) => {
                warn!(?entity, "blob fetch ended without a response");
                commands
                    .entity(entity)
                    .try_remove::<BlobPending>()
                    .try_insert(BlobResponse(Err(anyhow::anyhow!(
                        "fetch task ended without a response"
                    )
                    .into())));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_pending(world: &mut World, rx: async_channel::Receiver<Result<Bytes, BlobError>>) -> Entity {
        world
            .spawn(BlobPending {
                rx,
                _cancel: oneshot::channel().0,
            })
            .id()
    }

    #[test]
    fn a_failed_fetch_surfaces_an_error_response() {
        let mut app = App::new();
        app.add_systems(Update, recv_blob_responses);
        let (tx, rx) = async_channel::bounded(1);
        let entity = spawn_pending(app.world_mut(), rx);
        tx.send_blocking(Err(BlobError::TooLarge { size: 1 }))
            .expect("send");

        app.update();

        let world = app.world_mut();
        let response = world.get::<BlobResponse>(entity).expect("response");
        assert!(matches!(response.0, Err(BlobError::TooLarge { .. })));
        assert!(
            world.get::<BlobPending>(entity).is_none(),
            "a resolved fetch drops its pending state"
        );
    }

    #[test]
    fn a_dropped_channel_reports_failure() {
        let mut app = App::new();
        app.add_systems(Update, recv_blob_responses);
        let (tx, rx) = async_channel::bounded(1);
        let entity = spawn_pending(app.world_mut(), rx);
        drop(tx);

        app.update();

        let world = app.world_mut();
        let response = world.get::<BlobResponse>(entity).expect("response");
        assert!(
            matches!(response.0, Err(BlobError::Io(_))),
            "a task that vanished mid-fetch is a failure, not a hang"
        );
    }
}
