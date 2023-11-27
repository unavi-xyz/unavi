use unavi_system_api::menu::{Menu, MenuState};
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
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/unavi_system_api_component.wasm");
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
