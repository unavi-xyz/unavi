use std::collections::BTreeMap;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use hsd::{
    id::{
        BlobId,
        PrimId,
    },
    property::Property,
    state::event::SceneEvent,
};
use smol_str::SmolStr;

use crate::{
    Hsd,
    HsdBulk,
    HsdChild,
    HsdCommitGate,
    HsdPrimIndex,
    HsdRelationships,
    Prim,
    attributes::PARSERS,
    loaded::HsdSnapshotDrained,
};

/// Re-emits the whole realized scene when a document enters the world, so a
/// document built before its entity existed still reaches the ECS. Replaces
/// the export-then-reimport trick that forced Loro to replay its own state.
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
/// Component writes go through `Commands` and are not visible until the next
/// sync point, so two slots written in one batch would otherwise clobber each
/// other. Each map is seeded from the live component the first time its prim
/// is touched.
#[derive(Default)]
struct Staged {
    rels: HashMap<Entity, BTreeMap<SmolStr, PrimId>>,
    bulk: HashMap<Entity, BTreeMap<SmolStr, BlobId>>,
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

    fn bulk<'a>(
        &'a mut self,
        prim_ent: Entity,
        live: &Query<&HsdBulk>,
    ) -> &'a mut BTreeMap<SmolStr, BlobId> {
        self.bulk
            .entry(prim_ent)
            .or_insert_with(|| live.get(prim_ent).map(|b| b.0.clone()).unwrap_or_default())
    }
}

pub fn drain_scene_events(
    docs: Query<(Entity, &Hsd, Option<&HsdCommitGate>)>,
    mut indices: Query<&mut HsdPrimIndex>,
    rels_now: Query<&HsdRelationships>,
    bulk_now: Query<&HsdBulk>,
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
                &bulk_now,
                &mut commands,
            );
        }

        // Writing an identical map would still trip `Changed`, and the mesh
        // and collider rebuilds it drives tear down and respawn their blob
        // loaders.
        for (prim_ent, rels) in staged.rels {
            if rels_now.get(prim_ent).is_ok_and(|live| live.0 == rels) {
                continue;
            }
            commands.entity(prim_ent).insert(HsdRelationships(rels));
        }
        for (prim_ent, slots) in staged.bulk {
            if bulk_now.get(prim_ent).is_ok_and(|live| live.0 == slots) {
                continue;
            }
            commands.entity(prim_ent).insert(HsdBulk(slots));
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
    bulk_now: &Query<&HsdBulk>,
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
            staged.bulk.remove(&prim_ent);
            commands.entity(prim_ent).despawn();
        }
        SceneEvent::Property { prim, name, value } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, "prim not found for property {name}");
                return;
            };
            apply_property(commands, staged, rels_now, prim_ent, &name, value);
        }
        SceneEvent::Bulk { prim, slot, value } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, "prim not found for slot {slot}");
                return;
            };
            let slots = staged.bulk(prim_ent, bulk_now);
            match value {
                Some(value) => {
                    slots.insert(slot, value.hash);
                }
                None => {
                    slots.remove(&slot);
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

/// A property key holds either an attribute or a relationship, so removal
/// clears both — the key is gone and the reader cannot know which it was.
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
