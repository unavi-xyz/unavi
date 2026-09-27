use std::collections::BTreeMap;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use hsd::{
    id::PrimId,
    property::{
        name::PropName,
        value::Value,
    },
    state::event::SceneEvent,
};

use crate::{
    Hsd,
    HsdChild,
    HsdHeld,
    HsdPrimIndex,
    HsdRelationships,
    Prim,
    attributes,
    feed::DocFeed,
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

/// Per-prim relationship maps accumulated across one drain, seeded from the
/// live component the first time a prim is touched.
type StagedRels = HashMap<Entity, BTreeMap<PropName, PrimId>>;

fn staged_rels<'a>(
    staged: &'a mut StagedRels,
    prim_ent: Entity,
    live: &Query<&HsdRelationships>,
) -> &'a mut BTreeMap<PropName, PrimId> {
    staged
        .entry(prim_ent)
        .or_insert_with(|| live.get(prim_ent).map(|r| r.0.clone()).unwrap_or_default())
}

/// Drops what a held document's writes emitted: nothing listens while it is
/// out of the world, and placing it re-emits the scene in full.
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
    docs: Query<(Entity, &Hsd)>,
    mut indices: Query<&mut HsdPrimIndex>,
    rels_now: Query<&HsdRelationships>,
    drained: Query<(), With<HsdSnapshotDrained>>,
    feeds: Query<&DocFeed>,
    mut commands: Commands,
) {
    for (doc_ent, doc) in &docs {
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

        let mut staged = StagedRels::default();
        for event in events {
            process_event(
                event,
                doc_ent,
                &mut index,
                &mut staged,
                &rels_now,
                &mut commands,
            );
        }

        // Writing an identical map would still trip `Changed` and its rebuilds.
        for (prim_ent, rels) in staged {
            if rels_now.get(prim_ent).is_ok_and(|live| live.0 == rels) {
                continue;
            }
            commands.entity(prim_ent).insert(HsdRelationships(rels));
        }

        // A fed document is complete only once its feed's first read is.
        let complete = feeds.get(doc_ent).map_or(true, DocFeed::is_synced);
        if complete && !drained.contains(doc_ent) {
            commands.entity(doc_ent).insert(HsdSnapshotDrained);
        }
    }
}

fn process_event(
    event: SceneEvent,
    doc_ent: Entity,
    index: &mut HsdPrimIndex,
    staged: &mut StagedRels,
    rels_now: &Query<&HsdRelationships>,
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
            staged.remove(&prim_ent);
            commands.entity(prim_ent).despawn();
        }
        SceneEvent::Property { prim, name, value } => {
            let Some(&prim_ent) = index.0.get(&prim) else {
                warn!(%prim, %name, "prim not found for property");
                return;
            };
            apply_property(commands, staged, rels_now, prim_ent, &name, value);
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
    staged: &mut StagedRels,
    rels_now: &Query<&HsdRelationships>,
    prim_ent: Entity,
    name: &PropName,
    value: Option<Value>,
) {
    match value {
        Some(Value::Relationship(target)) => {
            staged_rels(staged, prim_ent, rels_now).insert(name.clone(), target);
        }
        Some(Value::Attribute(payload)) => {
            if let Err(err) = attributes::apply(commands, prim_ent, name, Some(payload.as_ref())) {
                error!(%name, ?err, "failed to apply attribute");
            }
        }
        None => {
            // Only a key already present as a relationship needs the map
            // cloned; an attribute removal never touches it.
            let was_relationship = staged.get(&prim_ent).map_or_else(
                || {
                    rels_now
                        .get(prim_ent)
                        .is_ok_and(|live| live.0.contains_key(name))
                },
                |staged_rels| staged_rels.contains_key(name),
            );
            if was_relationship {
                staged_rels(staged, prim_ent, rels_now).remove(name);
            }
            if let Err(err) = attributes::apply(commands, prim_ent, name, None) {
                error!(%name, ?err, "failed to remove attribute");
            }
        }
    }
}
