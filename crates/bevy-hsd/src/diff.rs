use std::collections::BTreeMap;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use hsd::{
    id::PrimId,
    property::Property,
    state::event::SceneEvent,
};
use smol_str::SmolStr;

use crate::{
    Hsd,
    HsdChild,
    HsdCommitGate,
    HsdHeld,
    HsdPrimIndex,
    HsdRelationships,
    HsdSlots,
    Prim,
    attributes::PARSERS,
    loaded::HsdSnapshotDrained,
};

/// Re-emits the whole realized scene when a document enters the world, so a
/// document built before its entity existed still reaches the ECS.
pub fn resync_on_spawn(trigger: On<Add, Hsd>, docs: Query<&Hsd>, mut commands: Commands) {
    if let Ok(doc) = docs.get(trigger.entity)
        && let Ok(mut state) = doc.0.lock()
    {
        state.resync();
    }
    commands
        .entity(trigger.entity)
        .insert(HsdPrimIndex::default());
}

/// Per-prim maps accumulated across a whole batch.
///
/// Writes go through `Commands`, invisible until the next sync point, so two
/// slots written in one batch would otherwise clobber each other. Each map
/// seeds from the live component the first time its prim is touched.
#[derive(Default)]
struct Staged {
    rels:  HashMap<Entity, BTreeMap<SmolStr, PrimId>>,
    slots: HashMap<Entity, BTreeMap<SmolStr, Vec<u8>>>,
}

impl Staged {
    fn rels<'a>(
        &'a mut self,
        prim_ent: Entity,
        live: &Query<&HsdRelationships>,
    ) -> &'a mut BTreeMap<SmolStr, PrimId> {
        self.rels
            .entry(prim_ent)
            .or_insert_with(|| live.get(prim_ent).map(|r| r.0.clone()).unwrap_or_default())
    }

    fn slots<'a>(
        &'a mut self,
        prim_ent: Entity,
        live: &Query<&HsdSlots>,
    ) -> &'a mut BTreeMap<SmolStr, Vec<u8>> {
        self.slots
            .entry(prim_ent)
            .or_insert_with(|| live.get(prim_ent).map(|s| s.0.clone()).unwrap_or_default())
    }
}

/// Drops what a held document's writes emitted. Nothing is listening, and
/// placing it re-emits the scene in full, so keeping them would only grow a
/// buffer for as long as the document stays out of the world.
pub fn discard_held_events(held: Query<&HsdHeld>) {
    for doc in &held {
        let Ok(mut state) = doc.0.lock() else {
            warn!("scene state poisoned");
            continue;
        };
        state.drain_events();
    }
}

pub fn drain_scene_events(
    docs: Query<(Entity, &Hsd, Option<&HsdCommitGate>)>,
    mut indices: Query<&mut HsdPrimIndex>,
    rels_now: Query<&HsdRelationships>,
    slots_now: Query<&HsdSlots>,
    drained: Query<(), With<HsdSnapshotDrained>>,
    mut commands: Commands,
) {
    for (doc_ent, doc, gate) in &docs {
        if gate.is_some_and(HsdCommitGate::is_held) {
            continue;
        }
        let Ok(mut state) = doc.0.lock() else {
            warn!("scene state poisoned");
            continue;
        };
        let events = state.drain_events();
        drop(state);

        if events.is_empty() && drained.contains(doc_ent) {
            continue;
        }

        let Ok(mut index) = indices.get_mut(doc_ent) else {
            continue;
        };

        let mut staged = Staged::default();
        for event in events {
            process_event(
                event,
                doc_ent,
                &mut index,
                &mut staged,
                &rels_now,
                &slots_now,
                &mut commands,
            );
        }

        // Writing an identical map would still trip `Changed` and its rebuilds.
        for (prim_ent, rels) in staged.rels {
            if rels_now.get(prim_ent).is_ok_and(|live| live.0 == rels) {
                continue;
            }
            commands.entity(prim_ent).insert(HsdRelationships(rels));
        }
        for (prim_ent, slots) in staged.slots {
            if slots_now.get(prim_ent).is_ok_and(|live| live.0 == slots) {
                continue;
            }
            commands.entity(prim_ent).insert(HsdSlots(slots));
        }

        if !drained.contains(doc_ent) {
            commands.entity(doc_ent).insert(HsdSnapshotDrained);
        }
    }
}

fn process_event(
    event: SceneEvent,
    doc_ent: Entity,
    index: &mut HsdPrimIndex,
    staged: &mut Staged,
    rels_now: &Query<&HsdRelationships>,
    slots_now: &Query<&HsdSlots>,
    commands: &mut Commands,
) {
    match event {
        SceneEvent::Realized { prim, parent } => {
            let prim_ent = commands.spawn((Prim(prim), HsdChild(doc_ent))).id();
            index.0.insert(prim, prim_ent);
            let parent_ent = parent_entity(index, doc_ent, parent);
            commands.entity(parent_ent).add_child(prim_ent);
        }
        SceneEvent::Reparented { prim, parent } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, "reparented prim not found");
                return;
            };
            let parent_ent = parent_entity(index, doc_ent, parent);
            commands.entity(parent_ent).add_child(prim_ent);
        }
        SceneEvent::Unrealized { prim } => {
            let Some(prim_ent) = index.0.remove(&prim) else {
                return;
            };
            staged.rels.remove(&prim_ent);
            staged.slots.remove(&prim_ent);
            commands.entity(prim_ent).despawn();
        }
        SceneEvent::Property { prim, name, value } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, "prim not found for property {name}");
                return;
            };
            apply_property(commands, staged, rels_now, prim_ent, &name, value);
        }
        SceneEvent::Slot { prim, name, value } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, "prim not found for slot {name}");
                return;
            };
            let slots = staged.slots(prim_ent, slots_now);
            match value {
                Some(value) => {
                    slots.insert(name, value);
                }
                None => {
                    slots.remove(&name);
                }
            }
        }
    }
}

fn parent_entity(index: &HsdPrimIndex, doc_ent: Entity, parent: Option<PrimId>) -> Entity {
    parent
        .and_then(|parent| index.0.get(&parent).copied())
        .unwrap_or(doc_ent)
}

/// A property key holds either an attribute or a relationship; removal clears
/// both because the key cannot tell which it was.
fn apply_property(
    commands: &mut Commands,
    staged: &mut Staged,
    rels_now: &Query<&HsdRelationships>,
    prim_ent: Entity,
    name: &str,
    value: Option<Property>,
) {
    match value {
        Some(Property::Relationship(target)) => {
            staged.rels(prim_ent, rels_now).insert(name.into(), target);
        }
        Some(Property::Attribute(payload)) => {
            let Some(parser) = PARSERS.get(name) else {
                return;
            };
            if let Err(err) = parser.lifecycle(commands, prim_ent, Some(&payload)) {
                error!(%name, ?err, "failed to apply attribute");
            }
        }
        None => {
            staged.rels(prim_ent, rels_now).remove(name);
            if let Some(parser) = PARSERS.get(name)
                && let Err(err) = parser.lifecycle(commands, prim_ent, None)
            {
                error!(%name, ?err, "failed to remove attribute");
            }
        }
    }
}
