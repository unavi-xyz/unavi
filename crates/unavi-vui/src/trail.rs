use arrayvec::ArrayVec;
use wired_math::types::Vec3;

use crate::{
    attention::Attention,
    mote::{
        MoteSpec,
        Role,
    },
    palette::Palette,
    tuning::Tuning,
    view::Style,
};

/// Levels the stack draws before collapsing the rest into a count.
pub const MAX_BEADS: usize = 5;

/// One level above the parent mote, drawn as a bead behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrailBead {
    /// Surface-local. Depth in the tree is depth in space, so the stack runs
    /// out of the plane toward the viewer.
    pub position: Vec3,
    pub radius:   f32,
    pub style:    Style,
    /// The level this bead stands for; selecting it climbs there.
    pub depth:    usize,
}

/// The breadcrumb: parent motes stacked along the tether. There is no separate
/// widget, no back button and no forward button.
#[derive(Debug, Clone, PartialEq)]
pub struct TrailView {
    pub beads:  ArrayVec<TrailBead, MAX_BEADS>,
    /// Levels above the stack that it does not show.
    pub hidden: usize,
}

/// Lays out the levels above the parent mote, nearest first, from
/// [`crate::tree::Tree::trail`].
#[must_use]
pub fn view(specs: &[MoteSpec], palette: &Palette, tuning: &Tuning) -> TrailView {
    let mut beads = ArrayVec::new();
    let mut taper = 1.0;
    for (step, spec) in specs.iter().take(MAX_BEADS).enumerate() {
        taper *= tuning.trail_taper;
        beads.push(TrailBead {
            position: Vec3::new(0.0, 0.0, (step + 1) as f32 * tuning.trail_pitch),
            radius:   tuning.mote_radius * tuning.parent_scale * taper,
            style:    Style {
                color:    palette.dim,
                alpha:    palette.solid_alpha * taper,
                emissive: palette.emissive(Attention::Idle) * taper,
            },
            depth:    depth_of(spec, step, specs.len()),
        });
    }
    TrailView {
        beads,
        hidden: specs.len().saturating_sub(MAX_BEADS),
    }
}

/// A trail spec carries its own level in its role; the stack position is the
/// fallback for anything that does not.
const fn depth_of(spec: &MoteSpec, step: usize, total: usize) -> usize {
    match spec.role {
        Role::Parent { depth } => depth,
        _ => total.saturating_sub(step + 1),
    }
}

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;

    use super::*;

    fn specs(count: usize) -> Vec<MoteSpec> {
        (0..count)
            .map(|index| MoteSpec {
                role:        Role::Parent {
                    depth: count - index - 1,
                },
                label:       SmolStr::new(format!("level{index}")),
                description: None,
            })
            .collect()
    }

    fn view_of(count: usize) -> TrailView {
        view(&specs(count), &Palette::DEFAULT, &Tuning::DEFAULT)
    }

    #[test]
    fn the_root_level_draws_no_stack() {
        let trail = view_of(0);
        assert!(trail.beads.is_empty());
        assert_eq!(trail.hidden, 0);
    }

    #[test]
    fn the_stack_runs_out_of_the_plane_toward_the_viewer() {
        let trail = view_of(3);
        assert_eq!(trail.beads.len(), 3);
        assert!(trail.beads[0].position.z > 0.0);
        assert!(
            trail.beads[1].position.z > trail.beads[0].position.z,
            "depth in the tree is literal depth in space"
        );
        assert!(trail.beads.iter().all(|bead| bead.position.x == 0.0));
    }

    #[test]
    fn beads_recede_rather_than_competing_with_the_level() {
        let trail = view_of(3);
        assert!(trail.beads[1].radius < trail.beads[0].radius);
        assert!(trail.beads[1].style.alpha < trail.beads[0].style.alpha);
        assert_eq!(trail.beads[0].style.color, Palette::DEFAULT.dim);
    }

    #[test]
    fn a_deep_stack_collapses_the_remainder_into_a_count() {
        let trail = view_of(MAX_BEADS + 4);
        assert_eq!(trail.beads.len(), MAX_BEADS);
        assert_eq!(trail.hidden, 4, "unbounded depth, bounded stack");
    }

    #[test]
    fn every_bead_says_which_level_it_climbs_to() {
        let trail = view_of(3);
        let depths = trail.beads.iter().map(|bead| bead.depth).collect::<Vec<_>>();
        assert_eq!(depths, vec![2, 1, 0], "nearest first, descending to root");
    }
}
