use bevy::prelude::*;
use wds::{Blobs, actor::Actor};

pub mod blob;
pub mod record;

pub struct WdsPlugin;

impl Plugin for WdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(blob::get::on_get_blob)
            .add_observer(blob::request::on_blob_request_add)
            .add_observer(blob::request::on_blob_request_remove)
            .add_observer(record::read::on_read_record)
            .add_observer(record::write::on_write_record)
            .add_systems(
                FixedUpdate,
                (
                    blob::deps::mark_blob_deps_loaded,
                    blob::request::recv_blob_responses,
                ),
            );
    }
}

#[derive(Component)]
pub struct LocalBlobs(pub Blobs);

#[derive(Component)]
#[require(SyncTargets)]
pub struct LocalActor(pub Actor);

#[derive(Component, Default)]
pub struct SyncTargets(pub Vec<Actor>);
