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
            .add_system(Schedule::Update, read_system)
            .add_system(Schedule::Update, write_system);
        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct MyPoint {
    x: f32,
    y: f32,
}

fn startup_system(commands: Commands) {
    println!("startup_system");

    let _ = commands.spawn();

    commands.spawn().insert(MyPoint { x: 1.0, y: 2.0 });
    commands.spawn().insert(MyPoint { x: 3.0, y: 4.0 });
}

fn read_system(points: Query<&MyPoint>) {
    println!("read_system");

    assert_eq!(points.len(), 2);

    let mut iter_ref = points.iter();
    let a = iter_ref.next().unwrap();
    assert_eq!(a.x, 1.0);
    assert_eq!(a.y, 2.0);
    let b = iter_ref.next().unwrap();
    assert_eq!(b.x, 3.0);
    assert_eq!(b.y, 4.0);
}

fn write_system(mut points: Query<&mut MyPoint>) {
    println!("write_system");

    assert_eq!(points.len(), 2);

    let mut iter_ref = points.iter();
    let a = iter_ref.next().unwrap();
    assert_eq!(a.x, 1.0);
    assert_eq!(a.y, 2.0);
    let b = iter_ref.next().unwrap();
    assert_eq!(b.x, 3.0);
    assert_eq!(b.y, 4.0);

    let mut iter_mut = points.iter_mut();
    let a = iter_mut.next().unwrap();
    assert_eq!(a.x, 1.0);
    assert_eq!(a.y, 2.0);
    let b = iter_mut.next().unwrap();
    assert_eq!(b.x, 3.0);
    assert_eq!(b.y, 4.0);
}

export!(World);
