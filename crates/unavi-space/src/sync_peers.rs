//! Naming peers that become known after a document has already joined.

use bevy::prelude::*;
use bevy_async::task;
use bevy_hsd::document::SyncPeers;
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use unavi_store::Store;

/// The addresses in `known` that `named` does not list, each once.
pub fn unnamed(
    named: &SyncPeers,
    known: impl IntoIterator<Item = EndpointAddr>,
) -> Vec<EndpointAddr> {
    let mut fresh: Vec<EndpointAddr> = Vec::new();
    for addr in known {
        let listed = named.0.iter().chain(&fresh).any(|have| have.id == addr.id);
        if !listed {
            fresh.push(addr);
        }
    }
    fresh
}

/// Syncs the joined document `ns` with `peers` in the background.
pub fn name_peers(store: &Store, ns: NamespaceId, peers: Vec<EndpointAddr>) {
    let store = store.clone();
    task::spawn(async move {
        if let Err(err) = store.add_peers(ns, peers).await {
            warn!(%ns, ?err, "failed naming sync peers");
        }
    });
}
