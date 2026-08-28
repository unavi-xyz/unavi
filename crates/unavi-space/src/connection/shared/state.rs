use anyhow::{
    Context,
    bail,
};
use bevy::prelude::World;
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
use unavi_util::async_commands::AsyncCommands;

use crate::{
    connection::{
        PeerLink,
        shared::StreamIdent,
    },
    state::{
        entities,
        message::StateMsg,
    },
};

const MAX_MSG_LEN: usize = 8 * 1024 * 1024;

pub async fn send_state_stream(link: &PeerLink, connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::State.write(&mut tx).await?;

    let me = link.view().me();
    let replicas = link.view().replicas();
    let (token, rx) = replicas.register_stream(me);
    let res = send_loop(&mut tx, &rx).await;
    replicas.unregister_stream(token);

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
    link: &PeerLink,
    peer: EndpointId,
    _tx: SendStream,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    // Racing connections to the same peer share one state entity; the
    // generation lets only the latest stream tear it down, so a superseded
    // connection's exit cannot erase the peer's replicated state.
    let generation = link.next_stream_gen();
    let (ent_tx, ent_rx) = async_channel::bounded(1);
    if AsyncCommands::default()
        .push(move |world: &mut World| {
            let ent = entities::claim_remote_peer(world, peer, generation);
            let _ = ent_tx.try_send(ent);
        })
        .send()
        .await
        .is_err()
    {
        bail!("async command queue closed");
    }
    let peer_ent = ent_rx.recv().await.context("claim remote peer")?;
    let res = recv_loop(peer_ent, peer, &mut rx).await;
    let _ = AsyncCommands::default()
        .push(move |world: &mut World| {
            entities::release_remote_peer(world, peer_ent, generation);
        })
        .send()
        .await;
    res
}

async fn recv_loop(
    peer_ent: bevy::prelude::Entity,
    peer: EndpointId,
    rx: &mut RecvStream,
) -> anyhow::Result<()> {
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
        entities::apply_remote(peer_ent, peer, msg);
    }
}
