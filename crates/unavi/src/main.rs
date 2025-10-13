use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(unavi::UnaviPlugin);
    app.run();
}
