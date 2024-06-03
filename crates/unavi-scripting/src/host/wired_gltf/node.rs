use std::sync::{Arc, RwLock, RwLockWriteGuard};

use anyhow::{anyhow, bail, Result};
use bevy::utils::{HashMap, HashSet};
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, OptionType, OptionValue,
    ResourceType, Store, StoreContextMut, Value, ValueType,
};

use crate::{load::EngineBackend, resource_table::ResourceTable, StoreData};

use super::{Data, WiredGltfAction};

#[derive(Clone)]
pub struct NodeResource {
    id: u32,
}

impl NodeResource {
    fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Default)]
struct LocalData {
    next_id: u32,
    nodes: HashMap<u32, NodeData>,
}

impl LocalData {
    fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Default)]
struct NodeData {
    children: HashSet<u32>,
    parent: Option<u32>,
    resources: HashSet<u32>,
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    sender: Sender<WiredGltfAction>,
    data: Arc<RwLock<Data>>,
) -> Result<()> {
    let resource_table = store.data().resource_table.clone();
    let interface = linker.define_instance("wired:gltf/node".try_into()?)?;

    let local_data = Arc::new(RwLock::new(LocalData::default()));

    let node_type = ResourceType::new::<NodeResource>(None);
    let node_list_type = ListType::new(ValueType::Own(node_type.clone()));
    let node_option_type = OptionType::new(ValueType::Own(node_type.clone()));

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

                results[0] = Value::U32(node.id);

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

                if let Some(data) = local_data.nodes.get(&node.id) {
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

                if let Some(data) = local_data.nodes.get_mut(&parent_node.id) {
                    data.children.insert(child_node.id);
                }

                if let Some(data) = local_data.nodes.get_mut(&child_node.id) {
                    data.parent = Some(parent_node.id);
                }

                Ok(())
            },
        )
    };

    let node_remove_child_fn = {
        let local_data = local_data.clone();
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

                if let Some(data) = local_data.nodes.get_mut(&parent_node.id) {
                    data.children.remove(&child_node.id);
                }

                if let Some(data) = local_data.nodes.get_mut(&child_node.id) {
                    data.parent = None;
                }

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

                if let Some(data) = local_data.nodes.get(&node.id) {
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

                sender.send(WiredGltfAction::RemoveNode { id: node.id })?;

                if let Some(data) = local_data.nodes.remove(&node.id) {
                    for id in data.resources {
                        resource_table.remove(&id);
                    }
                };

                // TODO: Remove children
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

    interface.define_func("list-nodes", list_nodes_fn)?;
    interface.define_func("create-node", create_node_fn)?;
    interface.define_func("remove-node", remove_node_fn)?;

    Ok(())
}

/// Creates a new node resource for the given node ID.
fn create_node_resource(
    id: u32,
    node_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), node_type.clone(), |_| {
            NodeResource::new(id)
        })?;

    let data = local_data
        .nodes
        .get_mut(&id)
        .ok_or(anyhow!("Node not found"))?;

    data.resources.insert(res_id);

    Ok(Value::Own(resource))
}
