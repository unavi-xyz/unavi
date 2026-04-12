use bevy::prelude::*;

use crate::asset::Wasm;

#[derive(EntityEvent, Clone)]
pub struct LoadLocalScript {
    pub entity: Entity,
    pub source: ScriptSource,
}

#[derive(Clone)]
pub enum ScriptSource {
    Bytes(Vec<u8>),
    Path(String),
}

#[derive(Component)]
pub struct PendingScript(Handle<Wasm>);

pub fn on_load_local_script(
    trigger: On<LoadLocalScript>,
    server: Res<AssetServer>,
    mut assets: ResMut<Assets<Wasm>>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    let mut entity = commands.entity(trigger.entity);

    match &ev.source {
        ScriptSource::Path(path) => {
            let name = path_to_name(path);
            let handle = server.load::<Wasm>(path);

            #[cfg(target_family = "wasm")]
            entity.insert((crate::web::WebPendingScript(handle), Name::new(name)));

            #[cfg(not(target_family = "wasm"))]
            entity.insert((PendingScript(handle), Name::new(name)));
        }
        ScriptSource::Bytes(bytes) => {
            let handle = assets.add(Wasm(bytes.clone()));

            #[cfg(target_family = "wasm")]
            entity.insert(crate::web::WebPendingScript(handle));

            #[cfg(not(target_family = "wasm"))]
            entity.insert(PendingScript(handle));
        }
    }
}

fn path_to_name(path: &str) -> String {
    let name = path.strip_prefix("wasm/").unwrap_or(path);
    let name = name.strip_suffix(".wasm").unwrap_or(name);
    name.replace('/', ":")
}

#[cfg(not(target_family = "wasm"))]
pub use native::{PendingRecord, PendingUpload, poll_local_scripts};

#[cfg(not(target_family = "wasm"))]
mod native {
    use std::{
        sync::{Arc, Mutex, mpsc},
        time::Duration,
    };

    use bevy::prelude::*;
    use bevy_hsd::{HsdDoc, HsdRecordId};
    use bevy_wds::LocalActor;
    use bytes::Bytes;
    use loro::{LoroDoc, LoroList, LoroTree, TreeParentId};
    use wired_schemas::SCHEMA_HSD;

    use super::{PendingScript, Wasm};

    #[derive(Component)]
    pub struct PendingUpload {
        actor: wds::actor::Actor,
        rx: Arc<Mutex<mpsc::Receiver<anyhow::Result<blake3::Hash>>>>,
    }

    #[derive(Component)]
    pub struct PendingRecord {
        rx: Arc<Mutex<mpsc::Receiver<anyhow::Result<(blake3::Hash, LoroDoc)>>>>,
    }

    pub fn poll_local_scripts(
        mut commands: Commands,
        wasm_assets: Res<Assets<Wasm>>,
        actors: Query<&LocalActor>,
        pending_scripts: Query<(Entity, &PendingScript)>,
        pending_records: Query<(Entity, &PendingRecord)>,
        pending_uploads: Query<(Entity, &PendingUpload)>,
    ) {
        for (ent, pending) in &pending_records {
            let Ok(lock) = pending.rx.try_lock() else {
                continue;
            };

            match lock.try_recv() {
                Ok(Ok((id, doc))) => {
                    commands
                        .entity(ent)
                        .remove::<PendingRecord>()
                        .insert((HsdDoc(Arc::new(doc)), HsdRecordId(id)));
                }
                Ok(Err(err)) => {
                    error!("local script record create: {err:?}");
                    commands.entity(ent).remove::<PendingRecord>();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("local script record create disconnected");
                    commands.entity(ent).remove::<PendingRecord>();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        for (ent, pending) in &pending_uploads {
            let Ok(lock) = pending.rx.try_lock() else {
                continue;
            };

            match lock.try_recv() {
                Ok(Ok(hash)) => {
                    let actor = pending.actor.clone();
                    let (tx, rx) = mpsc::channel();
                    unavi_wasm_compat::spawn_thread(async move {
                        let _ = tx.send(create_hsd_record(actor, hash).await);
                        tokio::time::sleep(Duration::from_mins(3)).await;
                    });
                    commands
                        .entity(ent)
                        .remove::<PendingUpload>()
                        .insert(PendingRecord {
                            rx: Arc::new(Mutex::new(rx)),
                        });
                }
                Ok(Err(err)) => {
                    error!("local script upload: {err:?}");
                    commands.entity(ent).remove::<PendingUpload>();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    error!("local script upload disconnected");
                    commands.entity(ent).remove::<PendingUpload>();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        let Ok(actor) = actors.single() else { return };

        for (ent, pending) in &pending_scripts {
            let Some(wasm) = wasm_assets.get(&pending.0) else {
                continue;
            };

            let bytes = Bytes::from(wasm.0.clone());
            let upload_actor = actor.0.clone();
            let record_actor = actor.0.clone();
            let (tx, rx) = mpsc::channel();
            unavi_wasm_compat::spawn_thread(async move {
                let _ = tx.send(upload_actor.upload_blob(bytes).await);
                tokio::time::sleep(Duration::from_mins(3)).await;
            });

            commands
                .entity(ent)
                .remove::<PendingScript>()
                .insert(PendingUpload {
                    actor: record_actor,
                    rx: Arc::new(Mutex::new(rx)),
                });
        }
    }

    async fn create_hsd_record(
        actor: wds::actor::Actor,
        hash: blake3::Hash,
    ) -> anyhow::Result<(blake3::Hash, LoroDoc)> {
        let result = actor
            .create_record()
            .add_schema("hsd", &*SCHEMA_HSD, |doc| {
                let hsd = doc.get_map("hsd");
                let nodes = hsd
                    .get_or_create_container("nodes", LoroTree::new())
                    .map_err(|e| anyhow::anyhow!("nodes tree: {e}"))?;
                let node_id = nodes
                    .create(TreeParentId::Root)
                    .map_err(|e| anyhow::anyhow!("create node: {e}"))?;
                let meta = nodes
                    .get_meta(node_id)
                    .map_err(|e| anyhow::anyhow!("node meta: {e}"))?;
                let scripts = meta
                    .get_or_create_container("scripts", LoroList::new())
                    .map_err(|e| anyhow::anyhow!("scripts list: {e}"))?;
                scripts
                    .push(hash.as_bytes().to_vec())
                    .map_err(|e| anyhow::anyhow!("push hash: {e}"))?;
                doc.commit();
                Ok(())
            })?
            .send()
            .await?;
        Ok((result.id, result.doc))
    }
}
