use bevy::prelude::*;
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::{
    attributes::portal::LinkId,
    id::{
        DocId,
        PrimId,
    },
    state::HsdState,
};

use super::{
    LinkedPortal,
    pair_links,
    resolve_destinations,
};
use crate::portal::{
    PortalHome,
    PortalLink,
    PortalTargetDoc,
};

const SPACE_A: DocId = DocId([1; 32]);
const SPACE_B: DocId = DocId([2; 32]);
const SPACE_C: DocId = DocId([3; 32]);
const LINK: LinkId = LinkId([9; 16]);

fn linked(index: u32, home: DocId, target: DocId, prim: u8) -> LinkedPortal {
    LinkedPortal {
        entity: Entity::from_raw_u32(index).expect("entity index"),
        key: (home, PrimId([prim; 16])),
        home,
        target,
        link: LINK,
    }
}

#[test]
fn test_halves_pair_with_each_other() {
    let near = linked(1, SPACE_A, SPACE_B, 1);
    let far = linked(2, SPACE_B, SPACE_A, 1);

    let pairs = pair_links(&[near, far]);

    assert_eq!(pairs.get(&near.entity), Some(&far.entity));
    assert_eq!(pairs.get(&far.entity), Some(&near.entity));
}

#[test]
fn test_half_aimed_elsewhere_is_not_a_partner() {
    let near = linked(1, SPACE_A, SPACE_B, 1);
    let stray = linked(2, SPACE_B, SPACE_C, 1);

    assert!(pair_links(&[near, stray]).is_empty());
}

#[test]
fn test_different_links_do_not_pair() {
    let near = linked(1, SPACE_A, SPACE_B, 1);
    let mut far = linked(2, SPACE_B, SPACE_A, 1);
    far.link = LinkId([8; 16]);

    assert!(pair_links(&[near, far]).is_empty());
}

#[test]
fn test_lowest_key_wins_regardless_of_order() {
    let near = linked(1, SPACE_A, SPACE_B, 1);
    let high = linked(2, SPACE_B, SPACE_A, 7);
    let low = linked(3, SPACE_B, SPACE_A, 3);

    for portals in [[near, high, low], [low, high, near]] {
        assert_eq!(pair_links(&portals).get(&near.entity), Some(&low.entity));
    }
}

struct Fixture {
    app:     App,
    space_b: Entity,
}

fn spawn_doc(app: &mut App, id: DocId) -> Entity {
    app.world_mut()
        .spawn((Hsd::new(HsdState::new()), HsdDocId(id)))
        .id()
}

fn spawn_portal(
    app: &mut App,
    doc: Entity,
    prim: u8,
    home: DocId,
    target: DocId,
    link: Option<LinkId>,
) -> Entity {
    let mut portal = app.world_mut().spawn((
        Prim(PrimId([prim; 16])),
        PrimOf(doc),
        PortalHome(home),
        PortalTargetDoc(target),
    ));
    if let Some(link) = link {
        portal.insert(PortalLink(link));
    }
    portal.id()
}

fn fixture() -> Fixture {
    let mut app = App::new();
    app.add_systems(Update, resolve_destinations);
    spawn_doc(&mut app, SPACE_A);
    let space_b = spawn_doc(&mut app, SPACE_B);
    Fixture { app, space_b }
}

fn destination(app: &App, portal: Entity) -> Option<Entity> {
    app.world().get::<super::Destination>(portal).map(|d| d.0)
}

#[test]
fn test_one_way_glues_to_target_root() {
    let Fixture { mut app, space_b } = fixture();
    let doc = spawn_doc(&mut app, DocId([4; 32]));
    let portal = spawn_portal(&mut app, doc, 1, SPACE_A, SPACE_B, None);

    app.update();

    assert_eq!(destination(&app, portal), Some(space_b));
}

#[test]
fn test_unpaired_link_glues_to_target_root() {
    let Fixture { mut app, space_b } = fixture();
    let doc = spawn_doc(&mut app, DocId([4; 32]));
    let portal = spawn_portal(&mut app, doc, 1, SPACE_A, SPACE_B, Some(LINK));

    app.update();

    assert_eq!(destination(&app, portal), Some(space_b));
}

#[test]
fn test_two_way_glues_portals_to_each_other() {
    let Fixture { mut app, .. } = fixture();
    let near_doc = spawn_doc(&mut app, DocId([4; 32]));
    let far_doc = spawn_doc(&mut app, DocId([5; 32]));
    let near = spawn_portal(&mut app, near_doc, 1, SPACE_A, SPACE_B, Some(LINK));
    let far = spawn_portal(&mut app, far_doc, 1, SPACE_B, SPACE_A, Some(LINK));

    app.update();

    assert_eq!(destination(&app, near), Some(far));
    assert_eq!(destination(&app, far), Some(near));
}
