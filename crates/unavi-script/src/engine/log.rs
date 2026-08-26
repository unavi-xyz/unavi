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
