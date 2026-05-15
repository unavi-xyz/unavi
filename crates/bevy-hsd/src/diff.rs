use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::{platform::collections::HashMap, prelude::*};
use loro::{TreeDiffItem, TreeExternalDiff, TreeID, TreeParentId, ValueOrContainer};

use crate::{
    HsdChild, Prim,
    attributes::{ApplyEvent, AttrDataEvent, PARSERS},
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

impl HsdDiffEvent {
    const fn target_prim(&self) -> TreeID {
        match self {
            Self::Prim(p) => p.target,
            Self::Attr { prim, .. } | Self::AttrData { prim, .. } => *prim,
        }
    }
}

#[derive(Component)]
pub struct DiffQueue(pub Arc<Mutex<Receiver<HsdDiffEvent>>>);

macro_rules! find_prim {
    ($created:ident, $prims:ident, $prim:ident, $doc:ident) => {{
        if let Some(found) = $created.get(&($doc, $prim)) {
            *found
        } else if let Some((found, _, _)) =
            $prims.iter().find(|(_, d, p)| d.0 == $doc && p.0 == $prim)
        {
            found
        } else {
            warn!("Prim not found: {}", $prim);
            continue;
        }
    }};
}

pub fn drain_diff_queues(
    prims: Query<(Entity, &HsdChild, &Prim)>,
    queues: Query<(Entity, &DiffQueue)>,
    mut commands: Commands,
    mut created: Local<HashMap<(Entity, TreeID), Entity>>,
) {
    for (doc_ent, queue) in queues {
        let Ok(queue) = queue.0.try_lock() else {
            continue;
        };

        while let Ok(event) = queue.try_recv() {
            let prim = event.target_prim();

            match event {
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Create { parent, .. },
                    ..
                }) => {
                    let prim_ent = commands.spawn((Prim(prim), HsdChild(doc_ent))).id();
                    created.insert((doc_ent, prim), prim_ent);

                    if let TreeParentId::Node(parent_id) = parent {
                        let parent_ent = find_prim!(created, prims, parent_id, doc_ent);
                        commands.entity(parent_ent).add_child(prim_ent);
                    }
                }
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Move { parent, .. },
                    ..
                }) => {
                    let prim_ent = find_prim!(created, prims, prim, doc_ent);

                    // Always orphan, in case the new parent is not found.
                    commands.entity(prim_ent).remove::<ChildOf>();

                    if let TreeParentId::Node(parent_id) = parent {
                        let parent_ent = find_prim!(created, prims, parent_id, doc_ent);
                        commands.entity(parent_ent).add_child(prim_ent);
                    }
                }
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Delete { .. },
                    ..
                }) => {
                    let prim_ent = find_prim!(created, prims, prim, doc_ent);

                    commands.entity(prim_ent).despawn();
                }
                HsdDiffEvent::Attr { attr, value, .. } => {
                    let prim_ent = find_prim!(created, prims, prim, doc_ent);

                    let Some(p) = PARSERS.get(attr.as_str()) else {
                        warn!("Unknown attribute: {attr}");
                        continue;
                    };

                    if let Err(err) = p.lifecycle(&mut commands, prim_ent, value) {
                        error!(%attr, ?err, "Failed to handle attribute lifecycle");
                    }
                }
                HsdDiffEvent::AttrData { data, .. } => {
                    let prim_ent = find_prim!(created, prims, prim, doc_ent);

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

    created.clear();
    if created.capacity() > 2048 {
        created.shrink_to(256);
    }
}
