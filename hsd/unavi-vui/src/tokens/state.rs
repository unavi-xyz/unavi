//! The states a mote is drawn in: how much attention it has, and the
//! condition it is in.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Attention {
    #[default]
    Idle,
    /// As warm as a mote gets without being chosen. Heat climbs toward this
    /// by how near the pointer has come; the tracker never reports it,
    /// because being approached is a matter of degree and this is only where
    /// it tops out.
    Near,
    /// The best candidate for a pointer.
    Attended,
    /// Grasped.
    Engaged,
}

impl Attention {
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Attended | Self::Engaged)
    }

    /// Where this state sits on the continuous scale a surface eases along.
    ///
    /// Attention is graded, so what a mote *reads* as has to be too: a body
    /// that jumps to its attended size the frame a pointer crosses it reads as
    /// a widget lighting up, where one that arrives there reads as a thing
    /// noticing. The state itself stays discrete — targeting, grasp and the
    /// event stream all turn on which mote is attended, not on how far.
    #[must_use]
    pub const fn heat(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Near => 0.22,
            Self::Attended => 0.62,
            Self::Engaged => 1.0,
        }
    }
}

/// What a mote is regardless of where the pointer is, so a mote can be active
/// and attended at once. Drawn through the palette alongside [`Attention`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the flags are independent: a mote can be active and busy at once"
)]
pub struct Condition {
    /// On and staying on: a toggle held until something clears it.
    pub active:   bool,
    /// Working.
    pub busy:     bool,
    /// Cannot be used now.
    pub disabled: bool,
    /// Something went wrong, and a notice says so.
    pub failed:   bool,
}

/// The two states `heat` falls between, and how far between them it is.
///
/// Everything attention-driven is sampled through this rather than
/// interpolated field by field, so a state's own values stay exactly what
/// [`Attention`] says they are at its own heat, and role-specific rules — a
/// parent mote receding, a container's glass, an item's warmth — keep working
/// without any of them learning about heat.
#[must_use]
pub fn bracket(heat: f32) -> (Attention, Attention, f32) {
    const STEPS: [Attention; 4] = [
        Attention::Idle,
        Attention::Near,
        Attention::Attended,
        Attention::Engaged,
    ];

    let heat = heat.clamp(0.0, 1.0);
    for pair in STEPS.windows(2) {
        let (low, high) = (pair[0], pair[1]);
        if heat <= high.heat() {
            let span = high.heat() - low.heat();
            let t = if span > f32::EPSILON {
                (heat - low.heat()) / span
            } else {
                0.0
            };
            return (low, high, t.clamp(0.0, 1.0));
        }
    }
    (Attention::Engaged, Attention::Engaged, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATES: [Attention; 4] = [
        Attention::Idle,
        Attention::Near,
        Attention::Attended,
        Attention::Engaged,
    ];

    #[test]
    fn heat_rises_with_attention_and_spans_the_whole_scale() {
        for pair in STATES.windows(2) {
            assert!(
                pair[0].heat() < pair[1].heat(),
                "{:?} must sit below {:?}",
                pair[0],
                pair[1]
            );
        }
        assert!(Attention::Idle.heat().abs() < f32::EPSILON);
        assert!((Attention::Engaged.heat() - 1.0).abs() < f32::EPSILON);
    }

    /// The property everything downstream leans on: at a state's own heat the
    /// bracket collapses onto that state, so sampling a settled mote returns
    /// exactly what the state says and nothing is rewritten.
    #[test]
    fn a_states_own_heat_brackets_onto_that_state() {
        for state in STATES {
            let (low, high, t) = bracket(state.heat());
            let landed = if t >= 1.0 { high } else { low };
            assert_eq!(landed, state, "{state:?} did not land on itself");
            assert!(t <= 0.0 || t >= 1.0, "{state:?} landed mid-blend at {t}");
        }
    }

    #[test]
    fn a_heat_between_two_states_brackets_between_them() {
        let (low, high, t) = bracket(0.42);
        assert_eq!((low, high), (Attention::Near, Attention::Attended));
        assert!(t > 0.0 && t < 1.0);
    }

    #[test]
    fn heat_outside_the_scale_is_clamped_rather_than_extrapolated() {
        for (heat, expected) in [(-5.0, Attention::Idle), (5.0, Attention::Engaged)] {
            let (low, high, t) = bracket(heat);
            let landed = if t >= 1.0 { high } else { low };
            assert_eq!(landed, expected);
        }
    }
}
