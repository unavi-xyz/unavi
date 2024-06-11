use std::sync::{Arc, RwLock};

use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};
use crossbeam::channel::Receiver;
use wasm_component_layer::{Linker, ResourceType, Store};

use crate::{load::EngineBackend, StoreData};

use self::{local_data::LocalData, material::MaterialResource, mesh::MeshResource};

pub mod handler;
mod local_data;
mod material;
mod mesh;
mod node;
pub mod query;

#[derive(Component, Deref, DerefMut)]
pub struct WiredGltfReceiver(pub Receiver<WiredGltfAction>);

#[derive(Debug)]
pub enum WiredGltfAction {
    CreateMaterial { id: u32 },
    CreateMesh { id: u32 },
    CreateNode { id: u32 },
    CreatePrimitive { id: u32, mesh: u32 },
    RemoveMaterial { id: u32 },
    RemoveMesh { id: u32 },
    RemoveNode { id: u32 },
    RemovePrimitive { id: u32 },
    SetMaterialColor { id: u32, color: Color },
    SetNodeMesh { id: u32, mesh: Option<u32> },
    SetNodeParent { id: u32, parent: Option<u32> },
    SetNodeTransform { id: u32, transform: Transform },
    SetPrimitiveIndices { id: u32, value: Vec<u32> },
    SetPrimitiveNormals { id: u32, value: Vec<f32> },
    SetPrimitivePositions { id: u32, value: Vec<f32> },
    SetPrimitiveUvs { id: u32, value: Vec<f32> },
}

#[derive(Component, Deref, DerefMut)]
pub struct WiredGltfData(pub Arc<RwLock<EcsData>>);

#[derive(Default)]
pub struct EcsData {
    nodes: HashMap<u32, Transform>,
}

struct SharedTypes {
    material_type: ResourceType,
    mesh_type: ResourceType,
}

impl Default for SharedTypes {
    fn default() -> Self {
        let material_type = ResourceType::new::<MaterialResource>(None);
        let mesh_type = ResourceType::new::<MeshResource>(None);
        Self {
            material_type,
            mesh_type,
        }
    }
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<(WiredGltfReceiver, WiredGltfData)> {
    let data = Arc::new(RwLock::new(EcsData::default()));
    let (send, recv) = crossbeam::channel::bounded::<WiredGltfAction>(100);

    let local_data = Arc::new(RwLock::new(LocalData::default()));
    let shared_types = SharedTypes::default();

    node::add_to_host(
        store,
        linker,
        &shared_types,
        send.clone(),
        local_data.clone(),
        data.clone(),
    )?;
    mesh::add_to_host(
        store,
        linker,
        &shared_types,
        send.clone(),
        local_data.clone(),
    )?;
    material::add_to_host(
        store,
        linker,
        &shared_types,
        send.clone(),
        local_data.clone(),
    )?;

    Ok((WiredGltfReceiver(recv), WiredGltfData(data)))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tracing_test::traced_test;

    use crate::{
        asset::{Wasm, WasmLoader},
        load::{LoadedScript, WasmStores},
        ScriptBundle,
    };

    use super::*;

    fn load_component_wasm(asset_server: &AssetServer, name: &str) -> Handle<Wasm> {
        let path = format!("components/{}_{}.wasm", name, env!("CARGO_PKG_VERSION"));
        asset_server.load(path)
    }

    #[tokio::test]
    #[traced_test]
    async fn test_wired_gltf() {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..Default::default()
            },
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Wasm>()
        .init_asset_loader::<WasmLoader>()
        .init_non_send_resource::<WasmStores>();

        app.add_systems(
            Update,
            (
                crate::load::load_scripts,
                crate::host::wired_gltf::query::query_node_data,
                crate::execution::init_scripts,
                crate::execution::update_scripts,
                handler::handle_wired_gltf_actions,
            )
                .chain(),
        );

        let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
        let name = "test_wired_gltf";
        let wasm = load_component_wasm(&asset_server, name);

        app.world.spawn(ScriptBundle {
            name: name.into(),
            wasm,
        });

        tokio::time::sleep(Duration::from_secs_f32(0.2)).await;
        app.update();

        let mut loaded_scripts = app.world.query::<(Entity, &LoadedScript)>();
        let len = loaded_scripts.iter(&app.world).len();
        assert_eq!(len, 1);

        assert!(logs_contain("Hello from script!"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("ERROR"));
    }
}
