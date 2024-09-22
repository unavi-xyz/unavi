use bevy::{ecs::system::RunSystemOnce, prelude::*};
use unavi_avatar::animation::{AnimationName, AvatarAnimationNodes, TargetAnimationWeights};

use crate::LocalPlayer;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuState {
    #[default]
    Closed,
    Open,
}

pub(crate) fn open_menu(world: &mut World) {
    world.run_system_once_with(true, set_menu_animation);
}

pub(crate) fn close_menu(world: &mut World) {
    world.run_system_once_with(false, set_menu_animation);
}

fn set_menu_animation(
    open: In<bool>,
    avatars: Query<Entity, With<AvatarAnimationNodes>>,
    mut animation_players: Query<(&mut TargetAnimationWeights, &Parent)>,
    players: Query<&Children, With<LocalPlayer>>,
) {
    let Ok(children) = players.get_single() else {
        return;
    };

    for child in children.iter() {
        if let Ok(avatar_ent) = avatars.get(*child) {
            for (mut targets, parent) in animation_players.iter_mut() {
                if parent.get() != avatar_ent {
                    continue;
                }

                if *open {
                    targets.insert(AnimationName::Menu, 1.0);
                } else {
                    targets.remove(&AnimationName::Menu);
                }
            }
        }
    }
}
