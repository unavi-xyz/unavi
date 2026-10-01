//! The avatar stream: one peer's root and bone poses, as keyframes and deltas.

use std::time::Duration;

use anyhow::Context;
use bevy::{
    platform::collections::HashMap,
    transform::components::Transform,
};
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        RecvStream,
    },
};
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};
use serde_vrm::vrm0::BoneName;
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use web_time::Instant;

use crate::{
    avatar_sync::{
        AgentSender,
        OutgoingPose,
        ResolvedPose,
    },
    link::{
        PeerLink,
        codec::{
            Delta,
            Keyframe,
            f16_vec3::F16Vec3,
            f32_vec3::F32Vec3,
            i8_vec3::I8Vec3,
            pose::Pose,
            rigid_transform::RigidTransform,
        },
        senders::PeerSender,
        streams::{
            StreamKind,
            read_disconnected,
        },
    },
    membership::SpaceId,
};

#[derive(Serialize, Deserialize, MaxSize)]
enum AgentMsg {
    Keyframe {
        id:    u32,
        space: [u8; 32],
        pose:  Pose<Keyframe>,
    },
    Delta {
        keyframe: u32,
        space:    [u8; 32],
        pose:     Pose<Delta>,
    },
}

// Frames are length-prefixed by one byte.
const _: () = assert!(AgentMsg::POSTCARD_MAX_SIZE <= u8::MAX as usize);

const KEYFRAME_INTERVAL: Duration = Duration::from_secs(5);

pub async fn send_agent_stream(link: &PeerLink, connection: &Connection) -> anyhow::Result<()> {
    let (mut tx, _rx) = connection.open_bi().await?;
    StreamKind::Avatar.write(&mut tx).await?;

    let (pose_tx, pose_rx) = async_channel::bounded::<OutgoingPose>(1);

    link.view()
        .commands()
        .spawn((PeerSender(connection.remote_id()), AgentSender(pose_tx)))
        .send()
        .await?;

    let mut keyframe_id = 0;
    let mut last_keyframe = Pose::default();
    let mut last_keyframe_time: Option<Instant> = None;
    let mut last_space = None;

    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];

    while let Ok(OutgoingPose { space, pose }) = pose_rx.recv().await {
        let now = Instant::now();
        let space_bytes = space.0;

        // A delta is only valid against a keyframe in the same space,
        // so a space change forces a fresh keyframe.
        let new_keyframe = last_keyframe_time
            .is_none_or(|last| now.duration_since(last) >= KEYFRAME_INTERVAL)
            || Some(space) != last_space;

        let msg = if new_keyframe {
            keyframe_id += 1;
            last_keyframe = pose.clone();
            last_keyframe_time = Some(now);
            last_space = Some(space);
            AgentMsg::Keyframe {
                id: keyframe_id,
                space: space_bytes,
                pose,
            }
        } else {
            AgentMsg::Delta {
                keyframe: keyframe_id,
                space:    space_bytes,
                pose:     delta_pose(pose, &last_keyframe),
            }
        };

        let out = postcard::to_slice(&msg, &mut buf)?;
        let len = out.len();
        tx.write_u8(len as u8).await?;
        tx.write_all(out).await?;
    }

    Ok(())
}

fn delta_pose(pose: Pose<Keyframe>, last: &Pose<Keyframe>) -> Pose<Delta> {
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
    id:    u32,
    root:  RigidTransform<F32Vec3>,
    bones: HashMap<BoneName, RigidTransform<F16Vec3>>,
}

/// Reconstructs a full-precision pose from a message, tracking the keyframe
/// baseline that later deltas apply against. Returns `None` for a delta that
/// arrives before its keyframe or references a stale one, and for any pose
/// that is not finite or leaves its space's cell.
fn resolve_msg(msg: AgentMsg, baseline: &mut Option<Baseline>) -> Option<ResolvedPose> {
    match msg {
        AgentMsg::Keyframe { id, space, pose } => {
            let root = pose.root.clone().into();
            let bones = pose
                .bones
                .iter()
                .map(|(name, bone)| (*name, bone.clone().into()))
                .collect();
            *baseline = Some(Baseline {
                id,
                root: pose.root,
                bones: pose.bones,
            });
            ResolvedPose {
                space: SpaceId(space),
                root,
                bones,
            }
            .checked()
        }
        AgentMsg::Delta {
            keyframe,
            space,
            pose,
        } => {
            let baseline = baseline.as_ref()?;
            if baseline.id != keyframe {
                return None;
            }
            let translation = pose.root.tra.apply_to(baseline.root.tra).into();
            let bones = pose
                .bones
                .iter()
                .filter_map(|(name, bone)| {
                    let base = baseline.bones.get(name)?;
                    Some((
                        *name,
                        Transform {
                            translation: bone.tra.apply_to(base.tra).into(),
                            rotation: bone.rot.into(),
                            ..Default::default()
                        },
                    ))
                })
                .collect();
            ResolvedPose {
                space: SpaceId(space),
                root: Transform {
                    translation,
                    rotation: pose.root.rot.into(),
                    ..Default::default()
                },
                bones,
            }
            .checked()
        }
    }
}

