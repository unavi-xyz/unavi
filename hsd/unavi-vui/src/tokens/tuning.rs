//! Every feel constant VUI animates by, in one place.

use wired_guest::style;

/// Metres from the eye at which a mote's label is read.
const READING: f32 = 0.7;
/// Metres at which a placard on its mote is read.
const PLACARD_READING: f32 = 0.57;
/// Metres at which a standing pane is read, since it labels something in the
/// room rather than being worked at arm's length.
const STAND_READING: f32 = 1.4;
/// Metres from the eye at which a pane's body type is read.
const PANE_READING: f32 = 0.85;

/// The em height in metres of a type step given in milliradians, read from
/// `distance` metres.
const fn em(mrad: f32, distance: f32) -> f32 {
    mrad * 0.001 * distance
}

/// Every feel constant in one place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tuning {
    pub orbit_radius: f32,
    pub mote_radius:  f32,
    /// Centre-slot target radius, as a fraction of the orbit radius.
    pub centre_frac:  f32,
    /// Nothing is attended past this multiple of the orbit radius. Also sizes
    /// the orbit's hit surface.
    pub reach_frac:   f32,
    /// How far the hit surface stands in front of the orbit. Must clear
    /// `mote_radius * group_scale * seize_scale + lean_dist`.
    pub field_lift:   f32,
    /// Extra angular half-width granted to the slot already attended.
    pub stick:        f32,
    /// The same hysteresis for a grid, as a fraction of a cell.
    pub grid_stick:   f32,
    /// Centre-to-centre spacing of a grid's cells.
    pub grid_pitch:   f32,
    /// A grid level: columns by rows cells on one page.
    pub grid_columns: usize,
    pub grid_rows:    usize,
    /// How far a page tab stands past the outermost slot of its layout.
    pub tab_gap:      f32,
    /// How near the pointer must come to a page tab to attend it.
    pub tab_hit:      f32,

    pub attend_scale:  f32,
    pub seize_scale:   f32,
    /// Seconds of unbroken attention before a placard appears.
    pub placard_delay: f32,
    /// Clearance between the attended mote's surface and the placard's lower
    /// edge.
    pub placard_gap:   f32,
    /// How far in front of its mote the placard stands.
    pub placard_lift:  f32,
    /// Metres in one em of a placard's type.
    pub placard_em:    f32,
    /// Thickness of the bar tethering a standing pane to what it labels.
    pub tether_width:  f32,
    /// Radius of the dot a tether ends in.
    pub tether_dot:    f32,
    /// How fast a standing pane eases toward where its script puts it, per
    /// second.
    pub stand_speed:   f32,

    /// Seconds a notice stays on its mote.
    pub notice_duration: f32,

    /// Em height of the name drawn under every mote.
    pub label_size: f32,
    /// Clearance between a mote's surface and its label.
    pub label_gap:  f32,
    /// How far in front of its mote a label sits.
    pub label_lift: f32,

    /// Seconds a consequential action fills for before it fires.
    pub cast_duration: f32,
    /// Full rings per second an abandoned cast unwinds, which is faster than
    /// it filled: letting go is a snap back, not a rewind.
    pub cast_recoil:   f32,

    /// How fast a surface opens and closes, in full reveals per second.
    pub reveal_speed:     f32,
    /// How far each slot lags the one before it, as a fraction of one slot's
    /// own ramp. Zero makes a level arrive all at once, which reads as a panel
    /// rather than as objects.
    pub reveal_stagger:   f32,
    /// How far past its size a mote lands before settling back onto it.
    pub reveal_overshoot: f32,

    /// How much of its brightness a mote keeps while another holds
    /// attention. The field receding is what makes the chosen one stand out;
    /// brightening it further only adds light to a form that is already lit.
    pub sibling_dim: f32,

    /// How fast a mote's look catches up to the attention it is under.
    pub heat_speed:  f32,
    /// How close to its target a mote's heat lands on exactly, so a settled
    /// surface stops redrawing itself.
    pub heat_settle: f32,

    /// Radians per second an icon turns inside its shell. Gentle by default;
    /// zero holds every icon still.
    pub icon_spin:     f32,
    /// An icon's diagonal, as a fraction of its shell's diameter. Every icon
    /// is measured and fit to this, so a ring of motes reads as one size
    /// whatever the content.
    pub icon_fill:     f32,
    /// Seconds a face takes to flip from one glyph to the next. The wipe
    /// across its columns is spread over this, so a longer flip is a slower
    /// sweep rather than a longer pause.
    pub flip_duration: f32,

    /// Radius of the readout drawn at an open orbit's centre. A good deal
    /// larger than a mote, so a display of thirteen dots to a side reads as a
    /// face to be looked at rather than as one more thing to point at: its
    /// dots come out about twice the size a glyph's do inside a shell, which
    /// is what lets two lines of text be read across an arm's length.
    pub readout_radius: f32,

    /// Metres the attended mote reaches toward the pointer.
    pub lean_dist:  f32,
    /// Distance at which lean has fallen off to nothing.
    pub lean_range: f32,
    pub lean_speed: f32,

    /// Pointer travel past which a release places rather than taps.
    pub seize_threshold: f32,

    /// The band a carried level opens in, in metres off the eye. It opens
    /// where it was let go, save for a height held between these two: a level
    /// dropped at the floor is one nobody can read, and one dropped overhead
    /// is one nobody can reach.
    pub land_low:  f32,
    pub land_high: f32,

    /// Role-driven body sizes, applied before attention scaling.
    pub group_scale:    f32,
    pub action_scale:   f32,
    pub parent_scale:   f32,
    /// A page tab's body, smaller than a mote so it reads as a control for the
    /// level rather than as part of it.
    pub tab_scale:      f32,
    /// A key in a bed, as a multiple of `mote_radius`: its half-width. Sized
    /// so a key grown by attention still clears its neighbour across the
    /// gap `grid_pitch` leaves.
    pub key_scale:      f32,
    /// How far down a key its name hangs, as a fraction of its half-width.
    pub key_label_drop: f32,
    /// Most interior bodies a group draws; beyond this it reports overflow.
    pub pip_cap:        usize,

    /// Metres in one em of a pane's body type. Every length a pane lays out
    /// is in ems, so this is the whole of its size.
    pub pane_em:   f32,
    /// The same for a pane that stands.
    pub stand_em:  f32,
    /// Metres its type and rules stand proud of the pane.
    pub pane_lift: f32,
}

