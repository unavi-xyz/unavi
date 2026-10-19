use bevy::{
    input::mouse::{
        AccumulatedMouseMotion,
        MouseScrollUnit,
        MouseWheel,
    },
    prelude::*,
    window::{
        PrimaryWindow,
        Window,
    },
};

use crate::{
    action::ActionState,
    config::{
        InputConfig,
        platform::mouse_motion_scale,
    },
};

/// One frame's wheel travel, both axes, in notches.
///
/// A notch is what every reader of the wheel means: a count for
/// [`Reach`](crate::action::Action::Reach), and the vector `wired:input` hands
/// a guest ("notches turned on each axis"). Hardware that reports
/// [`MouseScrollUnit::Line`] sends one notch as a line; a platform that reports
/// [`MouseScrollUnit::Pixel`] sends many, and the browser reports those in
/// physical pixels, so both Bevy's own conversion factor and the window's scale
/// factor divide it back down. Without that, one notch on the web moves a
/// hundred times as far as one notch on the desktop.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct Notches(pub Vec2);

/// `event`'s travel in notches, on a window at `scale`.
fn notches_of(event: &MouseWheel, scale: f32) -> Vec2 {
    let delta = Vec2::new(event.x, event.y);
    match event.unit {
        MouseScrollUnit::Line => delta,
        MouseScrollUnit::Pixel => delta / (MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR * scale),
    }
}

pub fn read(
    motion: Res<AccumulatedMouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    config: Res<InputConfig>,
    mut state: ResMut<ActionState>,
    mut notches: ResMut<Notches>,
) {
    // Screen space runs down; looking does not.
    let delta = Vec2::new(motion.delta.x, -motion.delta.y) * mouse_motion_scale();

    if delta != Vec2::ZERO {
        for (action, _) in config.bindings.axes().filter(|(_, b)| b.mouse_motion) {
            state.accumulate_delta(action, delta);
        }
    }

    let scale = window.single().map_or(1.0, Window::scale_factor);
    let mut turned = Vec2::ZERO;
    for event in wheel.read() {
        turned += notches_of(event, scale);
    }
    notches.0 = turned;

    if turned.y != 0.0 {
        for (action, _) in config.bindings.axes().filter(|(_, b)| b.mouse_wheel) {
            state.accumulate_delta(action, Vec2::new(0.0, turned.y));
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
            Vec2,
        },
    };

    use super::{
        AccumulatedMouseMotion,
        ActionState,
        InputConfig,
        MouseScrollUnit,
        MouseWheel,
        Notches,
        notches_of,
        read,
    };
    use crate::action::Action;

    /// The notches one wheel event turns, and the reach `read` gives it.
    fn wheel_turns(unit: MouseScrollUnit, y: f32) -> (Vec2, f32) {
        let mut app = App::new();
        app.add_message::<MouseWheel>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<InputConfig>()
            .init_resource::<ActionState>()
            .init_resource::<Notches>();
        app.world_mut().write_message(MouseWheel {
            unit,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
            phase: TouchPhase::Moved,
        });
        app.add_systems(Update, read);
        app.update();
        let world = app.world();
        (
            world.resource::<Notches>().0,
            world.resource::<ActionState>().delta(Action::Reach).y,
        )
    }

    /// A trackpad reporting [`MouseScrollUnit::Pixel`] and a wheel reporting
    /// [`MouseScrollUnit::Line`] must move `Reach` and a guest by comparable
    /// amounts for an equivalent scroll, not by a 100x-different amount
    /// because one side's unit went unconverted.
    #[test]
    fn pixel_and_line_wheel_units_produce_comparable_notches() {
        let (line, line_reach) = wheel_turns(MouseScrollUnit::Line, 1.0);
        let (pixel, pixel_reach) = wheel_turns(
            MouseScrollUnit::Pixel,
            MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        );

        assert!(
            (line.y - pixel.y).abs() < 1.0e-5,
            "line={line} pixel={pixel}"
        );
        assert!((line_reach - pixel_reach).abs() < 1.0e-5);
    }

    /// The browser reports wheel pixels in physical ones, so the same gesture
    /// on a 2x display must still turn one notch rather than two.
    #[test]
    fn a_hidpi_pixel_scroll_still_turns_one_notch() {
        let notches = notches_of(
            &MouseWheel {
                unit:   MouseScrollUnit::Pixel,
                x:      0.0,
                y:      2.0 * MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
                window: Entity::PLACEHOLDER,
                phase:  TouchPhase::Moved,
            },
            2.0,
        );

        assert!((notches.y - 1.0).abs() < 1.0e-5, "got {notches}");
    }

    /// The frame's whole travel reaches the resource, so the paths that do not
    /// read the messages themselves (picking, device listeners) see the same
    /// notches `Reach` does.
    #[test]
    fn the_frame_total_carries_every_event() {
        let mut app = App::new();
        app.add_message::<MouseWheel>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<InputConfig>()
            .init_resource::<ActionState>()
            .init_resource::<Notches>()
            .add_systems(Update, read);
        for y in [0.5, 0.5] {
            app.world_mut().write_message(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y,
                window: Entity::PLACEHOLDER,
                phase: TouchPhase::Moved,
            });
        }
        app.update();

        assert_eq!(app.world().resource::<Notches>().0, Vec2::new(0.0, 1.0));
    }
}
