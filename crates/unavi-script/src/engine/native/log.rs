use smol_str::SmolStr;
use tokio::io::{
    AsyncRead,
    AsyncReadExt,
    BufReader,
    DuplexStream,
};
use unavi_util::async_task::spawn_async_task;
use wasmtime_wasi::cli::AsyncStdoutStream;

use crate::engine::log::{
    Level,
    emit,
};

const KB: usize = 1024;
const STDERR_LEN: usize = KB;
const STDOUT_LEN: usize = 4 * KB;
/// How much of a run with no newline in it is held before it goes out
/// unfinished.
const BATCH_LEN: usize = 4 * KB;

pub struct ScriptStderr(DuplexStream);

impl ScriptStderr {
    #[must_use]
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDERR_LEN);
        (Self(reader), AsyncStdoutStream::new(STDERR_LEN, writer))
    }

    pub fn drain(self, script: SmolStr) {
        spawn_async_task(drain_stream(self.0, script, Level::Warn));
    }
}

pub struct ScriptStdout(DuplexStream);

impl ScriptStdout {
    #[must_use]
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDOUT_LEN);
        (Self(reader), AsyncStdoutStream::new(STDOUT_LEN, writer))
    }

    pub fn drain(self, script: SmolStr) {
        spawn_async_task(drain_stream(self.0, script, Level::Info));
    }
}

/// A run is everything readable at once, cut at the last newline in it, so a
/// line still being written is held back for the next read.
async fn drain_stream(stream: impl AsyncRead + Unpin, script: SmolStr, level: Level) {
    let mut reader = BufReader::new(stream);
    let mut held = Vec::<u8>::with_capacity(BATCH_LEN);
    let mut chunk = [0_u8; BATCH_LEN];

    while let Ok(read) = reader.read(&mut chunk).await {
        if read == 0 {
            break;
        }
        held.extend_from_slice(&chunk[..read]);

        let run = match held.iter().rposition(|byte| *byte == b'\n') {
            Some(end) => held.drain(..=end).collect::<Vec<_>>(),
            None if held.len() >= BATCH_LEN => std::mem::take(&mut held),
            None => continue,
        };

        emit(&script, level, &String::from_utf8_lossy(&run));
    }
}
