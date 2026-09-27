use std::collections::BTreeMap;

use super::*;
use crate::{
    id::{
        DocId,
        PrimId,
    },
    key,
    property::{
        Property,
        name::PropName,
        value::Value,
    },
    schema::{
        material::{
            BINDING,
            MaterialAttr,
        },
        name::NameAttr,
        parent::ParentAttr,
        reference::{
            LayerKey,
            ReferenceAttr,
        },
        script::ScriptAttr,
        xform::XformAttr,
    },
    state::{
        entry::Entry,
        event::SceneEvent,
        layer::{
            Layer,
            LayerId,
        },
    },
};

fn prim(n: u8) -> PrimId {
    PrimId([n; 16])
}

fn parent_key(prim: PrimId) -> String {
    key::Key::prop(prim, &ParentAttr::NAME).to_string()
}

fn root_entry(id: PrimId, timestamp: u64) -> Entry {
    Entry::new(
        parent_key(id),
        ParentAttr::to_wire(Some(ParentAttr::Root)),
        timestamp,
    )
}

fn child_entry(id: PrimId, parent: PrimId, timestamp: u64) -> Entry {
    Entry::new(
        parent_key(id),
        ParentAttr::to_wire(Some(ParentAttr::Prim(parent))),
        timestamp,
    )
}

fn attr_entry<A: Property>(id: PrimId, value: &A, timestamp: u64) -> Entry {
    Entry::new(
        key::Key::prop(id, &A::NAME).to_string(),
        Value::Attribute(value.encode().expect("encode")).encode(),
        timestamp,
    )
}

fn tombstone(key: String, timestamp: u64) -> Entry {
    Entry::new(key, Vec::new(), timestamp)
}

fn apply(state: &mut HsdState, entries: &[Entry]) {
    state.apply_all(entries).expect("apply");
}

fn shape(state: &HsdState) -> BTreeMap<PrimId, Option<PrimId>> {
    state
        .prims()
        .map(|prim| (prim, state.parent(prim)))
        .collect()
}

#[test]
fn realizes_a_root_and_its_child() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), child_entry(prim(2), prim(1), 2)],
    );

    assert_eq!(state.roots(), vec![prim(1)]);
    assert_eq!(state.children(prim(1)), vec![prim(2)]);
    assert_eq!(state.parent(prim(2)), Some(prim(1)));
}

#[test]
fn entry_order_does_not_change_the_result() {
    let entries = [
        attr_entry(prim(3), &NameAttr("leaf".into()), 4),
        child_entry(prim(3), prim(2), 3),
        child_entry(prim(2), prim(1), 2),
        root_entry(prim(1), 1),
    ];

    let mut forward = HsdState::new();
    apply(&mut forward, &entries);

    let mut reversed = HsdState::new();
    let mut flipped = entries.clone();
    flipped.reverse();
    apply(&mut reversed, &flipped);

    assert_eq!(shape(&forward), shape(&reversed));
    assert_eq!(forward.entries(), reversed.entries());
    assert_eq!(
        reversed
            .attribute::<NameAttr>(prim(3))
            .expect("name")
            .expect("decode"),
        NameAttr("leaf".into())
    );
}

#[test]
fn an_orphan_is_held_not_reparented_to_the_root() {
    let mut state = HsdState::new();
    apply(&mut state, &[child_entry(prim(2), prim(1), 2)]);

    assert!(state.exists(prim(2)));
    assert!(!state.is_realized(prim(2)));
    assert_eq!(state.roots().len(), 0);
}

#[test]
fn an_orphan_realizes_with_its_properties_when_its_parent_arrives() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            child_entry(prim(2), prim(1), 2),
            attr_entry(prim(2), &XformAttr::default(), 3),
        ],
    );
    assert_eq!(state.drain_events().len(), 0);

    apply(&mut state, &[root_entry(prim(1), 1)]);
    let events = state.drain_events();

    assert!(events.contains(&SceneEvent::Realized {
        prim:   prim(2),
        parent: Some(prim(1)),
    }));
    assert!(events.iter().any(|e| matches!(
        e,
        SceneEvent::Property { prim: p, name, .. } if *p == prim(2) && *name == XformAttr::NAME
    )));
}

#[test]
fn a_property_on_an_unrealized_prim_emits_nothing() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[attr_entry(prim(9), &NameAttr("held".into()), 1)],
    );
    assert_eq!(state.drain_events().len(), 0);
}

#[test]
fn a_cycle_breaks_at_its_greatest_stamp_regardless_of_order() {
    let entries = [
        child_entry(prim(1), prim(3), 10),
        child_entry(prim(2), prim(1), 20),
        child_entry(prim(3), prim(2), 30),
    ];

    let mut forward = HsdState::new();
    apply(&mut forward, &entries);

    let mut shuffled = HsdState::new();
    apply(
        &mut shuffled,
        &[entries[2].clone(), entries[0].clone(), entries[1].clone()],
    );

    assert_eq!(shape(&forward), shape(&shuffled));
    assert_eq!(forward.roots(), vec![prim(3)]);
    assert_eq!(forward.parent(prim(1)), Some(prim(3)));
    assert_eq!(forward.parent(prim(2)), Some(prim(1)));
}

