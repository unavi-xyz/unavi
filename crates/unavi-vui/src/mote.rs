use smol_str::SmolStr;

use crate::{
    attention::Attention,
    tuning::Tuning,
};

/// Most pips a mote can draw. Sizing the array here keeps [`Pips`] `Copy` and
/// off the heap; [`Tuning::pip_cap`] is the tunable limit within it.
pub const MAX_PIPS: usize = 8;

/// What a mote is, as far as a spatial UI can tell.
///
/// This is deliberately the *whole* vocabulary. An earlier `MoteKind` sat
/// beside it naming domain things — space, person, document, tool — and it
/// earned nothing: half its variants restated this enum (a folder is a group,
/// a command is an action), the rest were nouns a general UI library has no
/// business knowing, and all eight resolved to near-identical greys because
/// the palette reserves colour for state. What a mote *is* comes from its
/// label now that text exists; what the library needs to know is only what it
/// must draw differently, which is this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A verb: activating it does something, and it is reversible.
    Action,
    /// A noun: it can be pulled out of the orbit and put somewhere, which is
    /// what makes dragging it mean anything.
    Item,
    /// Contains other motes. `groups` of its `children` are themselves
    /// containers, which the pips show by drawing those see-through.
    Group { children: usize, groups: usize },
    /// A verb with consequences: opens a cast site rather than firing on
    /// release.
    Cast,
    /// The way back, carrying how deep you currently are. Always slot 0 and
    /// never anywhere else, so it never moves.
    Parent { depth: usize },
}

