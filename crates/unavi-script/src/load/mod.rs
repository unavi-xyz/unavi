use std::{sync::Arc, task::Poll};

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use log::{ScriptStderr, ScriptStdout};
use wasmtime::{
    AsContextMut, Store,
    component::{Linker, ResourceTable},
};
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use wired::ecs::WiredEcsData;

use crate::{
    Script, WasmBinary, WasmEngine,
    asset::Wasm,
    execute::{RuntimeCtx, ScriptRuntime},
};

pub(crate) mod log;
mod wired;

pub mod bindings {
    wasmtime::component::bindgen!({
        world: "guest",
        path: "../../protocol/wit/wired-ecs",
        async: true,
    });
}

type LoadResult = anyhow::Result<(Entity, bindings::Guest)>;

#[derive(Component)]
pub struct LoadingScript;

#[derive(Component)]
#[require(Executing)]
pub struct LoadedScript(pub Arc<bindings::Guest>);

#[derive(Component, Default, Deref, DerefMut)]
pub struct Executing(bool);

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    data: RuntimeData,
}

#[derive(Default)]
pub struct RuntimeData {
    pub wired_ecs: WiredEcsData,
}

impl IoView for StoreState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

impl WasiView for StoreState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

pub fn load_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    engines: Query<&WasmEngine>,
    to_load: Query<
        (Entity, &WasmBinary, &Script, Option<&Name>),
        (Without<LoadingScript>, Without<LoadedScript>),
    >,
    mut pool: TaskPool<LoadResult>,
) {
    for (ent, handle, script, name) in to_load {
        let Ok(engine) = engines.get(script.0) else {
            warn!("Script instantiation failed: engine not found");
            continue;
        };

        let wasm = match wasm_assets.get(&handle.0) {
            Some(a) => a,
            None => continue,
        };

        let name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        info!("Loading script {name}: size={}", wasm.0.len());

        let (stdout, stdout_stream) = ScriptStdout::new();
        let (stderr, stderr_stream) = ScriptStderr::new();
        let wasi = WasiCtxBuilder::new()
            .stdout(stdout_stream)
            .stderr(stderr_stream)
            .build();
        let state = StoreState {
            wasi,
            resource_table: ResourceTable::default(),
            data: RuntimeData::default(),
        };
        let mut store = Store::new(&engine.0, state);
        store.epoch_deadline_async_yield_and_update(1);

        let component = wasmtime::component::Component::from_binary(&engine.0, &wasm.0);

        let rt = ScriptRuntime::new(store, stdout, stderr);
        let ctx = rt.ctx.clone();
        commands.entity(ent).insert((LoadingScript, rt));

        pool.spawn(async move {
            let mut ctx = ctx.lock().await;
            let res = instantiate_component(ent, component, &mut ctx)
                .await
                .with_context(|| name)?;
            Ok(res)
        });
    }

    for task in pool.iter_poll() {
        match task {
            Poll::Ready(Ok((ent, script))) => {
                commands
                    .entity(ent)
                    .remove::<LoadingScript>()
                    .insert(LoadedScript(Arc::new(script)));
            }
            Poll::Ready(Err(e)) => {
                error!("Error loading script: {e:?}");
            }
            _ => {}
        }
    }
}

async fn instantiate_component(
    ent: Entity,
    component: Result<wasmtime::component::Component, anyhow::Error>,
    rt: &mut RuntimeCtx,
) -> LoadResult {
    let mut linker = Linker::new(rt.store.engine());
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).context("add wasi to linker")?;
    wired::add_to_linker(&mut linker)?;

    let component = component.context("component load")?;

    let guest = bindings::Guest::instantiate_async(rt.store.as_context_mut(), &component, &linker)
        .await
        .context("instantiate guest")?;

    Ok((ent, guest))
}
