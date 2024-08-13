use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_tnua::prelude::*;
use bevy_vrm::{
    first_person::{FirstPersonFlag, SetupFirstPerson, RENDER_LAYERS},
    loader::Vrm,
    VrmBundle,
};
use unavi_avatar::{
    default_character_animations, default_vrm, AvatarBundle, AverageVelocity, FallbackAvatar,
};
use unavi_constants::layers::LOCAL_PLAYER_LAYER;

use crate::{controls::InputState, menu::PlayerMenuOpen};

#[derive(Component)]
pub struct Player {
    pub input: InputState,
    pub jump_height: f32,
    pub speed: f32,
    pub velocity: Vec3,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            input: InputState::default(),
            jump_height: 2.0,
            speed: 7.0,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
pub struct PlayerCamera;

pub const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_WIDTH: f32 = 0.5;
pub const SPAWN: Vec3 = Vec3::new(0.0, PLAYER_HEIGHT * 2.0, 0.0);

pub(crate) fn spawn_player(asset_server: Res<AssetServer>, mut commands: Commands) {
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
            PlayerMenuOpen::default(),
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
            FirstPerson,
        ))
        .id();

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, (PLAYER_HEIGHT / 2.0) * 0.85, 0.0),
                ..default()
            },
            PlayerCamera,
            RenderLayers::layer(0).union(&RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
        ))
        .id();

    commands.entity(body).push_children(&[avatar, camera]);
}

#[derive(Component)]
pub struct FirstPerson;

pub(crate) fn setup_first_person(
    mut events: EventReader<AssetEvent<Vrm>>,
    mut writer: EventWriter<SetupFirstPerson>,
    players: Query<(Entity, &Handle<Vrm>), With<FirstPerson>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (ent, handle) in players.iter() {
                if handle.id() == *id {
                    writer.send(SetupFirstPerson(ent));
                }
            }
        }
    }
}
