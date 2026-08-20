use std::collections::BTreeMap;

use crate::{
    attributes::{
        Attribute,
        material::{
            BINDING,
            MaterialAttr,
        },
        name::NameAttr,
        xform::XformAttr,
    },
    id::PrimId,
    key,
    property::{
        Parent,
        Property,
    },
    state::{
        MAX_PRIM_DEPTH,
        SceneState,
        entry::Entry,
        event::SceneEvent,
        prim::Origin,
    },
};

fn prim(n: u8) -> PrimId {
    PrimId([n; 16])
}

fn root_entry(id: PrimId, timestamp: u64) -> Entry {
    Entry::bytes(key::parent(id), Parent::Root.encode(), timestamp)
}

fn child_entry(id: PrimId, parent: PrimId, timestamp: u64) -> Entry {
    Entry::bytes(key::parent(id), Parent::Prim(parent).encode(), timestamp)
}

fn attr_entry<A: Attribute>(id: PrimId, value: &A, timestamp: u64) -> Entry {
    Entry::bytes(
        key::prop(id, A::KEY),
        Property::Attribute(value.encode().expect("encode")).encode(),
        timestamp,
    )
}

fn tombstone(key: String, timestamp: u64) -> Entry {
    Entry::bytes(key, Vec::new(), timestamp)
}

fn apply(state: &mut SceneState, entries: &[Entry]) {
    state.apply_all(entries).expect("apply");
}

fn shape(state: &SceneState) -> BTreeMap<PrimId, Option<PrimId>> {
    state
        .prims()
        .map(|prim| (prim, state.parent(prim)))
        .collect()
}

#[test]
fn realizes_a_root_and_its_child() {
    let mut state = SceneState::new();
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

    let mut forward = SceneState::new();
    apply(&mut forward, &entries);

    let mut reversed = SceneState::new();
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
    let mut state = SceneState::new();
    apply(&mut state, &[child_entry(prim(2), prim(1), 2)]);

    assert!(state.exists(prim(2)));
    assert!(!state.is_realized(prim(2)));
    assert_eq!(state.roots().len(), 0);
}

#[test]
fn an_orphan_realizes_with_its_properties_when_its_parent_arrives() {
    let mut state = SceneState::new();
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
        SceneEvent::Property { prim: p, name, .. } if *p == prim(2) && name == XformAttr::KEY
    )));
}

#[test]
fn a_property_on_an_unrealized_prim_emits_nothing() {
    let mut state = SceneState::new();
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

    let mut forward = SceneState::new();
    apply(&mut forward, &entries);

    let mut shuffled = SceneState::new();
    apply(
        &mut shuffled,
        &[entries[2].clone(), entries[0].clone(), entries[1].clone()],
    );

    assert_eq!(shape(&forward), shape(&shuffled));
    assert_eq!(forward.roots(), vec![prim(3)]);
    assert_eq!(forward.parent(prim(1)), Some(prim(3)));
    assert_eq!(forward.parent(prim(2)), Some(prim(1)));
}

#[test]
fn a_prim_hanging_off_a_cycle_is_realized_under_its_own_parent() {
    let mut state = SceneState::new();
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
fn a_cross_author_tombstone_removes_a_prim_written_by_someone_else() {
    let mut state = SceneState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            attr_entry(prim(2), &NameAttr("gone".into()), 3),
        ],
    );
    state.drain_events();

    apply(&mut state, &[tombstone(key::parent(prim(2)), 10)]);

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
    let mut state = SceneState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            child_entry(prim(3), prim(2), 3),
        ],
    );

    apply(&mut state, &[tombstone(key::parent(prim(2)), 10)]);
    assert!(!state.is_realized(prim(3)));
    assert!(state.exists(prim(3)));

    apply(&mut state, &[child_entry(prim(2), prim(1), 20)]);
    assert!(state.is_realized(prim(3)));
}

#[test]
fn an_older_entry_never_overwrites_a_newer_one() {
    let mut state = SceneState::new();
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

    let mut forward = SceneState::new();
    apply(
        &mut forward,
        &[root_entry(prim(1), 1), a.clone(), b.clone()],
    );

    let mut reversed = SceneState::new();
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
    let mut state = SceneState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            Entry::bytes(
                key::prop(prim(1), "shader_graph"),
                Property::Attribute(payload.clone()).encode(),
                2,
            ),
        ],
    );

    let entries = state.entries();
    let stored = entries
        .get(&key::prop(prim(1), "shader_graph"))
        .expect("entry");
    assert_eq!(stored, &Property::Attribute(payload).encode());
}

#[test]
fn relationships_and_attributes_share_one_namespace() {
    let mut state = SceneState::new();
    apply(
        &mut state,
        &[root_entry(prim(1), 1), root_entry(prim(2), 1)],
    );

    state
        .set_attribute(prim(1), &MaterialAttr::default())
        .expect("attribute");
    state
        .set_relationship(prim(1), BINDING, prim(2))
        .expect("relationship");

    let prim_state = state.get(prim(1)).expect("prim");
    assert!(
        prim_state
            .property("material")
            .expect("attr")
            .as_attribute()
            .is_some()
    );
    assert_eq!(
        prim_state.property(BINDING).expect("rel").as_relationship(),
        Some(prim(2))
    );
}

