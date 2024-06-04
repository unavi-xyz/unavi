use std::sync::{Arc, RwLock, RwLockWriteGuard};

use anyhow::{anyhow, bail, Result};
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, ResourceType, Store,
    StoreContextMut, Value, ValueType,
};

use crate::{load::EngineBackend, resource_table::ResourceTable, StoreData};

use super::{LocalData, MeshData, WiredGltfAction};

#[derive(Clone)]
pub struct MeshResource {
    id: u32,
}

impl MeshResource {
    fn new(id: u32) -> Self {
        Self { id }
    }
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    sender: Sender<WiredGltfAction>,
    local_data: Arc<RwLock<LocalData>>,
) -> Result<()> {
    let resource_table = store.data().resource_table.clone();
    let interface = linker.define_instance("wired:gltf/mesh".try_into()?)?;

    let mesh_type = ResourceType::new::<MeshResource>(None);
    let mesh_list_type = ListType::new(ValueType::Own(mesh_type.clone()));

    let mesh_id_fn = {
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Borrow(mesh_type.clone())], [ValueType::U32]),
            move |ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let mesh: &MeshResource = resource.rep(&ctx_ref)?;

                results[0] = Value::U32(mesh.id);

                Ok(())
            },
        )
    };

    let list_meshes_fn = {
        let local_data = local_data.clone();
        let mesh_type = mesh_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::List(mesh_list_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let meshes = local_data
                    .meshes
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|id| {
                        create_mesh_resource(
                            id,
                            &mesh_type,
                            &mut ctx,
                            &mut local_data,
                            &mut resource_table,
                        )
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                results[0] = Value::List(
                    List::new(mesh_list_type.clone(), meshes).expect("failed to create list"),
                );

                Ok(())
            },
        )
    };

    let create_mesh_fn = {
        let local_data = local_data.clone();
        let mesh_type = mesh_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::Own(mesh_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let id = local_data.new_id();
                local_data.meshes.insert(id, MeshData::default());
                sender.send(WiredGltfAction::CreateMesh { id })?;

                let value = create_mesh_resource(
                    id,
                    &mesh_type,
                    &mut ctx,
                    &mut local_data,
                    &mut resource_table,
                )?;

                results[0] = value;

                Ok(())
            },
        )
    };

    let remove_mesh_fn = {
        let local_data = local_data.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Own(mesh_type.clone())], []),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Own(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let mesh: &MeshResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                sender.send(WiredGltfAction::RemoveMesh { id: mesh.id })?;

                if let Some(data) = local_data.meshes.remove(&mesh.id) {
                    for id in data.resources {
                        resource_table.remove(&id);
                    }
                };

                // TODO: Remove material (?)

                Ok(())
            },
        )
    };

    interface.define_resource("mesh", mesh_type)?;
    interface.define_func("[method]mesh.id", mesh_id_fn)?;

    interface.define_func("list-meshes", list_meshes_fn)?;
    interface.define_func("create-mesh", create_mesh_fn)?;
    interface.define_func("remove-mesh", remove_mesh_fn)?;

    Ok(())
}

/// Creates a new mesh resource for the given mesh ID.
fn create_mesh_resource(
    id: u32,
    mesh_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), mesh_type.clone(), |_| {
            MeshResource::new(id)
        })?;

    let data = local_data
        .meshes
        .get_mut(&id)
        .ok_or(anyhow!("Mesh not found"))?;

    data.resources.insert(res_id);

    Ok(Value::Own(resource))
}