pub async fn recv_agent_stream(
    link: &PeerLink,
    peer: EndpointId,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    let mut buf = [0; AgentMsg::POSTCARD_MAX_SIZE];
    let mut baseline: Option<Baseline> = None;

    loop {
        let len = match rx.read_u8().await {
            Ok(len) => len as usize,
            Err(err) if read_disconnected(&err) => return Ok(()),
            Err(err) => return Err(err).context("read len"),
        };
        if len > buf.len() {
            anyhow::bail!("agent frame length {len} exceeds max {}", buf.len());
        }
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let msg = postcard::from_bytes::<AgentMsg>(buf)?;

        if let Some(resolved) = resolve_msg(msg, &mut baseline) {
            let _ = link.poses().submit(peer, (Instant::now(), resolved));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{
        Quat,
        Vec3,
    };

    use super::*;

    const SPACE: [u8; 32] = [7; 32];

    fn keyframe_pose(t: Vec3) -> Pose<Keyframe> {
        Pose {
            root: RigidTransform::from(&Transform::from_translation(t)),
            ..Default::default()
        }
    }

    fn keyframe_pose_with_bone(t: Vec3, bone: BoneName, bone_local: Transform) -> Pose<Keyframe> {
        let mut pose = keyframe_pose(t);
        pose.bones.insert(bone, (&bone_local).into());
        pose
    }

    #[test]
    fn delta_before_keyframe_is_dropped() {
        let mut baseline = None;
        let msg = AgentMsg::Delta {
            keyframe: 1,
            space:    SPACE,
            pose:     Pose::default(),
        };
        assert!(resolve_msg(msg, &mut baseline).is_none());
    }

    #[test]
    fn keyframe_sets_baseline_and_resolves() {
        let mut baseline = None;
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let resolved = resolve_msg(
            AgentMsg::Keyframe {
                id:    1,
                space: SPACE,
                pose:  keyframe_pose(pos),
            },
            &mut baseline,
        )
        .expect("resolved");

        assert_eq!(SpaceId(SPACE), resolved.space);
        assert!((resolved.root.translation - pos).length() < 0.01);
        assert!(baseline.is_some());
    }

    #[test]
    fn delta_applies_delta_to_baseline() {
        let mut baseline = None;
        let base = Vec3::new(1.0, 2.0, 3.0);
        resolve_msg(
            AgentMsg::Keyframe {
                id:    1,
                space: SPACE,
                pose:  keyframe_pose(base),
            },
            &mut baseline,
        );

        let moved = base + Vec3::new(0.1, -0.2, 0.05);
        let delta = delta_pose(keyframe_pose(moved), &keyframe_pose(base));
        let resolved = resolve_msg(
            AgentMsg::Delta {
                keyframe: 1,
                space:    SPACE,
                pose:     delta,
            },
            &mut baseline,
        )
        .expect("resolved");

        assert!((resolved.root.translation - moved).length() < 0.05);
    }

    #[test]
    fn keyframe_resolves_bones() {
        let mut baseline = None;
        let bone_local = Transform {
            translation: Vec3::new(0.0, 0.1, 0.02),
            rotation: Quat::from_rotation_x(0.5),
            ..Default::default()
        };
        let resolved = resolve_msg(
            AgentMsg::Keyframe {
                id:    1,
                space: SPACE,
                pose:  keyframe_pose_with_bone(Vec3::ZERO, BoneName::LeftUpperArm, bone_local),
            },
            &mut baseline,
        )
        .expect("resolved");

        let bone = resolved.bones.get(&BoneName::LeftUpperArm).expect("bone");
        assert!((bone.translation - bone_local.translation).length() < 0.01);
        assert!(bone.rotation.angle_between(bone_local.rotation) < 0.02);
    }

    #[test]
    fn delta_applies_bone_delta() {
        let mut baseline = None;
        let base_bone = Transform {
            translation: Vec3::new(0.0, 0.1, 0.0),
            rotation: Quat::from_rotation_x(0.2),
            ..Default::default()
        };
        resolve_msg(
            AgentMsg::Keyframe {
                id:    1,
                space: SPACE,
                pose:  keyframe_pose_with_bone(Vec3::ZERO, BoneName::Head, base_bone),
            },
            &mut baseline,
        );

        let moved_bone = Transform {
            translation: base_bone.translation + Vec3::new(0.01, -0.02, 0.005),
            rotation: Quat::from_rotation_x(0.35),
            ..Default::default()
        };
        let delta = delta_pose(
            keyframe_pose_with_bone(Vec3::ZERO, BoneName::Head, moved_bone),
            &keyframe_pose_with_bone(Vec3::ZERO, BoneName::Head, base_bone),
        );
        let resolved = resolve_msg(
            AgentMsg::Delta {
                keyframe: 1,
                space:    SPACE,
                pose:     delta,
            },
            &mut baseline,
        )
        .expect("resolved");

        let bone = resolved.bones.get(&BoneName::Head).expect("bone");
        assert!((bone.translation - moved_bone.translation).length() < 0.02);
        assert!(bone.rotation.angle_between(moved_bone.rotation) < 0.05);
    }

    #[test]
    fn delta_with_stale_keyframe_id_is_dropped() {
        let mut baseline = None;
        resolve_msg(
            AgentMsg::Keyframe {
                id:    2,
                space: SPACE,
                pose:  keyframe_pose(Vec3::ZERO),
            },
            &mut baseline,
        );

        let stale = AgentMsg::Delta {
            keyframe: 1,
            space:    SPACE,
            pose:     Pose::default(),
        };
        assert!(resolve_msg(stale, &mut baseline).is_none());
    }
}
