use bevy::prelude::*;
use unavi_avatar::animation::{load::AvatarAnimationNodes, AnimationName, TargetAnimationWeights};

#[derive(Component, Default, Deref, DerefMut)]
pub struct PlayerMenuOpen(pub bool);

pub(crate) fn play_menu_animation(
    avatars: Query<Entity, With<AvatarAnimationNodes>>,
    mut animation_players: Query<(&mut TargetAnimationWeights, &Parent)>,
    players: Query<(&PlayerMenuOpen, &Children), Changed<PlayerMenuOpen>>,
) {
    for (open, children) in players.iter() {
        for child in children.iter() {
            if let Ok(avatar_ent) = avatars.get(*child) {
                for (mut targets, parent) in animation_players.iter_mut() {
                    if parent.get() != avatar_ent {
                        continue;
                    }

                    if **open {
                        targets.insert(AnimationName::Menu, 1.0);
                    } else {
                        targets.remove(&AnimationName::Menu);
                    }
                }
            }
        }
    }
}
