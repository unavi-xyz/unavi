use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::{pbr::MeshMaterial3d, prelude::*};
use hsd::attributes::{Attribute, material::MaterialAttr};
use loro::{TreeDiffItem, TreeExternalDiff, TreeID, TreeParentId, ValueOrContainer};

use crate::{
    HsdChild, HsdPrimIndex, HsdRelationships, Prim,
    attributes::{
        ApplyEvent, AttrDataEvent, PARSERS,
        material::{HsdMaterial, MaterialData},
    },
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

const DIFF_QUEUE_BACKPRESSURE_WARN: usize = 10_000;

pub fn drain_diff_queues(
    queues: Query<(Entity, &DiffQueue)>,
    mut indices: Query<&mut HsdPrimIndex>,
    mut relationships: Query<&mut HsdRelationships>,
    has_material_data: Query<(), With<MaterialData>>,
    mut events: Local<Vec<HsdDiffEvent>>,
    mut commands: Commands,
) {
    for (doc_ent, queue) in queues {
        let Ok(queue) = queue.0.try_lock() else {
            warn!("diff queue contended; will retry next frame");
            continue;
        };
        let Ok(mut index) = indices.get_mut(doc_ent) else {
            continue;
        };

        events.clear();
        events.extend(std::iter::from_fn(|| queue.try_recv().ok()));

        if events.len() > DIFF_QUEUE_BACKPRESSURE_WARN {
            warn!(
                drained = events.len(),
                threshold = DIFF_QUEUE_BACKPRESSURE_WARN,
                "hsd diff queue draining a large batch; producer may be outpacing consumer",
            );
        }

        // Creates first so attribute events can resolve the prim entity.
        events.sort_by_key(|e| {
            !matches!(
                e,
                HsdDiffEvent::Prim(TreeDiffItem {
                    action: TreeExternalDiff::Create { .. },
                    ..
                })
            )
        });

        for event in events.drain(..) {
            process_event(
                event,
                doc_ent,
                &mut index,
                &mut relationships,
                &has_material_data,
                &mut commands,
            );
        }
    }
}

fn process_event(
    event: HsdDiffEvent,
    doc_ent: Entity,
    index: &mut HsdPrimIndex,
    relationships: &mut Query<&mut HsdRelationships>,
    has_material_data: &Query<(), With<MaterialData>>,
    commands: &mut Commands,
) {
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
                return;
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
                return;
            };
            commands.entity(prim_ent).despawn();
        }
        HsdDiffEvent::Attr { attr, value, .. } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!("prim not found: {prim}");
                return;
            };
            let Some(parser) = PARSERS.get(attr.as_str()) else {
                warn!("unknown attribute: {attr}");
                return;
            };
            if let Err(err) = parser.lifecycle(commands, prim_ent, value) {
                error!(%attr, ?err, "failed to handle attribute lifecycle");
            }
        }
        HsdDiffEvent::AttrData { data, .. } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!("prim not found: {prim}");
                return;
            };
            dispatch_attr_data(commands, prim_ent, data);
        }
        HsdDiffEvent::Relationship { key, target, .. } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!("prim not found: {prim}");
                return;
            };
            apply_relationship(
                commands,
                relationships,
                has_material_data,
                prim_ent,
                key,
                target,
            );
        }
    }
}

fn dispatch_attr_data(commands: &mut Commands, prim_ent: Entity, data: AttrDataEvent) {
    match data {
        AttrDataEvent::Collider(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
        AttrDataEvent::Image(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
        AttrDataEvent::Material(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
        AttrDataEvent::Mesh(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
        AttrDataEvent::RigidBody(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
        AttrDataEvent::Xform(value) => {
            commands
                .entity(prim_ent)
                .trigger(|entity| ApplyEvent { entity, value });
        }
    }
}

fn apply_relationship(
    commands: &mut Commands,
    relationships: &mut Query<&mut HsdRelationships>,
    has_material_data: &Query<(), With<MaterialData>>,
    prim_ent: Entity,
    key: String,
    target: Option<TreeID>,
) {
    // A prim without a MaterialAttr can still receive a material via relationship
    // (inheriting from another prim's HsdMaterial). MaterialParser::lifecycle covers
    // the attr-driven path; this branch covers the relationship-only path so the
    // prim has the slots ready for propagate_material_to_dependents to fill.
    if key == MaterialAttr::KEY {
        match target {
            Some(_) => {
                commands.entity(prim_ent).insert((
                    HsdMaterial::default(),
                    MeshMaterial3d::<StandardMaterial>::default(),
                ));
            }
            None if !has_material_data.contains(prim_ent) => {
                commands
                    .entity(prim_ent)
                    .remove::<HsdMaterial>()
                    .remove::<MeshMaterial3d<StandardMaterial>>();
            }
            None => {}
        }
    }

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
