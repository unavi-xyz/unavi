use smol_str::SmolStr;

use crate::{
    attention::Attention,
    tuning::Tuning,
};

/// Most pips a mote can draw. Sizing the array here keeps [`Pips`] `Copy` and
/// off the heap; [`Tuning::pip_cap`] is the tunable limit within it.
pub const MAX_PIPS: usize = 8;

/// What a mote stands for. Picks the silhouette and hue; the structural
/// question of whether it contains anything is [`Role`], not this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoteKind {
    /// A generic grouping with no subject of its own — a plain folder.
    Folder,
    Command,
    Document,
    Space,
    Person,
    Tool,
    Item,
    Result,
}

impl MoteKind {
    pub const COUNT: usize = 8;

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Command => 0,
            Self::Folder => 1,
            Self::Document => 2,
            Self::Space => 3,
            Self::Person => 4,
            Self::Tool => 5,
            Self::Item => 6,
            Self::Result => 7,
        }
    }
}

/// A mote's structural place in the tree. This, not [`MoteKind`], decides
/// size, transparency and what pips mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Contains nothing; activating it does something.
    Leaf,
    /// Contains other motes. `folders` of its `children` are themselves
    /// containers, which the pips show by drawing those see-through.
    Branch { children: usize, folders: usize },
    /// Consequential: opens a cast site rather than firing on tap.
    Cast,
    /// The way back to the level above, carrying how deep you currently are.
    /// Always slot 0 and never anywhere else, so it never moves.
    Parent { depth: usize },
}

/// Whether a mote can be pulled out of its orbit. Opt-in: most motes are
/// commands, and dragging one nowhere means nothing.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// Behaves like a button: never leaves its slot, and a release that has
    /// wandered off it cancels instead of activating.
    #[default]
    Fixed,
    Takeable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoteSpec {
    pub kind:     MoteKind,
    pub role:     Role,
    pub label:    SmolStr,
    pub grab:     Grab,
    /// Whether the mote carries a body of its own — a tool's model — to draw
    /// instead of a kind silhouette.
    pub embodied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipKind {
    /// A child that holds nothing: drawn solid.
    Leaf,
    /// A child that is itself a container: drawn see-through, the same rule
    /// its own mote follows one level down.
    Branch,
}

/// Where a mote's pips sit, and the whole of the up-versus-down distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipPlacement {
    /// Contents, drawn within the body: "what is in here".
    Inside,
    /// Depth marks, drawn as a ring around the body: "how far you came".
    /// Structural rather than decorative — inside means descending, around
    /// means ascending, and no colour has to carry it.
    Around,
}

/// A mote's pip summary. Counts are real, never a density: a preview that
/// does not match what opening it yields is worse than no preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pips {
    pub kinds:     [PipKind; MAX_PIPS],
    pub count:     usize,
    /// More than `count` can show, so the shell says "and more" rather than
    /// reporting a number that is wrong.
    pub overflow:  bool,
    pub placement: PipPlacement,
}

impl Pips {
    pub const NONE: Self = Self {
        kinds:     [PipKind::Leaf; MAX_PIPS],
        count:     0,
        overflow:  false,
        placement: PipPlacement::Inside,
    };

    /// How many leading pips are containers. Pips are ordered containers
    /// first, so a renderer can draw two runs rather than tracking each one.
    #[must_use]
    pub fn branches(&self) -> usize {
        self.kinds
            .iter()
            .take(self.count)
            .take_while(|kind| matches!(kind, PipKind::Branch))
            .count()
    }
}