/// Folds a stream of scene events into the hierarchy a consumer builds,
/// failing if any prim is placed under a parent not yet realized.
fn replay(events: &[SceneEvent]) -> BTreeMap<PrimId, Option<PrimId>> {
    let mut scene = BTreeMap::new();
    for event in events {
        match event {
            SceneEvent::Realized { prim, parent } | SceneEvent::Reparented { prim, parent } => {
                if let Some(parent) = parent {
                    assert!(
                        scene.contains_key(parent),
                        "{prim} placed under {parent} before {parent} was realized"
                    );
                }
                scene.insert(*prim, *parent);
            }
            SceneEvent::Unrealized { prim } => {
                scene.remove(prim);
            }
            SceneEvent::Property { .. } => {}
        }
    }
    scene
}

#[test]
fn a_cycle_realizes_each_member_after_its_parent_in_any_order() {
    let entries = [
        child_entry(prim(1), prim(3), 10),
        child_entry(prim(2), prim(1), 20),
        child_entry(prim(3), prim(2), 30),
        child_entry(prim(4), prim(2), 40),
    ];
    let orders = [[0, 1, 2, 3], [2, 0, 1, 3], [3, 2, 1, 0], [1, 3, 0, 2], [0, 2, 3, 1]];

    for order in orders {
        let mut state = HsdState::new();
        let mut events = Vec::new();
        for i in order {
            apply(&mut state, &[entries[i].clone()]);
            events.extend(state.drain_events());
        }
        assert_eq!(
            replay(&events),
            shape(&state),
            "events for order {order:?} rebuild the state"
        );
    }
}

#[test]
fn a_prim_hanging_off_a_cycle_is_realized_under_its_own_parent() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            child_entry(prim(1), prim(2), 10),
            child_entry(prim(2), prim(1), 20),
            child_entry(prim(3), prim(1), 30),
        ],
    );

    assert_eq!(state.roots(), vec![prim(2)]);
    assert_eq!(state.parent(prim(3)), Some(prim(1)));
}

#[test]
fn a_reparent_closing_a_cycle_breaks_it_at_a_member_below_the_moved_prim() {
    let entries = [
        root_entry(prim(9), 1),
        child_entry(prim(1), prim(9), 10),
        child_entry(prim(2), prim(1), 20),
        child_entry(prim(3), prim(2), 30),
        child_entry(prim(1), prim(3), 15),
    ];

    let mut incremental = HsdState::new();
    apply(&mut incremental, &entries);

    let mut fresh = HsdState::new();
    apply(
        &mut fresh,
        &[
            entries[4].clone(),
            entries[3].clone(),
            entries[2].clone(),
            entries[0].clone(),
        ],
    );

    assert_eq!(incremental.roots(), vec![prim(3), prim(9)]);
    assert_eq!(shape(&incremental), shape(&fresh));
}

#[test]
fn a_cross_author_tombstone_removes_a_prim_written_by_someone_else() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            attr_entry(prim(2), &NameAttr("gone".into()), 3),
        ],
    );
    state.drain_events();

    apply(&mut state, &[tombstone(parent_key(prim(2)), 10)]);

    assert!(!state.exists(prim(2)));
    assert!(!state.is_realized(prim(2)));
    assert_eq!(state.children(prim(1)), Vec::new());
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Unrealized { prim: prim(2) }]
    );
}

#[test]
fn deleting_a_prim_holds_its_descendants_rather_than_dropping_them() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            child_entry(prim(3), prim(2), 3),
        ],
    );

    apply(&mut state, &[tombstone(parent_key(prim(2)), 10)]);
    assert!(!state.is_realized(prim(3)));
    assert!(state.exists(prim(3)));

    apply(&mut state, &[child_entry(prim(2), prim(1), 20)]);
    assert!(state.is_realized(prim(3)));
}

#[test]
fn an_older_entry_never_overwrites_a_newer_one() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("new".into()), 100),
            attr_entry(prim(1), &NameAttr("old".into()), 50),
        ],
    );

    assert_eq!(
        state
            .attribute::<NameAttr>(prim(1))
            .expect("name")
            .expect("decode"),
        NameAttr("new".into())
    );
}

#[test]
fn concurrent_writes_at_one_timestamp_resolve_the_same_way_both_orders() {
    let a = attr_entry(prim(1), &NameAttr("alpha".into()), 7);
    let b = attr_entry(prim(1), &NameAttr("beta".into()), 7);

    let mut forward = HsdState::new();
    apply(
        &mut forward,
        &[root_entry(prim(1), 1), a.clone(), b.clone()],
    );

    let mut reversed = HsdState::new();
    apply(&mut reversed, &[root_entry(prim(1), 1), b, a]);

    assert_eq!(
        forward
            .attribute::<NameAttr>(prim(1))
            .expect("name")
            .expect("decode"),
        reversed
            .attribute::<NameAttr>(prim(1))
            .expect("name")
            .expect("decode"),
    );
}

