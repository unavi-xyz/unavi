use anyhow::Result;
use bevy::log::{debug, error, info, info_span, warn};
use wasm_bridge::component::Linker;

use crate::state::StoreState;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-log"
    });

    pub use self::wired::log::*;
}

use self::bindings::api::LogLevel;

impl bindings::api::Host for StoreState {
    fn log(&mut self, level: LogLevel, message: String) -> wasm_bridge::Result<()> {
        let span = info_span!("Script", name = self.name);
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

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    Ok(())
}
