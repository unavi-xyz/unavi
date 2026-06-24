use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    time::Duration,
};

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
use loro::TreeID;
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
        object::{
            ObjectSender,
            OutgoingObject,
            ResolvedObject,
            submit_object,
        },
    },
    shared::StreamIdent,
    types::{
        f16_vec3::F16Vec3,
        f32_vec3::F32Vec3,
        rigid_transform::RigidTransform,
    },
};

/// A message on a per-document object stream. The stream's document id is fixed
/// in its header, so frames omit it; [`ObjectMsg::SpaceChange`] announces the
/// space the document currently sits in, sent only when it changes, and the
/// i/p-frames carry one prim's rigid-body update.
#[derive(Serialize, Deserialize, MaxSize)]
enum ObjectMsg {
    SpaceChange {
        space: [u8; 32],
    },
    IFrame {
        id:           u32,
        prim_peer:    u64,
        prim_counter: i32,
        root:         RigidTransform<F32Vec3>,
        lin:          F32Vec3,
        ang:          F32Vec3,
    },
    PFrame {
        iframe:       u32,
        prim_peer:    u64,
        prim_counter: i32,
        root:         RigidTransform<F16Vec3>,
        lin:          F32Vec3,
        ang:          F32Vec3,
    },
}

const IFRAME_FREQ: Duration = Duration::from_secs(5);

/// Per-prim i-frame baseline tracked by a sender, so its p-frame deltas resolve
/// against the right i-frame.
struct SendState {
    iframe_id:        u32,
    last_iframe_root: RigidTransform<F32Vec3>,
    last_iframe_time: Instant,
    last_space:       Hash,
}

/// One open stream carrying every dynamic prim of a single document. The space
/// is tracked here so a [`ObjectMsg::SpaceChange`] is emitted once per change
/// rather than per frame.
struct DocStream {
    tx:         SendStream,
    last_space: Option<Hash>,
    prims:      HashMap<TreeID, SendState>,
}

pub async fn send_object_stream(connection: &Connection) -> anyhow::Result<()> {
    let (obj_tx, obj_rx) = async_channel::bounded::<Vec<OutgoingObject>>(1);

    AsyncCommands::default()
        .spawn((PeerStream(connection.remote_id()), ObjectSender(obj_tx)))
        .send()
        .await?;

    let mut streams: HashMap<Hash, DocStream> = HashMap::new();
    let mut buf = [0; ObjectMsg::POSTCARD_MAX_SIZE];

    while let Ok(objects) = obj_rx.recv().await {
        let now = Instant::now();
        let mut owned = HashSet::new();
        for obj in &objects {
            owned.insert(obj.doc);
            send_object(connection, &mut streams, &mut buf, obj, now).await?;
        }

        // Documents no longer owned drop out of the batch; close their streams so
        // the receiver stops driving the replica and stale baselines are freed.
        let stale = streams
            .keys()
            .filter(|doc| !owned.contains(*doc))
            .copied()
            .collect::<Vec<_>>();
        for doc in stale {
            if let Some(mut stream) = streams.remove(&doc) {
                let _ = stream.tx.finish();
            }
        }
    }

    Ok(())
}

async fn send_object(
    connection: &Connection,
    streams: &mut HashMap<Hash, DocStream>,
    buf: &mut [u8],
    obj: &OutgoingObject,
    now: Instant,
) -> anyhow::Result<()> {
    let stream = match streams.entry(obj.doc) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => {
            let mut tx = connection.open_bi().await?.0;
            StreamIdent::Object.write(&mut tx).await?;
            tx.write_all(obj.doc.as_bytes()).await?;
            e.insert(DocStream {
                tx,
                last_space: None,
                prims: HashMap::new(),
            })
        }
    };

    if stream.last_space != Some(obj.space) {
        stream.last_space = Some(obj.space);
        let msg = ObjectMsg::SpaceChange {
            space: *obj.space.as_bytes(),
        };
        write_frame(&mut stream.tx, &msg, buf).await?;
    }

    let msg = build_msg(&mut stream.prims, obj, now);
    write_frame(&mut stream.tx, &msg, buf).await
}

async fn write_frame(tx: &mut SendStream, msg: &ObjectMsg, buf: &mut [u8]) -> anyhow::Result<()> {
    let out = postcard::to_slice(msg, buf)?;
    let len = out.len();
    tx.write_u8(u8::try_from(len).expect("max size")).await?;
    tx.write_all(out).await?;
    Ok(())
}