#[test]
fn an_unknown_attribute_round_trips_untouched() {
    let payload = vec![0xCA, 0xFE, 0xBA, 0xBE];
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            Entry::new(
                key::Key::prop(prim(1), &"custom/blob".parse().expect("name")).to_string(),
                Value::Attribute(payload.clone()).encode(),
                2,
            ),
        ],
    );

    let entries = state.entries();
    let stored = entries
        .get(&key::Key::prop(prim(1), &"custom/blob".parse().expect("name")).to_string())
        .expect("entry");
    assert_eq!(stored, &Value::Attribute(payload).encode());
}

#[test]
fn relationships_and_attributes_share_one_namespace() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), root_entry(prim(2), 1)],
    );

    state
        .set_attribute(prim(1), &MaterialAttr::default())
        .expect("attribute");
    state
        .set_relationship(prim(1), &BINDING, prim(2))
        .expect("relationship");

    let prim_state = state.get(prim(1)).expect("prim");
    assert!(
        prim_state
            .property(&MaterialAttr::NAME)
            .expect("attr")
            .as_attribute()
            .is_some()
    );
    assert_eq!(
        prim_state
            .property(&BINDING)
            .expect("rel")
            .as_relationship(),
        Some(prim(2))
    );
}

#[test]
fn removing_a_namespace_takes_its_relationships_with_it() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), root_entry(prim(2), 1)],
    );
    state
        .set_attribute(prim(1), &MaterialAttr::default())
        .expect("attribute");
    state
        .set_relationship(prim(1), &BINDING, prim(2))
        .expect("relationship");
    state
        .set_attribute(prim(1), &NameAttr("kept".into()))
        .expect("name");

    state.remove_group(prim(1), MaterialAttr::NAME.group());

    assert!(state.attribute::<MaterialAttr>(prim(1)).is_none());
    assert_eq!(state.relationship(prim(1), &BINDING), None);
    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("kept"));
}

#[test]
fn parent_is_only_written_through_set_parent() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    assert!(matches!(
        state.set_property(prim(1), &ParentAttr::NAME, name_attr("x")),
        Err(StateError::Reserved(_))
    ));
}

#[test]
fn script_created_prims_are_absent_from_the_save_set() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    let scratch = state.create_prim(Some(prim(1)));
    state
        .set_attribute(scratch, &NameAttr("transient".into()))
        .expect("attribute");

    assert!(state.is_realized(scratch));
    let entries = state.entries();
    assert!(entries.contains_key(&parent_key(prim(1))));
    assert!(!entries.contains_key(&parent_key(scratch)));
}

#[test]
fn a_script_editing_a_document_prim_changes_what_is_drawn_not_what_is_kept() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    let saved = state.entries();

    state
        .set_attribute(prim(1), &NameAttr("edited".into()))
        .expect("attribute");

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("edited"),
        "the edit is what every reader sees this session"
    );
    assert_eq!(
        state.entries(),
        saved,
        "and it reaches the document only through a commit, which is what \
             lets a document stay open to its keyholder instead of being \
             frozen to keep the two coherent"
    );
}

#[test]
fn an_attribute_carrying_bytes_round_trips() {
    // Bulk bytes are a field of the attribute payload, so a value holding them
    // is one property like any other.
    let payload = vec![9; 1024];
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.drain_events();

    apply(
        &mut state,
        &[attr_entry(prim(1), &ScriptAttr(payload.clone()), 2)],
    );

    assert_eq!(
        state
            .attribute::<ScriptAttr>(prim(1))
            .expect("script")
            .expect("decodes")
            .0,
        payload
    );
    assert_eq!(
        state
            .entries()
            .get(&key::Key::prop(prim(1), &ScriptAttr::NAME).to_string()),
        Some(&Value::Attribute(ScriptAttr(payload).encode().expect("encode")).encode()),
    );
}

#[test]
fn reparenting_emits_one_event_and_moves_the_subtree() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            root_entry(prim(2), 1),
            child_entry(prim(3), prim(1), 2),
            child_entry(prim(4), prim(3), 3),
        ],
    );
    state.drain_events();

    apply(&mut state, &[child_entry(prim(3), prim(2), 10)]);

    assert_eq!(state.children(prim(1)), Vec::new());
    assert_eq!(state.children(prim(2)), vec![prim(3)]);
    assert_eq!(state.parent(prim(4)), Some(prim(3)));
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Reparented {
            prim:   prim(3),
            parent: Some(prim(2)),
        }]
    );
}

