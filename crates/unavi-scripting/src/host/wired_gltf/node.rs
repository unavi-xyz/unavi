use std::sync::{Arc, RwLock};

use anyhow::{bail, Result};
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, ResourceType, Store, Value,
    ValueType,
};

use crate::{load::EngineBackend, StoreData};

use super::{Data, WiredGltfAction};

#[derive(Clone)]
pub struct NodeResource {
    pub id: u32,
}

#[derive(Default)]
struct LocalData {
    node_refs: Vec<u32>,
    nodes: Vec<u32>,
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
    let nodes_list_type = ListType::new(ValueType::Own(node_type.clone()));

    let node_id_fn = {
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Borrow(node_type.clone())], [ValueType::U32]),
            move |ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(r) => r,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let node: &NodeResource = resource.rep(&ctx_ref)?;

                results[0] = Value::U32(node.id);

                Ok(())
            },
        )
    };

    let nodes_fn = {
        let local_data = local_data.clone();
        let node_type = node_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::List(nodes_list_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let nodes = local_data
                    .nodes
                    .clone()
                    .into_iter()
                    .map(|num| -> Result<_, _> {
                        let (id, resource) =
                            resource_table.push(ctx.as_context_mut(), node_type.clone(), |_| {
                                NodeResource { id: num }
                            })?;

                        local_data.node_refs.push(id);

                        Ok(Value::Own(resource))
                    })
                    .collect::<Result<Vec<_>, anyhow::Error>>()?;

                results[0] = Value::List(
                    List::new(nodes_list_type.clone(), nodes).expect("failed to create list"),
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

                let (id, resource) =
                    resource_table.push(ctx.as_context_mut(), node_type.clone(), |id| {
                        NodeResource { id }
                    })?;

                local_data.nodes.push(id);

                sender.send(WiredGltfAction::CreateNode { id })?;

                results[0] = Value::Own(resource);

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
            FuncType::new([ValueType::U32], []),
            move |_ctx, args, _results| {
                let id = match args[0] {
                    Value::U32(num) => num,
                    _ => bail!("invalid arg"),
                };

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                sender.send(WiredGltfAction::RemoveNode { id })?;

                if let Some(index) = local_data.nodes.iter().find(|item| **item == id) {
                    let index = *index as usize;
                    local_data.nodes.remove(index);
                }

                resource_table.remove(&id);

                Ok(())
            },
        )
    };

    interface.define_resource("node", node_type)?;
    interface.define_func("[method]node.id", node_id_fn)?;

    interface.define_func("nodes", nodes_fn)?;
    interface.define_func("create-node", create_node_fn)?;
    interface.define_func("remove-node", remove_node_fn)?;

    Ok(())
}
