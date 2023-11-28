wit_bindgen::generate!({
    world: "menu",
    exports: {
        world: Menu,
    },
});

use wired::system::logger::{log, LogLevel};

pub struct Menu;

impl Guest for Menu {
    fn open() {
        log(LogLevel::Info, "Menu open");
    }

    fn close() {
        log(LogLevel::Info, "Menu close");
    }
}
