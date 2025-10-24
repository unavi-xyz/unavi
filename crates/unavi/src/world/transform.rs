use bevy::prelude::*;
use unavi_player::Player;

pub fn publish_user_transforms(body: Query<Entity, With<Player>>) {
    let Some(_body) = body.iter().next() else {
        return;
    };
}
