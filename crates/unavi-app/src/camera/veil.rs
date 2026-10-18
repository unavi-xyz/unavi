//! The fog veil: a scalar that thickens the cameras' distance fog until the
//! world is hidden in sky colour, so a change of scene never cuts.

use bevy::prelude::*;

use super::{
    FOG_END,
    FOG_START,
    sky::blend::SkyBlend,
};

/// Seconds the world takes to disappear into the fog.
pub const COVER: f32 = 0.35;
/// Seconds it takes to come back, longer so an arrival settles gently.
pub const REVEAL: f32 = 0.7;

/// Fog distances at full cover. The halo sits at 0.7 m, so it stays legible
/// while everything past arm's length melts into the sky.
const VEIL_START: f32 = 0.8;
const VEIL_END: f32 = 2.5;

/// How hidden the world is, from 0 (clear) to 1 (all fog).
#[derive(Resource, Clone, Copy, Debug)]
pub struct Veil {
    /// Linear position along the ease, which the level is a smoothstep of.
    /// Reversing walks it back from where it is, so the level never jumps.
    progress: f32,
    covering: bool,
}

impl Veil {
    #[must_use]
    pub const fn covered() -> Self {
        Self {
            progress: 1.0,
            covering: true,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub const fn revealed() -> Self {
        Self {
            progress: 0.0,
            covering: false,
        }
    }

    pub const fn cover(&mut self) {
        self.covering = true;
    }

    pub const fn reveal(&mut self) {
        self.covering = false;
    }

    /// Hides the world at once, for a drop that cannot wait for an ease.
    pub const fn snap_covered(&mut self) {
        *self = Self::covered();
    }

    /// Whether the world is fully hidden.
    #[must_use]
    pub fn is_covered(self) -> bool {
        self.progress >= 1.0
    }

    #[must_use]
    pub fn level(self) -> f32 {
        let p = self.progress;
        p * p * 2.0_f32.mul_add(-p, 3.0)
    }

    pub fn step(&mut self, dt: f32) {
        let rate = if self.covering {
            dt / COVER
        } else {
            -dt / REVEAL
        };
        self.progress = (self.progress + rate).clamp(0.0, 1.0);
    }

    /// The fog distances the cameras draw with at this level.
    #[must_use]
    pub fn falloff(self) -> FogFalloff {
        let level = self.level();
        FogFalloff::Linear {
            start: (VEIL_START - FOG_START).mul_add(level, FOG_START),
            end:   (VEIL_END - FOG_END).mul_add(level, FOG_END),
        }
    }
}

impl Default for Veil {
    /// Starts covered, hiding the first frames while the scene assembles.
    fn default() -> Self {
        Self::covered()
    }
}

pub fn reveal_veil(mut veil: ResMut<Veil>) {
    veil.reveal();
}

/// Advances the veil and hands it to every camera's fog, and the sky's fog
/// colour to the clear colour so frames drawn before pipelines compile show
/// sky instead of black.
pub fn apply_veil(
    time: Res<Time>,
    sky: Res<SkyBlend>,
    mut veil: ResMut<Veil>,
    mut clear: ResMut<ClearColor>,
    mut fogs: Query<&mut DistanceFog>,
) {
    veil.step(time.delta_secs());

    let falloff = veil.falloff();
    for mut fog in &mut fogs {
        fog.falloff = falloff.clone();
    }
    clear.0 = sky.palette().fog().into();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_only_after_the_cover_time() {
        let mut veil = Veil::revealed();
        veil.cover();

        veil.step(COVER * 0.99);
        assert!(!veil.is_covered(), "covered before the full time");
        assert!(veil.level() > 0.0 && veil.level() < 1.0);

        veil.step(COVER * 0.5);
        assert!(veil.is_covered());
        assert!((veil.level() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn revealing_takes_the_reveal_time() {
        let mut veil = Veil::covered();
        veil.reveal();

        veil.step(REVEAL * 0.99);
        assert!(veil.level() > 0.0);

        veil.step(REVEAL * 0.5);
        assert!(veil.level().abs() < f32::EPSILON);
    }

    #[test]
    fn reveal_mid_cover_reverses_without_a_jump() {
        let mut veil = Veil::revealed();
        veil.cover();
        veil.step(COVER * 0.6);
        let before = veil.level();

        veil.reveal();
        assert!((veil.level() - before).abs() < f32::EPSILON);

        veil.step(0.05);
        let after = veil.level();
        assert!(after < before, "level kept rising: {before} -> {after}");
        assert!(before - after < 0.2, "level jumped: {before} -> {after}");
        assert!(!veil.is_covered());
    }

    #[test]
    fn snap_covered_hides_at_once() {
        let mut veil = Veil::revealed();
        veil.snap_covered();
        assert!(veil.is_covered());
    }

    #[test]
    fn falloff_runs_from_the_world_fog_to_the_veil_fog() {
        let FogFalloff::Linear { start, end } = Veil::revealed().falloff() else {
            panic!("not linear");
        };
        assert!((start - FOG_START).abs() < 1.0e-3 && (end - FOG_END).abs() < 1.0e-3);

        let FogFalloff::Linear { start, end } = Veil::covered().falloff() else {
            panic!("not linear");
        };
        assert!((start - VEIL_START).abs() < 1.0e-3 && (end - VEIL_END).abs() < 1.0e-3);
    }
}
