use std::sync::{
    LazyLock,
    atomic::{AtomicUsize, Ordering},
};

use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::prelude::*;

wit_bindgen::generate!({
    generate_all,
    additional_derives: [wired_ecs::Component],
    with: {
        "wired:ecs/types": wired_ecs::types,
    },
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    app: App,
}

impl GuestScript for Script {
    fn new() -> Self {
        let mut app = App::default();
        app.add_system(Schedule::Startup, startup_system)
            .add_system(Schedule::Update, update_system);

        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct MyPoint {
    x: usize,
    y: usize,
}

const X1: usize = 1;
const Y1: usize = 2;
const X2: usize = 3;
const Y2: usize = 4;

fn startup_system(commands: Commands) {
    println!("startup_system");

    let _ = commands.spawn();

    commands.spawn().insert(MyPoint { x: X1, y: Y1 });
    commands.spawn().insert(MyPoint { x: X2, y: Y2 });
}

static COUNT: LazyLock<AtomicUsize> = LazyLock::new(AtomicUsize::default);

fn update_system(mut points: Query<&mut MyPoint>) {
    println!("update_system");

    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let x1 = X1 + count;
    let y1 = Y1 + count;
    let x2 = X2 + count;
    let y2 = Y2 + count;

    assert_eq!(points.len(), 2);

    let mut iter_ref = points.iter();
    let a = iter_ref.next().unwrap();
    assert_eq!(a.x, x1);
    assert_eq!(a.y, y1);
    let b = iter_ref.next().unwrap();
    assert_eq!(b.x, x2);
    assert_eq!(b.y, y2);

    let mut iter_mut = points.iter_mut();
    let a = iter_mut.next().unwrap();
    assert_eq!(a.x, x1);
    assert_eq!(a.y, y1);
    a.x += 1;
    a.y += 1;
    let b = iter_mut.next().unwrap();
    assert_eq!(b.x, x2);
    assert_eq!(b.y, y2);
    b.x += 1;
    b.y += 1;
}

export!(World);
