use std::sync::{Arc, mpsc};

use bevy::prelude::*;
use bevy_hsd::HsdDoc;
use bevy_wds::LocalActor;
use loro::{LoroDoc, LoroList, LoroTree, TreeParentId};

use crate::asset::Wasm;

/// Source of a WASM module for dev/test use.
pub enum WasmSource {
    Bytes(Vec<u8>),
    Path(String),
}

/// Plugin that loads a WASM module via Bevy's asset system and sets up an
/// HSD document for scripting. For use in examples and tests.
pub struct WasmDevPlugin(pub WasmSource);

impl Plugin for WasmDevPlugin {
    fn build(&self, app: &mut App) {
        match &self.0 {
            WasmSource::Bytes(bytes) => {
                app.insert_resource(DevWasmBytes(bytes.clone()))
                    .add_systems(Startup, load_from_bytes);
            }
            WasmSource::Path(path) => {
                app.insert_resource(DevWasmPath(path.clone()))
                    .add_systems(Startup, load_from_path);
            }
        }
        app.add_systems(Update, upload_and_spawn);
    }
}

#[derive(Resource)]
struct DevWasmBytes(Vec<u8>);

#[derive(Resource)]
struct DevWasmHandle(Handle<Wasm>);

#[derive(Resource)]
struct DevWasmPath(String);

fn load_from_bytes(
    mut commands: Commands,
    mut assets: ResMut<Assets<Wasm>>,
    bytes: Res<DevWasmBytes>,
) {
    let handle = assets.add(Wasm(bytes.0.clone()));
    commands.insert_resource(DevWasmHandle(handle));
}

fn load_from_path(
    mut commands: Commands,
    server: Res<AssetServer>,
    path: Res<DevWasmPath>,
) {
    let handle = server.load::<Wasm>(&path.0);
    commands.insert_resource(DevWasmHandle(handle));
}

fn upload_and_spawn(
    mut commands: Commands,
    handle: Option<Res<DevWasmHandle>>,
    wasm_assets: Res<Assets<Wasm>>,
    actors: Query<&LocalActor>,
    mut pending: Local<Vec<mpsc::Receiver<anyhow::Result<blake3::Hash>>>>,
) {
    let mut still_pending = Vec::new();
    for rx in pending.drain(..) {
        match rx.try_recv() {
            Ok(Ok(hash)) => spawn_hsd_doc(&mut commands, hash),
            Ok(Err(e)) => error!("dev wasm upload: {e}"),
            Err(mpsc::TryRecvError::Empty) => still_pending.push(rx),
            Err(mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *pending = still_pending;

    let Some(ref dev_handle) = handle else { return };
    let Some(wasm) = wasm_assets.get(&dev_handle.0) else { return };
    let Ok(actor) = actors.single() else { return };

    let bytes = bytes::Bytes::from(wasm.0.clone());
    let actor = actor.0.clone();
    let (tx, rx) = mpsc::channel();
    pending.push(rx);
    commands.remove_resource::<DevWasmHandle>();

    unavi_wasm_compat::spawn_thread(async move {
        let _ = tx.send(actor.upload_blob(bytes).await);
    });
}

fn spawn_hsd_doc(commands: &mut Commands, hash: blake3::Hash) {
    let doc = Arc::new(LoroDoc::new());
    {
        let hsd = doc.get_map("hsd");
        let nodes = hsd
            .get_or_create_container("nodes", LoroTree::new())
            .expect("nodes tree");
        let node_id = nodes.create(TreeParentId::Root).expect("create node");
        let meta = nodes.get_meta(node_id).expect("node meta");
        let scripts = meta
            .get_or_create_container("scripts", LoroList::new())
            .expect("scripts list");
        scripts
            .push(hash.as_bytes().to_vec())
            .expect("push hash");
        doc.commit();
    }
    commands.spawn(HsdDoc(doc));
}
