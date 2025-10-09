use wired_ecs::prelude::*;

use crate::wired::scene::types::Name;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup);
}

const NAME: &str = "my-comp-a";

fn startup(commands: Commands) {
    let a = commands.spawn();
    a.insert(Name {
        value: NAME.to_string(),
    });
}
