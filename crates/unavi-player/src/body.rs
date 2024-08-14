use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_tnua::prelude::*;
use bevy_vrm::{
    first_person::{FirstPersonFlag, SetupFirstPerson, RENDER_LAYERS},
    loader::Vrm,
    BoneName, VrmBundle,
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

#[derive(Component, Deref, DerefMut)]
pub struct EyeOffset(pub Vec3);

pub(crate) fn setup_first_person(
    avatars: Query<(Entity, &Handle<Vrm>, &Handle<Scene>), With<FirstPerson>>,
    mut commands: Commands,
    mut events: EventReader<AssetEvent<Vrm>>,
    mut scenes: ResMut<Assets<Scene>>,
    mut writer: EventWriter<SetupFirstPerson>,
    mut to_process: Local<Vec<AssetId<Vrm>>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            to_process.push(*id);
        }
    }

    let mut to_remove = None;

    for id in to_process.iter() {
        if to_remove.is_some() {
            break;
        }

        for (avatar_ent, handle_vrm, handle_scene) in avatars.iter() {
            if handle_vrm.id() != *id {
                continue;
            }

            writer.send(SetupFirstPerson(avatar_ent));

            let Some(scene) = scenes.get_mut(handle_scene) else {
                continue;
            };

            let mut bones = scene.world.query::<(Entity, &BoneName)>();

            let mut left_eye = None;
            let mut right_eye = None;

            for (bone_ent, bone_name) in bones.iter(&scene.world) {
                if *bone_name == BoneName::LeftEye {
                    left_eye = Some(bone_ent);
                }
                if *bone_name == BoneName::RightEye {
                    right_eye = Some(bone_ent);
                }
            }

            let mut offset = Vec3::default();

            if left_eye.is_some() && right_eye.is_some() {
                let left_tr = scene
                    .world
                    .entity(left_eye.unwrap())
                    .get::<Transform>()
                    .unwrap();
                let right_tr = scene
                    .world
                    .entity(right_eye.unwrap())
                    .get::<Transform>()
                    .unwrap();

                offset = left_tr.translation + right_tr.translation;
                offset /= 2.0;
            }

            commands.entity(avatar_ent).insert(EyeOffset(offset));

            to_remove = Some(id);
        }
    }

    if let Some(to_remove) = to_remove {
        let i = to_process.iter().position(|n| n == to_remove).unwrap();
        to_process.remove(i);
    }
}

#[derive(Component)]
pub struct AvatarHead(pub Entity);

pub(crate) fn set_avatar_head(
    avatars: Query<Entity, (With<EyeOffset>, Without<AvatarHead>)>,
    bones: Query<(Entity, &BoneName)>,
    mut commands: Commands,
    parents: Query<&Parent>,
) {
    for avatar_ent in avatars.iter() {
        for (bone_ent, bone_name) in bones.iter() {
            if *bone_name != BoneName::Head {
                continue;
            }

            if is_child(bone_ent, avatar_ent, &parents) {
                commands.entity(avatar_ent).insert(AvatarHead(bone_ent));
                break;
            }
        }
    }
}

/// Walks up the parent tree, searching for a specific Entity.
fn is_child(target_child: Entity, target_parent: Entity, parents: &Query<&Parent>) -> bool {
    if target_child == target_parent {
        true
    } else if let Ok(parent) = parents.get(target_child) {
        is_child(parent.get(), target_parent, parents)
    } else {
        false
    }
}

#[derive(Component)]
pub struct BaseRotation(pub Quat);

pub(crate) fn rotate_avatar_head(
    avatars: Query<&AvatarHead>,
    cameras: Query<&Transform, With<PlayerCamera>>,
    mut bones: Query<
        (&mut Transform, Option<&BaseRotation>),
        (With<BoneName>, Without<PlayerCamera>),
    >,
    mut commands: Commands,
) {
    for head in avatars.iter() {
        let Ok((mut head_tr, base)) = bones.get_mut(head.0) else {
            continue;
        };

        let Some(base) = base else {
            commands
                .entity(head.0)
                .insert(BaseRotation(head_tr.rotation));
            continue;
        };

        let camera_tr = cameras.single();
        let new_rot = base.0 * camera_tr.rotation;

        head_tr.rotation = new_rot;
    }
}
