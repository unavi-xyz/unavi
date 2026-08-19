use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::config::{
    InputConfig,
    Tuning,
    bindings::{
        AxisBinding,
        Bindings,
        ButtonBinding,
        Dpad,
        PerPointer,
        Stick,
        XrBinding,
    },
};

/// What a config file may say. Every binding is optional and applied over the
/// defaults, so naming one of them never silently drops the rest.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigPatch {
    pub bindings: BindingsPatch,
    pub tuning:   Tuning,
}

impl ConfigPatch {
    #[must_use]
    pub fn resolve(self) -> InputConfig {
        InputConfig {
            bindings: self.bindings.apply(Bindings::default()),
            tuning:   self.tuning.sanitized(),
        }
    }
}

impl From<&InputConfig> for ConfigPatch {
    fn from(config: &InputConfig) -> Self {
        Self {
            bindings: (&config.bindings).into(),
            tuning:   config.tuning,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BindingsPatch {
    #[serde(rename = "move")]
    pub movement: AxisPatch,
    pub look:     AxisPatch,
    pub jump:     ButtonPatch,
    pub sprint:   ButtonPatch,
    pub trigger:  PerPointer<ButtonPatch>,
    pub grip:     PerPointer<ButtonPatch>,
    pub menu:     PerPointer<ButtonPatch>,
}

impl BindingsPatch {
    fn apply(self, base: Bindings) -> Bindings {
        Bindings {
            movement: self.movement.apply(base.movement),
            look:     self.look.apply(base.look),
            jump:     self.jump.apply(base.jump),
            sprint:   self.sprint.apply(base.sprint),
            trigger:  apply_per_pointer(self.trigger, base.trigger),
            grip:     apply_per_pointer(self.grip, base.grip),
            menu:     apply_per_pointer(self.menu, base.menu),
        }
    }
}

impl From<&Bindings> for BindingsPatch {
    fn from(bindings: &Bindings) -> Self {
        Self {
            movement: (&bindings.movement).into(),
            look:     (&bindings.look).into(),
            jump:     (&bindings.jump).into(),
            sprint:   (&bindings.sprint).into(),
            trigger:  per_pointer_patch(&bindings.trigger),
            grip:     per_pointer_patch(&bindings.grip),
            menu:     per_pointer_patch(&bindings.menu),
        }
    }
}

fn apply_per_pointer(
    patch: PerPointer<ButtonPatch>,
    base: PerPointer<ButtonBinding>,
) -> PerPointer<ButtonBinding> {
    PerPointer {
        screen:     patch.screen.apply(base.screen),
        left_hand:  patch.left_hand.apply(base.left_hand),
        right_hand: patch.right_hand.apply(base.right_hand),
    }
}

fn per_pointer_patch(base: &PerPointer<ButtonBinding>) -> PerPointer<ButtonPatch> {
    PerPointer {
        screen:     (&base.screen).into(),
        left_hand:  (&base.left_hand).into(),
        right_hand: (&base.right_hand).into(),
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AxisPatch {
    pub dpads:        Option<Vec<Dpad>>,
    pub sticks:       Option<Vec<Stick>>,
    pub mouse_motion: Option<bool>,
    pub xr:           Option<Vec<XrBinding>>,
}

impl AxisPatch {
    fn apply(self, base: AxisBinding) -> AxisBinding {
        AxisBinding {
            dpads:        self.dpads.unwrap_or(base.dpads),
            sticks:       self.sticks.unwrap_or(base.sticks),
            mouse_motion: self.mouse_motion.unwrap_or(base.mouse_motion),
            xr:           self.xr.unwrap_or(base.xr),
        }
    }
}

impl From<&AxisBinding> for AxisPatch {
    fn from(binding: &AxisBinding) -> Self {
        Self {
            dpads:        Some(binding.dpads.clone()),
            sticks:       Some(binding.sticks.clone()),
            mouse_motion: Some(binding.mouse_motion),
            xr:           Some(binding.xr.clone()),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ButtonPatch {
    pub keys:  Option<Vec<KeyCode>>,
    pub mouse: Option<Vec<MouseButton>>,
    pub pad:   Option<Vec<GamepadButton>>,
    pub xr:    Option<Vec<XrBinding>>,
}

impl ButtonPatch {
    fn apply(self, base: ButtonBinding) -> ButtonBinding {
        ButtonBinding {
            keys:  self.keys.unwrap_or(base.keys),
            mouse: self.mouse.unwrap_or(base.mouse),
            pad:   self.pad.unwrap_or(base.pad),
            xr:    self.xr.unwrap_or(base.xr),
        }
    }
}

impl From<&ButtonBinding> for ButtonPatch {
    fn from(binding: &ButtonBinding) -> Self {
        Self {
            keys:  Some(binding.keys.clone()),
            mouse: Some(binding.mouse.clone()),
            pad:   Some(binding.pad.clone()),
            xr:    Some(binding.xr.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::file::{
        parse,
        to_text,
    };

    #[test]
    fn naming_one_dpad_keeps_the_stick_and_the_headset() {
        let config =
            parse("(bindings: (move: (dpads: [(up: KeyI, down: KeyK, left: KeyJ, right: KeyL)])))")
                .expect("parse");

        assert_eq!(config.bindings.movement.dpads.len(), 1);
        assert_eq!(
            config.bindings.movement.sticks.len(),
            Bindings::default().movement.sticks.len(),
            "a named dpad list is not a claim about sticks"
        );
        assert_eq!(
            config.bindings.movement.xr.len(),
            Bindings::default().movement.xr.len()
        );
    }

    #[test]
    fn a_second_dpad_lives_alongside_the_first() {
        let config = parse(
            "(bindings: (move: (dpads: [
                (up: KeyW, down: KeyS, left: KeyA, right: KeyD),
                (up: KeyI, down: KeyK, left: KeyJ, right: KeyL),
            ])))",
        )
        .expect("parse");

        assert_eq!(config.bindings.movement.dpads.len(), 2);
    }

    #[test]
    fn touching_one_binding_leaves_its_siblings_alone() {
        let config = parse("(bindings: (jump: (keys: [Enter])))").expect("parse");

        assert_eq!(config.bindings.jump.keys, vec![KeyCode::Enter]);
        assert_eq!(
            config.bindings.jump.xr.len(),
            Bindings::default().jump.xr.len()
        );
        assert_eq!(config.bindings.sprint.keys, Bindings::default().sprint.keys);
    }

    #[test]
    fn an_empty_list_is_an_unbinding_rather_than_a_silence() {
        let config = parse("(bindings: (look: (mouse_motion: false, sticks: [])))").expect("parse");

        assert!(config.bindings.look.sticks.is_empty());
        assert!(!config.bindings.look.mouse_motion);
    }

    #[test]
    fn what_is_written_out_reads_back_the_same() {
        let config = InputConfig::default();
        let text = to_text(&config).expect("serialize");
        let parsed = parse(&text).expect("parse");

        assert_eq!(
            parsed.bindings.movement.xr.len(),
            config.bindings.movement.xr.len()
        );
        assert_eq!(
            parsed.bindings.trigger.screen.mouse,
            config.bindings.trigger.screen.mouse
        );
        assert!(
            !text.contains("Some("),
            "the file a person edits should not be full of Some"
        );
    }
}
