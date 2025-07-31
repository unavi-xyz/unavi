use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired::math::types::{Vec2, Vec3};
use wired_ecs::{
    App, Component,
    param::Query,
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
        app.add_system(Schedule::Startup, startup_system);
        app.add_system(Schedule::Startup, point_system);
        // app.add_system(Schedule::Startup, multi_system);

        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

fn startup_system() {
    println!("running on startup!");
}

#[derive(Component, Clone, Copy, Debug)]
struct MyPoint {
    x: f32,
    y: f32,
}

fn point_system(points: Query<MyPoint>) {
    println!("point system");

    for point in &points.items {
        println!("point: {point:?}");
    }
}

fn multi_system(a: Query<Vec2>, b: Query<Vec3>) {}

export!(World);
