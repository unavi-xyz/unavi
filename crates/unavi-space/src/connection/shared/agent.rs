use std::time::Duration;

use iroh::endpoint::{Connection, RecvStream, SendStream};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use unavi_util::async_commands::AsyncCommands;

use crate::connection::{
    ecs::{PeerStream, agent::AgentSender},
    shared::StreamIdent,
    types::{
        IFrame, PFrame, f16_vec3::F16Vec3, i8_vec3::I8Vec3, pose::Pose,
        rigid_transform::RigidTransform,
    },
};

#[derive(Serialize, Deserialize, MaxSize)]
enum AgentMsg {
    IFrame { id: usize, pose: Pose<IFrame> },
    PFrame { iframe: usize, pose: Pose<PFrame> },
}

const IFRAME_FREQ: Duration = Duration::from_secs(5);

pub async fn send_agent_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::Agent.write(&mut tx).await?;

    // TODO Read `rx` for tickrate backpressure

    let (pose_tx, pose_rx) = async_channel::bounded::<Pose<IFrame>>(1);

    // Send channel to ECS.
    AsyncCommands::default()
        .spawn((PeerStream(connection.remote_id()), AgentSender(pose_tx)))
        .send()
        .await?;

    let mut iframe_id = 0;
    let mut last_iframe = Pose::default();
    let mut last_iframe_time = n0_future::time::Instant::now() - IFRAME_FREQ;

    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];

    while let Ok(pose) = pose_rx.recv().await {
        let now = n0_future::time::Instant::now();

        // Convert to i-frame or p-frame.
        let msg = if now.duration_since(last_iframe_time) >= IFRAME_FREQ {
            iframe_id += 1;
            last_iframe = pose.clone();
            last_iframe_time = now;
            AgentMsg::IFrame {
                id: iframe_id,
                pose,
            }
        } else {
            AgentMsg::PFrame {
                iframe: iframe_id,
                pose: delta_pose(pose, &last_iframe),
            }
        };

        // Serialize and send.
        let out = postcard::to_slice(&msg, &mut buf)?;
        let len = out.len();
        tx.write_u8(u8::try_from(len).expect("max size")).await?;
        tx.write_all(&buf).await?;
    }

    Ok(())
}

fn delta_pose(pose: Pose<IFrame>, last: &Pose<IFrame>) -> Pose<PFrame> {
    let root = RigidTransform::<F16Vec3>::delta(&pose.root, &last.root);
    let bones = pose
        .bones
        .into_iter()
        .map(|(name, bone)| {
            let baseline = last.bones.get(&name).cloned().unwrap_or_default();
            (name, RigidTransform::<I8Vec3>::delta(&bone, &baseline))
        })
        .collect();
    Pose { root, bones }
}

pub async fn recv_agent_stream(_tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];

    loop {
        let len = rx.read_u8().await? as usize;
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let _msg = postcard::from_bytes::<AgentMsg>(buf)?;

        // TODO send to ecs + apply
    }
}
