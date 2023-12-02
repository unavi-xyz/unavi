use self::wired::script::{
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
        log(LogLevel::Info, "Initializing unavi-system.");
    }

    fn update(_delta: f32) {
        if time::elapsed_seconds() > 3.0 {
            let msg = format!(
                "Exiting unavi-system after {} seconds",
                time::elapsed_seconds()
            );
            log(LogLevel::Info, msg.as_str());

            process::exit();
        }
    }

    fn cleanup() {
        log(LogLevel::Info, "Cleaning up unavi-system!");
    }
}
