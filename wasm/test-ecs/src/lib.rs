use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::{
    App, Component,
    param::{Commands, Query},
    types::{ParamData, Schedule, SystemId},
};

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
            .add_system(Schedule::Update, single_system);
        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

fn startup_system(commands: Commands) {
    println!("startup_system");

    let _ = commands.spawn();

    let ent_a = commands.spawn();
    ent_a.insert(MyPoint { y: 1.0, x: 2.0 });

    let ent_b = commands.spawn();
    ent_b.insert(MyPoint { y: 3.0, x: 4.0 });
}

#[derive(Component, Clone, Copy, Debug)]
struct MyPoint {
    x: f32,
    y: f32,
}

fn single_system(points: Query<MyPoint>) {
    println!("single_system");

    assert_eq!(points.items.len(), 2);

    let a = &points.items[0];
    assert_eq!(a.x, 1.0);
    assert_eq!(a.y, 2.0);

    let b = &points.items[1];
    assert_eq!(b.x, 3.0);
    assert_eq!(b.y, 4.0);
}

export!(World);