impl Tuning {
    pub const DEFAULT: Self = Self {
        orbit_radius: 0.155,
        mote_radius:  0.032,
        centre_frac:  0.34,
        reach_frac:   1.9,
        field_lift:   0.09,
        stick:        0.18,
        grid_stick:   0.18,
        grid_pitch:   0.11,
        grid_columns: 3,
        grid_rows:    3,
        tab_gap:      0.12,
        tab_hit:      0.035,

        attend_scale:  1.18,
        seize_scale:   1.30,
        placard_delay: 0.25,
        placard_gap:   0.018,
        placard_lift:  0.012,
        placard_em:    em(style::MRAD_BODY, PLACARD_READING),
        tether_width:  0.004,
        tether_dot:    0.0125,
        stand_speed:   18.0,

        notice_duration: 3.0,

        label_size: em(style::MRAD_SMALL, READING),
        label_gap:  0.007,
        label_lift: 0.004,

        cast_duration: 0.9,
        cast_recoil:   4.0,

        reveal_speed:     6.5,
        reveal_stagger:   0.22,
        reveal_overshoot: 1.5,

        sibling_dim: 0.8,

        heat_speed:  14.0,
        heat_settle: 0.004,

        icon_spin:     0.15,
        icon_fill:     0.62,
        flip_duration: 0.3,

        readout_radius: 0.0512,

        lean_dist:  0.022,
        lean_range: 0.35,
        lean_speed: 12.0,

        seize_threshold: 0.04,

        land_low:  -0.85,
        land_high: 0.10,

        group_scale:    1.25,
        action_scale:   1.0,
        parent_scale:   0.8,
        tab_scale:      0.6,
        key_scale:      1.25,
        key_label_drop: 0.52,
        pip_cap:        9,

        pane_em:   em(style::MRAD_BODY, PANE_READING),
        stand_em:  em(style::MRAD_BODY, STAND_READING),
        pane_lift: 0.0015,
    };
}

impl Default for Tuning {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hit_surface_stands_clear_of_the_furthest_a_mote_reaches() {
        let tuning = Tuning::DEFAULT;
        let furthest = tuning
            .mote_radius
            .mul_add(tuning.group_scale * tuning.seize_scale, tuning.lean_dist);
        assert!(
            tuning.field_lift >= furthest,
            "a mote can reach {furthest} out of the plane but the surface the \
             reticle rides on is only {} in front of it, so the mote being \
             pointed at is the one that hides the reticle",
            tuning.field_lift
        );
    }

    #[test]
    fn the_readout_fills_the_centre_without_reaching_the_motes() {
        let tuning = Tuning::DEFAULT;
        assert!(
            tuning.readout_radius > tuning.mote_radius * 1.5,
            "larger than a mote, so it reads as a face"
        );
        let nearest = tuning.mote_radius.mul_add(
            -(tuning.group_scale * tuning.seize_scale),
            tuning.orbit_radius,
        );
        assert!(tuning.readout_radius < nearest, "clear of the ring");
    }

    #[test]
    fn text_sits_on_its_mote_rather_than_out_at_the_hit_surface() {
        let tuning = Tuning::DEFAULT;
        for lift in [tuning.placard_lift, tuning.label_lift] {
            assert!(lift > 0.0, "still clear of the body");
            assert!(
                lift < tuning.mote_radius,
                "text further out than a mote is wide stops reading as that \
                 mote's, and floats in front of the whole dial"
            );
        }
    }

    #[test]
    fn a_key_grown_by_attention_still_clears_its_neighbour() {
        let tuning = Tuning::DEFAULT;
        let grown = tuning.mote_radius * tuning.key_scale * tuning.seize_scale;
        assert!(
            grown * 2.0 < tuning.grid_pitch,
            "a pressed key is {} wide in a bed whose cells are {}",
            grown * 2.0,
            tuning.grid_pitch
        );
    }

    #[test]
    fn a_label_reads_smaller_than_the_placard_it_expands_into() {
        let tuning = Tuning::DEFAULT;
        assert!(tuning.label_size < tuning.placard_em);
    }

    #[test]
    fn a_pane_that_stands_reads_larger_than_one_that_floats() {
        let tuning = Tuning::DEFAULT;
        assert!(tuning.stand_em > tuning.pane_em);
    }
}
