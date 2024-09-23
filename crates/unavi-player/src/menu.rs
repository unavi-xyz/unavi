use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_vr_controller::{
    animation::{weights::TargetAnimationWeights, AnimationName, AvatarAnimationNodes},
    player::{CameraFreeLook, PlayerAvatar},
};

pub const MENU_ANIMATION: &str = "menu";

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
    avatars: Query<Entity, (With<AvatarAnimationNodes>, With<PlayerAvatar>)>,
    mut animation_players: Query<(&mut TargetAnimationWeights, &Parent)>,
    mut free_look: Query<&mut CameraFreeLook>,
) {
    for avatar_ent in avatars.iter() {
        for (mut targets, parent) in animation_players.iter_mut() {
            if parent.get() != avatar_ent {
                continue;
            }

            free_look.single_mut().0 = *open;

            if *open {
                targets.insert(AnimationName::Other(MENU_ANIMATION), 1.0);
            } else {
                targets.remove(&AnimationName::Other(MENU_ANIMATION));
            }
        }
    }
}
