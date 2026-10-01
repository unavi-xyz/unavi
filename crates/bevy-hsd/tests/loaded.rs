use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_hsd::{
    document::HsdDocId,
    loaded::HsdLoaded,
};
use bevy_iroh::store::LocalStore;
use bytemuck::cast_slice;
use hsd::{
    attributes::{
        collider::ColliderKind,
        parent::ParentAttr,
        reference::ReferenceAttr,
    },
    id::{
        DocId,
        PrimId,
    },
    key,
    property::{
        Payload,
        Property,
        value::Value,
    },
    state::entry::Entry,
};
use rstest::rstest;
use tracing_test::traced_test;
use unavi_util::UtilPlugin;

use crate::common::*;

mod common;

const VERTS: [[f32; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
const IDXS: [[u32; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];

/// A chain of real documents long enough that its last hop sits past
/// `bevy_hsd::reference::MAX_REF_DEPTH` (8).
const CHAIN_LEN: usize = 8;
const ROOT_PRIM: PrimId = PrimId([1; 16]);
/// Never opened. The depth cap refuses this hop before any open is attempted.
const DANGLING: DocId = DocId([99; 32]);

fn has<C: Component>(world: &mut World) -> bool {
    world.query::<&C>().iter(world).next().is_some()
}

fn root_entry(prim: PrimId) -> Entry {
    Entry::new(
        key::Key::prop(prim, &ParentAttr::NAME).to_string(),
        ParentAttr::to_wire(Some(ParentAttr::Root)),
        1,
    )
}

fn reference_entry(prim: PrimId, target: DocId) -> Entry {
    let value = Value::Attribute(ReferenceAttr(target).encode().expect("encode").into()).encode();
    Entry::new(
        key::Key::prop(prim, &ReferenceAttr::NAME).to_string(),
        value,
        1,
    )
}

#[traced_test]
#[rstest]
fn test_loaded_when_no_blob_work(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &ColliderKind::Cuboid {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    );

    ctx.tick_until(has::<HsdLoaded>);
}

/// A collider that never builds still settles to `HsdLoaded`. A broken build
/// clears the pending marker on the same attempt that fails it, rather than
/// blocking readiness forever.
#[traced_test]
#[rstest]
fn test_loaded_despite_a_broken_collider(mut ctx: TestContext) {
    let root = ctx.create_prim();

    // Garbage bytes for a trimesh. The collider cannot be built.
    ctx.set_attr(root, &ColliderKind::Trimesh);
    ctx.set_collider_vertices(root, b"not-vertices".to_vec());
    ctx.set_collider_indices(root, b"not-indices".to_vec());

    ctx.tick_until(has::<HsdLoaded>);

    let world = ctx.app.world_mut();
    assert!(
        !has::<Collider>(world),
        "the broken collider must never have built"
    );
}

/// An image that fails to decode still settles to `HsdLoaded`, rather than
/// blocking limbo exit on a decode that will never succeed.
#[traced_test]
#[rstest]
fn test_loaded_despite_an_undecodable_image(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_image_data(root, b"not a real image".to_vec());

    ctx.tick_until(has::<HsdLoaded>);
}

#[traced_test]
#[rstest]
fn test_loaded_after_collider_built(mut ctx: TestContext) {
    let vertices = cast_slice::<[f32; 3], u8>(&VERTS).to_vec();
    let indices = cast_slice::<[u32; 3], u8>(&IDXS).to_vec();

    let root = ctx.create_prim();
    ctx.set_attr(root, &ColliderKind::Trimesh);
    ctx.set_collider_vertices(root, vertices);
    ctx.set_collider_indices(root, indices);

    ctx.tick_until(has::<Collider>);
    ctx.tick_until(has::<HsdLoaded>);
}

/// A reference past the depth cap is refused rather than added to the scene.
/// The refusal must not block the referencing document's own `HsdLoaded`.
#[traced_test]
#[rstest]
fn test_loaded_despite_a_reference_past_the_depth_cap(mut ctx: TestContext) {
    let backing = Backing::new();
    ctx.app.add_plugins(UtilPlugin);
    ctx.app.world_mut().spawn(LocalStore(backing.store()));
    ctx.app
        .world_mut()
        .entity_mut(ctx.doc)
        .insert(HsdDocId(DocId([0; 32])));

    let chain: Vec<_> = (0..CHAIN_LEN).map(|_| backing.document()).collect();
    for (i, doc) in chain.iter().enumerate() {
        let target = chain
            .get(i + 1)
            .map_or(DANGLING, |next| DocId(*next.id().as_bytes()));
        set_entry(doc, &root_entry(ROOT_PRIM));
        set_entry(doc, &reference_entry(ROOT_PRIM, target));
    }

    let root = ctx.create_prim();
    let first = DocId(*chain[0].id().as_bytes());
    ctx.set_attr(root, &ReferenceAttr(first));

    let doc = ctx.doc;
    ctx.tick_until(move |world| world.get::<HsdLoaded>(doc).is_some());
}
