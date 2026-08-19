use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    action::Action,
    pointer::PointerKind,
};

/// Every runtime supports this one, so it is what a controller UNAVI ships no
/// bindings for falls back to. It has two buttons and no stick.
const SIMPLE_PROFILE: &str = "/interaction_profiles/khr/simple_controller";
const TOUCH_PROFILE: &str = "/interaction_profiles/oculus/touch_controller";
const INDEX_PROFILE: &str = "/interaction_profiles/valve/index_controller";
const VIVE_PROFILE: &str = "/interaction_profiles/htc/vive_controller";
const WMR_PROFILE: &str = "/interaction_profiles/microsoft/motion_controller";

#[derive(Clone, Debug)]
pub struct Bindings {
    pub movement: AxisBinding,
    pub look:     AxisBinding,
    pub jump:     ButtonBinding,
    pub sprint:   ButtonBinding,
    pub trigger:  PerPointer<ButtonBinding>,
    pub grip:     PerPointer<ButtonBinding>,
    pub menu:     PerPointer<ButtonBinding>,
}

impl Bindings {
    pub fn axes(&self) -> impl Iterator<Item = (Action, &AxisBinding)> {
        [(Action::Move, &self.movement), (Action::Look, &self.look)].into_iter()
    }

    pub fn buttons(&self) -> impl Iterator<Item = (Action, &ButtonBinding)> {
        [(Action::Jump, &self.jump), (Action::Sprint, &self.sprint)]
            .into_iter()
            .chain(
                PointerKind::ALL
                    .into_iter()
                    .map(|kind| (Action::Trigger(kind), self.trigger.get(kind))),
            )
            .chain(
                PointerKind::ALL
                    .into_iter()
                    .map(|kind| (Action::Grip(kind), self.grip.get(kind))),
            )
            .chain(
                PointerKind::ALL
                    .into_iter()
                    .map(|kind| (Action::Menu(kind), self.menu.get(kind))),
            )
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PerPointer<T> {
    pub screen:     T,
    pub left_hand:  T,
    pub right_hand: T,
}

impl<T> PerPointer<T> {
    pub const fn get(&self, kind: PointerKind) -> &T {
        match kind {
            PointerKind::Screen => &self.screen,
            PointerKind::LeftHand => &self.left_hand,
            PointerKind::RightHand => &self.right_hand,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AxisBinding {
    pub dpads:        Vec<Dpad>,
    pub sticks:       Vec<Stick>,
    pub mouse_motion: bool,
    pub xr:           Vec<XrBinding>,
}

#[derive(Clone, Debug, Default)]
pub struct ButtonBinding {
    pub keys:  Vec<KeyCode>,
    pub mouse: Vec<MouseButton>,
    pub pad:   Vec<GamepadButton>,
    pub xr:    Vec<XrBinding>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dpad {
    pub up:    KeyCode,
    pub down:  KeyCode,
    pub left:  KeyCode,
    pub right: KeyCode,
}

impl Dpad {
    #[must_use]
    pub fn value(self, keys: &ButtonInput<KeyCode>) -> Vec2 {
        let axis = |positive: KeyCode, negative: KeyCode| {
            f32::from(keys.pressed(positive)) - f32::from(keys.pressed(negative))
        };
        Vec2::new(axis(self.right, self.left), axis(self.up, self.down))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Stick {
    Left,
    Right,
}

impl Stick {
    #[must_use]
    pub const fn axes(self) -> (GamepadAxis, GamepadAxis) {
        match self {
            Self::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
            Self::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XrBinding {
    pub profile: String,
    pub path:    String,
}

const LEFT: &str = "left";
const RIGHT: &str = "right";

/// A profile and what follows `/user/hand/<side>/` on it.
fn xr(side: &str, bindings: &[(&str, &str)]) -> Vec<XrBinding> {
    bindings
        .iter()
        .map(|(profile, path)| XrBinding {
            profile: (*profile).to_owned(),
            path:    format!("/user/hand/{side}/{path}"),
        })
        .collect()
}

const STICK: [(&str, &str); 4] = [
    (TOUCH_PROFILE, "input/thumbstick"),
    (INDEX_PROFILE, "input/thumbstick"),
    (WMR_PROFILE, "input/thumbstick"),
    (VIVE_PROFILE, "input/trackpad"),
];

/// The one input every headset has under the index finger. Analogue where the
/// runtime reports a pull, so a light touch reads as one.
const TRIGGER: [(&str, &str); 5] = [
    (SIMPLE_PROFILE, "input/select/click"),
    (TOUCH_PROFILE, "input/trigger/value"),
    (INDEX_PROFILE, "input/trigger/value"),
    (WMR_PROFILE, "input/trigger/value"),
    (VIVE_PROFILE, "input/trigger/value"),
];

/// Closing the hand. The simple profile has no second button to spare, so a
/// controller UNAVI ships no bindings for can act but not carry.
const GRIP: [(&str, &str); 4] = [
    (TOUCH_PROFILE, "input/squeeze/value"),
    (INDEX_PROFILE, "input/squeeze/value"),
    (WMR_PROFILE, "input/squeeze/click"),
    (VIVE_PROFILE, "input/squeeze/click"),
];

const JUMP: [(&str, &str); 4] = [
    (TOUCH_PROFILE, "input/a/click"),
    (INDEX_PROFILE, "input/a/click"),
    (WMR_PROFILE, "input/trackpad/click"),
    (VIVE_PROFILE, "input/trackpad/click"),
];

/// A wand has a trigger, a grip, a menu and a trackpad and nothing else, so
/// once the first three are spoken for and the fourth is walking there is no
/// button left to sprint with.
const SPRINT: [(&str, &str); 3] = [
    (TOUCH_PROFILE, "input/thumbstick/click"),
    (INDEX_PROFILE, "input/thumbstick/click"),
    (WMR_PROFILE, "input/thumbstick/click"),
];

fn movement() -> AxisBinding {
    AxisBinding {
        dpads: vec![Dpad {
            up:    KeyCode::KeyW,
            down:  KeyCode::KeyS,
            left:  KeyCode::KeyA,
            right: KeyCode::KeyD,
        }],
        sticks: vec![Stick::Left],
        xr: xr(LEFT, &STICK),
        ..default()
    }
}

fn look() -> AxisBinding {
    AxisBinding {
        sticks: vec![Stick::Right],
        mouse_motion: true,
        xr: xr(RIGHT, &STICK),
        ..default()
    }
}

fn jump() -> ButtonBinding {
    ButtonBinding {
        keys: vec![KeyCode::Space],
        pad: vec![GamepadButton::South],
        xr: xr(RIGHT, &JUMP),
        ..default()
    }
}

fn sprint() -> ButtonBinding {
    ButtonBinding {
        keys: vec![KeyCode::ShiftLeft],
        pad: vec![GamepadButton::LeftThumb],
        xr: xr(LEFT, &SPRINT),
        ..default()
    }
}

/// Acting on what is pointed at: the button everything picks with.
fn trigger() -> PerPointer<ButtonBinding> {
    PerPointer {
        screen:     ButtonBinding {
            mouse: vec![MouseButton::Left],
            pad: vec![GamepadButton::RightTrigger2],
            ..default()
        },
        left_hand:  ButtonBinding {
            xr: xr(LEFT, &TRIGGER),
            ..default()
        },
        right_hand: ButtonBinding {
            xr: xr(RIGHT, &TRIGGER),
            ..default()
        },
    }
}

/// Taking hold of what is pointed at. Never bound to the same input as
/// [`trigger`], because the whole point of the two is that a press means one
/// thing or the other and never both.
fn grip() -> PerPointer<ButtonBinding> {
    PerPointer {
        screen:     ButtonBinding {
            mouse: vec![MouseButton::Right],
            pad: vec![GamepadButton::LeftTrigger2],
            ..default()
        },
        left_hand:  ButtonBinding {
            xr: xr(LEFT, &GRIP),
            ..default()
        },
        right_hand: ButtonBinding {
            xr: xr(RIGHT, &GRIP),
            ..default()
        },
    }
}

/// Touch is the odd one out: its second face button is `y` on the left hand
/// and `b` on the right.
fn menu() -> PerPointer<ButtonBinding> {
    let common = |touch| {
        [
            (SIMPLE_PROFILE, "input/menu/click"),
            (TOUCH_PROFILE, touch),
            (INDEX_PROFILE, "input/b/click"),
            (WMR_PROFILE, "input/menu/click"),
            (VIVE_PROFILE, "input/menu/click"),
        ]
    };

    PerPointer {
        screen:     ButtonBinding {
            keys: vec![KeyCode::Tab],
            pad: vec![GamepadButton::West],
            ..default()
        },
        left_hand:  ButtonBinding {
            xr: xr(LEFT, &common("input/y/click")),
            ..default()
        },
        right_hand: ButtonBinding {
            xr: xr(RIGHT, &common("input/b/click")),
            ..default()
        },
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            movement: movement(),
            look:     look(),
            jump:     jump(),
            sprint:   sprint(),
            trigger:  trigger(),
            grip:     grip(),
            menu:     menu(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILES: [&str; 5] = [
        SIMPLE_PROFILE,
        TOUCH_PROFILE,
        INDEX_PROFILE,
        VIVE_PROFILE,
        WMR_PROFILE,
    ];

    #[test]
    fn a_dpad_reads_its_own_keys() {
        let mut keys = ButtonInput::<KeyCode>::default();
        let dpad = Dpad {
            up:    KeyCode::KeyW,
            down:  KeyCode::KeyS,
            left:  KeyCode::KeyA,
            right: KeyCode::KeyD,
        };

        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        assert_eq!(dpad.value(&keys), Vec2::new(1.0, 1.0));

        keys.press(KeyCode::KeyS);
        assert_eq!(
            dpad.value(&keys),
            Vec2::new(1.0, 0.0),
            "opposite keys cancel rather than fighting"
        );
    }

    #[test]
    fn every_action_is_reachable_from_the_default_bindings() {
        let bindings = Bindings::default();
        let named = bindings
            .axes()
            .map(|(action, _)| action)
            .chain(bindings.buttons().map(|(action, _)| action))
            .collect::<Vec<_>>();

        for action in Action::ALL {
            assert!(named.contains(&action), "{action:?} has no binding slot");
        }
    }

    #[test]
    fn every_shipped_profile_can_act_and_open_the_menu() {
        let bindings = Bindings::default();

        for profile in PROFILES {
            for kind in [PointerKind::LeftHand, PointerKind::RightHand] {
                for (name, binding) in [
                    ("trigger", bindings.trigger.get(kind)),
                    ("menu", bindings.menu.get(kind)),
                ] {
                    assert!(
                        binding.xr.iter().any(|b| b.profile == profile),
                        "{profile} cannot {name} with its {}",
                        kind.name()
                    );
                }
            }
        }
    }

    #[test]
    fn no_profile_binds_one_input_to_both_the_trigger_and_the_grip() {
        let bindings = Bindings::default();

        for kind in PointerKind::ALL {
            for pulled in &bindings.trigger.get(kind).xr {
                assert!(
                    !bindings
                        .grip
                        .get(kind)
                        .xr
                        .iter()
                        .any(|held| held.path == pulled.path),
                    "{} would act and take hold at once",
                    pulled.path
                );
            }
        }
    }

    /// The trigger is the input everything picks with, so anything else on it
    /// would fire whenever a mote is pressed.
    #[test]
    fn nothing_else_is_bound_to_a_trigger() {
        let bindings = Bindings::default();
        let triggers = PointerKind::ALL
            .into_iter()
            .flat_map(|kind| bindings.trigger.get(kind).xr.iter())
            .map(|binding| binding.path.as_str())
            .collect::<Vec<_>>();

        for (action, binding) in bindings.buttons() {
            if matches!(action, Action::Trigger(_)) {
                continue;
            }
            for bound in &binding.xr {
                assert!(
                    !triggers.contains(&bound.path.as_str()),
                    "{action:?} shares {} with a trigger",
                    bound.path
                );
            }
        }
    }

    #[test]
    fn every_profile_with_a_stick_can_walk_look_jump_and_take_hold() {
        let bindings = Bindings::default();

        for profile in PROFILES.into_iter().filter(|p| *p != SIMPLE_PROFILE) {
            for (action, binding) in bindings.buttons() {
                let skip = match action {
                    // Covered by its own test, and the desktop pointer has no
                    // headset binding to find.
                    Action::Menu(_) | Action::Trigger(_) => true,
                    Action::Grip(kind) => kind == PointerKind::Screen,
                    // Sprint is the one thing a wand has no button left for.
                    Action::Sprint => profile == VIVE_PROFILE,
                    _ => false,
                };
                if skip {
                    continue;
                }
                assert!(
                    binding.xr.iter().any(|b| b.profile == profile),
                    "{profile} has no {action:?}"
                );
            }
            for (action, binding) in bindings.axes() {
                assert!(
                    binding.xr.iter().any(|b| b.profile == profile),
                    "{profile} has no {action:?}"
                );
            }
        }
    }

    #[test]
    fn every_suggested_path_belongs_to_a_hand() {
        let bindings = Bindings::default();
        let paths = bindings
            .axes()
            .map(|(_, b)| &b.xr)
            .chain(bindings.buttons().map(|(_, b)| &b.xr))
            .flatten();

        for binding in paths {
            assert!(
                binding.path.starts_with("/user/hand/left/")
                    || binding.path.starts_with("/user/hand/right/"),
                "{} is not a hand path",
                binding.path
            );
        }
    }
}
