use tokio::io::DuplexStream;
use wasmtime_wasi::cli::AsyncStdoutStream;

const KB: usize = 1024;
const STDERR_LEN: usize = KB;
const STDOUT_LEN: usize = 4 * KB;

pub struct ScriptStderr(pub DuplexStream);

impl ScriptStderr {
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDERR_LEN);
        (Self(reader), AsyncStdoutStream::new(STDERR_LEN, writer))
    }
}

pub struct ScriptStdout(pub DuplexStream);

impl ScriptStdout {
    pub fn new() -> (Self, AsyncStdoutStream) {
        let (writer, reader) = tokio::io::duplex(STDOUT_LEN);
        (Self(reader), AsyncStdoutStream::new(STDOUT_LEN, writer))
    }
}
