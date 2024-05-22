use std::{
    cell::RefCell,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use anyhow::anyhow;
use futures_io::AsyncWrite;
use tracing::warn;
use xwt_core::{session::stream::SendSpec, stream::Write};

pub struct WriteCompat<T: SendSpec> {
    send: RefCell<T::SendStream>,
}

impl<T: SendSpec> WriteCompat<T> {
    pub fn new(send: T::SendStream) -> Self {
        Self {
            send: RefCell::new(send),
        }
    }
}

impl<T: SendSpec> AsyncWrite for WriteCompat<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<futures_io::Result<usize>> {
        #[allow(clippy::await_holding_refcell_ref)]
        let write = async {
            self.as_mut()
                .send
                .borrow_mut()
                .write(buf)
                .await
                .map_err(|e| {
                    warn!("Stream write error: {}", e);
                    futures_io::Error::new(futures_io::ErrorKind::Other, anyhow!("{}", e))
                })
        };
        let mut pinned_write = pin!(write);
        pinned_write.as_mut().poll(cx)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<futures_io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<futures_io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
