use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_vrm::VrmBundle;
use controls::{InputState, PitchTag, YawTag};
use unavi_avatar::{
    default_character_animations, default_vrm, AvatarBundle, AvatarPlugin, AverageVelocity,
    FallbackAvatar,
};
use unavi_constants::layers::LOCAL_PLAYER_LAYER;

mod controls;
mod input;
mod look;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AvatarPlugin,
            TnuaAvian3dPlugin::default(),
            TnuaControllerPlugin::default(),
        ))
        .insert_resource(input::InputMap::default())
        .add_event::<look::YawEvent>()
        .add_event::<look::PitchEvent>()
        .init_resource::<look::LookDirection>()
        .add_systems(Startup, spawn_player)
        .add_systems(
            Update,
            (
                input::handle_raycast_input,
                look::grab_mouse,
                (controls::void_teleport, input::read_keyboard_input).before(controls::move_player),
                (
                    look::read_mouse_input,
                    (controls::apply_yaw, controls::apply_pitch),
                    controls::move_player,
                )
                    .chain(),
            ),
        );
    }
}

#[derive(Component)]
pub struct Player {
    pub input: InputState,
    pub jump_height: f32,
    pub speed: f32,
    pub sprint_speed: f32,
    pub velocity: Vec3,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            input: InputState::default(),
            jump_height: 2.0,
            speed: 7.0,
            sprint_speed: 10.0,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
pub struct PlayerCamera;

const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_WIDTH: f32 = 0.5;
const SPAWN: Vec3 = Vec3::new(0.0, PLAYER_HEIGHT * 2.0, 0.0);

fn spawn_player(asset_server: Res<AssetServer>, mut commands: Commands) {
    let animations = default_character_animations(&asset_server);

    let body = commands
        .spawn((
            Collider::capsule(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT - PLAYER_WIDTH),
            CollisionLayers {
                memberships: LOCAL_PLAYER_LAYER,
                ..default()
            },
            LinearVelocity::default(),
            Player::default(),
            RigidBody::Dynamic,
            TnuaControllerBundle::default(),
            SpatialBundle {
                global_transform: GlobalTransform::from_translation(SPAWN),
                ..default()
            },
        ))
        .id();

    let avatar = commands
        .spawn((
            AvatarBundle {
                animations,
                fallback: FallbackAvatar,
                velocity: AverageVelocity {
                    target: Some(body),
                    ..default()
                },
            },
            VrmBundle {
                scene_bundle: SceneBundle {
                    transform: Transform::from_xyz(0.0, -PLAYER_HEIGHT / 2.0, 0.0),
                    ..default()
                },
                vrm: default_vrm(&asset_server),
                ..default()
            },
        ))
        .id();

    let yaw = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(
                0.0,
                (PLAYER_HEIGHT / 2.0) * 0.85,
                0.0,
            )),
            YawTag,
        ))
        .id();
    let pitch = commands.spawn((TransformBundle::default(), PitchTag)).id();
    let camera = commands
        .spawn((Camera3dBundle::default(), PlayerCamera))
        .id();

    commands.entity(body).push_children(&[avatar, yaw]);
    commands.entity(yaw).push_children(&[pitch]);
    commands.entity(pitch).push_children(&[camera]);
}
