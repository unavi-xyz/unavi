use bevy::ecs::world::CommandQueue;
use blake3::Hash;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use unavi_util::async_commands::ASYNC_COMMAND_QUEUE;

use crate::{
    connection::shared::StreamIdent,
    peer::AddSpaceStateSender,
    state::space::{SPACES, SpaceStateUpdate},
};

#[derive(Serialize, Deserialize)]
enum StateMsg {
    Update { space: Hash, data: Vec<u8> },
}

pub async fn send_state_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::State.write(&mut tx).await?;

    let (ss_tx, mut ss_rx) = tokio::sync::mpsc::channel(4);

    let mut queue = CommandQueue::default();
    queue.push(bevy::ecs::system::command::trigger(AddSpaceStateSender {
        peer: connection.remote_id(),
        sender: ss_tx,
    }));
    let _ = ASYNC_COMMAND_QUEUE.0.send(queue).await;

    // TODO Request / send full state snapshot

    while let Some(SpaceStateUpdate { space, data }) = ss_rx.recv().await {
        let msg = StateMsg::Update { space, data };

        let mut buf = Vec::new();
        let out = postcard::to_slice(&msg, &mut buf)?;
        let len = out.len();
        tx.write_u16(u16::try_from(len).expect("max size")).await?;
        tx.write_all(&buf).await?;
    }

    Ok(())
}

pub async fn recv_state_stream(_tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    loop {
        let len = rx.read_u16().await? as usize;
        let mut buf = Vec::with_capacity(len);
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let msg = postcard::from_bytes::<StateMsg>(buf)?;

        match msg {
            StateMsg::Update { space, data } => {
                let lock = SPACES.lock().await;
                let Some(state) = lock.get(&space) else {
                    continue;
                };
                state.doc.import(&data)?;
                drop(lock);
            }
        }
    }
}
