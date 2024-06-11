use std::sync::{Arc, RwLock, RwLockWriteGuard};

use anyhow::{anyhow, bail, Result};
use bevy::render::color::Color;
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, Record, RecordType,
    ResourceType, Store, StoreContextMut, Value, ValueType,
};

use crate::{load::EngineBackend, resource_table::ResourceTable, StoreData};

use super::{
    local_data::{LocalData, MaterialData},
    SharedTypes, WiredGltfAction,
};

#[derive(Clone)]
pub struct MaterialResource(pub u32);

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    shared_types: &SharedTypes,
    sender: Sender<WiredGltfAction>,
    local_data: Arc<RwLock<LocalData>>,
) -> Result<()> {
    let resource_table = store.data().resource_table.clone();
    let interface = linker.define_instance("wired:gltf/material".try_into()?)?;

    let material_type = &shared_types.material_type;

    let material_list_type = ListType::new(ValueType::Own(material_type.clone()));
    let color_type = RecordType::new(
        None,
        [
            ("r", ValueType::F32),
            ("g", ValueType::F32),
            ("b", ValueType::F32),
            ("a", ValueType::F32),
        ]
        .into_iter(),
    )?;

    let material_id_fn = Func::new(
        store.as_context_mut(),
        FuncType::new([ValueType::Borrow(material_type.clone())], [ValueType::U32]),
        move |ctx, args, results| {
            let resource = match &args[0] {
                Value::Borrow(v) => v,
                _ => bail!("invalid arg"),
            };

            let ctx_ref = ctx.as_context();
            let material: &MaterialResource = resource.rep(&ctx_ref)?;

            results[0] = Value::U32(material.0);

            Ok(())
        },
    );

    let material_color_fn = {
        let color_type = color_type.clone();
        let local_data = local_data.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(material_type.clone())],
                [ValueType::Record(color_type.clone())],
            ),
            move |ctx, args, results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let material: &MaterialResource = resource.rep(&ctx_ref)?;

                let local_data = local_data.read().unwrap();

                if let Some(data) = local_data.materials.get(&material.0) {
                    let record = Record::new(
                        color_type.clone(),
                        [
                            ("r", Value::F32(data.color.r())),
                            ("g", Value::F32(data.color.g())),
                            ("b", Value::F32(data.color.b())),
                            ("a", Value::F32(data.color.a())),
                        ]
                        .into_iter(),
                    )?;

                    results[0] = Value::Record(record);
                }

                Ok(())
            },
        )
    };

    let material_set_color_fn = {
        let local_data = local_data.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(material_type.clone()),
                    ValueType::Record(color_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Borrow(v) => v,
                    _ => bail!("invalid arg"),
                };

                let color = match &args[1] {
                    Value::Record(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let material: &MaterialResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();

                if let Some(data) = local_data.materials.get_mut(&material.0) {
                    let r = match color.field("r").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let g = match color.field("g").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let b = match color.field("b").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };
                    let a = match color.field("a").unwrap() {
                        Value::F32(v) => v,
                        _ => bail!("invalid arg"),
                    };

                    data.color = Color::rgba_from_array([r, g, b, a]);

                    sender.send(WiredGltfAction::SetMaterialColor {
                        id: material.0,
                        color: data.color.clone(),
                    })?;
                }

                Ok(())
            },
        )
    };

    let list_materials_fn = {
        let local_data = local_data.clone();
        let material_type = material_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::List(material_list_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let materials = local_data
                    .materials
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|id| {
                        create_material_resource(
                            id,
                            &material_type,
                            &mut ctx,
                            &mut local_data,
                            &mut resource_table,
                        )
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                results[0] = Value::List(
                    List::new(material_list_type.clone(), materials)
                        .expect("failed to create list"),
                );

                Ok(())
            },
        )
    };

    let create_material_fn = {
        let local_data = local_data.clone();
        let material_type = material_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([], [ValueType::Own(material_type.clone())]),
            move |mut ctx, _args, results| {
                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                let id = local_data.new_id();
                local_data.materials.insert(id, MaterialData::default());
                sender.send(WiredGltfAction::CreateMaterial { id })?;

                let value = create_material_resource(
                    id,
                    &material_type,
                    &mut ctx,
                    &mut local_data,
                    &mut resource_table,
                )?;

                results[0] = value;

                Ok(())
            },
        )
    };

    let remove_material_fn = {
        let local_data = local_data.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new([ValueType::Own(material_type.clone())], []),
            move |ctx, args, _results| {
                let resource = match &args[0] {
                    Value::Own(v) => v,
                    _ => bail!("invalid arg"),
                };

                let ctx_ref = ctx.as_context();
                let material: &MaterialResource = resource.rep(&ctx_ref)?;

                let mut local_data = local_data.write().unwrap();
                let mut resource_table = resource_table.write().unwrap();

                if let Some(data) = local_data.materials.remove(&material.0) {
                    for id in data.resources {
                        resource_table.remove(&id);
                    }

                    // TODO: Remove textures (?)
                };

                sender.send(WiredGltfAction::RemoveMaterial { id: material.0 })?;

                Ok(())
            },
        )
    };

    interface.define_resource("material", material_type.clone())?;
    interface.define_func("[method]material.id", material_id_fn)?;
    interface.define_func("[method]material.color", material_color_fn)?;
    interface.define_func("[method]material.set-color", material_set_color_fn)?;

    interface.define_func("list-materials", list_materials_fn)?;
    interface.define_func("create-material", create_material_fn)?;
    interface.define_func("remove-material", remove_material_fn)?;

    Ok(())
}

pub fn create_material_resource(
    id: u32,
    material_type: &ResourceType,
    ctx: &mut StoreContextMut<StoreData, EngineBackend>,
    local_data: &mut RwLockWriteGuard<LocalData>,
    resource_table: &mut RwLockWriteGuard<ResourceTable>,
) -> anyhow::Result<Value> {
    let (res_id, resource) =
        resource_table.push(ctx.as_context_mut(), material_type.clone(), |_| {
            MaterialResource(id)
        })?;

    let data = local_data
        .materials
        .get_mut(&id)
        .ok_or(anyhow!("Material not found"))?;

    data.resources.insert(res_id);

    Ok(Value::Own(resource))
}
