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
/// [`Self::Trigger`] and [`Self::Grip`] are separate actions: a trigger points
/// and picks, a grip closes around what is already there. One button doing
/// both would force the host and an equipped tool to arbitrate over a single
/// press.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Move,
    Look,
    /// Pulling a held object closer or pushing it away. An axis rather than a
    /// button: a scroll wheel reports it as notches, not a hold.
    Reach,
    Jump,
    Sprint,
    /// Letting go of whatever is holding the input, such as a locked cursor.
    Release,
    Trigger(PointerKind),
    Grip(PointerKind),
    Menu(PointerKind),
}

impl Action {
    pub const ALL: [Self; 15] = [
        Self::Move,
        Self::Look,
        Self::Reach,
        Self::Jump,
        Self::Sprint,
        Self::Release,
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
    /// Derived from [`Self::ALL`] rather than hand-counted, so it cannot
    /// silently fall out of step with it the way a `4 + 3 * N` formula can.
    pub const COUNT: usize = Self::ALL.len();

    /// `self`'s slot: its position in [`Self::ALL`], found by scanning it
    /// rather than a hand-maintained offset formula, so a variant added here
    /// and to the enum cannot land on the wrong slot or collide with another.
    /// `every_action_has_its_own_slot` still checks the result has no
    /// collisions.
    const fn index(self) -> usize {
        let mut i = 0;
        while i < Self::ALL.len() {
            if self.same(Self::ALL[i]) {
                return i;
            }
            i += 1;
        }
        panic!("every Action is a member of Action::ALL")
    }

    /// Structural equality, usable from the `const fn` [`Self::index`]: a
    /// derived `PartialEq::eq` is not callable there.
    const fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Move, Self::Move)
            | (Self::Look, Self::Look)
            | (Self::Reach, Self::Reach)
            | (Self::Jump, Self::Jump)
            | (Self::Sprint, Self::Sprint)
            | (Self::Release, Self::Release) => true,
            (Self::Trigger(a), Self::Trigger(b))
            | (Self::Grip(a), Self::Grip(b))
            | (Self::Menu(a), Self::Menu(b)) => a.index() == b.index(),
            _ => false,
        }
    }

    /// Whether the action carries a direction/strength rather than a press.
    /// An axis action's `pressed` is never read.
    const fn is_axis(self) -> bool {
        matches!(self, Self::Move | Self::Look | Self::Reach)
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
    /// released: what was held reaches its readers as the release it would
    /// have got anyway, rather than sticking down behind whatever took the
    /// input.
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
    fn capture_lets_go_of_everything_the_frame_was_holding() {
        let mut state = ActionState::default();
        let grip = Action::Grip(PointerKind::Screen);
        let tuning = Tuning::default();

        state.begin_frame();
        state.press(grip, 1.0);
        state.end_frame(&tuning);
        assert!(state.pressed(grip));

        state.begin_frame();
        state.press(grip, 1.0);
        state.accumulate(Action::Move, Vec2::Y);
        state.accumulate_delta(Action::Look, Vec2::X);
        state.silence();
        state.end_frame(&tuning);

        assert!(
            state.just_released(grip),
            "a grab must let go rather than stick down behind whatever took \
             the input"
        );
        assert!(!state.pressed(grip));
        assert_eq!(state.axis(Action::Move), Vec2::ZERO);
        assert_eq!(state.delta(Action::Look), Vec2::ZERO);
    }

    /// `Reach` is an axis: a wheel notch is a one-frame travel like a mouse
    /// delta, so it must not be clamped away or read as a press.
    #[test]
    fn reach_is_an_axis_fed_by_delta_and_never_a_press() {
        let mut state = ActionState::default();
        state.begin_frame();
        state.accumulate_delta(Action::Reach, Vec2::new(0.0, 3.0));
        state.end_frame(&Tuning::default());

        assert_eq!(state.delta(Action::Reach), Vec2::new(0.0, 3.0));
        assert!(!state.pressed(Action::Reach));
    }

    /// `Release` is an ordinary button, silenced like everything else once
    /// something else captures the input.
    #[test]
    fn release_is_a_button_silenced_under_capture() {
        let mut state = ActionState::default();
        let tuning = Tuning::default();

        state.begin_frame();
        state.press(Action::Release, 1.0);
        state.end_frame(&tuning);
        assert!(state.just_pressed(Action::Release));

        state.begin_frame();
        state.press(Action::Release, 1.0);
        state.silence();
        state.end_frame(&tuning);
        assert!(
            state.just_released(Action::Release),
            "capture must let go of Release rather than hold it pressed"
        );
    }
}
