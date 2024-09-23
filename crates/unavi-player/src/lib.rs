use avian3d::prelude::CollisionLayers;
use bevy::prelude::*;
use bevy_vr_controller::{
    animation::{defaults::default_character_animations, load::AvatarAnimation, AnimationName},
    player::{PlayerSettings, SpawnedPlayer},
    VrControllerPlugin,
};
use unavi_constants::player::{layers::LAYER_LOCAL_PLAYER, DEFAULT_VRM, LOCAL_PLAYER_ID};

pub mod id;
mod input;
mod menu;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrControllerPlugin)
            .init_resource::<input::UnaviInputMap>()
            .init_state::<menu::MenuState>()
            .add_systems(Startup, spawn_player)
            .add_systems(OnEnter(menu::MenuState::Open), menu::open_menu)
            .add_systems(OnExit(menu::MenuState::Open), menu::close_menu)
            .add_systems(
                Update,
                (
                    id::id_player_bones,
                    input::read_keyboard_input
                        .after(bevy_vr_controller::input::read_keyboard_input),
                ),
            );
    }
}

fn spawn_player(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut animations = default_character_animations(&asset_server);

    animations.0.insert(
        AnimationName::Other(menu::MENU_ANIMATION),
        AvatarAnimation {
            clip: asset_server.load("models/idle-menu.glb#Animation0"),
            gltf: asset_server.load("models/idle-menu.glb"),
        },
    );

    let SpawnedPlayer { avatar, body, .. } = PlayerSettings {
        animations: Some(animations),
        void_level: Some(-20.0),
        vrm: Some(asset_server.load(DEFAULT_VRM)),
        ..default()
    }
    .spawn(&mut commands);

    commands
        .entity(avatar)
        .insert(id::PlayerId(LOCAL_PLAYER_ID));
    commands.entity(body).insert((
        id::PlayerId(LOCAL_PLAYER_ID),
        CollisionLayers {
            memberships: LAYER_LOCAL_PLAYER,
            ..default()
        },
    ));
}
