use bevy::prelude::*;
use unavi_player::LocalPlayer;

pub fn publish_user_transforms(player: Query<Entity, With<LocalPlayer>>) {
    let Some(_player) = player.iter().next() else {
        return;
    };
}
