use std::task::Poll;

use anyhow::Context;
use bevy::prelude::*;
use bevy_async_task::TaskPool;
use wasmtime::{Store, component::Linker};

use crate::{Script, WasmBinary, WasmEngine, wasm::Wasm};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-script",
        async: true,
    });
}

type LoadResult = anyhow::Result<(Entity, bindings::Script)>;

#[derive(Component)]
pub struct LoadingScript;

#[derive(Component)]
pub struct LoadedScript(pub bindings::Script);

pub fn load_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    engines: Query<&WasmEngine>,
    to_load: Query<(Entity, &WasmBinary, &Script), (Without<LoadingScript>, Without<LoadedScript>)>,
    mut pool: TaskPool<LoadResult>,
) {
    for (ent, handle, script) in to_load {
        let Ok(engine) = engines.get(script.0) else {
            warn!("Script instantiation failed: engine not found");
            continue;
        };

        let wasm = match wasm_assets.get(&handle.0) {
            Some(a) => a,
            None => continue,
        };

        info!("Loading script. size={}", wasm.0.len());

        let component = wasmtime::component::Component::from_binary(&engine.0, &wasm.0);
        let mut store = Store::new(&engine.0, ());
        let linker = Linker::new(&engine.0);

        pool.spawn(async move {
            let component = component.context("component load")?;
            let out = bindings::Script::instantiate_async(&mut store, &component, &linker)
                .await
                .context("instantiate")?;
            let script = out
                .wired_script_types()
                .script()
                .call_constructor(&mut store)
                .await?;
            out.wired_script_types()
                .script()
                .call_update(&mut store, script, 0.2)
                .await?;
            Ok((ent, out))
        });

        commands.entity(ent).insert(LoadingScript);
    }

    for task in pool.iter_poll() {
        match task {
            Poll::Ready(Ok((ent, script))) => {
                commands
                    .entity(ent)
                    .remove::<LoadingScript>()
                    .insert(LoadedScript(script));
            }
            Poll::Ready(Err(e)) => {
                error!("Error loading script: {e:?}");
            }
            _ => {}
        }
    }
}