#[test]
fn removing_a_property_emits_an_absent_value() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &XformAttr::default(), 2),
        ],
    );
    state.drain_events();

    apply(
        &mut state,
        &[tombstone(
            key::Key::prop(prim(1), &XformAttr::NAME).to_string(),
            3,
        )],
    );

    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  XformAttr::NAME,
            value: None,
        }]
    );
}

#[test]
fn the_save_set_round_trips_through_a_fresh_state() {
    let mut original = HsdState::new();
    apply(
        &mut original,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            attr_entry(prim(2), &NameAttr("kept".into()), 3),
            attr_entry(prim(2), &ScriptAttr(vec![7; 32]), 4),
        ],
    );

    let mut restored = HsdState::new();
    let entries = original
        .entries()
        .into_iter()
        .map(|(key, value)| Entry {
            key,
            value,
            timestamp: 100,
        })
        .collect::<Vec<_>>();
    apply(&mut restored, &entries);

    assert_eq!(shape(&original), shape(&restored));
    assert_eq!(original.entries(), restored.entries());
}

#[test]
fn nesting_past_the_depth_cap_is_not_realized() {
    let mut state = HsdState::new();

    let deep = MAX_PRIM_DEPTH + 8;
    let ids = (0..deep)
        .map(|i| {
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&(i as u64).to_be_bytes());
            PrimId::from_digest(&bytes)
        })
        .collect::<Vec<_>>();

    state.apply(&root_entry(ids[0], 0)).expect("root");
    for (i, window) in ids.windows(2).enumerate() {
        state
            .apply(&child_entry(window[1], window[0], i as u64 + 1))
            .expect("child");
    }

    assert!(state.is_realized(ids[0]), "the root realizes");
    assert!(
        state.is_realized(ids[MAX_PRIM_DEPTH - 1]),
        "prims within the cap realize"
    );
    assert!(
        !state.is_realized(ids[deep - 1]),
        "prims past the cap are held"
    );
}

#[test]
fn moving_a_subtree_under_a_deep_chain_holds_what_passes_the_cap() {
    let mut state = HsdState::new();

    let ids = (0..MAX_PRIM_DEPTH - 2)
        .map(|i| {
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&(i as u64).to_be_bytes());
            PrimId::from_digest(&bytes)
        })
        .collect::<Vec<_>>();
    state.apply(&root_entry(ids[0], 0)).expect("root");
    for (i, window) in ids.windows(2).enumerate() {
        state
            .apply(&child_entry(window[1], window[0], i as u64 + 1))
            .expect("child");
    }
    let deepest = ids[ids.len() - 1];

    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 1),
            child_entry(prim(3), prim(2), 1),
        ],
    );
    assert!(state.is_realized(prim(3)));

    apply(&mut state, &[child_entry(prim(1), deepest, 10_000)]);

    assert!(state.is_realized(prim(2)), "the last prim within the cap");
    assert!(!state.is_realized(prim(3)), "one past the cap");
}

#[test]
fn an_open_tick_withholds_its_own_writes() {
    let mut state = HsdState::new();
    state.open_tick();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    assert!(
        state.drain_events().is_empty(),
        "a prim whose creating tick has not positioned it yet must not be \
             drawn at the origin"
    );

    state.close_tick();
    assert!(
        state.drain_events().contains(&SceneEvent::Realized {
            prim:   prim(1),
            parent: None,
        }),
        "closing the tick releases it"
    );
}

#[test]
fn writes_made_before_a_tick_opened_still_drain() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.open_tick();
    apply(&mut state, &[root_entry(prim(2), 2)]);

    let events = state.drain_events();
    assert!(events.contains(&SceneEvent::Realized {
        prim:   prim(1),
        parent: None,
    }));
    assert!(
        !events.contains(&SceneEvent::Realized {
            prim:   prim(2),
            parent: None,
        }),
        "only the open tick's tail is held back"
    );
    state.close_tick();
}

#[test]
fn a_prim_and_its_properties_leave_together() {
    let mut state = HsdState::new();
    state.open_tick();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &XformAttr::default(), 2),
        ],
    );
    assert_eq!(state.drain_events().len(), 0);
    state.close_tick();

    let events = state.drain_events();
    assert!(events.contains(&SceneEvent::Realized {
        prim:   prim(1),
        parent: None,
    }));
    assert!(
        events.iter().any(|e| matches!(
            e,
            SceneEvent::Property { prim: p, name, .. }
                if *p == prim(1) && *name == XformAttr::NAME
        )),
        "the transform arrives in the same drain as the prim it belongs to"
    );
}

#[test]
fn boundaries_nest_so_two_writers_both_have_to_finish() {
    let mut state = HsdState::new();
    state.open_tick();
    state.open_tick();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    state.close_tick();
    assert!(
        state.drain_events().is_empty(),
        "one writer finishing does not release another's partial work"
    );

    state.close_tick();
    assert_ne!(state.drain_events().len(), 0);
}

