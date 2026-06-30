use anyhow::{
    Context,
    bail,
};
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        RecvStream,
        SendStream,
    },
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};

use crate::{
    connection::shared::StreamIdent,
    state::{
        message::StateMsg,
        replicas,
    },
};

const MAX_MSG_LEN: usize = 8 * 1024 * 1024;

pub async fn send_state_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::State.write(&mut tx).await?;

    let (token, rx) = replicas::register_stream();
    let res = send_loop(&mut tx, &rx).await;
    replicas::unregister_stream(token);

    res
}

async fn send_loop(
    tx: &mut SendStream,
    rx: &async_channel::Receiver<StateMsg>,
) -> anyhow::Result<()> {
    while let Ok(msg) = rx.recv().await {
        let buf = postcard::to_allocvec(&msg)?;
        let len = buf.len();
        if len > MAX_MSG_LEN {
            bail!("message too large")
        }
        tx.write_u32(u32::try_from(len)?).await?;
        tx.write_all(&buf).await?;
    }
    Ok(())
}

pub async fn recv_state_stream(
    peer: EndpointId,
    _tx: SendStream,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    let res = recv_loop(peer, &mut rx).await;
    replicas::remove_peer(*peer.as_bytes());
    res
}

async fn recv_loop(peer: EndpointId, rx: &mut RecvStream) -> anyhow::Result<()> {
    loop {
        let len = match rx.read_u32().await {
            Ok(len) => len as usize,
            Err(err) if super::read_disconnected(&err) => return Ok(()),
            Err(err) => return Err(err).context("read len"),
        };
        if len > MAX_MSG_LEN {
            bail!("message too large")
        }

        let mut buf = vec![0; len];
        rx.read_exact(&mut buf).await.context("read msg")?;
        let msg = postcard::from_bytes::<StateMsg>(&buf).context("parse msg")?;
        replicas::apply_remote(*peer.as_bytes(), msg);
    }
}
