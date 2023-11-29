use self::wired::host::logger::{log, LogLevel};

wit_bindgen::generate!({
    world: "menu",
    exports: {
        world: Menu,
    },
});

pub struct Menu;

impl Guest for Menu {
    fn open() {
        log(LogLevel::Info, "Menu open");
    }

    fn close() {
        log(LogLevel::Info, "Menu close");
    }
}
