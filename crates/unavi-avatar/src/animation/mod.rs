use bevy::prelude::*;
use bevy_vrm::HumanoidBones;

use crate::AvatarAnimations;

mod mixamo;

pub fn apply_avatar_animations(
    avatars: Query<(&AvatarAnimations, &HumanoidBones)>,
    clips: Res<Assets<AnimationClip>>,
) {
    for (handle, _bones) in avatars.iter() {
        let _walk = match clips.get(handle.walk.id()) {
            Some(a) => a,
            None => continue,
        };

        // TODO: Update to Bevy 0.14 to continue work on this, current API makes things difficult.
    }
}
