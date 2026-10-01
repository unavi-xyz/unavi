use std::collections::{
    BTreeMap,
    HashSet,
};

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
    attributes,
    document::{
        Hsd,
        Unplaced,
    },
    feed::DocFeed,
    loaded::SnapshotDrained,
    prim::{
        HsdRelationships,
        Prim,
        PrimIndex,
        PrimOf,
    },
};

/// Re-emits the whole scene when a placed document spawns.
pub fn resync_on_add(trigger: On<Add, Hsd>, docs: Query<(&Hsd, Has<Unplaced>)>) {
    let Ok((doc, unplaced)) = docs.get(trigger.entity) else {
        return;
    };
    if unplaced {
        return;
    }
    if let Some(mut state) = doc.lock() {
        state.resync();
    }
}

/// Re-emits the whole scene when a document is placed.
pub fn resync_on_place(trigger: On<Remove, Unplaced>, docs: Query<&Hsd>) {
    if let Ok(doc) = docs.get(trigger.entity)
        && let Some(mut state) = doc.lock()
    {
        state.resync();
    }
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

pub fn discard_unplaced_events(docs: Query<&Hsd, With<Unplaced>>) {
    for doc in &docs {
        if let Some(mut state) = doc.lock() {
            state.drain_events();
        }
    }
}

pub fn drain_scene_events(
    docs: Query<(Entity, &Hsd), Without<Unplaced>>,
    mut indices: Query<&mut PrimIndex>,
    rels_now: Query<&HsdRelationships>,
    drained: Query<(), With<SnapshotDrained>>,
    feeds: Query<&DocFeed>,
    mut commands: Commands,
) {
    for (doc_ent, doc) in &docs {
        let Some(mut state) = doc.lock() else {
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
            let live = rels_now.get(prim_ent).ok().map(|live| &live.0);
            if live.is_some_and(|live| *live == rels) {
                continue;
            }

            let old_targets: HashSet<PrimId> = live
                .map(|live| live.values().copied().collect())
                .unwrap_or_default();
            let new_targets: HashSet<PrimId> = rels.values().copied().collect();
            for target in old_targets.difference(&new_targets) {
                index.unlink(*target, prim_ent);
            }
            for target in new_targets.difference(&old_targets) {
                index.link(*target, prim_ent);
            }

            commands.entity(prim_ent).insert(HsdRelationships(rels));
        }

        // A fed document is complete only once its feed's first read is.
        let complete = feeds.get(doc_ent).map_or(true, DocFeed::is_synced);
        if complete && !drained.contains(doc_ent) {
            commands.entity(doc_ent).insert(SnapshotDrained);
        }
    }
}

fn process_event(
    event: SceneEvent,
    doc_ent: Entity,
    index: &mut PrimIndex,
    staged: &mut StagedRels,
    rels_now: &Query<&HsdRelationships>,
    commands: &mut Commands,
) {
    match event {
        SceneEvent::Added { prim, parent } => {
            let prim_ent = commands.spawn((Prim(prim), PrimOf(doc_ent))).id();
            index.insert(prim, prim_ent);
            let parent_ent = parent_entity(index, doc_ent, parent);
            commands.entity(parent_ent).add_child(prim_ent);
        }
        SceneEvent::Reparented { prim, parent } => {
            let Some(prim_ent) = index.get(prim) else {
                warn!(%prim, "reparented prim not found");
                return;
            };
            let parent_ent = parent_entity(index, doc_ent, parent);
            commands.entity(parent_ent).add_child(prim_ent);
        }
        SceneEvent::Removed { prim } => {
            let Some(prim_ent) = index.remove(prim) else {
                return;
            };
            let final_rels = staged.remove(&prim_ent).unwrap_or_else(|| {
                rels_now
                    .get(prim_ent)
                    .map(|r| r.0.clone())
                    .unwrap_or_default()
            });
            for target in final_rels.values() {
                index.unlink(*target, prim_ent);
            }
            commands.entity(prim_ent).despawn();
        }
        SceneEvent::Property { prim, name, value } => {
            let Some(prim_ent) = index.get(prim) else {
                warn!(%prim, %name, "prim not found for property");
                return;
            };
            apply_property(commands, staged, rels_now, prim_ent, &name, value);
        }
    }
}

fn parent_entity(index: &PrimIndex, doc_ent: Entity, parent: Option<PrimId>) -> Entity {
    parent
        .and_then(|parent| index.get(parent))
        .unwrap_or(doc_ent)
}

/// A property key holds either an attribute or a relationship. Removal
/// clears both, because the key cannot tell which it was.
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
