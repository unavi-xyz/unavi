use std::sync::RwLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_iroh::store::BlobProviders;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;

pub mod presence;

static SELF_PEER: RwLock<Option<EndpointId>> = RwLock::new(None);

/// The local endpoint id, once the iroh endpoint exists.
#[must_use]
pub fn self_peer_id() -> Option<EndpointId> {
    *SELF_PEER.read().expect("SELF_PEER poisoned")
}

pub fn set_self_peer_id(peer: EndpointId) {
    let mut current = SELF_PEER.write().expect("SELF_PEER poisoned");
    if let Some(existing) = *current
        && existing != peer
    {
        info!("self peer id changed (endpoint re-created)");
    }
    *current = Some(peer);
}

#[must_use]
pub fn self_did() -> Option<String> {
    crate::identity::local().map(|l| l.identity.did().to_string())
}

/// Offers the connected peers to the blob downloader.
///
/// A space's document syncs from its occupants, so its content lives with them
/// too; a fetch knowing only the configured sync targets asks a server that may
/// never have seen the space.
pub fn publish_blob_providers(peers: Query<&Peer>, mut providers: Query<&mut BlobProviders>) {
    let Ok(mut providers) = providers.single_mut() else {
        return;
    };
    let connected = peers.iter().map(|p| p.0.id).collect::<Vec<_>>();
    if providers.0 != connected {
        providers.0 = connected;
    }
}

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<NamespaceId, f32>);