fn build_msg(
    prims: &mut HashMap<TreeID, SendState>,
    obj: &OutgoingObject,
    now: Instant,
) -> ObjectMsg {
    let root = RigidTransform::<F32Vec3>::from(&obj.root);
    let lin = obj.lin.into();
    let ang = obj.ang.into();

    let state = prims.entry(obj.prim).or_insert_with(|| SendState {
        iframe_id:        0,
        last_iframe_root: root.clone(),
        last_iframe_time: now - IFRAME_FREQ,
        last_space:       Hash::from_bytes([0; 32]),
    });

    // A p-frame delta is only valid against an i-frame in the same space, so a
    // space change forces a fresh i-frame.
    let new_iframe =
        now.duration_since(state.last_iframe_time) >= IFRAME_FREQ || state.last_space != obj.space;

    if new_iframe {
        state.iframe_id += 1;
        state.last_iframe_root = root.clone();
        state.last_iframe_time = now;
        state.last_space = obj.space;
        ObjectMsg::IFrame {
            id: state.iframe_id,
            prim_peer: obj.prim.peer,
            prim_counter: obj.prim.counter,
            root,
            lin,
            ang,
        }
    } else {
        ObjectMsg::PFrame {
            iframe: state.iframe_id,
            prim_peer: obj.prim.peer,
            prim_counter: obj.prim.counter,
            root: RigidTransform::<F16Vec3>::delta(&root, &state.last_iframe_root),
            lin,
            ang,
        }
    }
}

/// Per-prim i-frame baseline tracked by a receiver.
struct Baseline {
    id:   u32,
    root: RigidTransform<F32Vec3>,
}

/// Reconstructs a full-precision update for a frame, tracking the i-frame
/// baseline per prim. Returns `None` for a [`ObjectMsg::SpaceChange`], a
/// p-frame that arrives before its i-frame, or one referencing a stale i-frame.
fn resolve_msg(
    msg: ObjectMsg,
    doc: Hash,
    space: Hash,
    baselines: &mut HashMap<TreeID, Baseline>,
) -> Option<ResolvedObject> {
    match msg {
        ObjectMsg::SpaceChange { .. } => None,
        ObjectMsg::IFrame {
            id,
            prim_peer,
            prim_counter,
            root,
            lin,
            ang,
        } => {
            let prim = TreeID::new(prim_peer, prim_counter);
            let transform = root.clone().into();
            baselines.insert(prim, Baseline { id, root });
            Some(ResolvedObject {
                doc,
                space,
                prim,
                root: transform,
                lin: lin.into(),
                ang: ang.into(),
            })
        }
        ObjectMsg::PFrame {
            iframe,
            prim_peer,
            prim_counter,
            root,
            lin,
            ang,
        } => {
            let prim = TreeID::new(prim_peer, prim_counter);
            let baseline = baselines.get(&prim)?;
            if baseline.id != iframe {
                return None;
            }
            let translation = root.tra.apply_to(baseline.root.tra).into();
            Some(ResolvedObject {
                doc,
                space,
                prim,
                root: Transform {
                    translation,
                    rotation: root.rot.into(),
                    ..Default::default()
                },
                lin: lin.into(),
                ang: ang.into(),
            })
        }
    }
}

