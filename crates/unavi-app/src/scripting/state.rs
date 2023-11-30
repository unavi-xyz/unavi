use wired_host_bindgen::script::wired::host::logger;

pub struct State;

impl logger::Host for State {
    fn log(&mut self, level: logger::LogLevel, msg: String) -> wasmtime::Result<()> {
        match level {
            logger::LogLevel::Debug => tracing::debug!("{}", msg),
            logger::LogLevel::Info => tracing::info!("{}", msg),
            logger::LogLevel::Warn => tracing::warn!("{}", msg),
            logger::LogLevel::Error => tracing::error!("{}", msg),
        };

        Ok(())
    }
}
