use std::sync::{Arc, RwLock, RwLockWriteGuard};

use anyhow::{anyhow, bail, Result};
use bevy::prelude::*;
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, OptionType, OptionValue,
    Record, RecordType, ResourceType, Store, StoreContextMut, Value, ValueType,
};

use crate::{load::EngineBackend, resource_table::ResourceTable, StoreData};

use super::{
    local_data::{LocalData, NodeData},
    mesh::{create_mesh_resource, MeshResource},
    EcsData, SharedTypes, WiredGltfAction,
};

#[derive(Clone)]
pub struct NodeResource(pub u32);

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    shared_types: &SharedTypes,
    sender: Sender<WiredGltfAction>,
    local_data: Arc<RwLock<LocalData>>,
    ecs_data: Arc<RwLock<EcsData>>,
) -> Result<()> {
    let resource_table = store.data().resource_table.clone();
    let interface = linker.define_instance("wired:gltf/node".try_into()?)?;

    let mesh_type = &shared_types.mesh_type;

    let node_type = ResourceType::new::<NodeResource>(None);
    let node_list_type = ListType::new(ValueType::Own(node_type.clone()));
    let node_option_type = OptionType::new(ValueType::Own(node_type.clone()));

    let vec3_type = RecordType::new(
        None,
        [
            ("x", ValueType::F32),
            ("y", ValueType::F32),
            ("z", ValueType::F32),
        ]
        .into_iter(),
    )?;
    let quat_type = RecordType::new(
        None,
        [
            ("x", ValueType::F32),
            ("y", ValueType::F32),
            ("z", ValueType::F32),
            ("w", ValueType::F32),
        ]
        .into_iter(),
    )?;
    let transform_type = RecordType::new(
        None,
        [
            ("translation", ValueType::Record(vec3_type.clone())),
            ("rotation", ValueType::Record(quat_type.clone())),
            ("scale", ValueType::Record(vec3_type.clone())),
        ]
        .into_iter(),
    )?;

    let node_id_fn = {
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Borrow(node_type.clone())], [ValueType::U32]),
            move |ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                results[0] = Value::U32(node.0);

                Ok(())
            },
        )
    };

    let node_children_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let node_list_type = node_list_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(node_type.clone())],
                [ValueType::List(node_list_type.clone())],
            ),
            move |mut ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.nodes.get(&node.0) {
                    let children = data
                        .children
                        .clone()
                        .into_iter()
                        .map(|id| {
                            create_node_resource(
                                id,
                                &node_type,
                                &mut ctx,
                                &mut local_data,
                                &mut resource_table,
                            )
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?;

                    results[0] = Value::List(
                        List::new(node_list_type.clone(), children).expect("failed to create list"),
                    );
                };

                Ok(())
            },
        )
    };

    let node_add_child_fn = {
        let local_data = local_data.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(node_type.clone()),
                    ValueType::Borrow(node_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let parent = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let child = match &args[1] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let parent_node: &NodeResource = parent.rep(&ctx_ref)?;
                let child_node: &NodeResource = child.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&parent_node.0) {
                    data.children.insert(child_node.0);
                }

                if let Some(data) = local_data.nodes.get_mut(&child_node.0) {
                    if let Some(prev_id) = data.parent {
                        if let Some(prev_data) = local_data.nodes.get_mut(&prev_id) {
                            prev_data.children.remove(&child_node.0);
                        }
                    }
                }

                if let Some(data) = local_data.nodes.get_mut(&child_node.0) {
                    data.parent = Some(parent_node.0);
                }

                sender.send(WiredGltfAction::SetNodeParent {
                    id: child_node.0,
                    parent: Some(parent_node.0),
                })?;

                Ok(())
            },
        )
    };

    let node_remove_child_fn = {
        let local_data = local_data.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(node_type.clone()),
                    ValueType::Borrow(node_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let parent = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let child = match &args[1] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let parent_node: &NodeResource = parent.rep(&ctx_ref)?;
                let child_node: &NodeResource = child.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&parent_node.0) {
                    data.children.remove(&child_node.0);
                }

                if let Some(data) = local_data.nodes.get_mut(&child_node.0) {
                    data.parent = None;
                }

                sender.send(WiredGltfAction::SetNodeParent {
                    id: child_node.0,
                    parent: None,
                })?;

                Ok(())
            },
        )
    };

    let node_parent_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(node_type.clone())],
                [ValueType::Option(node_option_type.clone())],
            ),
            move |mut ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.nodes.get(&node.0) {
                    let value = match data.parent {
                        Some(parent) => {
                            let value = create_node_resource(
                                parent,
                                &node_type,
                                &mut ctx,
                                &mut local_data,
                                &mut resource_table,
                            )?;
                            Some(value)
                        }
                        None => None,
                    };

                    let value = OptionValue::new(node_option_type.clone(), value)?;

                    results[0] = Value::Option(value);
                }

                Ok(())
            },
        )
    };

    let node_transform_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let transform_type = transform_type.clone();
        let vec3_type = vec3_type.clone();
        let vec4_type = quat_type.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(node_type.clone())],
                [ValueType::Record(transform_type.clone())],
            ),
            move |ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let local_data = local_data.read().unwrap();

                if let Some(data) = local_data.nodes.get(&node.0) {
                    let tr = data.transform;

                    let translation = Record::new(
                        vec3_type.clone(),
                        [
                            ("x", Value::F32(tr.translation.x)),
                            ("y", Value::F32(tr.translation.y)),
                            ("z", Value::F32(tr.translation.z)),
                        ]
                        .into_iter(),
                    )?;
                    let rotation = Record::new(
                        vec4_type.clone(),
                        [
                            ("x", Value::F32(tr.rotation.x)),
                            ("y", Value::F32(tr.rotation.y)),
                            ("z", Value::F32(tr.rotation.z)),
                            ("w", Value::F32(tr.rotation.w)),
                        ]
                        .into_iter(),
                    )?;
                    let scale = Record::new(
                        vec3_type.clone(),
                        [
                            ("x", Value::F32(tr.scale.x)),
                            ("y", Value::F32(tr.scale.y)),
                            ("z", Value::F32(tr.scale.z)),
                        ]
                        .into_iter(),
                    )?;
                    let record = Record::new(
                        transform_type.clone(),
                        [
                            ("translation", Value::Record(translation)),
                            ("rotation", Value::Record(rotation)),
                            ("scale", Value::Record(scale)),
                        ]
                        .into_iter(),
                    )?;

                    results[0] = Value::Record(record);
                }

                Ok(())
            },
        )
    };

    let node_set_transform_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(node_type.clone()),
                    ValueType::Record(transform_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let transform = match &args[1] {
                    Value::Record(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&node.0) {
                    let translation = match transform.field("translation").unwrap() {
                        Value::Record(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let rotation = match transform.field("rotation").unwrap() {
                        Value::Record(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let scale = match transform.field("scale").unwrap() {
                        Value::Record(v) => v,
                        _ => bail!("invalid arg"),
                    };

                    let tr_x = match translation.field("x").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let tr_y = match translation.field("y").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let tr_z = match translation.field("z").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };

                    let rot_x = match rotation.field("x").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let rot_y = match rotation.field("y").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let rot_z = match rotation.field("z").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let rot_w = match rotation.field("w").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };

                    let scale_x = match scale.field("x").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let scale_y = match scale.field("y").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let scale_z = match scale.field("z").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };

                    data.transform.translation.x = tr_x;
                    data.transform.translation.y = tr_y;
                    data.transform.translation.z = tr_z;

                    data.transform.rotation.x = rot_x;
                    data.transform.rotation.y = rot_y;
                    data.transform.rotation.z = rot_z;
                    data.transform.rotation.w = rot_w;

                    data.transform.scale.x = scale_x;
                    data.transform.scale.y = scale_y;
                    data.transform.scale.z = scale_z;

                    sender.send(WiredGltfAction::SetNodeTransform {
                        id: node.0,
                        transform: Transform {
                            translation: Vec3::new(tr_x, tr_y, tr_z),
                            rotation: Quat::from_xyzw(rot_x, rot_y, rot_z, rot_w),
                            scale: Vec3::new(scale_x, scale_y, scale_z),
                        },
                    })?;
                }

                Ok(())
            },
        )
    };

    let node_mesh_fn = {
        let local_data = local_data.clone();
        let mesh_type = mesh_type.clone();
        let resource_table = resource_table.clone();

        let mesh_option_type = OptionType::new(ValueType::Own(mesh_type.clone()));

        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(node_type.clone())],
                [ValueType::Option(mesh_option_type.clone())],
            ),
            move |mut ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&node.0) {
                    let mesh_res = match data.mesh {
                        Some(mesh_id) => Some(create_mesh_resource(
                            mesh_id,
                            &mesh_type,
                            &mut ctx.as_context_mut(),
                            &mut local_data,
                            &mut resource_table,
                        )?),

                        None => None,
                    };

                    results[0] =
                        Value::Option(OptionValue::new(mesh_option_type.clone(), mesh_res)?);
                }

                Ok(())
            },
        )
    };

    let node_set_mesh_fn = {
        let local_data = local_data.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(node_type.clone()),
                    ValueType::Borrow(mesh_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let node_res = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let mesh_res = match &args[1] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = node_res.rep(&ctx_ref)?;
                let mesh: &MeshResource = mesh_res.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetNodeMesh {
                    id: node.0,
                    mesh: Some(mesh.0),
                })?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&node.0) {
                    data.mesh = Some(mesh.0);
                }

                Ok(())
            },
        )
    };

    let node_remove_mesh_fn = {
        let local_data = local_data.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Borrow(node_type.clone())], []),
            move |ctx, args, _results| {
                let node_res = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = node_res.rep(&ctx_ref)?;

                sender.send(WiredGltfAction::SetNodeMesh {
                    id: node.0,
                    mesh: None,
                })?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.nodes.get_mut(&node.0) {
                    data.mesh = None;
                }

                Ok(())
            },
        )
    };

    let list_nodes_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::List(node_list_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let nodes = local_data
                    .nodes
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|id| {
                        create_node_resource(
                            id,
                            &node_type,
                            &mut ctx,
                            &mut local_data,
                            &mut resource_table,
                        )
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                results[0] = Value::List(
                    List::new(node_list_type.clone(), nodes).expect("failed to create list"),
                );

                Ok(())
            },
        )
    };

    let create_node_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::Own(node_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let id = local_data.new_id();
                local_data.nodes.insert(id, NodeData::default());
                sender.send(WiredGltfAction::CreateNode { id })?;

                let value = create_node_resource(
                    id,
                    &node_type,
                    &mut ctx,
                    &mut local_data,
                    &mut resource_table,
                )?;

                results[0] = value;

                Ok(())
            },
        )
    };

    let remove_node_fn = {
        let local_data = local_data.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Own(node_type.clone())], []),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Own(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                sender.send(WiredGltfAction::RemoveNode { id: node.0 })?;

                if let Some(data) = local_data.nodes.remove(&node.0) {
                    for id in data.resources {
                        resource_table.remove(&id);
                    }
                };

                // TODO: Remove children (?)
                // TODO: Remove mesh (?)

                Ok(())
            },
        )
    };

    interface.define_resource("node", node_type)?;
    interface.define_func("[method]node.id", node_id_fn)?;
    interface.define_func("[method]node.children", node_children_fn)?;
    interface.define_func("[method]node.add-child", node_add_child_fn)?;
    interface.define_func("[method]node.remove-child", node_remove_child_fn)?;
    interface.define_func("[method]node.parent", node_parent_fn)?;
    interface.define_func("[method]node.transform", node_transform_fn)?;
    interface.define_func("[method]node.set-transform", node_set_transform_fn)?;
    interface.define_func("[method]node.mesh", node_mesh_fn)?;
    interface.define_func("[method]node.set-mesh", node_set_mesh_fn)?;
    interface.define_func("[method]node.remove-mesh", node_remove_mesh_fn)?;

    interface.define_func("list-nodes", list_nodes_fn)?;
    interface.define_func("create-node", create_node_fn)?;
    interface.define_func("remove-node", remove_node_fn)?;

    Ok(())
}

fn create_node_resource(
    id: u32,
    node_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), node_type.clone(), |_| {
            NodeResource(id)
        })?;

    let data = local_data
        .nodes
        .get_mut(&id)
        .ok_or(anyhow!("Node not found"))?;

    data.resources.insert(res_id);

    Ok(Value::Own(resource))
}