pub async fn recv_object_stream(
    peer: EndpointId,
    _tx: SendStream,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    let mut doc = [0u8; 32];
    rx.read_exact(&mut doc).await.context("read doc header")?;
    let doc = Hash::from_bytes(doc);

    let mut buf = [0; ObjectMsg::POSTCARD_MAX_SIZE];
    let mut baselines: HashMap<TreeID, Baseline> = HashMap::new();
    let mut current_space: Option<Hash> = None;

    loop {
        let len = match rx.read_u8().await {
            Ok(len) => len as usize,
            Err(err) if super::read_disconnected(&err) => return Ok(()),
            Err(err) => return Err(err).context("read len"),
        };
        let buf = &mut buf[..len];
        rx.read_exact(buf).await?;
        let msg = postcard::from_bytes::<ObjectMsg>(buf)?;

        match msg {
            ObjectMsg::SpaceChange { space } => current_space = Some(Hash::from_bytes(space)),
            frame => {
                let Some(space) = current_space else {
                    continue;
                };
                if let Some(resolved) = resolve_msg(frame, doc, space, &mut baselines) {
                    submit_object(peer, resolved);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::*;

    fn h(seed: &[u8]) -> Hash {
        blake3::hash(seed)
    }

    fn prim() -> TreeID {
        TreeID::new(42, 7)
    }

    fn outgoing(doc: Hash, space: Hash, t: Vec3) -> OutgoingObject {
        OutgoingObject {
            doc,
            space,
            prim: prim(),
            root: Transform::from_translation(t),
            lin: Vec3::new(1.0, 0.0, 0.0),
            ang: Vec3::new(0.0, 2.0, 0.0),
        }
    }

    #[test]
    fn first_msg_is_iframe_and_resolves() {
        let mut prims = HashMap::new();
        let mut baselines = HashMap::new();
        let doc = h(b"doc");
        let space = h(b"space");
        let pos = Vec3::new(1.0, 2.0, 3.0);

        let msg = build_msg(&mut prims, &outgoing(doc, space, pos), Instant::now());
        assert!(matches!(msg, ObjectMsg::IFrame { .. }));

        let resolved = resolve_msg(msg, doc, space, &mut baselines).expect("resolved");
        assert_eq!(resolved.doc, doc);
        assert_eq!(resolved.space, space);
        assert_eq!(resolved.prim, prim());
        assert!((resolved.root.translation - pos).length() < 0.01);
        assert!((resolved.lin - Vec3::new(1.0, 0.0, 0.0)).length() < 0.01);
        assert!((resolved.ang - Vec3::new(0.0, 2.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn pframe_applies_delta_to_baseline() {
        let mut prims = HashMap::new();
        let mut baselines = HashMap::new();
        let doc = h(b"doc");
        let space = h(b"space");
        let now = Instant::now();

        let iframe = build_msg(
            &mut prims,
            &outgoing(doc, space, Vec3::new(1.0, 2.0, 3.0)),
            now,
        );
        resolve_msg(iframe, doc, space, &mut baselines);

        let moved = Vec3::new(1.1, 1.8, 3.05);
        let pframe = build_msg(&mut prims, &outgoing(doc, space, moved), now);
        assert!(matches!(pframe, ObjectMsg::PFrame { .. }));

        let resolved = resolve_msg(pframe, doc, space, &mut baselines).expect("resolved");
        assert!((resolved.root.translation - moved).length() < 0.05);
    }

    #[test]
    fn space_change_forces_iframe() {
        let mut prims = HashMap::new();
        let doc = h(b"doc");
        let now = Instant::now();

        let first = build_msg(&mut prims, &outgoing(doc, h(b"a"), Vec3::ZERO), now);
        assert!(matches!(first, ObjectMsg::IFrame { .. }));

        let second = build_msg(&mut prims, &outgoing(doc, h(b"b"), Vec3::ZERO), now);
        assert!(matches!(second, ObjectMsg::IFrame { .. }));
    }

    #[test]
    fn pframe_before_iframe_is_dropped() {
        let mut baselines = HashMap::new();
        let msg = ObjectMsg::PFrame {
            iframe:       1,
            prim_peer:    42,
            prim_counter: 7,
            root:         RigidTransform::default(),
            lin:          F32Vec3::default(),
            ang:          F32Vec3::default(),
        };
        assert!(resolve_msg(msg, h(b"doc"), h(b"space"), &mut baselines).is_none());
    }

    #[test]
    fn pframe_with_stale_iframe_id_is_dropped() {
        let mut prims = HashMap::new();
        let mut baselines = HashMap::new();
        let doc = h(b"doc");
        let space = h(b"space");

        let iframe = build_msg(
            &mut prims,
            &outgoing(doc, space, Vec3::ZERO),
            Instant::now(),
        );
        resolve_msg(iframe, doc, space, &mut baselines);

        let stale = ObjectMsg::PFrame {
            iframe:       99,
            prim_peer:    42,
            prim_counter: 7,
            root:         RigidTransform::default(),
            lin:          F32Vec3::default(),
            ang:          F32Vec3::default(),
        };
        assert!(resolve_msg(stale, doc, space, &mut baselines).is_none());
    }

    #[test]
    fn distinct_prims_track_independent_baselines() {
        let mut prims = HashMap::new();
        let mut baselines = HashMap::new();
        let doc = h(b"doc");
        let space = h(b"space");
        let now = Instant::now();

        let a = OutgoingObject {
            prim: TreeID::new(1, 1),
            ..outgoing(doc, space, Vec3::new(5.0, 0.0, 0.0))
        };
        let b = OutgoingObject {
            prim: TreeID::new(2, 2),
            ..outgoing(doc, space, Vec3::new(-5.0, 0.0, 0.0))
        };

        let ra =
            resolve_msg(build_msg(&mut prims, &a, now), doc, space, &mut baselines).expect("a");
        let rb =
            resolve_msg(build_msg(&mut prims, &b, now), doc, space, &mut baselines).expect("b");
        assert!((ra.root.translation - Vec3::new(5.0, 0.0, 0.0)).length() < 0.01);
        assert!((rb.root.translation - Vec3::new(-5.0, 0.0, 0.0)).length() < 0.01);
        assert_eq!(baselines.len(), 2);
    }
}
