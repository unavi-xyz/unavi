use unavi_system_bindgen::menu::{wired::system::logger, Menu};
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};

struct MenuState;

impl logger::Host for MenuState {
    fn log(&mut self, level: logger::LogLevel, msg: String) -> wasmtime::Result<()> {
        match level {
            logger::LogLevel::Debug => tracing::debug!("{}", msg),
            logger::LogLevel::Info => tracing::info!("{}", msg),
            logger::LogLevel::Warn => tracing::warn!("{}", msg),
            logger::LogLevel::Error => tracing::error!("{}", msg),
        };

        Ok(())
    }
}

fn main() {
    if let Err(e) = load_wasm() {
        panic!("Error: {}", e);
    }
}

fn load_wasm() -> wasmtime::Result<()> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config)?;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/unavi_system.wasm");
    println!("Loading wasm file: {:?}", path);

    let component = Component::from_file(&engine, &path)?;

    let mut linker = Linker::new(&engine);
    Menu::add_to_linker(&mut linker, |state: &mut MenuState| state)?;

    let mut store = Store::new(&engine, MenuState {});

    let (bindings, _) = Menu::instantiate(&mut store, &component, &linker)?;

    bindings.call_open(&mut store)?;
    bindings.call_close(&mut store)?;

    Ok(())
}
