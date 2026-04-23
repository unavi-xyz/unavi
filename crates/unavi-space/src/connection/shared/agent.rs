use std::time::Duration;

use bevy::ecs::world::CommandQueue;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::Instant,
};
use unavi_util::async_commands::ASYNC_COMMAND_QUEUE;

use crate::connection::{
    ecs::{PeerStream, agent::AgentSender},
    shared::StreamIdent,
    types::{IFrame, PFrame, pose::Pose},
};

#[derive(Serialize, Deserialize, MaxSize)]
enum AgentFrame {
    IFrame { id: usize, pose: Pose<IFrame> },
    PFrame { iframe: usize, pose: Pose<PFrame> },
}

const IFRAME_FREQ: Duration = Duration::from_secs(5);

pub async fn send_agent_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::Agent.write(&mut tx).await?;

    // TODO Read `rx` for tickrate backpressure

    let (pose_tx, mut pose_rx) = tokio::sync::mpsc::channel::<Pose<IFrame>>(1);

    // Send channel to ECS.
    {
        let mut commands = CommandQueue::default();
        commands.push(bevy::ecs::system::command::spawn_batch([(
            PeerStream(connection.remote_id()),
            AgentSender(pose_tx),
        )]));
        ASYNC_COMMAND_QUEUE.0.send(commands).await?;
    }

    let mut iframe_id = 0;
    let mut last_iframe = Pose::default();
    let mut last_iframe_time = Instant::now() - IFRAME_FREQ;

    let mut buf = [0; AgentFrame::POSTCARD_MAX_SIZE];

    while let Some(pose) = pose_rx.recv().await {
        let now = Instant::now();

        // Convert to frame.
        let frame = if now.duration_since(last_iframe_time) >= IFRAME_FREQ {
            iframe_id += 1;
            last_iframe = pose.clone();
            last_iframe_time = now;
            AgentFrame::IFrame {
                id: iframe_id,
                pose,
            }
        } else {
            AgentFrame::PFrame {
                iframe: iframe_id,
                pose: delta_pose(pose, &last_iframe),
            }
        };

        // Serialize and send.
        let out = postcard::to_slice(&frame, &mut buf)?;
        let len = out.len();
        tx.write_u8(u8::try_from(len).expect("max size < 256"))
            .await?;
        tx.write_all(&buf).await?;
    }

    Ok(())
}

fn delta_pose(pose: Pose<IFrame>, last: &Pose<IFrame>) -> Pose<PFrame> {
    todo!()
}

pub async fn recv_agent_stream(_tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    let mut buf = [0; AgentFrame::POSTCARD_MAX_SIZE];

    loop {
        let len = rx.read_u8().await? as usize;
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let _frame = postcard::from_bytes::<AgentFrame>(buf)?;

        // TODO send to ecs
    }
}