impl Role {
    /// Whether this mote leaves its slot when dragged.
    ///
    /// A separate `Grab` axis used to answer this, and it was the same
    /// mistake `MoteKind` was: "an action you happen to be able to pick up"
    /// is a noun, and saying so twice lets the two drift. Everything else
    /// behaves like a button — a release that has wandered off it cancels
    /// rather than firing.
    #[must_use]
    pub const fn is_takeable(self) -> bool {
        matches!(self, Self::Item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoteSpec {
    pub role:        Role,
    /// The name, drawn under the body at all times.
    pub label:       SmolStr,
    /// What it does, in the author's words. The label names it; this explains
    /// it, and only appears once attention has been held — which is the whole
    /// division of labour between a label and a placard.
    pub description: Option<SmolStr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipKind {
    /// A child that holds nothing: drawn solid.
    Item,
    /// A child that is itself a container: drawn see-through, the same rule
    /// its own mote follows one level down.
    Group,
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
        kinds:     [PipKind::Item; MAX_PIPS],
        count:     0,
        overflow:  false,
        placement: PipPlacement::Inside,
    };

    /// How many leading pips are containers. Pips are ordered containers
    /// first, so a renderer can draw two runs rather than tracking each one.
    #[must_use]
    pub fn groups(&self) -> usize {
        self.kinds
            .iter()
            .take(self.count)
            .take_while(|kind| matches!(kind, PipKind::Group))
            .count()
    }
}

/// How a mote is drawn, once role and attention are accounted for.
///
/// The bubble/silhouette LOD described in `docs/vui-system.md` §3.1 is not
/// here: it was carried as a `Shell` and a `detail` fraction that every
/// binding ignored, and unread values drift. It comes back when something
/// draws it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Presentation {
    pub radius: f32,
    pub pips:   Pips,
}

fn contents(children: usize, groups: usize, tuning: &Tuning) -> Pips {
    let cap = tuning.pip_cap.min(MAX_PIPS);
    let count = children.min(cap);
    let groups = groups.min(count);
    let mut kinds = [PipKind::Item; MAX_PIPS];
    for kind in kinds.iter_mut().take(groups) {
        *kind = PipKind::Group;
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
        kinds:     [PipKind::Item; MAX_PIPS],
        count:     depth.min(cap),
        overflow:  depth > cap,
        placement: PipPlacement::Around,
    }
}

#[must_use]
pub fn present(spec: &MoteSpec, attention: Attention, tuning: &Tuning) -> Presentation {
    let attention_scale = match attention {
        Attention::Engaged => tuning.seize_scale,
        Attention::Attended => tuning.attend_scale,
        Attention::Idle | Attention::Near => 1.0,
    };
    // Role drives size before attention does, so a container reads as bigger
    // than an action at any distance and without being pointed at.
    let role_scale = match spec.role {
        Role::Group { .. } => tuning.group_scale,
        Role::Action | Role::Item | Role::Cast => tuning.action_scale,
        Role::Parent { .. } => tuning.parent_scale,
    };

    Presentation {
        radius: tuning.mote_radius * role_scale * attention_scale,
        pips:   match spec.role {
            Role::Group { children, groups } => contents(children, groups, tuning),
            Role::Parent { depth } => depth_marks(depth, tuning),
            Role::Action | Role::Item | Role::Cast => Pips::NONE,
        },
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
            role,
            label: SmolStr::new_static("test"),
            description: None,
        }
    }

    fn present_at(role: Role, attention: Attention) -> Presentation {
        present(&spec(role), attention, &tuning())
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let pips = present_at(
            Role::Group {
                children: 3,
                groups:   0,
            },
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.count, 3);
        assert!(!pips.overflow);
        assert_eq!(pips.groups(), 0);
    }

    #[test]
    fn container_children_are_marked_so_they_can_be_drawn_see_through() {
        let pips = present_at(
            Role::Group {
                children: 5,
                groups:   2,
            },
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.groups(), 2);
        assert_eq!(pips.kinds[0], PipKind::Group);
        assert_eq!(pips.kinds[2], PipKind::Item);
    }

    #[test]
    fn an_oversized_group_reports_overflow_rather_than_lying() {
        let cap = tuning().pip_cap;
        let pips = present_at(
            Role::Group {
                children: cap + 5,
                groups:   0,
            },
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.count, cap);
        assert!(pips.overflow);
    }

    #[test]
    fn more_groups_than_shown_pips_does_not_overrun() {
        let pips = present_at(
            Role::Group {
                children: 40,
                groups:   40,
            },
            Attention::Idle,
        )
        .pips;
        assert_eq!(pips.groups(), pips.count);
        assert!(pips.count <= MAX_PIPS);
    }

    #[test]
    fn the_parent_mote_shows_depth_around_itself_not_inside() {
        let pips = present_at(Role::Parent { depth: 3 }, Attention::Idle).pips;
        assert_eq!(pips.count, 3);
        assert_eq!(
            pips.placement,
            PipPlacement::Around,
            "inside means descending, around means ascending"
        );
    }

    #[test]
    fn actions_and_casts_carry_no_pips() {
        for role in [Role::Action, Role::Item, Role::Cast] {
            assert_eq!(present_at(role, Attention::Idle).pips.count, 0);
        }
    }

    #[test]
    fn only_an_item_leaves_its_slot_when_dragged() {
        assert!(Role::Item.is_takeable());
        for button in [
            Role::Action,
            Role::Cast,
            Role::Parent { depth: 1 },
            Role::Group {
                children: 0,
                groups:   0,
            },
        ] {
            assert!(!button.is_takeable(), "{button:?} behaves like a button");
        }
    }

    #[test]
    fn attention_grows_the_body() {
        let idle = present_at(Role::Action, Attention::Idle).radius;
        let attended = present_at(Role::Action, Attention::Attended).radius;
        let engaged = present_at(Role::Action, Attention::Engaged).radius;
        assert!(idle < attended);
        assert!(attended < engaged);
    }

    #[test]
    fn a_group_reads_bigger_than_an_action_before_attention_touches_either() {
        let group = present_at(
            Role::Group {
                children: 3,
                groups:   0,
            },
            Attention::Idle,
        );
        assert!(group.radius > present_at(Role::Action, Attention::Idle).radius);
        assert!(present_at(Role::Parent { depth: 1 }, Attention::Idle).radius < group.radius);
    }
}