#[test]
fn an_unmatched_close_does_not_underflow() {
    let mut state = HsdState::new();
    state.close_tick();
    state.open_tick();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    assert!(state.drain_events().is_empty(), "the boundary still holds");
    state.close_tick();
}

#[test]
fn a_consumer_attaching_mid_tick_gets_the_scene_as_it_stands() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.drain_events();

    state.open_tick();
    state.resync();
    assert!(
        state.drain_events().contains(&SceneEvent::Realized {
            prim:   prim(1),
            parent: None,
        }),
        "a resync is a description of now, not part of anyone's tick"
    );

    apply(&mut state, &[root_entry(prim(2), 2)]);
    assert!(
        state.drain_events().is_empty(),
        "writes after the resync are still the open tick's"
    );
    state.close_tick();
}

fn runtime_property(state: &mut HsdState, prim: PrimId, name: PropName, value: Option<Value>) {
    match value {
        Some(value) => state.set_property(prim, &name, value).expect("set"),
        None => state.remove_property(prim, &name),
    }
}

fn name_attr(value: &str) -> Value {
    Value::Attribute(NameAttr(value.into()).encode().expect("encode"))
}

fn name_of(state: &HsdState, prim: PrimId) -> Option<String> {
    Some(state.attribute::<NameAttr>(prim)?.expect("decodes").0)
}

#[test]
fn a_runtime_opinion_shadows_the_document_beneath_it() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    state.drain_events();

    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("runtime")),
    );

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: Some(name_attr("runtime")),
        }]
    );
}

#[test]
fn a_spawn_despawn_loop_leaves_nothing_behind() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    let resolved = state.resolved.len();

    for _ in 0..100 {
        let spawned = state.create_prim(Some(prim(1)));
        state
            .set_attribute(spawned, &NameAttr("bullet".into()))
            .expect("set");
        state.remove_prim(spawned);
    }

    assert_eq!(state.resolved.len(), resolved);
    assert_eq!(state.layers[LayerId::Runtime.idx()].prims().count(), 0);
    assert_eq!(state.children(prim(1)), Vec::new());
}

#[test]
fn removing_a_document_prim_blocks_it_rather_than_forgetting_it() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), child_entry(prim(2), prim(1), 2)],
    );

    state.remove_prim(prim(2));

    assert!(!state.exists(prim(2)));
    assert!(state.entries().contains_key(&parent_key(prim(2))));
}

#[test]
fn back_to_back_local_writes_keep_the_last_value() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    for i in 0..1000 {
        let value = format!("{i}");
        state
            .set_attribute(prim(1), &NameAttr(value.clone()))
            .expect("set");
        assert_eq!(name_of(&state, prim(1)), Some(value));
    }
}

#[test]
fn a_document_write_under_a_runtime_opinion_emits_nothing() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("runtime")),
    );
    state.drain_events();

    apply(
        &mut state,
        &[attr_entry(prim(1), &NameAttr("later".into()), 3)],
    );

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
    assert_eq!(
        state.drain_events(),
        Vec::new(),
        "the composed value did not move, so there is nothing to apply; a \
             regression here floods the ECS every frame"
    );
}

#[test]
fn a_blocked_opinion_hides_the_document_value_without_dropping_it() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    let saved = state.entries();
    state.drain_events();

    runtime_property(&mut state, prim(1), NameAttr::NAME, None);

    assert_eq!(
        name_of(&state, prim(1)),
        None,
        "Blocked resolves to absent and stops; the weaker value must not \
             show through"
    );
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: None,
        }]
    );
    assert_eq!(
        state.entries(),
        saved,
        "blocking is an opinion of a live layer, not an edit to the document"
    );
}

#[test]
fn a_runtime_write_leaves_the_save_set_byte_identical() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    let saved = state.entries();

    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("runtime")),
    );
    runtime_property(
        &mut state,
        prim(1),
        "custom/scratch".parse().expect("name"),
        Some(name_attr("new key")),
    );

    assert_eq!(
        state.entries(),
        saved,
        "this is the guarantee sync_document could not make, and the whole \
             reason it had to freeze a document"
    );
}

#[test]
fn a_prim_the_runtime_layer_alone_states_exists_and_reparents() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.drain_events();

    state.write_parent(
        LayerId::Runtime,
        prim(9),
        Some(ParentAttr::Prim(prim(1))),
        None,
    );

    assert!(state.exists(prim(9)));
    assert!(state.is_realized(prim(9)));
    assert_eq!(state.children(prim(1)), vec![prim(9)]);

    state.write_parent(LayerId::Runtime, prim(9), Some(ParentAttr::Root), None);

    assert_eq!(state.parent(prim(9)), None);
    assert_eq!(state.children(prim(1)), Vec::new());
    assert!(
        !state.entries().contains_key(&parent_key(prim(9))),
        "a prim only a live layer states never reaches the save set"
    );
}