/// How a mote's body is drawn.
///
/// A container is a bubble you can see into; a thing that is already
/// recognizable is drawn as itself, because wrapping a tool in a sphere only
/// makes it smaller and harder to identify.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bare,
    Bubble { fill: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Presentation {
    pub shell:      Shell,
    pub radius:     f32,
    /// Collider radius. Independent of attention so it is not rewritten every
    /// frame of a hover, and larger than the resting body so it covers the
    /// grown one and forgives a near miss.
    pub hit_radius: f32,
    /// 0 draws a silhouette only; 1 resolves the contents. A function of
    /// angular size and attention, so detail arrives on approach.
    pub detail:     f32,
    pub pips:       Pips,
}

/// Apparent angular size of a body of `radius` seen from `distance`.
#[must_use]
pub fn angular_size(radius: f32, distance: f32) -> f32 {
    if distance <= f32::EPSILON {
        return std::f32::consts::PI;
    }
    2.0 * (radius / distance).atan()
}

fn contents(children: usize, folders: usize, tuning: &Tuning) -> Pips {
    let cap = tuning.pip_cap.min(MAX_PIPS);
    let count = children.min(cap);
    let folders = folders.min(count);
    let mut kinds = [PipKind::Leaf; MAX_PIPS];
    for kind in kinds.iter_mut().take(folders) {
        *kind = PipKind::Branch;
    }
    Pips {
        kinds,
        count,
        overflow: children > cap,
        placement: PipPlacement::Inside,
    }
}

fn depth_marks(depth: usize, tuning: &Tuning) -> Pips {
    let cap = tuning.pip_cap.min(MAX_PIPS);
    Pips {
        kinds:     [PipKind::Leaf; MAX_PIPS],
        count:     depth.min(cap),
        overflow:  depth > cap,
        placement: PipPlacement::Around,
    }
}

#[must_use]
pub fn present(
    spec: &MoteSpec,
    distance: f32,
    attention: Attention,
    tuning: &Tuning,
) -> Presentation {
    let attention_scale = match attention {
        Attention::Engaged => tuning.seize_scale,
        Attention::Attended => tuning.attend_scale,
        Attention::Idle | Attention::Near => 1.0,
    };
    // Role drives size before attention does, so a container reads as bigger
    // than a leaf at any distance and without being pointed at.
    let role_scale = match spec.role {
        Role::Branch { .. } => tuning.branch_scale,
        Role::Leaf | Role::Cast => tuning.leaf_scale,
        Role::Parent { .. } => tuning.parent_scale,
    };
    let resting = tuning.mote_radius * role_scale;
    let hit_radius = resting * tuning.hit_scale;
    let radius = resting * attention_scale;

    let (shell, pips) = match spec.role {
        Role::Branch { children, folders } => (
            Shell::Bubble {
                fill: (children as f32 / tuning.fill_saturation).clamp(0.0, 1.0),
            },
            contents(children, folders, tuning),
        ),
        Role::Parent { depth } => (Shell::Bare, depth_marks(depth, tuning)),
        Role::Leaf | Role::Cast => (Shell::Bare, Pips::NONE),
    };

    let span = tuning.detail_full - tuning.detail_min;
    let base = if span <= f32::EPSILON {
        0.0
    } else {
        ((angular_size(radius, distance) - tuning.detail_min) / span).clamp(0.0, 1.0)
    };
    let boost = if attention.is_active() {
        tuning.detail_attend
    } else {
        0.0
    };

    Presentation {
        shell,
        radius,
        hit_radius,
        detail: (base + boost).clamp(0.0, 1.0),
        pips,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn spec(role: Role) -> MoteSpec {
        MoteSpec {
            kind: MoteKind::Folder,
            role,
            label: SmolStr::new_static("test"),
            grab: Grab::Fixed,
            embodied: false,
        }
    }

    fn present_at(role: Role, distance: f32, attention: Attention) -> Presentation {
        present(&spec(role), distance, attention, &tuning())
    }

    #[test]
    fn a_branch_is_a_bubble_and_a_leaf_is_not() {
        let branch = present_at(
            Role::Branch {
                children: 3,
                folders:  0,
            },
            1.0,
            Attention::Idle,
        );
        assert!(matches!(branch.shell, Shell::Bubble { .. }));
        assert_eq!(
            present_at(Role::Leaf, 1.0, Attention::Idle).shell,
            Shell::Bare
        );
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let pips = present_at(
            Role::Branch {
                children: 3,
                folders:  0,
            },
            1.0,
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.count, 3);
        assert!(!pips.overflow);
        assert_eq!(pips.branches(), 0);
    }

    #[test]
    fn container_children_are_marked_so_they_can_be_drawn_see_through() {
        let pips = present_at(
            Role::Branch {
                children: 5,
                folders:  2,
            },
            1.0,
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.branches(), 2);
        assert_eq!(pips.kinds[0], PipKind::Branch);
        assert_eq!(pips.kinds[2], PipKind::Leaf);
    }

    #[test]
    fn an_oversized_branch_reports_overflow_rather_than_lying() {
        let cap = tuning().pip_cap;
        let pips = present_at(
            Role::Branch {
                children: cap + 5,
                folders:  0,
            },
            1.0,
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.count, cap);
        assert!(pips.overflow);
    }

    #[test]
    fn more_folders_than_shown_pips_does_not_overrun() {
        let pips = present_at(
            Role::Branch {
                children: 40,
                folders:  40,
            },
            1.0,
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.branches(), pips.count);
        assert!(pips.count <= MAX_PIPS);
    }

    #[test]
    fn the_parent_mote_shows_depth_around_itself_not_inside() {
        let pips = present_at(Role::Parent { depth: 3 }, 1.0, Attention::Idle).pips;
        assert_eq!(pips.count, 3);
        assert_eq!(
            pips.placement,
            PipPlacement::Around,
            "inside means descending, around means ascending"
        );
    }

    #[test]
    fn leaves_and_casts_carry_no_pips() {
        for role in [Role::Leaf, Role::Cast] {
            assert_eq!(present_at(role, 1.0, Attention::Idle).pips.count, 0);
        }
    }

    #[test]
    fn detail_arrives_on_approach() {
        let far = present_at(Role::Leaf, 6.0, Attention::Idle);
        let near = present_at(Role::Leaf, 0.3, Attention::Idle);
        assert!(far.detail < near.detail);
        assert!(
            far.detail.abs() < 1.0e-5,
            "a distant mote is a silhouette only"
        );
    }

    #[test]
    fn attention_reveals_detail_regardless_of_distance() {
        assert!(
            present_at(Role::Leaf, 6.0, Attention::Attended).detail
                > present_at(Role::Leaf, 6.0, Attention::Idle).detail
        );
    }

    #[test]
    fn the_collider_covers_the_body_at_its_largest_and_never_moves() {
        let idle = present_at(Role::Leaf, 1.0, Attention::Idle);
        let attended = present_at(Role::Leaf, 1.0, Attention::Attended);
        let engaged = present_at(Role::Leaf, 1.0, Attention::Engaged);

        assert!(
            idle.hit_radius >= attended.radius,
            "a hover must stay hittable"
        );
        assert!(idle.hit_radius >= engaged.radius);
        assert!(
            (idle.hit_radius - attended.hit_radius).abs() < 1.0e-6,
            "a collider that changed with attention would be rewritten every frame"
        );
        assert!((idle.hit_radius - engaged.hit_radius).abs() < 1.0e-6);
    }

    #[test]
    fn a_branch_is_easier_to_hit_than_a_leaf() {
        let branch = present_at(
            Role::Branch {
                children: 2,
                folders:  0,
            },
            1.0,
            Attention::Idle,
        );
        assert!(branch.hit_radius > present_at(Role::Leaf, 1.0, Attention::Idle).hit_radius);
    }

    #[test]
    fn attention_grows_the_body() {
        let idle = present_at(Role::Leaf, 1.0, Attention::Idle).radius;
        let attended = present_at(Role::Leaf, 1.0, Attention::Attended).radius;
        let engaged = present_at(Role::Leaf, 1.0, Attention::Engaged).radius;
        assert!(idle < attended);
        assert!(attended < engaged);
    }

    #[test]
    fn a_degenerate_distance_does_not_divide_by_zero() {
        assert!(
            present_at(Role::Leaf, 0.0, Attention::Idle)
                .detail
                .is_finite()
        );
    }
}
