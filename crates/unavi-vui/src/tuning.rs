use serde::{
    Deserialize,
    Serialize,
};

/// Every feel constant in one place.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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

    pub attend_scale:     f32,
    pub seize_scale:      f32,
    /// Seconds of unbroken attention before a placard appears.
    pub placard_delay:    f32,
    /// Seconds the placard takes to reach full opacity once the delay has
    /// passed.
    pub placard_fade:     f32,
    /// Clearance between the attended mote's surface and the placard's lower
    /// edge.
    pub placard_gap:      f32,
    /// How far in front of its mote the placard stands.
    pub placard_lift:     f32,
    pub placard_title:    f32,
    pub placard_row:      f32,
    /// Baseline-to-baseline spacing, as a multiple of the row size.
    pub placard_line:     f32,
    pub placard_pad:      f32,
    pub placard_width:    f32,
    /// Average glyph width, in ems, that placard wrapping assumes.
    pub advance_estimate: f32,

    /// Em height of the name drawn under every mote.
    pub label_size: f32,
    /// Clearance between a mote's surface and its label.
    pub label_gap:  f32,
    /// How far in front of its mote a label sits.
    pub label_lift: f32,

    /// Metres the attended mote reaches toward the pointer.
    pub lean_dist:  f32,
    /// Distance at which lean has fallen off to nothing.
    pub lean_range: f32,
    pub lean_speed: f32,

    /// Pointer travel past which a release places rather than taps.
    pub seize_threshold: f32,

    /// Role-driven body sizes, applied before attention scaling.
    pub group_scale:  f32,
    pub action_scale: f32,
    pub parent_scale: f32,
    /// Most interior bodies a group draws; beyond this it reports overflow.
    pub pip_cap:      usize,
}

impl Tuning {
    pub const DEFAULT: Self = Self {
        orbit_radius: 0.18,
        mote_radius:  0.032,
        centre_frac:  0.34,
        reach_frac:   1.9,
        field_lift:   0.09,
        stick:        0.18,

        attend_scale:     1.18,
        seize_scale:      1.30,
        placard_delay:    0.25,
        placard_fade:     0.12,
        placard_gap:      0.018,
        placard_lift:     0.012,
        placard_title:    0.016,
        placard_row:      0.011,
        placard_line:     1.5,
        placard_pad:      0.013,
        placard_width:    0.24,
        advance_estimate: 0.62,

        label_size: 0.011,
        label_gap:  0.007,
        label_lift: 0.004,

        lean_dist:  0.022,
        lean_range: 0.35,
        lean_speed: 12.0,

        seize_threshold: 0.04,

        group_scale:  1.45,
        action_scale: 1.0,
        parent_scale: 0.8,
        pip_cap:      7,
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
    fn a_label_reads_smaller_than_the_placard_it_expands_into() {
        let tuning = Tuning::DEFAULT;
        assert!(tuning.label_size < tuning.placard_title);
        assert!(tuning.placard_row < tuning.placard_title);
    }
}
