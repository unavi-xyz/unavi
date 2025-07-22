use std::{borrow::Cow, future::poll_fn, pin::Pin, task::Poll};

use bevy::prelude::*;
use tokio::io::{AsyncRead, DuplexStream, ReadBuf};

pub async fn try_read_text_stream<'a>(
    buf: &'a mut [u8],
    stream: &mut DuplexStream,
) -> Option<Cow<'a, str>> {
    let mut pinned = Pin::new(stream);
    let mut read_buf = ReadBuf::new(buf);

    let n = poll_fn(|cx| match pinned.as_mut().poll_read(cx, &mut read_buf) {
        Poll::Ready(Ok(())) => Poll::Ready(read_buf.filled().len()),
        Poll::Ready(Err(e)) => Poll::Ready({
            error!("Error reading buf: {e}");
            0
        }),
        Poll::Pending => Poll::Ready(0),
    })
    .await;

    if n == 0 {
        return None;
    }

    let s = String::from_utf8_lossy(&buf[0..n]);
    Some(s)
}
