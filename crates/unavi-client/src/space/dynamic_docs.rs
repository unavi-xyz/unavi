use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::{LocalActor, SyncTargets};
use loro::LoroDoc;

use crate::networking::event::PendingDynamicDocs;

/// Lazily fetches HSD documents for dynamic objects whose record IDs arrive
/// via physics gossip before the document is known locally.
///
/// When a peer publishes physics for an object in a document we haven't seen,
/// we attempt to read it from WDS. If the record is public the fetch succeeds
/// and the HSD entity is spawned, which lets subsequent physics updates find
/// it normally. Private / not-found records are silently ignored.
pub fn fetch_dynamic_docs(
    mut pending: ResMut<PendingDynamicDocs>,
    existing: Query<&HsdRecordId>,
    local_actors: Query<&LocalActor>,
    sync_targets: Query<&SyncTargets>,
    mut commands: Commands,
    mut receivers: Local<Vec<std::sync::mpsc::Receiver<anyhow::Result<(blake3::Hash, LoroDoc)>>>>,
) {
    let spawned: std::collections::HashSet<blake3::Hash> = existing.iter().map(|r| r.0).collect();
    pending.0.retain(|h| !spawned.contains(h));

    if pending.0.is_empty() && receivers.is_empty() {
        return;
    }

    let Ok(local_actor) = local_actors.single() else {
        return;
    };
    let actor = local_actor.0.clone();
    let remote = sync_targets
        .single()
        .ok()
        .and_then(|t| t.0.first().cloned());

    for hash in pending.0.drain() {
        let actor = actor.clone();
        let remote = remote.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        receivers.push(rx);
        unavi_wasm_compat::spawn_thread(async move {
            let mut builder = actor.read(hash);
            if let Some(r) = remote {
                builder = builder.sync_from(r.host().clone());
            }
            let result = builder.send().await.map(|doc| (hash, doc));
            let _ = tx.send(result);
        });
    }

    receivers.retain(|rx| match rx.try_recv() {
        Ok(Ok((hash, doc))) => {
            commands.spawn((HsdDoc(Arc::new(doc)), HsdRecordId(hash)));
            false
        }
        Ok(Err(e)) => {
            debug!("dynamic doc fetch skipped: {e}");
            false
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => true,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
    });
}
