use self::wired::host::{
    logger::{log, LogLevel},
    process, time,
};

wit_bindgen::generate!({
    world: "script",
    exports: {
        world: Script,
    },
});

pub struct Script;

impl Guest for Script {
    fn init() {
        log(
            LogLevel::Info,
            format!("Initializing unavi-system at {}", time::elapsed_seconds()).as_str(),
        );
    }

    fn update(delta_seconds: f32) {
        log(
            LogLevel::Info,
            format!("Updating with delta {}", delta_seconds).as_str(),
        );

        if time::elapsed_seconds() > 1000.0 {
            process::exit();
        }
    }

    fn cleanup() {
        log(LogLevel::Info, "Cleaning up unavi-system!");
    }
}
