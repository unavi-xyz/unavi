use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Update, update_1)
        .add_system(Schedule::Update, update_2)
        .add_system(Schedule::Update, update_3)
        .order_systems(update_1, SystemOrder::Before, update_2)
        .order_systems(update_3, SystemOrder::After, update_2);
}

fn startup() {
    println!("startup_system");
}

fn update_1() {
    println!("update_1");
}

fn update_2() {
    println!("update_2");
}

fn update_3() {
    println!("update_3");
}
