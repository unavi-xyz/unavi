use iroh::endpoint::{Connection, RecvStream, SendStream};

use crate::connection::shared::StreamIdent;

pub async fn send_object_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::Object.write(&mut tx).await?;

    Ok(())
}

pub async fn recv_object_stream(_tx: SendStream, _rx: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
