use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_vr_controller::player::PlayerCamera;
use unavi_constants::player::layers::{LAYER_OTHER_PLAYER, LAYER_WORLD};

use crate::api::wired::input::{
    bindings::types::InputAction,
    input_handler::{InputHandlerSender, ScriptInputEvent},
};

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
            mask: LAYER_OTHER_PLAYER | LAYER_WORLD,
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
                handler
                    .send(ScriptInputEvent::Raycast {
                        action,
                        origin: translation,
                        orientation: rotation,
                    })
                    .expect("Failed to send script input event");
                break;
            }
        }
    };
}
