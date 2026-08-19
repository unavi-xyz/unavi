use bevy::prelude::*;
use bevy_xr_utils::actions::{
    ActionType,
    ActiveSet,
    XRUtilsAction,
    XRUtilsActionSet,
    XRUtilsActionState,
    XRUtilsBinding,
};

use crate::{
    action::{
        Action,
        ActionState,
    },
    config::{
        InputConfig,
        bindings::XrBinding,
    },
    source::deadzone,
};

/// Which UNAVI action an `OpenXR` action feeds.
#[derive(Component, Clone, Copy)]
pub struct XrAction(pub Action);

/// Builds one `OpenXR` action per bound UNAVI action, with a binding entity per
/// suggested path. `bevy_xr_utils` turns these into a real action set at
/// session start, so they must exist before it runs.
pub fn setup(config: Res<InputConfig>, mut commands: Commands) {
    let set = commands
        .spawn((
            XRUtilsActionSet {
                name:        "unavi".into(),
                pretty_name: "UNAVI".into(),
                priority:    0,
            },
            ActiveSet,
        ))
        .id();

    for (action, binding) in config.bindings.axes() {
        spawn_action(
            &mut commands,
            set,
            action,
            ActionType::Vector,
            binding.xr.iter(),
        );
    }

    for (action, binding) in config.bindings.buttons() {
        spawn_action(
            &mut commands,
            set,
            action,
            ActionType::Float,
            binding.xr.iter(),
        );
    }
}

fn spawn_action<'a>(
    commands: &mut Commands,
    set: Entity,
    action: Action,
    kind: ActionType,
    bindings: impl Iterator<Item = &'a XrBinding>,
) {
    let mut bindings = bindings.peekable();
    if bindings.peek().is_none() {
        return;
    }

    let name = name_of(action);
    let entity = commands
        .spawn((
            XrAction(action),
            XRUtilsAction {
                action_name:    name.clone().into(),
                localized_name: name.into(),
                action_type:    kind,
            },
            ChildOf(set),
        ))
        .id();

    for binding in bindings {
        commands.spawn((
            XRUtilsBinding {
                profile: binding.profile.clone().into(),
                binding: binding.path.clone().into(),
            },
            ChildOf(entity),
        ));
    }
}

fn name_of(action: Action) -> String {
    match action {
        Action::Move => "move".to_owned(),
        Action::Look => "look".to_owned(),
        Action::Jump => "jump".to_owned(),
        Action::Sprint => "sprint".to_owned(),
        Action::Trigger(kind) => format!("trigger_{}", kind.name()),
        Action::Grip(kind) => format!("grip_{}", kind.name()),
        Action::Menu(kind) => format!("menu_{}", kind.name()),
    }
}

pub fn read(
    actions: Query<(&XrAction, &XRUtilsActionState)>,
    config: Res<InputConfig>,
    mut state: ResMut<ActionState>,
) {
    for (action, xr_state) in actions {
        match xr_state {
            XRUtilsActionState::Bool(value) if value.is_active => {
                state.press(action.0, f32::from(value.current_state));
            }
            XRUtilsActionState::Float(value) if value.is_active => {
                state.press(action.0, value.current_state);
            }
            XRUtilsActionState::Vector(value) if value.is_active => {
                let raw = Vec2::from_array(value.current_state);
                state.accumulate(action.0, deadzone(raw, config.tuning.stick_deadzone));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::PointerKind;

    #[test]
    fn every_action_gets_a_distinct_openxr_name() {
        let mut names = Action::ALL.map(name_of).to_vec();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    /// `bevy_xr_utils` unwraps `create_action`, so a name `OpenXR` rejects is
    /// a panic at session start rather than a missing binding.
    #[test]
    fn every_name_is_one_openxr_accepts() {
        for action in Action::ALL {
            let name = name_of(action);
            assert!(!name.is_empty(), "{action:?} has no name");
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || c == '_'
                    || c == '-'
                    || c == '.'),
                "{name} would be rejected"
            );
        }
    }

    #[test]
    fn the_hands_are_told_apart_in_the_name() {
        assert_ne!(
            name_of(Action::Grip(PointerKind::LeftHand)),
            name_of(Action::Grip(PointerKind::RightHand))
        );
    }
}
