use exports::wired::host::logger::{Guest, LogLevel};

wit_bindgen::generate!({
    world: "host",
    exports: {
        "wired:host/logger": Logger,
    },
});

pub struct Logger;

impl Guest for Logger {
    fn log(level: LogLevel, msg: String) {
        println!("{:?}: {}", level, msg);
    }
}
