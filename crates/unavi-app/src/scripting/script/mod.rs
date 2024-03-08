use std::sync::{Arc, Mutex};

use bevy::{ecs::system::CommandQueue, prelude::*, tasks::Task};
use wasm_bridge::{
    component::{new_component_async, Linker},
    AsContextMut, Config, Engine, Store,
};
use wasm_bridge_wasi::preview2::{command, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use super::{asset::Wasm, stream::OutStream};

pub mod logic;
#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod wasm;

wasm_bridge::component::bindgen!({
    async: true,
    path: "../../wired-protocol/spatial/wired-script/world.wit",
    world: "script",
});

#[derive(Resource, Default)]
pub struct ScriptLoadQueue(pub Vec<Handle<Wasm>>);

#[derive(Resource)]
pub struct WasmEngine(pub Arc<Engine>);

impl Default for WasmEngine {
    fn default() -> Self {
        let mut config = Config::default();
        config.async_support(true);
        config.wasm_component_model(true);

        let engine = Engine::new(&config).expect("Failed to create engine");

        Self(Arc::new(engine))
    }
}

pub struct WasmScript {
    pub initialized: bool,
    pub script: Script,
    pub stdout: Arc<Mutex<Vec<u8>>>,
    pub store: Arc<Mutex<Store<StoreState>>>,
}

impl WasmScript {
    pub async fn new(engine: &Engine, bytes: &[u8]) -> Result<Self, wasm_bridge::Error> {
        let mut linker = Linker::new(engine);
        command::add_to_linker(&mut linker)?;

        let out_bytes = Arc::new(Mutex::new(Vec::<u8>::new()));
        let out_stream = OutStream {
            data: out_bytes.clone(),
            max: 512,
        };

        let wasi = WasiCtxBuilder::new().stdout(out_stream).build();

        let mut store = Store::new(
            engine,
            StoreState {
                table: ResourceTable::new(),
                wasi,
            },
        );

        let component = new_component_async(engine, bytes).await?;
        let (script, _) =
            Script::instantiate_async(store.as_context_mut(), &component, &linker).await?;

        Ok(Self {
            initialized: false,
            script,
            stdout: out_bytes,
            store: Arc::new(Mutex::new(store)),
        })
    }
}

pub struct StoreState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl WasiView for StoreState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

#[derive(Component)]
struct LoadScript(Task<CommandQueue>);

pub type ScriptsVec = Arc<Mutex<Vec<Arc<Mutex<WasmScript>>>>>;
