use std::sync::{Arc, Mutex};

use bytes::Bytes;
use wasm_bridge_wasi::preview2::{HostOutputStream, StdoutStream, StreamResult, Subscribe};

#[derive(Clone, Debug)]
pub struct OutStream {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub max: usize,
}

#[wasm_bridge::async_trait]
impl Subscribe for OutStream {
    async fn ready(&mut self) {}
}

impl HostOutputStream for OutStream {
    fn write(&mut self, buf: Bytes) -> StreamResult<()> {
        assert!(
            buf.len() <= self.max,
            "We specified to write at most {} bytes at a time.",
            self.max
        );
        self.data.try_lock().unwrap().extend(buf);
        StreamResult::Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        StreamResult::Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        StreamResult::Ok(self.max)
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
