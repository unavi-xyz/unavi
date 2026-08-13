//! How a surface arrives and how it goes away.
//!
//! Two motions at once, and they are not the same motion:
//!
//! - the **form** draws in toward its anchor, so a surface collapses to the
//!   point it was summoned at rather than fading where it stands;
//! - each **mote** scales up in turn, staggered, so what arrives reads as
//!   objects settling rather than as a panel popping in. Simultaneous arrival
//!   is the thing that makes an interface look like a UI.
//!
//! Both are pure: the surface multiplies its slot positions by [`Bloom::form`]
//! and hands each slot its own [`Bloom::slot`], and the binding transcribes.

use crate::tuning::Tuning;

/// How far a surface has opened, and how far each of its motes has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bloom {
    /// 0 shut, 1 fully open.
    t:    f32,
    /// Which way it is travelling.
    open: bool,
}

impl Default for Bloom {
    fn default() -> Self {
        Self::SHUT
    }
}

impl Bloom {
    pub const SHUT: Self = Self {
        t:    0.0,
        open: false,
    };

    pub const OPEN: Self = Self {
        t:    1.0,
        open: true,
    };

    /// Shut, and on its way open. Where a surface starts: one is put up to be
    /// seen, and arriving is how it gets there.
    pub const ARRIVING: Self = Self {
        t:    0.0,
        open: true,
    };

    pub const fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Whether anything is still drawn. A shut surface that has finished
    /// closing needs no more frames.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.open || self.t > 0.0
    }

    pub fn update(&mut self, delta: f32, tuning: &Tuning) {
        let step = delta * tuning.bloom_speed;
        self.t = if self.open {
            (self.t + step).min(1.0)
        } else {
            (self.t - step).max(0.0)
        };
    }

    /// How far the form itself has come in, which its slot positions scale by.
    ///
    /// No overshoot here: the ring's radius springing past where it settles
    /// reads as a wobble, where the same overshoot on a single body reads as
    /// weight.
    #[must_use]
    pub const fn form(&self) -> f32 {
        self.t
    }

    /// How far the mote in `slot` of `count` has arrived.
    ///
    /// Later slots start later, so the level fills in order, and each lands
    /// with a slight overshoot.
    #[must_use]
    pub fn slot(&self, slot: usize, count: usize, tuning: &Tuning) -> f32 {
        let stagger = tuning.bloom_stagger;
        // The whole level still finishes at t = 1, so the stagger stretches
        // the ramp rather than delaying the end.
        let window = stagger.mul_add(count.saturating_sub(1) as f32, 1.0);
        let start = slot as f32 * stagger / window;
        let span = 1.0 / window;

        let local = ((self.t - start) / span).clamp(0.0, 1.0);
        overshoot(local, tuning.bloom_overshoot)
    }
}

/// Eases to 1 having gone a little past it, so a mote lands rather than
/// stopping dead.
fn overshoot(t: f32, amount: f32) -> f32 {
    let back = t - 1.0;
    (amount + 1.0).mul_add(back * back * back, amount.mul_add(back * back, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLOTS: usize = 5;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn opened() -> Bloom {
        let mut bloom = Bloom::SHUT;
        bloom.set_open(true);
        bloom
    }

    /// Runs a bloom to a standstill, so a test asserts about where it settles
    /// rather than about one frame of it.
    fn settle(bloom: &mut Bloom) {
        for _ in 0..1000 {
            bloom.update(1.0 / 60.0, &tuning());
        }
    }

    #[test]
    fn a_shut_surface_draws_nothing_and_needs_no_frames() {
        let bloom = Bloom::SHUT;
        assert!(!bloom.is_visible());
        assert!(bloom.form().abs() < 1.0e-6);
        for slot in 0..SLOTS {
            assert!(bloom.slot(slot, SLOTS, &tuning()).abs() < 1.0e-6);
        }
    }

    #[test]
    fn opening_settles_fully_open() {
        let mut bloom = opened();
        settle(&mut bloom);
        assert!((bloom.form() - 1.0).abs() < 1.0e-6);
        for slot in 0..SLOTS {
            assert!((bloom.slot(slot, SLOTS, &tuning()) - 1.0).abs() < 1.0e-5);
        }
    }

    #[test]
    fn closing_settles_shut_and_then_stops_asking_for_frames() {
        let mut bloom = Bloom::OPEN;
        bloom.set_open(false);
        assert!(bloom.is_visible(), "still on its way out");
        settle(&mut bloom);
        assert!(bloom.form().abs() < 1.0e-6);
        assert!(!bloom.is_visible());
    }

    #[test]
    fn the_form_draws_in_toward_its_anchor() {
        let mut bloom = opened();
        bloom.update(1.0 / 60.0, &tuning());
        let part_way = bloom.form();
        assert!(
            part_way > 0.0 && part_way < 1.0,
            "slot positions scale by this, so a partly open form stands closer \
             in than a settled one"
        );
    }

    #[test]
    fn earlier_slots_arrive_first() {
        let mut bloom = opened();
        // Far enough in that the first slot has moved and the last has not
        // caught up.
        for _ in 0..3 {
            bloom.update(1.0 / 60.0, &tuning());
        }
        let first = bloom.slot(0, SLOTS, &tuning());
        let last = bloom.slot(SLOTS - 1, SLOTS, &tuning());
        assert!(
            first > last,
            "staggered arrival is what reads as settling rather than popping \
             in; {first} vs {last}"
        );
    }

    #[test]
    fn a_lone_slot_has_nothing_to_stagger_against() {
        let mut bloom = opened();
        bloom.update(1.0 / 60.0, &tuning());
        assert!(bloom.slot(0, 1, &tuning()) > 0.0);
    }

    #[test]
    fn a_mote_lands_past_its_size_before_settling_on_it() {
        let past = (0..100)
            .map(|step| overshoot(step as f32 / 100.0, Tuning::DEFAULT.bloom_overshoot))
            .fold(0.0_f32, f32::max);
        assert!(
            past > 1.0,
            "an ease that only approaches its target reads as stopping dead"
        );
    }

    #[test]
    fn the_ease_starts_where_it_is_and_ends_where_it_is_going() {
        let amount = Tuning::DEFAULT.bloom_overshoot;
        assert!(overshoot(0.0, amount).abs() < 1.0e-6);
        assert!((overshoot(1.0, amount) - 1.0).abs() < 1.0e-6);
    }
}