#[test]
fn commit_promotes_a_live_opinion_into_the_document() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    state.drain_events();

    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("runtime")),
    );
    state.drain_events();
    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
    assert!(
        state.drain_events().is_empty(),
        "the opinion travelled whole, so what every reader sees never moved; \
             only the save set changed"
    );
    assert_eq!(
        state
            .entries()
            .get(&key::Key::prop(prim(1), &NameAttr::NAME).to_string()),
        Some(&name_attr("runtime").encode()),
        "the promoted opinion is exactly what a save now writes"
    );

    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);
    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("runtime"),
        "a second commit is a no-op: the live opinion is gone, so there is \
             nothing left to promote"
    );
}

#[test]
fn commit_with_no_writable_key_writes_the_session_layer() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    let saved = state.entries();
    state.drain_events();

    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("runtime")),
    );
    state.commit(CommitTarget::Session, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
    assert_eq!(
        state.entries(),
        saved,
        "the session fallback changes nothing a save writes — that is what \
             makes a guest's commit visible without persisting it"
    );
    apply(
        &mut state,
        &[attr_entry(prim(1), &NameAttr("later".into()), 3)],
    );
    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("runtime"),
        "the session opinion still beats a document write, so it shadows \
             until the owner adopts it with keep"
    );
}

#[test]
fn committing_a_script_created_prim_adds_it_to_the_save_set() {
    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.drain_events();

    let scratch = state.create_prim(Some(prim(1)));
    state
        .set_attribute(scratch, &NameAttr("kept".into()))
        .expect("attribute");
    assert!(!state.entries().contains_key(&parent_key(scratch)));

    state.commit(
        CommitTarget::Document,
        &[(scratch, ParentAttr::NAME), (scratch, NameAttr::NAME)],
    );

    let entries = state.entries();
    assert!(
        entries.contains_key(&parent_key(scratch)),
        "committing the parent is what makes a spawned prim survive a save"
    );
    assert!(entries.contains_key(&key::Key::prop(scratch, &NameAttr::NAME).to_string()));
    assert!(state.is_realized(scratch));
    assert_eq!(state.children(prim(1)), vec![scratch]);
}

#[test]
fn committing_a_blocked_opinion_removes_a_document_property() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    state.drain_events();

    state.remove_property(prim(1), &NameAttr::NAME);
    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(name_of(&state, prim(1)), None);
    assert!(
        !state
            .entries()
            .contains_key(&key::Key::prop(prim(1), &NameAttr::NAME).to_string())
    );
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: None,
        }]
    );
}

#[test]
fn committing_a_blocked_parent_removes_a_document_prim() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), child_entry(prim(2), prim(1), 2)],
    );
    state.drain_events();

    state.remove_prim(prim(2));
    assert!(
        !state.exists(prim(2)),
        "a script can hide a document prim for this session"
    );
    assert!(
        state.entries().contains_key(&parent_key(prim(2))),
        "but hiding is a live opinion; the document still holds the prim"
    );

    state.commit(CommitTarget::Document, &[(prim(2), ParentAttr::NAME)]);

    assert!(
        !state.entries().contains_key(&parent_key(prim(2))),
        "committing the block is what removes the prim from the document; \
             the key falls out of the save set and a diff deletes it"
    );
    assert!(!state.is_realized(prim(2)));
}

#[test]
fn committing_a_key_with_no_live_opinion_changes_nothing() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("document".into()), 2),
        ],
    );
    let saved = state.entries();

    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("document"));
    assert_eq!(state.entries(), saved);
}

fn override_entry(site: PrimId, target: PrimId, name: PropName, value: &Value) -> Entry {
    Entry::new(LayerKey { target, name }.key(site), value.encode(), 3)
}

/// A referenced document, holding one prim with a name of its own.
fn referenced() -> HsdState {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("couch".into()), 2),
        ],
    );
    state.drain_events();
    state
}

/// What a referencing document says about `site`'s target, as the realizer
/// reads it back out to install.
fn stated(site: PrimId, entries: &[Entry]) -> Layer {
    let mut referencing = HsdState::new();
    apply(&mut referencing, entries);
    referencing
        .reference_layer_for(site)
        .cloned()
        .unwrap_or_default()
}

#[test]
fn an_override_beats_the_document_it_speaks_for() {
    let mut state = referenced();
    state.install_reference_layer(&stated(
        prim(7),
        &[override_entry(
            prim(7),
            prim(1),
            NameAttr::NAME,
            &name_attr("recoloured"),
        )],
    ));

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("recoloured"));
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: Some(name_attr("recoloured")),
        }],
        "installing an override changes what is drawn, so it emits"
    );
    assert_eq!(
        state
            .entries()
            .get(&key::Key::prop(prim(1), &NameAttr::NAME).to_string()),
        Some(&name_attr("couch").encode()),
        "the opinion is durable in the referencing document, not in this one"
    );
}

