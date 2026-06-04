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
    connection::shared::StreamIdent,
    peer::state::AddSpaceStateSender,
    state::space::{
        SpaceStateUpdate,
        space_state,
    },
};

#[derive(Serialize, Deserialize)]
enum StateMsg {
    Update { space: Hash, data: Vec<u8> },
}

pub async fn send_state_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::State.write(&mut tx).await?;

    let (ss_tx, ss_rx) = async_channel::bounded(4);

    let _ = AsyncCommands::default()
        .trigger(AddSpaceStateSender {
            peer:   connection.remote_id(),
            sender: ss_tx,
        })
        .send()
        .await;

    // TODO Request / send full state snapshot

    while let Ok(SpaceStateUpdate { space, data }) = ss_rx.recv().await {
        let msg = StateMsg::Update { space, data };

        let buf = postcard::to_allocvec(&msg)?;
        let len = buf.len();
        tx.write_u16(u16::try_from(len).expect("max size")).await?;
        tx.write_all(&buf).await?;
    }

    Ok(())
}

pub async fn recv_state_stream(_tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    loop {
        let len = rx.read_u16().await? as usize;
        let mut buf = vec![0u8; len];
        rx.read_exact(&mut buf).await?;
        let msg = postcard::from_bytes::<StateMsg>(&buf)?;

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
