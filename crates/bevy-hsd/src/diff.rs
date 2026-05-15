use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;
use loro::{TreeDiffItem, TreeExternalDiff, TreeID, ValueOrContainer};

use crate::{
    HsdChild, Prim,
    attributes::{ApplyEvent, AttrDataEvent},
};

pub type DiffSender = Arc<Sender<HsdDiffEvent>>;

pub enum HsdDiffEvent {
    Prim(TreeDiffItem),
    Attr {
        prim: TreeID,
        attr: String,
        value: Option<ValueOrContainer>,
    },
    AttrData {
        prim: TreeID,
        data: AttrDataEvent,
    },
}

#[derive(Component)]
pub struct DiffQueue(pub Arc<Mutex<Receiver<HsdDiffEvent>>>);

pub fn drain_diff_queues(
    prims: Query<(Entity, &HsdChild, &Prim)>,
    queues: Query<(Entity, &DiffQueue)>,
    mut commands: Commands,
) {
    for (doc_ent, queue) in queues {
        let Ok(queue) = queue.0.try_lock() else {
            continue;
        };

        while let Ok(event) = queue.try_recv() {
            match event {
                HsdDiffEvent::Prim(TreeDiffItem {
                    target,
                    action:
                        TreeExternalDiff::Create {
                            parent,
                            index,
                            position,
                        },
                }) => {}
                HsdDiffEvent::Prim(TreeDiffItem {
                    target,
                    action:
                        TreeExternalDiff::Move {
                            parent,
                            index,
                            position,
                            old_parent,
                            old_index,
                        },
                }) => {}
                HsdDiffEvent::Prim(TreeDiffItem {
                    target,
                    action:
                        TreeExternalDiff::Delete {
                            old_parent,
                            old_index,
                        },
                }) => {}
                HsdDiffEvent::Attr { prim, attr, value } => {}
                HsdDiffEvent::AttrData { prim, data } => {
                    let Some((prim_ent, _, _)) =
                        prims.iter().find(|(_, d, p)| d.0 == doc_ent && p.0 == prim)
                    else {
                        continue;
                    };

                    match data {
                        AttrDataEvent::Name(value) => commands
                            .entity(prim_ent)
                            .trigger(|entity| ApplyEvent { entity, value }),
                        AttrDataEvent::Xform(value) => commands
                            .entity(prim_ent)
                            .trigger(|entity| ApplyEvent { entity, value }),
                    };
                }
            }
        }
    }
}
