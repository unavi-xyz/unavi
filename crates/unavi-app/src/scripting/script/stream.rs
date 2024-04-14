use std::sync::mpsc::{Receiver, SyncSender};

use bytes::Bytes;
use tracing::debug;
use wasm_bridge_wasi::{HostOutputStream, StdoutStream, StreamResult, Subscribe};

#[derive(Clone, Debug)]
pub struct OutStream(SyncSender<Bytes>);

impl OutStream {
    pub fn new() -> (Self, Receiver<Bytes>) {
        let (send, recv) = std::sync::mpsc::sync_channel(100);
        (Self(send), recv)
    }
}

#[wasm_bridge::async_trait]
impl Subscribe for OutStream {
    async fn ready(&mut self) {}
}

impl HostOutputStream for OutStream {
    fn write(&mut self, buf: Bytes) -> StreamResult<()> {
        if let Err(err) = self.0.try_send(buf) {
            debug!("Failed to send bytes to out stream: {}", err)
        }

        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        StreamResult::Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        StreamResult::Ok(1)
    }
}

impl StdoutStream for OutStream {
    fn stream(&self) -> Box<(dyn HostOutputStream + 'static)> {
        Box::new((*self).clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}
