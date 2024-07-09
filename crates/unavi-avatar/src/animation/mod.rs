use bevy::prelude::*;

use crate::AvatarAnimations;

mod mixamo;

pub fn apply_avatar_animations(
    avatars: Query<&AvatarAnimations>,
    clips: Res<Assets<AnimationClip>>,
) {
    for animations in avatars.iter() {
        let _walk = match clips.get(animations.walk.id()) {
            Some(a) => a,
            None => continue,
        };

        // TODO: Update to Bevy 0.14 to continue work on this, animations got change.
    }
}
