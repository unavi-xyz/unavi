use wired_host_bindgen::script::wired::host::{logger, process, time};

#[derive(Default)]
pub struct ScriptState {
    /// Marks whether the script should exit.
    pub exit: bool,
}

impl logger::Host for ScriptState {
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

impl process::Host for ScriptState {
    fn exit(&mut self) -> wasmtime::Result<()> {
        tracing::info!("Process exiting...");
        self.exit = true;
        Ok(())
    }
}

impl time::Host for ScriptState {
    fn elapsed_seconds(&mut self) -> wasmtime::Result<f32> {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        Ok(duration.as_millis() as f32)
    }
}
