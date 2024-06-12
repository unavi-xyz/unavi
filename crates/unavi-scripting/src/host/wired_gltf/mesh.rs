use std::sync::{Arc, RwLock, RwLockWriteGuard};

use anyhow::{anyhow, bail, Result};
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, OptionType, ResourceType,
    Store, StoreContextMut, Value, ValueType,
};

use crate::{load::EngineBackend, resource_table::ResourceTable, StoreData};

use super::{
    local_data::{LocalData, MeshData, PrimitiveData},
    material::MaterialResource,
    SharedTypes, WiredGltfAction,
};

#[derive(Clone)]
pub struct MeshResource(pub u32);

#[derive(Clone)]
pub struct PrimitiveResource(pub u32);

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    shared_types: &SharedTypes,
    sender: Sender<WiredGltfAction>,
    local_data: Arc<RwLock<LocalData>>,
) -> Result<()> {
    let resource_table = store.data().resource_table.clone();
    let interface = linker.define_instance("wired:gltf/mesh".try_into()?)?;

    let material_type = &shared_types.material_type;
    let mesh_type = &shared_types.mesh_type;

    let mesh_list_type = ListType::new(ValueType::Own(mesh_type.clone()));
    let primitive_type = ResourceType::new::<PrimitiveResource>(None);
    let primitive_list_type = ListType::new(ValueType::Own(primitive_type.clone()));

    let f32_list_type = ListType::new(ValueType::F32);
    let u32_list_type = ListType::new(ValueType::U32);

    let primitive_id_fn = Func::new(
        store.as_context_mut(),
        FuncType::new(
            [ValueType::Borrow(primitive_type.clone())],
            [ValueType::U32],
        ),
        move |ctx, args, results| {
            let resource = match &args[0] {
                Value::Borrow(v) => v,
                _ => bail!("invalid arg"),
            };

            let ctx_ref = ctx.as_context();
            let primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

            results[0] = Value::U32(primitive.0);

            Ok(())
        },
    );

    let primitive_set_indices_fn = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(primitive_type.clone()),
                    ValueType::List(u32_list_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let value: Vec<u32> = match &args[1] {
                    Value::List(v) => v.typed().unwrap().to_vec(),
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetPrimitiveIndices {
                    id: primitive.0,
                    value,
                })?;

                Ok(())
            },
        )
    };

    let primitive_set_normals_fn = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(primitive_type.clone()),
                    ValueType::List(f32_list_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let value: Vec<f32> = match &args[1] {
                    Value::List(v) => v.typed().unwrap().to_vec(),
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetPrimitiveNormals {
                    id: primitive.0,
                    value,
                })?;

                Ok(())
            },
        )
    };

    let primitive_set_positions_fn = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(primitive_type.clone()),
                    ValueType::List(f32_list_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let value: Vec<f32> = match &args[1] {
                    Value::List(v) => v.typed().unwrap().to_vec(),
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetPrimitivePositions {
                    id: primitive.0,
                    value,
                })?;

                Ok(())
            },
        )
    };

    let primitive_set_uvs_fn = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(primitive_type.clone()),
                    ValueType::List(f32_list_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let value: Vec<f32> = match &args[1] {
                    Value::List(v) => v.typed().unwrap().to_vec(),
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetPrimitiveUvs {
                    id: primitive.0,
                    value,
                })?;

                Ok(())
            },
        )
    };

    let primitive_material_fn = {
        let local_data = local_data.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(primitive_type.clone())],
                [ValueType::Borrow(material_type.clone())],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let _primitive: &PrimitiveResource = resource.rep(&ctx_ref)?;

                let _local_data = local_data.read().unwrap();

                todo!();
            },
        )
    };

    let primitive_set_material_fn = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(primitive_type.clone()),
                    ValueType::Borrow(material_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let res_primitive = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let res_material = match &args[1] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let primitive: &PrimitiveResource = res_primitive.rep(&ctx_ref)?;
                let material: &MaterialResource = res_material.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetPrimitiveMaterial {
                    id: primitive.0,
                    material: material.0,
                })?;

                Ok(())
            },
        )
    };

    let mesh_id_fn = Func::new(
        store.as_context_mut(),
        FuncType::new([ValueType::Borrow(mesh_type.clone())], [ValueType::U32]),
        move |ctx, args, results| {
            let resource = match &args[0] {
                Value::Borrow(v) => v,
                _ => bail!("invalid arg"),
            };

            let ctx_ref = ctx.as_context();
            let mesh: &MeshResource = resource.rep(&ctx_ref)?;

            results[0] = Value::U32(mesh.0);

            Ok(())
        },
    );

    let mesh_list_primitives_fn = {
        let local_data = local_data.clone();
        let primitive_type = primitive_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(mesh_type.clone())],
                [ValueType::List(primitive_list_type.clone())],
            ),
            move |mut ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let mesh: &MeshResource = resource.rep(&ctx_ref)?;
                let mesh_id = mesh.0;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.meshes.get(&mesh_id) {
                    let primitives = data
                        .primitives
                        .keys()
                        .copied()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|id| {
                            create_primitive_resource(
                                id,
                                mesh_id,
                                &primitive_type,
                                &mut ctx,
                                &mut local_data,
                                &mut resource_table,
                            )
                        })
                        .collect::<Result<Vec<_>>>()?;

                    results[0] = Value::List(List::new(primitive_list_type.clone(), primitives)?);
                }

                Ok(())
            },
        )
    };

    let mesh_create_primitive_fn = {
        let local_data = local_data.clone();
        let primitive_type = primitive_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(mesh_type.clone())],
                [ValueType::Own(primitive_type.clone())],
            ),
            move |mut ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let mesh: &MeshResource = resource.rep(&ctx_ref)?;
                let mesh_id = mesh.0;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let id = local_data.new_id();

                if let Some(data) = local_data.meshes.get_mut(&mesh_id) {
                    data.primitives.insert(id, PrimitiveData::default());

                    sender.send(WiredGltfAction::CreatePrimitive { id, mesh: mesh_id })?;

                    let value = create_primitive_resource(
                        id,
                        mesh_id,
                        &primitive_type,
                        &mut ctx,
                        &mut local_data,
                        &mut resource_table,
                    )?;

                    results[0] = value;
                }

                Ok(())
            },
        )
    };

    let mesh_remove_primitive_fn = {
        let local_data = local_data.clone();
        let primitive_type = primitive_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(mesh_type.clone()),
                    ValueType::Own(primitive_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let mesh_res = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let primitive_res = match &args[1] {
                    Value::Own(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let mesh: &MeshResource = mesh_res.rep(&ctx_ref)?;
                let primitive: &PrimitiveResource = primitive_res.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::RemovePrimitive { id: primitive.0 })?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.meshes.get_mut(&mesh.0) {
                    if let Some(primitive_data) = data.primitives.remove(&primitive.0) {
                        for id in primitive_data.resources {
                            resource_table.remove(&id);
                        }
                    }
                }

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

                sender.send(WiredGltfAction::RemoveMesh { id: mesh.0 })?;

                if let Some(data) = local_data.meshes.remove(&mesh.0) {
                    for id in data.resources {
                        resource_table.remove(&id);
                    }

                    for p_data in data.primitives.values() {
                        // TODO: move into a function with mesh_remove_primitive_fn logic

                        for id in &p_data.resources {
                            resource_table.remove(id);
                        }
                    }
                };

                Ok(())
            },
        )
    };

    interface.define_resource("primitive", primitive_type)?;
    interface.define_func("[method]primitive.id", primitive_id_fn)?;
    interface.define_func("[method]primitive.set-indices", primitive_set_indices_fn)?;
    interface.define_func("[method]primitive.set-normals", primitive_set_normals_fn)?;
    interface.define_func(
        "[method]primitive.set-positions",
        primitive_set_positions_fn,
    )?;
    interface.define_func("[method]primitive.set-uvs", primitive_set_uvs_fn)?;
    interface.define_func("[method]primitive.material", primitive_material_fn)?;
    interface.define_func("[method]primitive.set-material", primitive_set_material_fn)?;

    interface.define_resource("mesh", mesh_type.clone())?;
    interface.define_func("[method]mesh.id", mesh_id_fn)?;
    interface.define_func("[method]mesh.list-primitives", mesh_list_primitives_fn)?;
    interface.define_func("[method]mesh.create-primitive", mesh_create_primitive_fn)?;
    interface.define_func("[method]mesh.remove-primitive", mesh_remove_primitive_fn)?;

    interface.define_func("list-meshes", list_meshes_fn)?;
    interface.define_func("create-mesh", create_mesh_fn)?;
    interface.define_func("remove-mesh", remove_mesh_fn)?;

    Ok(())
}

pub fn create_mesh_resource(
    id: u32,
    mesh_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), mesh_type.clone(), |_| {
            MeshResource(id)
        })?;

    let data = local_data
        .meshes
        .get_mut(&id)
        .ok_or(anyhow!("Mesh not found"))?;

    data.resources.insert(res_id);

    Ok(Value::Own(resource))
}

fn create_primitive_resource(
    id: u32,
    mesh_id: u32,
    primitive_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), primitive_type.clone(), |_| {
            PrimitiveResource(id)
        })?;

    let mesh_data = local_data
        .meshes
        .get_mut(&mesh_id)
        .ok_or(anyhow!("Mesh not found"))?;

    let primitive_data = match mesh_data.primitives.get_mut(&id) {
        Some(d) => d,
        None => {
            mesh_data.primitives.insert(id, PrimitiveData::default());
            mesh_data.primitives.get_mut(&id).unwrap()
        }
    };

    primitive_data.resources.insert(res_id);

    Ok(Value::Own(resource))
}
