use bevy::prelude::*;

use crate::{
    capture::Captured,
    config::{
        InputConfig,
        Tuning,
    },
    pointer::PointerKind,
};

/// What a hand does, as opposed to what is bound to it.
///
/// [`Self::Trigger`] and [`Self::Grip`] are the two halves of acting on the
/// world and are deliberately separate: a trigger points and picks, a grip
/// closes around what is already there. One button doing both is what forces
/// the host and an equipped tool to arbitrate over a single press.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Move,
    Look,
    Jump,
    Sprint,
    Trigger(PointerKind),
    Grip(PointerKind),
    Menu(PointerKind),
}

impl Action {
    pub const ALL: [Self; Self::COUNT] = [
        Self::Move,
        Self::Look,
        Self::Jump,
        Self::Sprint,
        Self::Trigger(PointerKind::Screen),
        Self::Trigger(PointerKind::LeftHand),
        Self::Trigger(PointerKind::RightHand),
        Self::Grip(PointerKind::Screen),
        Self::Grip(PointerKind::LeftHand),
        Self::Grip(PointerKind::RightHand),
        Self::Menu(PointerKind::Screen),
        Self::Menu(PointerKind::LeftHand),
        Self::Menu(PointerKind::RightHand),
    ];
    pub const COUNT: usize = 4 + 3 * PointerKind::COUNT;

    const fn index(self) -> usize {
        match self {
            Self::Move => 0,
            Self::Look => 1,
            Self::Jump => 2,
            Self::Sprint => 3,
            Self::Trigger(kind) => 4 + kind.index(),
            Self::Grip(kind) => 4 + PointerKind::COUNT + kind.index(),
            Self::Menu(kind) => 4 + 2 * PointerKind::COUNT + kind.index(),
        }
    }

    /// Whether the action carries a direction rather than a strength. An axis
    /// action's `pressed` is never read.
    const fn is_axis(self) -> bool {
        matches!(self, Self::Move | Self::Look)
    }
}

#[derive(Clone, Copy, Default)]
struct ActionValue {
    axis:     Vec2,
    delta:    Vec2,
    strength: f32,
    pressed:  bool,
    previous: bool,
}

/// Every action's value for this frame, gathered from every bound source.
#[derive(Resource)]
pub struct ActionState {
    values: [ActionValue; Action::COUNT],
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            values: [ActionValue::default(); Action::COUNT],
        }
    }
}

impl ActionState {
    /// How far a held source is pushed, `-1..=1` per component. A rate: what
    /// it means depends on how long it is held, so a reader scales it by the
    /// frame's time.
    #[must_use]
    pub const fn axis(&self, action: Action) -> Vec2 {
        self.values[action.index()].axis
    }

    /// How far a mouse moved this frame. Already a travel rather than a rate,
    /// so scaling it by the frame's time would make the same hand movement
    /// mean less the faster the game runs.
    #[must_use]
    pub const fn delta(&self, action: Action) -> Vec2 {
        self.values[action.index()].delta
    }

    /// How hard the action is held, `0..=1`. Analogue where the binding is —
    /// an `OpenXR` squeeze reports its pull, a key reports all or nothing.
    #[must_use]
    pub const fn value(&self, action: Action) -> f32 {
        self.values[action.index()].strength
    }

    #[must_use]
    pub const fn pressed(&self, action: Action) -> bool {
        self.values[action.index()].pressed
    }

    #[must_use]
    pub const fn just_pressed(&self, action: Action) -> bool {
        let value = &self.values[action.index()];
        value.pressed && !value.previous
    }

    #[must_use]
    pub const fn just_released(&self, action: Action) -> bool {
        let value = &self.values[action.index()];
        !value.pressed && value.previous
    }

    pub fn accumulate(&mut self, action: Action, axis: Vec2) {
        self.values[action.index()].axis += axis;
    }

    pub fn accumulate_delta(&mut self, action: Action, delta: Vec2) {
        self.values[action.index()].delta += delta;
    }

    /// Raises an action's strength to `value`, so the source pulling hardest
    /// wins rather than several adding up past full.
    pub const fn press(&mut self, action: Action, value: f32) {
        let held = &mut self.values[action.index()].strength;
        *held = held.max(value);
    }

    /// Drops everything read this frame, leaving every action to end it
    /// released.
    ///
    /// What was held reaches its readers as the release it would have got
    /// anyway, rather than sticking down behind whatever took the input: a
    /// grab lets go, a tool stops firing, and the agent stands still.
    pub fn silence(&mut self) {
        for value in &mut self.values {
            value.axis = Vec2::ZERO;
            value.delta = Vec2::ZERO;
            value.strength = 0.0;
        }
    }

    pub fn begin_frame(&mut self) {
        for value in &mut self.values {
            value.previous = value.pressed;
            value.axis = Vec2::ZERO;
            value.delta = Vec2::ZERO;
            value.strength = 0.0;
        }
    }

    pub fn end_frame(&mut self, tuning: &Tuning) {
        for action in Action::ALL.into_iter().filter(|a| !a.is_axis()) {
            let value = &mut self.values[action.index()];
            value.pressed = value.strength >= tuning.press_threshold;
        }

        for action in Action::ALL.into_iter().filter(|a| a.is_axis()) {
            let axis = &mut self.values[action.index()].axis;
            *axis = axis.clamp_length_max(1.0);
        }
    }
}

pub fn begin_frame(mut state: ResMut<ActionState>) {
    state.begin_frame();
}

