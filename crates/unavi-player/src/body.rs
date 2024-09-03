use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_tnua::prelude::*;
use bevy_vrm::{
    first_person::{FirstPersonFlag, SetupFirstPerson, RENDER_LAYERS},
    loader::Vrm,
    BoneName, VrmBundle,
};
use unavi_avatar::{
    default_character_animations, AvatarBundle, AverageVelocity, FallbackAvatar, DEFAULT_VRM,
};
use unavi_constants::player::{PLAYER_HEIGHT, PLAYER_WIDTH};
use unavi_scripting::api::wired::player::systems::{PlayerId, LOCAL_PLAYER_ID};

use crate::{controls::InputState, layers::LOCAL_PLAYER_LAYER};

#[derive(Component)]
pub struct LocalPlayer {
    pub input: InputState,
    pub jump_height: f32,
    pub speed: f32,
    pub velocity: Vec3,
}

impl Default for LocalPlayer {
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
            LocalPlayer::default(),
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
                vrm: asset_server.load(DEFAULT_VRM),
                ..default()
            },
            FirstPerson,
            PlayerId(LOCAL_PLAYER_ID),
        ))
        .id();

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, -PLAYER_HEIGHT / 2.0, 0.0),
                ..default()
            },
            PlayerCamera,
            RenderLayers::layer(0).union(&RENDER_LAYERS[&FirstPersonFlag::FirstPersonOnly]),
        ))
        .id();

    commands.entity(body).push_children(&[avatar, camera]);
}

pub(crate) fn id_player_bones(
    mut commands: Commands,
    new_bones: Query<Entity, Added<BoneName>>,
    parents: Query<&Parent>,
    player_ids: Query<&PlayerId>,
) {
    for ent in new_bones.iter() {
        if let Some(id) = find_player_id(ent, &parents, &player_ids) {
            commands.entity(ent).insert(PlayerId(id));
        }
    }
}

fn find_player_id(
    ent: Entity,
    parents: &Query<&Parent>,
    players: &Query<&PlayerId>,
) -> Option<usize> {
    if let Ok(parent) = parents.get(ent) {
        if let Ok(id) = players.get(**parent) {
            Some(id.0)
        } else {
            find_player_id(**parent, parents, players)
        }
    } else {
        None
    }
}

#[derive(Component)]
pub struct FirstPerson;

#[derive(Component, Deref, DerefMut)]
pub struct EyeOffset(pub Vec3);

pub(crate) fn setup_first_person(
    avatars: Query<(Entity, &Handle<Vrm>), With<FirstPerson>>,
    mut events: EventReader<AssetEvent<Vrm>>,
    mut writer: EventWriter<SetupFirstPerson>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (avatar_ent, handle_vrm) in avatars.iter() {
                if handle_vrm.id() != *id {
                    continue;
                }

                writer.send(SetupFirstPerson(avatar_ent));
            }
        }
    }
}

pub(crate) fn calc_eye_offset(
    mut commands: Commands,
    mut scene_assets: ResMut<Assets<Scene>>,
    mut to_calc: Local<Vec<Entity>>,
    mut to_remove: Local<Vec<Entity>>,
    new_scenes: Query<Entity, (With<FirstPerson>, Added<Handle<Scene>>)>,
    scenes: Query<&Handle<Scene>>,
) {
    for ent in new_scenes.iter() {
        to_calc.push(ent);
    }

    for ent in to_calc.iter() {
        let Ok(handle_scene) = scenes.get(*ent) else {
            continue;
        };

        let Some(scene) = scene_assets.get_mut(handle_scene) else {
            continue;
        };

        let mut bones = scene.world.query::<(Entity, &BoneName)>();

        let mut left_eye = None;
        let mut right_eye = None;
        let mut head = None;

        for (bone_ent, bone_name) in bones.iter(&scene.world) {
            if *bone_name == BoneName::LeftEye {
                left_eye = Some(bone_ent);
            }
            if *bone_name == BoneName::RightEye {
                right_eye = Some(bone_ent);
            }
            if *bone_name == BoneName::Head {
                head = Some(bone_ent);
            }
        }

        let mut offset = if left_eye.is_some() && right_eye.is_some() {
            let left_tr = scene
                .world
                .entity(left_eye.unwrap())
                .get::<GlobalTransform>()
                .unwrap();
            let right_tr = scene
                .world
                .entity(right_eye.unwrap())
                .get::<GlobalTransform>()
                .unwrap();

            (left_tr.translation() + right_tr.translation()) / 2.0
        } else {
            let head_tr = scene
                .world
                .entity(head.unwrap())
                .get::<GlobalTransform>()
                .unwrap();

            head_tr.translation()
        };

        offset.y += 0.08;
        offset.z -= 0.08;

        commands.entity(*ent).insert(EyeOffset(offset));

        to_remove.push(*ent);
    }

    for ent in to_remove.iter() {
        let new_calc = to_calc
            .iter()
            .copied()
            .filter(|x| x == ent)
            .collect::<Vec<_>>();
        *to_calc = new_calc;
    }

    to_remove.clear();
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
    avatars: Query<(&AvatarHead, &EyeOffset)>,
    mut bones: Query<
        (&mut Transform, Option<&BaseRotation>),
        (With<BoneName>, Without<PlayerCamera>),
    >,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    mut commands: Commands,
) {
    for (head, offset) in avatars.iter() {
        let Ok((mut head_tr, base)) = bones.get_mut(head.0) else {
            continue;
        };

        let Some(base) = base else {
            commands
                .entity(head.0)
                .insert(BaseRotation(head_tr.rotation));
            continue;
        };

        let mut camera_tr = cameras.single_mut();
        camera_tr.translation = Vec3::new(0.0, -PLAYER_HEIGHT / 2.0, 0.0) + offset.0;

        let new_rot = base.0 * camera_tr.rotation;
        head_tr.rotation = new_rot;
    }
}