#[test]
fn a_live_opinion_beats_an_override() {
    let mut state = referenced();
    state.install_reference_layer(&stated(
        prim(7),
        &[override_entry(
            prim(7),
            prim(1),
            NameAttr::NAME,
            &name_attr("room says"),
        )],
    ));
    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("script says")),
    );

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("script says"),
        "an override is durable, and everything durable loses to a live opinion"
    );
}

#[test]
fn an_override_the_referencing_document_dropped_stops_resolving() {
    let mut state = referenced();
    state.install_reference_layer(&stated(
        prim(7),
        &[override_entry(
            prim(7),
            prim(1),
            NameAttr::NAME,
            &name_attr("recoloured"),
        )],
    ));
    state.drain_events();

    state.install_reference_layer(&Layer::default());

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("couch"),
        "a layer is installed whole, so a dropped opinion falls back to the \
             document's own"
    );
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: Some(name_attr("couch")),
        }]
    );
}

#[test]
fn a_blocked_override_hides_a_prim_of_the_referenced_document() {
    let mut state = referenced();
    apply(&mut state, &[child_entry(prim(2), prim(1), 3)]);
    state.drain_events();

    state.install_reference_layer(&stated(
        prim(7),
        &[tombstone(
            LayerKey {
                target: prim(2),
                name:   ParentAttr::NAME,
            }
            .key(prim(7)),
            4,
        )],
    ));

    assert!(
        !state.is_realized(prim(2)),
        "a referencing document hides a prim it did not author by blocking \
             its parent"
    );
    assert!(
        state.entries().contains_key(&parent_key(prim(2))),
        "hiding it does not remove it: the prim is still the target's"
    );
}

#[test]
fn overrides_round_trip_through_the_entry_set() {
    let site = prim(7);
    let entry = override_entry(site, prim(1), NameAttr::NAME, &name_attr("recoloured"));

    let mut referencing = HsdState::new();
    apply(&mut referencing, &[root_entry(site, 1), entry.clone()]);
    let saved = referencing.entries();
    assert_eq!(
        saved.get(&entry.key),
        Some(&entry.value),
        "an override is authored content and is written back like any"
    );

    let mut reread = HsdState::new();
    apply(
        &mut reread,
        &saved
            .into_iter()
            .map(|(key, value)| Entry::new(key, value, 1))
            .collect::<Vec<_>>(),
    );

    let mut target = referenced();
    target.install_reference_layer(reread.reference_layer_for(site).expect("kept the override"));
    assert_eq!(name_of(&target, prim(1)).as_deref(), Some("recoloured"));
}

#[test]
fn an_override_whose_site_the_document_does_not_state_is_not_saved() {
    let mut referencing = HsdState::new();
    let entry = override_entry(prim(7), prim(1), NameAttr::NAME, &name_attr("recoloured"));
    apply(&mut referencing, std::slice::from_ref(&entry));

    assert!(
        !referencing.entries().contains_key(&entry.key),
        "an override rides on the prim that references its document; with \
             no such prim in the document there is nothing for it to ride"
    );
    assert!(
        referencing.reference_layer_for(prim(7)).is_some(),
        "it is still held, so it saves once the site prim is committed"
    );
}

#[test]
fn committing_to_an_override_answers_what_the_referencing_document_must_hold() {
    let site = prim(7);
    let mut target = referenced();
    let saved = target.entries();

    runtime_property(
        &mut target,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("recoloured")),
    );
    target.drain_events();

    let entries = target.commit(
        CommitTarget::Override { site },
        &[(prim(1), NameAttr::NAME)],
    );

    assert_eq!(
        entries,
        vec![Entry::new(
            LayerKey {
                target: prim(1),
                name:   NameAttr::NAME,
            }
            .key(site),
            name_attr("recoloured").encode(),
            entries[0].timestamp,
        )],
        "the promotion answers the entry the referencing document has to hold"
    );
    assert_eq!(
        name_of(&target, prim(1)).as_deref(),
        Some("recoloured"),
        "and holds the same opinion locally, so nothing blinks while the \
             referencing document is written"
    );
    assert!(state_is_quiet(&mut target));
    assert_eq!(
        target.entries(),
        saved,
        "the target document was authored elsewhere and is untouched"
    );

    let mut referencing = HsdState::new();
    apply(&mut referencing, &[root_entry(site, 1)]);
    apply(&mut referencing, &entries);
    assert_eq!(
        referencing.entries().get(&entries[0].key),
        Some(&entries[0].value),
        "which is what makes the edit survive the session"
    );

    target.install_reference_layer(referencing.reference_layer_for(site).expect("holds it"));
    assert_eq!(name_of(&target, prim(1)).as_deref(), Some("recoloured"));
    assert!(
        state_is_quiet(&mut target),
        "the loop closes on the same value, so re-installing emits nothing"
    );
}

fn state_is_quiet(state: &mut HsdState) -> bool {
    state.drain_events().is_empty()
}

