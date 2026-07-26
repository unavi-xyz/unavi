use std::time::Duration;

use anyhow::Context;
use bevy::transform::components::Transform;
use blake3::Hash;
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        RecvStream,
        SendStream,
    },
};
use n0_future::time::Instant;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use unavi_util::async_commands::AsyncCommands;

use crate::connection::{
    ecs::{
        PeerStream,
        agent::{
            inbound::{
                ResolvedPose,
                submit_pose,
            },
            outbound::{
                AgentSender,
                OutgoingPose,
            },
        },
    },
    shared::StreamIdent,
    types::{
        IFrame,
        PFrame,
        f16_vec3::F16Vec3,
        f32_vec3::F32Vec3,
        i8_vec3::I8Vec3,
        pose::Pose,
        rigid_transform::RigidTransform,
    },
};

#[derive(Serialize, Deserialize, MaxSize)]
enum AgentMsg {
    IFrame {
        id:    u32,
        space: [u8; 32],
        pose:  Pose<IFrame>,
    },
    PFrame {
        iframe: u32,
        space:  [u8; 32],
        pose:   Pose<PFrame>,
    },
}

const IFRAME_FREQ: Duration = Duration::from_secs(5);

pub async fn send_agent_stream(connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamIdent::Agent.write(&mut tx).await?;

    // TODO Read `rx` for tickrate backpressure

    let (pose_tx, pose_rx) = async_channel::bounded::<OutgoingPose>(1);

    // Send channel to ECS.
    AsyncCommands::default()
        .spawn((PeerStream(connection.remote_id()), AgentSender(pose_tx)))
        .send()
        .await?;

    let mut iframe_id = 0;
    let mut last_iframe = Pose::default();
    let mut last_iframe_time = Instant::now() - IFRAME_FREQ;
    let mut last_space = Hash::from_bytes([0; 32]);

    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];

    while let Ok(OutgoingPose { space, pose }) = pose_rx.recv().await {
        let now = Instant::now();
        let space_bytes = *space.as_bytes();

        // A p-frame delta is only valid against an i-frame in the same space, so
        // a space change forces a fresh i-frame.
        let new_iframe = now.duration_since(last_iframe_time) >= IFRAME_FREQ || space != last_space;

        let msg = if new_iframe {
            iframe_id += 1;
            last_iframe = pose.clone();
            last_iframe_time = now;
            last_space = space;
            AgentMsg::IFrame {
                id: iframe_id,
                space: space_bytes,
                pose,
            }
        } else {
            AgentMsg::PFrame {
                iframe: iframe_id,
                space:  space_bytes,
                pose:   delta_pose(pose, &last_iframe),
            }
        };

        // Serialize and send.
        let out = postcard::to_slice(&msg, &mut buf)?;
        let len = out.len();
        tx.write_u8(u8::try_from(len).expect("max size")).await?;
        tx.write_all(out).await?;
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

struct Baseline {
    id:   u32,
    root: RigidTransform<F32Vec3>,
}

/// Reconstructs a full-precision pose from a message, tracking the i-frame
/// baseline that subsequent p-frame deltas are applied against. Returns `None`
/// for p-frames that arrive before their i-frame or reference a stale one.
fn resolve_msg(msg: AgentMsg, baseline: &mut Option<Baseline>) -> Option<ResolvedPose> {
    match msg {
        AgentMsg::IFrame { id, space, pose } => {
            let root = pose.root.clone().into();
            *baseline = Some(Baseline {
                id,
                root: pose.root,
            });
            Some(ResolvedPose {
                space: Hash::from_bytes(space),
                root,
            })
        }
        AgentMsg::PFrame {
            iframe,
            space,
            pose,
        } => {
            let baseline = baseline.as_ref()?;
            if baseline.id != iframe {
                return None;
            }
            let translation = pose.root.tra.apply_to(baseline.root.tra).into();
            Some(ResolvedPose {
                space: Hash::from_bytes(space),
                root:  Transform {
                    translation,
                    rotation: pose.root.rot.into(),
                    ..Default::default()
                },
            })
        }
    }
}

pub async fn recv_agent_stream(
    peer: EndpointId,
    _tx: SendStream,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];
    let mut baseline: Option<Baseline> = None;

    loop {
        let len = match rx.read_u8().await {
            Ok(len) => len as usize,
            Err(err) if super::read_disconnected(&err) => return Ok(()),
            Err(err) => return Err(err).context("read len"),
        };
        if len > buf.len() {
            anyhow::bail!("agent frame length {len} exceeds max {}", buf.len());
        }
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let msg = postcard::from_bytes::<AgentMsg>(buf)?;

        if let Some(resolved) = resolve_msg(msg, &mut baseline) {
            submit_pose(peer, resolved);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::*;

    const SPACE: [u8; 32] = [7; 32];

    fn iframe_pose(t: Vec3) -> Pose<IFrame> {
        Pose {
            root: RigidTransform::from(&Transform::from_translation(t)),
            ..Default::default()
        }
    }

    #[test]
    fn pframe_before_iframe_is_dropped() {
        let mut baseline = None;
        let msg = AgentMsg::PFrame {
            iframe: 1,
            space:  SPACE,
            pose:   Pose::default(),
        };
        assert!(resolve_msg(msg, &mut baseline).is_none());
    }

    #[test]
    fn iframe_sets_baseline_and_resolves() {
        let mut baseline = None;
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let resolved = resolve_msg(
            AgentMsg::IFrame {
                id:    1,
                space: SPACE,
                pose:  iframe_pose(pos),
            },
            &mut baseline,
        )
        .expect("resolved");

        assert_eq!(Hash::from_bytes(SPACE), resolved.space);
        assert!((resolved.root.translation - pos).length() < 0.01);
        assert!(baseline.is_some());
    }

    #[test]
    fn pframe_applies_delta_to_baseline() {
        let mut baseline = None;
        let base = Vec3::new(1.0, 2.0, 3.0);
        resolve_msg(
            AgentMsg::IFrame {
                id:    1,
                space: SPACE,
                pose:  iframe_pose(base),
            },
            &mut baseline,
        );

        let moved = base + Vec3::new(0.1, -0.2, 0.05);
        let pframe = delta_pose(iframe_pose(moved), &iframe_pose(base));
        let resolved = resolve_msg(
            AgentMsg::PFrame {
                iframe: 1,
                space:  SPACE,
                pose:   pframe,
            },
            &mut baseline,
        )
        .expect("resolved");

        assert!((resolved.root.translation - moved).length() < 0.05);
    }

    #[test]
    fn pframe_with_stale_iframe_id_is_dropped() {
        let mut baseline = None;
        resolve_msg(
            AgentMsg::IFrame {
                id:    2,
                space: SPACE,
                pose:  iframe_pose(Vec3::ZERO),
            },
            &mut baseline,
        );

        let stale = AgentMsg::PFrame {
            iframe: 1,
            space:  SPACE,
            pose:   Pose::default(),
        };
        assert!(resolve_msg(stale, &mut baseline).is_none());
    }
}
