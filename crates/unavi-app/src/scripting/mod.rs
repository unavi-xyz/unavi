use crate::state::AppState;
use bevy::prelude::*;
use wasmtime::Engine;

pub mod asset;
pub mod state;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .add_systems(OnEnter(AppState::InWorld), setup_runtimes)
            .add_systems(Update, add_scripts_to_runtime);
    }
}

#[derive(Component)]
pub struct WasmRuntime {
    pub engine: wasmtime::Engine,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);

        let engine = Engine::new(&config).unwrap();

        Self { engine }
    }
}

#[derive(Component)]
pub struct WasmScript {
    pub asset: Handle<asset::Wasm>,
    /// Whether the script has been consumed by a runtime.
    pub consumed: bool,
}

impl WasmScript {
    pub fn new(asset: Handle<asset::Wasm>) -> Self {
        Self {
            asset,
            consumed: false,
        }
    }
}

fn setup_runtimes(mut commands: Commands, asset_server: Res<AssetServer>) {
    let unavi_system = commands
        .spawn(WasmScript::new(
            asset_server.load::<asset::Wasm>("scripts/unavi_system.wasm"),
        ))
        .id();

    commands
        .spawn((Name::new("system"), WasmRuntime::default()))
        .add_child(unavi_system);

    commands.spawn((Name::new("world"), WasmRuntime::default()));
}

fn add_scripts_to_runtime(
    assets: Res<Assets<asset::Wasm>>,
    runtimes: Query<&WasmRuntime>,
    mut scripts: Query<(&Parent, &mut WasmScript)>,
) {
    for (parent, mut script) in scripts.iter_mut() {
        if script.consumed {
            continue;
        }

        let wasm = match assets.get(&script.asset) {
            Some(wasm) => wasm,
            None => continue,
        };

        let runtime = match runtimes.get(parent.get()) {
            Ok(runtime) => runtime,
            Err(e) => {
                error!("Failed to get runtime: {}", e);
                continue;
            }
        };

        script.consumed = true;

        let component =
            match wasmtime::component::Component::from_binary(&runtime.engine, &wasm.bytes) {
                Ok(component) => component,
                Err(e) => {
                    error!("Failed to load script: {}", e);
                    continue;
                }
            };

        let mut linker = wasmtime::component::Linker::new(&runtime.engine);

        if let Err(e) = wired_host_bindgen::script::Script::add_to_linker(
            &mut linker,
            |state: &mut state::State| state,
        ) {
            error!("Failed to add script to linker: {}", e);
            continue;
        }

        let mut store = wasmtime::Store::new(&runtime.engine, state::State);

        match wired_host_bindgen::script::Script::instantiate(&mut store, &component, &linker) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to instantiate script: {}", e);
                continue;
            }
        }

        // let mut linker = wasmtime::component::Linker::new(&runtime.engine);
    }
}
