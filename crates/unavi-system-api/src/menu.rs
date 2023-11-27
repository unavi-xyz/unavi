wit_bindgen::generate!({
    world: "menu",
    exports: {
        world: Menu,
    },
});

impl Guest for Menu {
    fn open() {
        log("Menu open");
    }

    fn close() {
        log("Menu close");
    }
}

wasmtime::component::bindgen!("menu");

pub struct MenuState;

impl MenuImports for MenuState {
    fn log(&mut self, msg: String) -> wasmtime::Result<()> {
        println!("Menu log: {}", msg);
        Ok(())
    }
}
