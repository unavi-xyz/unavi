use tracing::{debug, error, info, warn};

#[allow(warnings)]
mod bindings;

use bindings::exports::unavi::log::api::{Guest, LogLevel};

struct Component;

impl Guest for Component {
    fn log(level: LogLevel, msg: String) {
        match level {
            LogLevel::Debug => debug!("{}", msg),
            LogLevel::Error => error!("{}", msg),
            LogLevel::Info => info!("{}", msg),
            LogLevel::Warn => warn!("{}", msg),
        }
    }
}

bindings::export!(Component with_types_in bindings);