pub fn end_frame(
    mut state: ResMut<ActionState>,
    config: Res<InputConfig>,
    captured: Res<Captured>,
) {
    if captured.0 {
        state.silence();
    }
    state.end_frame(&config.tuning);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_its_own_slot() {
        let mut seen = [false; Action::COUNT];
        for action in Action::ALL {
            assert!(!seen[action.index()], "{action:?} shares a slot");
            seen[action.index()] = true;
        }
    }

    #[test]
    fn the_hardest_pull_wins_rather_than_summing() {
        let mut state = ActionState::default();
        state.press(Action::Trigger(PointerKind::Screen), 0.8);
        state.press(Action::Trigger(PointerKind::Screen), 0.3);
        assert!((state.value(Action::Trigger(PointerKind::Screen)) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn two_keyboards_pushing_the_same_way_do_not_move_twice_as_fast() {
        let mut state = ActionState::default();
        state.accumulate(Action::Move, Vec2::Y);
        state.accumulate(Action::Move, Vec2::Y);
        state.end_frame(&Tuning::default());
        assert!((state.axis(Action::Move).length() - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn two_sticks_aiming_the_same_way_do_not_turn_twice_as_fast() {
        let mut state = ActionState::default();
        state.accumulate(Action::Look, Vec2::X);
        state.accumulate(Action::Look, Vec2::X);
        state.end_frame(&Tuning::default());
        assert!((state.axis(Action::Look).length() - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn a_mouse_delta_keeps_its_full_reach() {
        let mut state = ActionState::default();
        state.accumulate_delta(Action::Look, Vec2::new(40.0, -12.0));
        state.end_frame(&Tuning::default());
        assert_eq!(state.delta(Action::Look), Vec2::new(40.0, -12.0));
    }

    #[test]
    fn a_mouse_and_a_stick_reach_the_reader_apart() {
        let mut state = ActionState::default();
        state.accumulate(Action::Look, Vec2::new(0.5, 0.0));
        state.accumulate_delta(Action::Look, Vec2::new(40.0, 0.0));
        state.end_frame(&Tuning::default());

        assert_eq!(state.axis(Action::Look), Vec2::new(0.5, 0.0));
        assert_eq!(state.delta(Action::Look), Vec2::new(40.0, 0.0));
    }

    #[test]
    fn a_frame_starts_with_nothing_held_over() {
        let mut state = ActionState::default();
        state.accumulate(Action::Move, Vec2::Y);
        state.accumulate_delta(Action::Look, Vec2::X);
        state.press(Action::Jump, 1.0);

        state.begin_frame();

        assert_eq!(state.axis(Action::Move), Vec2::ZERO);
        assert_eq!(state.delta(Action::Look), Vec2::ZERO);
        assert!(state.value(Action::Jump).abs() < f32::EPSILON);
    }

    #[test]
    fn an_edge_is_reported_once() {
        let mut state = ActionState::default();
        let jump = Action::Jump;

        state.begin_frame();
        state.press(jump, 1.0);
        state.end_frame(&Tuning::default());
        assert!(state.just_pressed(jump));

        state.begin_frame();
        state.press(jump, 1.0);
        state.end_frame(&Tuning::default());
        assert!(state.pressed(jump) && !state.just_pressed(jump));

        state.begin_frame();
        state.end_frame(&Tuning::default());
        assert!(state.just_released(jump));
    }

    #[test]
    fn a_squeeze_short_of_the_threshold_is_not_a_press() {
        let mut state = ActionState::default();
        let grip = Action::Grip(PointerKind::RightHand);
        let tuning = Tuning::default();

        state.begin_frame();
        state.press(grip, tuning.press_threshold - 0.01);
        state.end_frame(&tuning);
        assert!(!state.pressed(grip), "but its pull is still readable");
        assert!(state.value(grip) > 0.0);
    }

    #[test]
    fn a_trigger_and_a_grip_on_one_hand_are_read_apart() {
        let mut state = ActionState::default();
        let kind = PointerKind::RightHand;

        state.begin_frame();
        state.press(Action::Grip(kind), 1.0);
        state.end_frame(&Tuning::default());

        assert!(state.pressed(Action::Grip(kind)));
        assert!(
            !state.pressed(Action::Trigger(kind)),
            "closing a hand is not pulling its trigger"
        );
    }

    #[test]
    fn what_was_held_when_the_input_was_taken_lets_go_of_it() {
        let mut state = ActionState::default();
        let grip = Action::Grip(PointerKind::Screen);
        let tuning = Tuning::default();

        state.begin_frame();
        state.press(grip, 1.0);
        state.end_frame(&tuning);
        assert!(state.pressed(grip));

        state.begin_frame();
        state.press(grip, 1.0);
        state.silence();
        state.end_frame(&tuning);

        assert!(
            state.just_released(grip),
            "a grab must let go rather than stick down behind whatever took \
             the input"
        );
        assert!(!state.pressed(grip));
    }

    #[test]
    fn silence_takes_the_axes_with_the_buttons() {
        let mut state = ActionState::default();

        state.begin_frame();
        state.accumulate(Action::Move, Vec2::Y);
        state.accumulate_delta(Action::Look, Vec2::X);
        state.press(Action::Jump, 1.0);
        state.silence();
        state.end_frame(&Tuning::default());

        assert_eq!(state.axis(Action::Move), Vec2::ZERO);
        assert_eq!(state.delta(Action::Look), Vec2::ZERO);
        assert!(!state.pressed(Action::Jump));
    }

    #[test]
    fn aiming_hard_to_the_right_is_not_a_press() {
        let mut state = ActionState::default();
        state.accumulate(Action::Move, Vec2::X);
        state.end_frame(&Tuning::default());
        assert!(!state.pressed(Action::Move));
    }
}