#[test]
fn script_created_prims_are_absent_from_the_save_set() {
    let mut state = SceneState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    let scratch = state.create_prim(Some(prim(1)));
    state
        .set_attribute(scratch, &NameAttr("transient".into()))
        .expect("attribute");

    assert!(state.is_realized(scratch));
    let entries = state.entries();
    assert!(entries.contains_key(&key::parent(prim(1))));
    assert!(!entries.contains_key(&key::parent(scratch)));
}

#[test]
fn a_document_prim_edited_by_a_script_stays_persistent() {
    let mut state = SceneState::new();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state
        .set_attribute(prim(1), &NameAttr("edited".into()))
        .expect("attribute");

    let entries = state.entries();
    assert!(entries.contains_key(&key::prop(prim(1), NameAttr::KEY)));
    assert_eq!(state.get(prim(1)).expect("prim").origin, Origin::Document);
}

#[test]
fn a_slot_is_tracked_as_inline_bytes() {
    let mut state = SceneState::new();
    let payload = vec![9; 1024];
    apply(&mut state, &[root_entry(prim(1), 1)]);
    state.drain_events();

    apply(
        &mut state,
        &[Entry::new(
            key::prop(prim(1), "mesh:POSITION"),
            payload.clone(),
            2,
        )],
    );

    assert_eq!(
        state.get(prim(1)).expect("prim").slot("mesh:POSITION"),
        Some(payload.as_slice())
    );
    assert!(state.drain_events().contains(&SceneEvent::Slot {
        prim:  prim(1),
        name:  "mesh:POSITION".into(),
        value: Some(payload),
    }));
}

#[test]
fn a_zero_size_slot_entry_reads_as_absence() {
    let mut state = SceneState::new();
    apply(
        &mut state,
        &[
            root_entry(prim(1), 1),
            Entry::new(key::prop(prim(1), "script"), vec![1; 64], 2),
            Entry::new(key::prop(prim(1), "script"), Vec::new(), 3),
        ],
    );

    assert_eq!(state.get(prim(1)).expect("prim").slot("script"), None);
}

#[test]
fn reparenting_emits_one_event_and_moves_the_subtree() {
    let mut state = SceneState::new();
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
    let mut state = SceneState::new();
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
        &[tombstone(key::prop(prim(1), XformAttr::KEY), 3)],
    );

    assert_eq!(
        state.drain_events(),
        vec![SceneEvent::Property {
            prim:  prim(1),
            name:  XformAttr::KEY.into(),
            value: None,
        }]
    );
}

#[test]
fn the_save_set_round_trips_through_a_fresh_state() {
    let mut original = SceneState::new();
    apply(
        &mut original,
        &[
            root_entry(prim(1), 1),
            child_entry(prim(2), prim(1), 2),
            attr_entry(prim(2), &NameAttr("kept".into()), 3),
            Entry::new(key::prop(prim(2), "script"), vec![7; 32], 4),
        ],
    );

    let mut restored = SceneState::new();
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

/// A chain deeper than the cap holds its tail rather than realizing it, so a
/// hostile document cannot force an unbounded walk or an unbounded ECS
/// hierarchy.
#[test]
fn nesting_past_the_depth_cap_is_not_realized() {
    let mut state = SceneState::new();

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
fn an_open_tick_withholds_its_own_writes() {
    let mut state = SceneState::new();
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
    let mut state = SceneState::new();
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
    let mut state = SceneState::new();
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
                if *p == prim(1) && name == XformAttr::KEY
        )),
        "the transform arrives in the same drain as the prim it belongs to"
    );
}

#[test]
fn boundaries_nest_so_two_writers_both_have_to_finish() {
    let mut state = SceneState::new();
    state.open_tick();
    state.open_tick();
    apply(&mut state, &[root_entry(prim(1), 1)]);

    state.close_tick();
    assert!(
        state.drain_events().is_empty(),
        "one writer finishing does not release another's partial work"
    );
    assert!(state.is_ticking());

    state.close_tick();
    assert!(!state.is_ticking());
    assert_ne!(state.drain_events().len(), 0);
}

#[test]
fn an_unmatched_close_does_not_underflow() {
    let mut state = SceneState::new();
    state.close_tick();
    assert!(!state.is_ticking());
    state.open_tick();
    apply(&mut state, &[root_entry(prim(1), 1)]);
    assert!(state.drain_events().is_empty(), "the boundary still holds");
    state.close_tick();
}

#[test]
fn a_consumer_attaching_mid_tick_gets_the_scene_as_it_stands() {
    let mut state = SceneState::new();
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
