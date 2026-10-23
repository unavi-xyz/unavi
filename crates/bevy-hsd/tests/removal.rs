use std::sync::{
    Mutex,
    Once,
};

use bevy::prelude::{
    Entity,
    With,
};
use bevy_hsd::prim::{
    Prim,
    PrimIndex,
};
use rstest::rstest;

use crate::common::*;

mod common;

/// Bevy's error handler reports a command aimed at a dead entity through
/// `log`, which no subscriber in the test app reads.
struct Caught(Mutex<Vec<String>>);

static CAUGHT: Caught = Caught(Mutex::new(Vec::new()));

impl log::Log for Caught {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        self.0
            .lock()
            .expect("lock the log")
            .push(record.args().to_string());
    }

    fn flush(&self) {}
}

fn caught_logs() -> Vec<String> {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        log::set_logger(&CAUGHT).expect("install the test logger");
        log::set_max_level(log::LevelFilter::Warn);
    });
    std::mem::take(&mut *CAUGHT.0.lock().expect("lock the log"))
}

/// A removed prim is despawned with its descendants by the scene tree, so the
/// `Removed` event each descendant gets on its own must not despawn it again.
#[rstest]
fn removing_a_prim_despawns_its_descendants_once(mut ctx: TestContext) {
    let root = ctx.create_prim();
    let child = ctx.create_child(root);
    let _grandchild = ctx.create_child(child);
    let _sibling = ctx.create_child(root);
    ctx.app.update();
    let _ = caught_logs();

    // Every descendant's event is processed in the same drain, after the
    // root's despawn has already taken the tree with it.
    ctx.state.lock().expect("lock state").remove_prim(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let prims = world
        .query_filtered::<Entity, With<Prim>>()
        .iter(world)
        .collect::<Vec<_>>();
    assert!(
        prims.is_empty(),
        "descendants outlived the prim they hung under: {prims:?}"
    );

    let mut indices = world.query::<&PrimIndex>();
    for index in indices.iter(world) {
        assert_eq!(index.iter().count(), 0, "the index kept dead prims");
    }

    let twice = caught_logs()
        .into_iter()
        .filter(|line| line.contains("Entity despawned"))
        .collect::<Vec<_>>();
    assert!(twice.is_empty(), "despawned an entity twice: {twice:#?}");
}
