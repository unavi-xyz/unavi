use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader, DuplexStream};
use tracing::{Instrument, Span, info, warn};
use unavi_util::async_task::spawn_async_task;
use wasmtime_wasi::cli::AsyncStdoutStream;

const KB: usize = 1024;
const STDERR_LEN: usize = KB;
const STDOUT_LEN: usize = 4 * KB;

pub struct ScriptStderr(DuplexStream);

impl ScriptStderr {
    #[must_use]
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDERR_LEN);
        (Self(reader), AsyncStdoutStream::new(STDERR_LEN, writer))
    }

    pub fn drain(self, span: Span) {
        spawn_async_task(drain_stream(self.0, Level::Warn).instrument(span));
    }
}

pub struct ScriptStdout(DuplexStream);

impl ScriptStdout {
    #[must_use]
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDOUT_LEN);
        (Self(reader), AsyncStdoutStream::new(STDOUT_LEN, writer))
    }

    pub fn drain(self, span: Span) {
        spawn_async_task(drain_stream(self.0, Level::Info).instrument(span));
    }
}

enum Level {
    Info,
    Warn,
}

async fn drain_stream(stream: impl AsyncRead + Unpin, level: Level) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        match level {
            Level::Info => info!("{line}"),
            Level::Warn => warn!("{line}"),
        }
    }
}
