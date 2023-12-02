use wired_script_bindgen::wired::script::{logger, logger::Host, process, time};

pub struct RuntimeState {
    /// The name of the runtime.
    pub name: String,
    /// Whether the runtime should exit.
    pub exit: bool,
    /// The time the runtime started.
    pub start_time: std::time::Instant,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            name: String::new(),
            exit: false,
            start_time: std::time::Instant::now(),
        }
    }
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
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f32();
        Ok(elapsed)
    }
}
