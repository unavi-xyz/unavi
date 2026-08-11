use serde::{
    Deserialize,
    Serialize,
};

/// Every feel constant in one place.
///
/// A running example can edit them and the tuned result ships back as
/// [`Tuning::DEFAULT`]. Answering "does 4 cm feel right?" should be a slider
/// in the headset, not a rebuild.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    pub orbit_radius: f32,
    pub mote_radius:  f32,
    /// Centre-slot target radius, as a fraction of the orbit radius.
    pub centre_frac:  f32,
    /// Nothing is attended past this multiple of the orbit radius.
    pub reach_frac:   f32,
    /// Extra angular half-width granted to the slot already attended, so a
    /// sweep does not flicker between neighbours.
    pub stick:        f32,

    pub attend_scale:  f32,
    pub seize_scale:   f32,
    /// Seconds of unbroken attention before a placard appears. The mote
    /// itself reacts immediately; only the text waits.
    pub placard_delay: f32,

    /// Metres the attended mote reaches toward the pointer.
    pub lean_dist:  f32,
    /// Distance at which lean has fallen off to nothing.
    pub lean_range: f32,
    pub lean_speed: f32,

    /// Pointer travel past which a release places rather than taps.
    pub seize_threshold: f32,

    pub open_speed:  f32,
    pub raise_speed: f32,
    /// Seconds between successive motes arriving when an orbit opens.
    pub stagger:     f32,

    /// Role-driven body sizes, applied before any attention scaling so a
    /// container reads as a container without being pointed at.
    pub branch_scale: f32,
    pub leaf_scale:   f32,
    pub parent_scale: f32,
    /// Most interior bodies a branch draws. Beyond this the shell reports
    /// overflow rather than showing a count that does not match reality.
    pub pip_cap:      usize,
    /// Collider size relative to the resting body. Generous on purpose: it
    /// covers the mote at its grown size and gives the pointer something
    /// forgiving to land on. Steady across a hover, so it is written only
    /// when a mote's role changes.
    pub hit_scale:    f32,

    /// Child count at which a branch's shell reads as completely full.
    pub fill_saturation: f32,
    /// Angular size (radians) below which only a silhouette is drawn.
    pub detail_min:      f32,
    /// Angular size at which contents are fully resolved.
    pub detail_full:     f32,
    /// Detail granted to the attended mote regardless of its angular size.
    pub detail_attend:   f32,
}

impl Tuning {
    pub const DEFAULT: Self = Self {
        orbit_radius: 0.18,
        mote_radius:  0.032,
        centre_frac:  0.34,
        reach_frac:   1.9,
        stick:        0.18,

        attend_scale:  1.18,
        seize_scale:   1.30,
        placard_delay: 0.25,

        lean_dist:  0.022,
        lean_range: 0.35,
        lean_speed: 12.0,

        seize_threshold: 0.04,

        open_speed:  7.0,
        raise_speed: 12.0,
        stagger:     0.015,

        branch_scale: 1.45,
        leaf_scale:   1.0,
        parent_scale: 0.8,
        pip_cap:      7,
        hit_scale:    1.6,

        fill_saturation: 12.0,
        detail_min:      0.035,
        detail_full:     0.14,
        detail_attend:   0.45,
    };
}

impl Default for Tuning {
    fn default() -> Self {
        Self::DEFAULT
    }
}
