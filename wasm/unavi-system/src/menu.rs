wit_bindgen::generate!({
    world: "menu",
    exports: {
        world: Menu,
    },
});

pub struct Menu;

impl Guest for Menu {
    fn open() {
        log("Menu open");
    }

    fn close() {
        log("Menu close");
    }
}
