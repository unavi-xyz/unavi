use super::{asset::Wasm, state::ScriptState};
use bevy::prelude::*;
use wired_host_bindgen::script::Script;

#[derive(Component)]
pub struct WasmEngine(wasmtime::Engine);

#[derive(Component)]
pub struct WasmLinker<T>(wasmtime::component::Linker<T>);

#[derive(Component)]
pub struct WasmStore<T>(pub(crate) wasmtime::Store<T>);

#[derive(Bundle)]
pub struct WasmRuntimeBundle<T: Send + Sync + 'static> {
    pub engine: WasmEngine,
    pub linker: WasmLinker<T>,
    pub store: WasmStore<T>,
}

impl<T: Send + Sync + 'static> WasmRuntimeBundle<T> {
    fn new(state: T) -> Self {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);

        let engine = wasmtime::Engine::new(&config).unwrap();
        let linker = wasmtime::component::Linker::new(&engine);
        let store = wasmtime::Store::new(&engine, state);

        Self {
            engine: WasmEngine(engine),
            linker: WasmLinker(linker),
            store: WasmStore(store),
        }
    }
}

pub type ScriptRuntimeBundle = WasmRuntimeBundle<ScriptState>;

impl Default for ScriptRuntimeBundle {
    fn default() -> Self {
        Self::new(ScriptState)
    }
}

#[derive(Component)]
pub struct WasmScript {
    pub asset: Handle<Wasm>,
    /// Whether the script has been processed.
    pub processed: bool,
}

impl WasmScript {
    pub fn new(asset: Handle<Wasm>) -> Self {
        Self {
            asset,
            processed: false,
        }
    }

    pub fn load(asset_server: Res<AssetServer>, path: String) -> Self {
        let handle = asset_server.load::<Wasm>(path);
        Self::new(handle)
    }
}

#[derive(Component)]
pub struct InstantiatedScript {
    pub script: Script,
    pub initialized: bool,
}

impl InstantiatedScript {
    pub fn new(script: Script) -> Self {
        Self {
            script,
            initialized: false,
        }
    }
}

pub fn instantiate_scripts(
    assets: Res<Assets<Wasm>>,
    mut commands: Commands,
    mut runtimes: Query<(
        &WasmEngine,
        &mut WasmLinker<ScriptState>,
        &mut WasmStore<ScriptState>,
    )>,
    mut scripts: Query<(Entity, &Parent, &mut WasmScript)>,
) {
    for (entity, parent, mut wasm_script) in scripts.iter_mut() {
        if wasm_script.processed {
            continue;
        }

        let wasm = match assets.get(&wasm_script.asset) {
            Some(wasm) => wasm,
            None => continue,
        };

        let (engine, mut linker, mut store) = match runtimes.get_mut(parent.get()) {
            Ok(runtime) => runtime,
            Err(e) => {
                error!("Failed to get runtime: {}", e);
                continue;
            }
        };

        wasm_script.processed = true;

        let component = match wasmtime::component::Component::from_binary(&engine.0, &wasm.bytes) {
            Ok(component) => component,
            Err(e) => {
                error!("Failed to load script: {}", e);
                continue;
            }
        };

        if let Err(e) = Script::add_to_linker(&mut linker.0, |state: &mut ScriptState| state) {
            error!("Failed to add script to linker: {}", e);
            continue;
        }

        let script = match Script::instantiate(&mut store.0, &component, &linker.0) {
            Ok((script, _)) => script,
            Err(e) => {
                error!("Failed to instantiate script: {}", e);
                continue;
            }
        };

        commands
            .entity(entity)
            .insert(InstantiatedScript::new(script));
    }
}
