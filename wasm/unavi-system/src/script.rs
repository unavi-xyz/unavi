use self::wired::host::logger::{log, LogLevel};

wit_bindgen::generate!({
    world: "script",
    exports: {
        world: Script,
    },
});

pub struct Script;

impl Guest for Script {
    fn init() {
        log(LogLevel::Info, "Initializing unavi-system!");
    }

    fn update() {
        log(LogLevel::Info, "Updating unavi-system!");
    }

    fn cleanup() {
        log(LogLevel::Info, "Cleaning up unavi-system!");
    }
}
