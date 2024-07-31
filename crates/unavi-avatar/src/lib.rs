use animation::{AvatarAnimations, TargetAnimation};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_vrm::VrmPlugins;
use velocity::AverageVelocity;

pub mod animation;
mod fallback;
mod velocity;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugins)
            .add_systems(Startup, fallback::init_fallback_assets)
            .add_systems(
                Update,
                (
                    animation::init_animation_transitions,
                    animation::load::load_animation_nodes,
                    animation::play_avatar_animations,
                    animation::play_target_animation,
                    fallback::despawn_fallback_children,
                    fallback::remove_fallback_avatar,
                    fallback::spawn_fallback_children,
                    velocity::calc_average_velocity,
                ),
            );
    }
}

#[derive(Bundle)]
pub struct AvatarBundle {
    pub animations: AvatarAnimations,
    pub collider: Collider,
    pub fallback: FallbackAvatar,
    pub rigid_body: RigidBody,
    pub spatial: SpatialBundle,
    pub target_animation: TargetAnimation,
    pub velocity: AverageVelocity,
}

const PLAYER_RADIUS: f32 = 0.2;
const PLAYER_HEIGHT: f32 = 0.8;

impl AvatarBundle {
    pub fn new(animations: AvatarAnimations) -> Self {
        Self {
            animations,
            collider: Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT),
            fallback: FallbackAvatar,
            rigid_body: RigidBody::Kinematic,
            spatial: SpatialBundle::default(),
            target_animation: TargetAnimation::default(),
            velocity: AverageVelocity::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct FallbackAvatar;
