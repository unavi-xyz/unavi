use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8};

use bevy::prelude::*;
use unavi_avatar::animation::{AnimationName, weights::TargetAnimationWeights};
use unavi_input::{actions::MenuAction, schminput::BoolActionValue};

use crate::{LocalAgent, LocalAgentEntities, movement::MenuAnimationState};

pub const MIN_MENU_MOVEMENT: f32 = 0.1;
pub const PITCH_BOUND_H: f32 = super::PITCH_BOUND - FRAC_PI_4;
pub const PITCH_BOUND_L: f32 = -super::PITCH_BOUND - FRAC_PI_8;
pub const YAW_BOUND: f32 = FRAC_PI_2;

pub fn apply_menu_animation(
    menu_action: Query<&BoolActionValue, With<MenuAction>>,
    mut animations: Query<(&mut TargetAnimationWeights, &ChildOf)>,
    local_agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    mut menu_state: ResMut<MenuAnimationState>,
    mut prev_state: Local<bool>,
) {
    let Ok(LocalAgentEntities { avatar, .. }) = local_agent.single() else {
        return;
    };

    let Ok(menu_action) = menu_action.single() else {
        return;
    };

    for (mut weights, child_of) in &mut animations {
        if child_of.parent() != *avatar {
            continue;
        }

        let just_pressed = menu_action.any && !*prev_state;
        *prev_state = menu_action.any;

        if just_pressed {
            menu_state.0 = !menu_state.0;
        }

        let menu_weight = if menu_state.0 { 1.0 } else { 0.0 };
        weights.insert(AnimationName::Menu, menu_weight);

        break;
    }
}
