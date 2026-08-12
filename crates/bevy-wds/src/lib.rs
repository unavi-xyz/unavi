use std::sync::RwLock;

use bevy::prelude::*;
use iroh_blobs::api::{
    blobs::Blobs,
    downloader::Downloader,
};
use iroh_docs::{
    NamespaceId,
    protocol::Docs,
};
use iroh_gossip::Gossip;
use unavi_registry::client::RegistryClient;
use wds::actor::Actor;

pub mod blob;
pub mod doc;

static LOCAL_ACTOR: RwLock<Option<Actor>> = RwLock::new(None);
static ROOT_DOC: RwLock<Option<NamespaceId>> = RwLock::new(None);
static REGISTRIES: RwLock<Vec<NamespaceId>> = RwLock::new(Vec::new());
static REGISTRY_CLIENTS: RwLock<Vec<RegistryClient>> = RwLock::new(Vec::new());

/// Publishes the process's root doc namespace for off-world access.
pub fn set_root_doc(ns: NamespaceId) {
    *ROOT_DOC.write().expect("root doc lock poisoned") = Some(ns);
}

#[must_use]
pub fn root_doc() -> Option<NamespaceId> {
    *ROOT_DOC.read().expect("root doc lock poisoned")
}

/// Publishes the view docs of the registries this client follows, for
/// off-world access.
///
/// Views are the curated docs a registry publishes.
pub fn set_registries(namespaces: Vec<NamespaceId>) {
    *REGISTRIES.write().expect("registries lock poisoned") = namespaces;
}

#[must_use]
pub fn registries() -> Vec<NamespaceId> {
    REGISTRIES.read().expect("registries lock poisoned").clone()
}

/// Publishes clients for the registries this process announces to.
pub fn set_registry_clients(clients: Vec<RegistryClient>) {
    *REGISTRY_CLIENTS
        .write()
        .expect("registry clients lock poisoned") = clients;
}

#[must_use]
pub fn registry_clients() -> Vec<RegistryClient> {
    REGISTRY_CLIENTS
        .read()
        .expect("registry clients lock poisoned")
        .clone()
}

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
            .add_observer(doc::on_doc_create)
            .add_observer(doc::on_doc_set)
            .add_observer(doc::on_doc_delete)
            .add_observer(doc::on_doc_get)
            .add_observer(doc::on_doc_list)
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

/// The store backing [`LocalBlobs`], for tag management the blobs client does
/// not expose, such as pinning content against garbage collection.
#[derive(Component)]
pub struct LocalBlobStore(pub iroh_blobs::api::Store);

/// Pulls blobs from named providers. Holds internal state, so it is built once
/// with the store rather than per fetch.
#[derive(Component)]
pub struct LocalDownloader(pub Downloader);

#[derive(Component)]
pub struct LocalDocs(pub Docs);

/// The store's gossip, which is the only one on this endpoint: the router
/// accepts `iroh_gossip::ALPN` once, so a second instance would silently take
/// every inbound connection from the first.
#[derive(Component)]
pub struct LocalGossip(pub Gossip);

#[derive(Component)]
#[require(SyncTargets)]
pub struct LocalActor(pub Actor);

#[derive(Component, Default)]
pub struct SyncTargets(pub Vec<Actor>);
