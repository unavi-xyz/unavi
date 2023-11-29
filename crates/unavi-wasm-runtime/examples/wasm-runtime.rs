use unavi_system_bindgen::menu::Menu;
use unavi_wasm_runtime::state::State;
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};

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

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/wasm/unavi_system.wasm");
    println!("Loading wasm file: {:?}", path);

    let component = Component::from_file(&engine, &path)?;

    let mut linker = Linker::new(&engine);
    Menu::add_to_linker(&mut linker, |state: &mut State| state)?;

    let mut store = Store::new(&engine, State {});

    let (bindings, _) = Menu::instantiate(&mut store, &component, &linker)?;

    bindings.call_open(&mut store)?;
    bindings.call_close(&mut store)?;

    Ok(())
}
