use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired::math::types::{Vec2, Vec3};
use wired_ecs::{
    App, Param,
    types::{ParamData, SystemId},
};

wit_bindgen::generate!({
    generate_all,
    additional_derives: [wired_ecs::wired_ecs_derive::Param],
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
        println!("new");

        let k = Vec3::param_key();
        let t = Vec3::param_type();
        println!("vec3:\n{k}\n{t:?}");

        let v = Vec3 {
            x: 1.2,
            y: 3.0,
            z: 4.567,
        };
        let v_bytes = v.to_bytes();
        let v2 = Vec3::from_bytes(&v_bytes);
        println!("v: {v:?}\nv2: {v2:?}");

        let mut app = App::default();
        app.add_system(wired_ecs::types::Schedule::Startup, startup_system);

        Self { app }
    }

    fn exec_system(&self, id: SystemId, params: Vec<ParamData>) {
        println!("exec system {id}");
        self.app.exec_system(id, params);
    }
}

fn startup_system() {
    println!("running on startup!");
}

#[derive(Debug)]
struct MyPoint {
    x: f32,
    y: f32,
}

fn point_system(points: Vec<&Vec2>) {
    for point in points {
        println!("point: {point:?}");
    }
}

export!(World);
