use tracing::{
    info,
    warn,
};

#[derive(Clone, Copy)]
pub enum Level {
    Info,
    Warn,
}

/// One entry per run of script output rather than one per line.
///
/// A tick that prints five lines reads as five lines under one heading,
/// instead of five entries interleaved with whatever else logged in between.
///
/// What counts as a run belongs to the transport — a read off wasmtime's
/// stream, a microtask's worth of `jco` writes — so a caller hands one over
/// already gathered. Both then land in `tracing`, and so obey the same filter
/// as everything else the client logs.
pub fn emit(script: &str, level: Level, run: &str) {
    let run = run.trim_end();
    if run.is_empty() {
        return;
    }
    match level {
        Level::Info => info!("{script}\n{run}"),
        Level::Warn => warn!("{script}\n{run}"),
    }
}
