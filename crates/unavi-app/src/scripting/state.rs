use wired_host_bindgen::script::wired::host::{logger, logger::Host, process, time};

#[derive(Default)]
pub struct RuntimeState {
    /// The name of the runtime.
    pub name: String,
    /// Whether the runtime should exit.
    pub exit: bool,
}

impl logger::Host for RuntimeState {
    fn log(&mut self, level: logger::LogLevel, msg: String) -> wasmtime::Result<()> {
        let name = if self.name.is_empty() {
            "unknown"
        } else {
            self.name.as_str()
        };

        let msg = format!("[{}] {}", name, msg);

        match level {
            logger::LogLevel::Debug => tracing::debug!("{}", msg),
            logger::LogLevel::Info => tracing::info!("{}", msg),
            logger::LogLevel::Warn => tracing::warn!("{}", msg),
            logger::LogLevel::Error => tracing::error!("{}", msg),
        };

        Ok(())
    }
}

impl process::Host for RuntimeState {
    fn exit(&mut self) -> wasmtime::Result<()> {
        let _ = self.log(logger::LogLevel::Info, "Exiting process...".to_string());
        self.exit = true;
        Ok(())
    }
}

impl time::Host for RuntimeState {
    fn elapsed_seconds(&mut self) -> wasmtime::Result<f32> {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        Ok(duration.as_millis() as f32)
    }
}
