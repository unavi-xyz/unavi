use avian3d::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use unavi_scripting::api::wired::input::{
    bindings::types::InputAction,
    input_handler::{InputHandlerSender, ScriptInputEvent},
};

use crate::{
    layers::{OTHER_PLAYER_LAYER, WORLD_LAYER},
    menu::MenuState,
    PlayerCamera,
};

use super::LocalPlayer;

#[derive(Resource)]
pub struct InputMap {
    pub key_menu: KeyCode,
    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_jump: KeyCode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            key_menu: KeyCode::Tab,
            key_forward: KeyCode::KeyW,
            key_backward: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_jump: KeyCode::Space,
        }
    }
}

pub fn read_keyboard_input(
    input_map: Res<InputMap>,
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<State<MenuState>>,
    mut next_menu: ResMut<NextState<MenuState>>,
    mut players: Query<&mut LocalPlayer>,
) {
    for mut player in players.iter_mut() {
        player.input.forward = keys.pressed(input_map.key_forward);
        player.input.backward = keys.pressed(input_map.key_backward);
        player.input.left = keys.pressed(input_map.key_left);
        player.input.right = keys.pressed(input_map.key_right);
        player.input.jump = keys.pressed(input_map.key_jump);

        let did_move = player.input.forward
            || player.input.backward
            || player.input.left
            || player.input.right
            || player.input.jump;

        if did_move {
            next_menu.set(MenuState::Closed);
        } else if keys.just_pressed(input_map.key_menu) {
            match menu.get() {
                MenuState::Closed => next_menu.set(MenuState::Open),
                MenuState::Open => next_menu.set(MenuState::Closed),
            }
        }
    }
}

const RAYCAST_DISTANCE: f32 = 5.0;
const CROSSHAIR_RADIUS: f32 = 0.012;

pub fn handle_raycast_input(
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    input_handlers: Query<(Entity, &InputHandlerSender)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut gizmos: Gizmos,
    query: SpatialQuery,
) {
    if camera.is_empty() {
        return;
    }

    let transform = camera.single();
    let (_, rotation, translation) = transform.to_scale_rotation_translation();

    let direction = rotation.normalize() * Dir3::NEG_Z;

    if let Some(hit) = query.cast_ray(
        translation,
        direction,
        RAYCAST_DISTANCE,
        false,
        SpatialQueryFilter {
            mask: OTHER_PLAYER_LAYER | WORLD_LAYER,
            ..default()
        },
    ) {
        // Crosshair.
        // TODO: Fix showing double on non-60hz monitors
        if let Ok(normal) = Dir3::from_xyz(hit.normal.x, hit.normal.y, hit.normal.z) {
            gizmos.circle(
                translation + direction * (hit.time_of_impact - 0.001),
                normal,
                CROSSHAIR_RADIUS * (hit.time_of_impact / RAYCAST_DISTANCE),
                Color::WHITE,
            );
        }

        // Script input.
        for (ent, handler) in input_handlers.iter() {
            // TODO: Recursive check if children were hit.

            let action = if mouse.just_pressed(MouseButton::Left) {
                InputAction::Collision
            } else {
                InputAction::Hover
            };

            if hit.entity == ent {
                if let Err(e) = handler.send(ScriptInputEvent::Raycast {
                    action,
                    origin: translation,
                    orientation: rotation,
                }) {
                    error!("Failed to send script input event: {}", e);
                };
                break;
            }
        }
    };
}
