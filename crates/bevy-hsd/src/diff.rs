use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;
use loro::{TreeDiffItem, TreeExternalDiff, TreeID, TreeParentId, ValueOrContainer};

use crate::{
    HsdChild, HsdPrimIndex, HsdRelationships, Prim,
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
    Relationship {
        prim: TreeID,
        key: String,
        target: Option<TreeID>,
    },
}

impl HsdDiffEvent {
    const fn target_prim(&self) -> TreeID {
        match self {
            Self::Prim(p) => p.target,
            Self::Attr { prim, .. }
            | Self::AttrData { prim, .. }
            | Self::Relationship { prim, .. } => *prim,
        }
    }
}

#[derive(Component)]
pub struct DiffQueue(pub Arc<Mutex<Receiver<HsdDiffEvent>>>);

pub fn drain_diff_queues(
    queues: Query<(Entity, &DiffQueue)>,
    mut indices: Query<&mut HsdPrimIndex>,
    mut relationships: Query<&mut HsdRelationships>,
    mut commands: Commands,
) {
    for (doc_ent, queue) in queues {
        let Ok(queue) = queue.0.try_lock() else {
            continue;
        };
        let Ok(mut index) = indices.get_mut(doc_ent) else {
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
                    index.0.insert(prim, prim_ent);

                    if let TreeParentId::Node(parent_id) = parent
                        && let Some(&parent_ent) = index.0.get(&parent_id)
                    {
                        commands.entity(parent_ent).add_child(prim_ent);
                    }
                }
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Move { parent, .. },
                    ..
                }) => {
                    let Some(&prim_ent) = index.0.get(&prim) else {
                        warn!("prim not found: {prim}");
                        continue;
                    };
                    commands.entity(prim_ent).remove::<ChildOf>();
                    if let TreeParentId::Node(parent_id) = parent
                        && let Some(&parent_ent) = index.0.get(&parent_id)
                    {
                        commands.entity(parent_ent).add_child(prim_ent);
                    }
                }
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Delete { .. },
                    ..
                }) => {
                    let Some(prim_ent) = index.0.remove(&prim) else {
                        warn!("prim not found: {prim}");
                        continue;
                    };
                    commands.entity(prim_ent).despawn();
                }
                HsdDiffEvent::Attr { attr, value, .. } => {
                    let Some(&prim_ent) = index.0.get(&prim) else {
                        warn!("prim not found: {prim}");
                        continue;
                    };
                    let Some(parser) = PARSERS.get(attr.as_str()) else {
                        warn!("unknown attribute: {attr}");
                        continue;
                    };
                    if let Err(err) = parser.lifecycle(&mut commands, prim_ent, value) {
                        error!(%attr, ?err, "failed to handle attribute lifecycle");
                    }
                }
                HsdDiffEvent::AttrData { data, .. } => {
                    let Some(&prim_ent) = index.0.get(&prim) else {
                        warn!("prim not found: {prim}");
                        continue;
                    };
                    match data {
                        AttrDataEvent::Image(value) => commands
                            .entity(prim_ent)
                            .trigger(|entity| ApplyEvent { entity, value }),
                        AttrDataEvent::Mesh(value) => commands
                            .entity(prim_ent)
                            .trigger(|entity| ApplyEvent { entity, value }),
                        AttrDataEvent::Xform(value) => commands
                            .entity(prim_ent)
                            .trigger(|entity| ApplyEvent { entity, value }),
                    };
                }
                HsdDiffEvent::Relationship { key, target, .. } => {
                    let Some(&prim_ent) = index.0.get(&prim) else {
                        warn!("prim not found: {prim}");
                        continue;
                    };
                    apply_relationship(&mut commands, &mut relationships, prim_ent, key, target);
                }
            }
        }
    }
}

fn apply_relationship(
    commands: &mut Commands,
    relationships: &mut Query<&mut HsdRelationships>,
    prim_ent: Entity,
    key: String,
    target: Option<TreeID>,
) {
    if let Ok(mut rels) = relationships.get_mut(prim_ent) {
        match target {
            Some(target) => {
                rels.0.insert(key, target);
            }
            None => {
                rels.0.remove(&key);
            }
        }
        if rels.0.is_empty() {
            commands.entity(prim_ent).remove::<HsdRelationships>();
        }
    } else if let Some(target) = target {
        let mut rels = HsdRelationships::default();
        rels.0.insert(key, target);
        commands.entity(prim_ent).insert(rels);
    }
}
