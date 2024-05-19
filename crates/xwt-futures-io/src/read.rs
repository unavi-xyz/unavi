use std::{
    cell::RefCell,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures_io::AsyncRead;
use xwt_core::{session::stream::RecvSpec, stream::Read};

pub struct ReadCompat<T: RecvSpec> {
    recv: RefCell<T::RecvStream>,
}

impl<T: RecvSpec> ReadCompat<T> {
    pub fn new(recv: T::RecvStream) -> Self {
        Self {
            recv: RefCell::new(recv),
        }
    }
}

impl<T: RecvSpec> AsyncRead for ReadCompat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        #[allow(clippy::await_holding_refcell_ref)]
        let read = async {
            match self.recv.borrow_mut().read(buf).await {
                Ok(v) => Ok(v.unwrap_or_default()),
                Err(e) => Err(futures_io::Error::new(
                    futures_io::ErrorKind::Other,
                    anyhow::anyhow!("{}", e),
                )),
            }
        };
        let mut pinned_read = pin!(read);
        pinned_read.as_mut().poll(cx)
    }
}