/// What a present peer says, arriving as a state message does.
fn session_property(state: &mut HsdState, prim: PrimId, value: &Value, at: u64) {
    state
        .apply_session(&Entry::new(
            key::Key::prop(prim, &NameAttr::NAME).to_string(),
            value.encode(),
            at,
        ))
        .expect("apply session");
}

#[test]
fn a_session_opinion_beats_a_script_computing_the_same_key() {
    let mut state = referenced();
    let saved = state.entries();
    runtime_property(
        &mut state,
        prim(1),
        NameAttr::NAME,
        Some(name_attr("animated")),
    );
    session_property(&mut state, prim(1), &name_attr("what the peer said"), 5);

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("what the peer said"),
        "a holder's broadcast must beat a bystander's local animation of \
             the same property, or the holder is not authoritative"
    );
    assert_eq!(
        state.entries(),
        saved,
        "and never reaches the save set on its own; only a commit puts it \
             there"
    );
}

#[test]
fn clearing_a_session_opinion_composes_the_layers_beneath_it_again() {
    let mut state = referenced();
    session_property(&mut state, prim(1), &name_attr("what the peer said"), 5);
    state.drain_events();

    state.clear_session(prim(1), &NameAttr::NAME);

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("couch"),
        "the absence of an opinion is not an opinion: what a peer leaving \
             takes with it falls through"
    );
    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  NameAttr::NAME,
            value: Some(name_attr("couch")),
        }]
    );
}

#[test]
fn clearing_a_key_no_peer_stated_changes_nothing() {
    let mut state = referenced();
    state.clear_session(prim(1), &NameAttr::NAME);

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("couch"));
    assert!(state_is_quiet(&mut state));
}

#[test]
fn keeping_a_session_opinion_promotes_it_into_the_document() {
    let mut state = referenced();
    session_property(&mut state, prim(1), &name_attr("a guest recoloured it"), 5);

    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(
        state
            .entries()
            .get(&key::Key::prop(prim(1), &NameAttr::NAME).to_string()),
        Some(&name_attr("a guest recoloured it").encode()),
        "keep is the owner's own commit over an opinion they did not \
             author, and this is the mechanism it rides on"
    );
    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("a guest recoloured it"),
        "what everyone sees does not move; only where the value lives"
    );
}

#[test]
fn a_promotion_lands_even_when_the_opinion_predates_the_document_value() {
    let mut state = HsdState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            attr_entry(prim(1), &NameAttr("couch".into()), 1_000),
        ],
    );
    session_property(&mut state, prim(1), &name_attr("recoloured"), 500);

    state.commit(CommitTarget::Document, &[(prim(1), NameAttr::NAME)]);

    assert_eq!(name_of(&state, prim(1)).as_deref(), Some("recoloured"));
    assert_eq!(
        state
            .entries()
            .get(&key::Key::prop(prim(1), &NameAttr::NAME).to_string()),
        Some(&name_attr("recoloured").encode()),
    );
}

#[test]
fn a_session_opinion_is_refused_when_an_older_one_arrives_late() {
    let mut state = referenced();
    session_property(&mut state, prim(1), &name_attr("newer"), 9);
    session_property(&mut state, prim(1), &name_attr("older"), 2);

    assert_eq!(
        name_of(&state, prim(1)).as_deref(),
        Some("newer"),
        "session opinions converge by stamp like every other layer's"
    );
}

#[test]
fn the_reference_target_round_trips_through_the_entry_set() {
    let site = prim(7);
    let target = DocId([7; 32]);

    let mut state = HsdState::new();
    apply(&mut state, &[root_entry(site, 1)]);
    state
        .set_attribute(site, &ReferenceAttr(target))
        .expect("reference");
    state.commit(CommitTarget::Document, &[(site, ReferenceAttr::NAME)]);

    let saved = state.entries();
    assert!(
        saved.contains_key(&key::Key::prop(site, &ReferenceAttr::NAME).to_string()),
        "the target is written at its structural key"
    );
    assert!(
        !saved.contains_key(&format!("p/{site}/ref/")),
        "nothing lives on the `ref/` spine"
    );

    let mut reread = HsdState::new();
    apply(
        &mut reread,
        &saved
            .into_iter()
            .map(|(key, value)| Entry::new(key, value, 1))
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        reread
            .attribute::<ReferenceAttr>(site)
            .expect("ref")
            .expect("decode"),
        ReferenceAttr(target),
        "and reads back as the ordinary `ref` property"
    );
}

#[test]
fn a_reference_layer_entry_round_trips_at_its_structural_key() {
    let site = prim(7);
    let entry = override_entry(site, prim(1), NameAttr::NAME, &name_attr("recoloured"));

    let mut referencing = HsdState::new();
    apply(&mut referencing, &[root_entry(site, 1), entry.clone()]);

    assert_eq!(
        referencing.entries().get(&entry.key),
        Some(&entry.value),
        "the entry set carries the reference layer under the site's prefix"
    );
}
