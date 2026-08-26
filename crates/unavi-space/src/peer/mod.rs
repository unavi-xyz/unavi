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

static SELF_PEER: RwLock<Option<[u8; 32]>> = RwLock::new(None);

#[must_use]
pub fn self_peer_id() -> Option<[u8; 32]> {
    *SELF_PEER.read().expect("SELF_PEER poisoned")
}

/// The local endpoint id, once the iroh endpoint exists.
pub fn self_endpoint_id() -> anyhow::Result<EndpointId> {
    let peer = self_peer_id().ok_or_else(|| anyhow::anyhow!("no local endpoint"))?;
    Ok(EndpointId::from_bytes(&peer)?)
}

pub fn set_self_peer_id(peer: [u8; 32]) {
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
    unavi_identity::identity::local_did().map(|did| did.to_string())
}

/// Trust scores key off the local DID, so the table loaded at startup is worth
/// nothing until the identity this process runs as is known.
pub fn score_once_identified(mut scored: Local<bool>) {
    if *scored || unavi_identity::identity::local_did().is_none() {
        return;
    }
    *scored = true;
    unavi_policy::trust::recompute(&[]);
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
