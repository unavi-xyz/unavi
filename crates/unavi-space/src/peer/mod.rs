use std::sync::{
    Arc,
    RwLock,
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_wds::LocalActor;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use wds::identity::Identity;

pub mod presence;

static SELF_PEER: RwLock<Option<[u8; 32]>> = RwLock::new(None);
static SELF_IDENTITY: RwLock<Option<Arc<Identity>>> = RwLock::new(None);

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
pub fn self_identity() -> Option<Arc<Identity>> {
    SELF_IDENTITY
        .read()
        .expect("SELF_IDENTITY poisoned")
        .clone()
}

#[must_use]
pub fn self_did() -> Option<String> {
    self_identity().map(|i| i.did().to_string())
}

pub fn capture_self_identity(trigger: On<Add, LocalActor>, actors: Query<&LocalActor>) {
    if let Ok(actor) = actors.get(trigger.entity) {
        let identity = actor.0.identity();
        unavi_policy::identity::set_self(identity.did().clone());
        *SELF_IDENTITY.write().expect("SELF_IDENTITY poisoned") = Some(Arc::clone(identity));
        // Scores are measured from the local DID, so nothing could be computed
        // until it existed.
        unavi_policy::trust::recompute(&[]);
    }
}

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<NamespaceId, f32>);
