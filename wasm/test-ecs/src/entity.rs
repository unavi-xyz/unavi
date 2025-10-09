use std::sync::{
    LazyLock,
    atomic::{AtomicU64, Ordering},
};

use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Update, test_entity);
}

static ID: LazyLock<AtomicU64> = LazyLock::new(AtomicU64::default);

#[derive(Decode, Encode, Component, Clone)]
struct EntMarker;

fn startup(commands: Commands) {
    let a = commands.spawn();
    a.insert(EntMarker);

    ID.store(a.id(), Ordering::Release);
}

fn test_entity(ents: Query<Entity, With<EntMarker>>) {
    assert_eq!(ents.len(), 1);

    let a = ents.iter().next().unwrap();
    let id = ID.load(Ordering::Acquire);
    assert_ne!(id, 0);
    assert_eq!(id, a.id());
}
