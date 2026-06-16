use std::future::pending;

use iroh::endpoint::{
    RecvStream,
    SendStream,
};

pub async fn recv_object_stream(_tx: SendStream, _rx: RecvStream) -> anyhow::Result<()> {
    // WIP: hold the stream open until the connection closes rather than
    // returning, which would finish the stream and churn the connection.
    pending().await
}
