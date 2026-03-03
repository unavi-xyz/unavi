use std::sync::mpsc::Receiver;

use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdNodeTreeId, HsdScripts};
use bevy_wds::LocalBlobs;
use smol_str::SmolStr;

use crate::{ScriptEngine, WasmBinary, WasmEngine, asset::Wasm};

/// Links a script entity back to its HSD node.
#[derive(Component)]
pub struct HsdScriptSource {
    pub node_entity: Entity,
    pub doc_entity: Entity,
    pub tree_id: SmolStr,
}

type FetchOut = anyhow::Result<(Entity, Entity, SmolStr, bytes::Bytes)>;

pub fn load_hsd_scripts(
    mut commands: Commands,
    mut wasm_assets: ResMut<Assets<Wasm>>,
    added: Query<(Entity, &HsdScripts, &HsdChild, &HsdNodeTreeId), Added<HsdScripts>>,
    engine: Query<Entity, With<WasmEngine>>,
    local_blobs: Query<&LocalBlobs>,
    mut pending: Local<Vec<Receiver<FetchOut>>>,
) {
    let Ok(engine_ent) = engine.single() else {
        return;
    };

    let blobs = local_blobs.single().ok().map(|b| b.0.clone());

    for (node_ent, scripts, child, tree_id) in &added {
        let Some(ref blobs) = blobs else {
            warn!("no local blobs available for HSD scripts");
            continue;
        };

        let doc_ent = child.doc;
        let tid: SmolStr = tree_id.0.clone();

        for &hash in &scripts.0 {
            let blobs = blobs.clone();
            let tid = tid.clone();
            let (tx, rx) = std::sync::mpsc::channel::<FetchOut>();
            pending.push(rx);

            unavi_wasm_compat::spawn_thread(async move {
                let result = blobs
                    .get_bytes(hash)
                    .await
                    .map(|bytes| (node_ent, doc_ent, tid, bytes))
                    .map_err(|e| anyhow::anyhow!("blob fetch: {e}"));
                let _ = tx.send(result);
            });
        }
    }

    let mut still_pending = Vec::new();
    for rx in pending.drain(..) {
        match rx.try_recv() {
            Ok(Ok((node_ent, doc_ent, tree_id, bytes))) => {
                let handle = wasm_assets.add(Wasm(bytes.to_vec()));
                commands.spawn((
                    ScriptEngine(engine_ent),
                    WasmBinary(handle),
                    HsdScriptSource {
                        node_entity: node_ent,
                        doc_entity: doc_ent,
                        tree_id,
                    },
                ));
            }
            Ok(Err(e)) => {
                error!("failed to fetch HSD script blob: {e:?}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                still_pending.push(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *pending = still_pending;
}

pub fn cleanup_hsd_scripts(
    mut commands: Commands,
    mut removed: RemovedComponents<HsdScripts>,
    sources: Query<(Entity, &HsdScriptSource)>,
) {
    for node_ent in removed.read() {
        for (script_ent, source) in &sources {
            if source.node_entity == node_ent {
                commands.entity(script_ent).despawn();
            }
        }
    }
}
