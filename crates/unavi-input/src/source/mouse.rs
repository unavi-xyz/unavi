use bevy::{
    input::mouse::{
        AccumulatedMouseMotion,
        MouseScrollUnit,
        MouseWheel,
    },
    prelude::*,
};

use crate::{
    action::ActionState,
    config::{
        InputConfig,
        platform::mouse_motion_scale,
    },
};

/// A [`MouseWheel`] event's `y` in whichever [`MouseScrollUnit`] the
/// platform sent, normalized to lines: [`Reach`](crate::action::Action::Reach)
/// reads wheel notches as a count, and a line is what one notch means on
/// hardware that reports [`MouseScrollUnit::Line`] directly. A platform that
/// reports [`MouseScrollUnit::Pixel`] instead is divided down by Bevy's own
/// [`MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR`] (documented there as
/// "correct for Microsoft Edge", the same approximation Bevy ships for this
/// exact mismatch), so a trackpad's pixel-granular scroll and a mouse's
/// line-granular wheel move the same action by comparable amounts.
fn lines(event: &MouseWheel) -> f32 {
    match event.unit {
        MouseScrollUnit::Line => event.y,
        MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
    }
}

pub fn read(
    motion: Res<AccumulatedMouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    config: Res<InputConfig>,
    mut state: ResMut<ActionState>,
) {
    // Screen space runs down; looking does not.
    let delta = Vec2::new(motion.delta.x, -motion.delta.y) * mouse_motion_scale();

    if delta != Vec2::ZERO {
        for (action, _) in config.bindings.axes().filter(|(_, b)| b.mouse_motion) {
            state.accumulate_delta(action, delta);
        }
    }

    let notches: f32 = wheel.read().map(lines).sum();
    if notches != 0.0 {
        for (action, _) in config.bindings.axes().filter(|(_, b)| b.mouse_wheel) {
            state.accumulate_delta(action, Vec2::new(0.0, notches));
        }
    }

    for (action, binding) in config.bindings.buttons() {
        if binding.mouse.iter().any(|button| buttons.pressed(*button)) {
            state.press(action, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        input::touch::TouchPhase,
        prelude::{
            App,
            ButtonInput,
            Entity,
            MouseButton,
            Update,
        },
    };

    use super::{
        AccumulatedMouseMotion,
        ActionState,
        InputConfig,
        MouseScrollUnit,
        MouseWheel,
        read,
    };
    use crate::action::Action;

    fn reach_delta_for(unit: MouseScrollUnit, y: f32) -> f32 {
        let mut app = App::new();
        app.add_message::<MouseWheel>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<InputConfig>()
            .init_resource::<ActionState>();
        app.world_mut().write_message(MouseWheel {
            unit,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
            phase: TouchPhase::Moved,
        });
        app.add_systems(Update, read);
        app.update();
        app.world().resource::<ActionState>().delta(Action::Reach).y
    }

    /// A trackpad reporting [`MouseScrollUnit::Pixel`] and a wheel reporting
    /// [`MouseScrollUnit::Line`] must move [`Action::Reach`] by comparable
    /// amounts for an equivalent scroll, not by a 100x-different amount
    /// because one side's unit went unconverted.
    #[test]
    fn pixel_and_line_wheel_units_produce_comparable_reach_deltas() {
        let line = reach_delta_for(MouseScrollUnit::Line, 1.0);
        let pixel = reach_delta_for(
            MouseScrollUnit::Pixel,
            MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        );

        assert!(
            (line - pixel).abs() < 1.0e-5,
            "line={line} pixel={pixel} should match"
        );
    }
}
