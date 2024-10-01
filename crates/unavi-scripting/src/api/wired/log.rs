use bevy::log::{debug, error, info, info_span, warn};
use wasm_bridge::component::Linker;

use crate::data::ScriptData;

use self::bindings::api::LogLevel;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-log"
    });

    pub use self::wired::log::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> anyhow::Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredLog {
    pub name: String,
}

impl bindings::api::Host for ScriptData {
    fn log(&mut self, level: LogLevel, message: String) -> wasm_bridge::Result<()> {
        let span = info_span!("Script", name = self.api.wired_log.as_ref().unwrap().name);
        let span = span.enter();

        match level {
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        };

        drop(span);

        Ok(())
    }
}
