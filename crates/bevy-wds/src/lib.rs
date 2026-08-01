use std::sync::RwLock;

use bevy::prelude::*;
use wds::{
    Blobs,
    actor::Actor,
};

pub mod blob;
pub mod record;

static LOCAL_ACTOR: RwLock<Option<Actor>> = RwLock::new(None);

/// Publishes the process's local actor for off-world async access, so callers
/// on background tasks can reach it without a main-world command hop.
pub fn set_local_actor(actor: Actor) {
    *LOCAL_ACTOR.write().expect("local actor lock poisoned") = Some(actor);
}

#[must_use]
pub fn local_actor() -> Option<Actor> {
    LOCAL_ACTOR
        .read()
        .expect("local actor lock poisoned")
        .clone()
}

pub struct WdsPlugin;

impl Plugin for WdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(blob::get::on_get_blob)
            .add_observer(blob::request::on_blob_request_add)
            .add_observer(blob::request::on_blob_request_remove)
            .add_observer(record::acl::on_set_record_public)
            .add_observer(record::query::on_query_record)
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
