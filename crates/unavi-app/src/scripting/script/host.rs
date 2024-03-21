use wasm_bridge::{component::Linker, Store};

use super::StoreState;

mod ui_imports {
    wasm_bridge::component::bindgen!({
        async: true,
        path: "../../components/unavi-ui/wit/world.wit",
        world: "imports",
    });
}

mod ui_host {
    wasm_bridge::component::bindgen!({
        async: true,
        path: "../../components/unavi-ui/wit/world.wit",
        world: "host",
    });
}

/// Add provided host components to the linker and store.
pub async fn add_host_components(
    _linker: &mut Linker<StoreState>,
    _store: &mut Store<StoreState>,
) -> Result<(), wasm_bridge::Error> {
    // let bytes = UNAVI_UI.to_vec();
    //
    // let component = new_component_async(store.engine(), bytes).await?;
    // let _ =
    //     self::ui_host::Host_::instantiate_async(store.as_context_mut(), &component, linker).await?;

    Ok(())
}
