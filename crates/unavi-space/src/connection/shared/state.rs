use anyhow::{
    Context,
    bail,
};
use blake3::Hash;
use iroh::endpoint::{
    Connection,
    RecvStream,
    SendStream,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use unavi_util::async_commands::AsyncCommands;

use crate::{
    connection::{
        ecs::PeerStream,
        shared::StreamIdent,
    },
    peer::state::SpaceStateSender,
    state::space::{
        SpaceStateUpdate,
        space_state,
    },
};

const MAX_MSG_LEN: usize = 8 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
enum StateMsg {
    Update { space: Hash, data: Vec<u8> },
}

pub async fn send_state_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::State.write(&mut tx).await?;

    let (ss_tx, ss_rx) = async_channel::bounded(4);

    AsyncCommands::default()
        .spawn((PeerStream(connection.remote_id()), SpaceStateSender(ss_tx)))
        .send()
        .await?;

    // TODO Request / send full state snapshot

    while let Ok(SpaceStateUpdate { space, data }) = ss_rx.recv().await {
        let msg = StateMsg::Update { space, data };
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

pub async fn recv_state_stream(_tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
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

        match msg {
            StateMsg::Update { space, data } => {
                let Some(state) = space_state(space) else {
                    continue;
                };
                state.doc.import(&data)?;
            }
        }
    }
}
