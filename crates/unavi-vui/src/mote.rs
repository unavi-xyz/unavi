use smol_str::SmolStr;

use crate::{
    attention::Attention,
    tuning::Tuning,
};

/// Most pips a mote can draw. [`Tuning::pip_cap`] is the tunable limit within
/// it.
///
/// Nine, so a full preview of a grid is a square: any smaller cap tiles with a
/// short last row.
pub const MAX_PIPS: usize = 9;

/// How a level lays its children out when it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrange {
    /// Around the parent, which stays in the middle.
    Orbit,
    /// In rows, filling from the top left.
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Fires when activated.
    Action,
    /// Can be pulled out of the orbit and placed. `unique` marks the one of
    /// its thing rather than a source of them.
    Item { unique: bool },
    /// Contains other motes; `groups` of its `children` are containers, and
    /// `arrange` is the shape they will open into.
    Group {
        children: usize,
        groups:   usize,
        arrange:  Arrange,
    },
    /// Opens a cast site rather than firing on release.
    Cast,
    /// The way back; always slot 0, carrying the current depth.
    Parent { depth: usize },
}

impl Role {
    /// Whether this mote leaves its slot when dragged.
    #[must_use]
    pub const fn is_takeable(self) -> bool {
        matches!(self, Self::Item { .. })
    }

    /// Whether placing this mote yields another of its thing rather than
    /// moving the one there is.
    #[must_use]
    pub const fn is_source(self) -> bool {
        matches!(self, Self::Item { unique: false })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoteSpec {
    pub role:        Role,
    /// The name, drawn under the body at all times.
    pub label:       SmolStr,
    /// What it does, shown on the placard once attention has been held.
    pub description: Option<SmolStr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipKind {
    /// A child that holds nothing.
    Item,
    /// A child that is itself a container.
    Group,
}

/// Where a mote's pips sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipPlacement {
    /// Contents, drawn within the body.
    Inside,
    /// Depth marks, drawn around the outside of the body.
    Around,
}

/// A mote's pip summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pips {
    pub kinds:     [PipKind; MAX_PIPS],
    pub count:     usize,
    /// Whether more motes exist than `count` can show.
    pub overflow:  bool,
    pub placement: PipPlacement,
    /// The shape the pips sit in, which for contents is the shape the level
    /// opens into: what a mote holds is previewed as it will be laid out.
    pub arrange:   Arrange,
}

impl Pips {
    pub const NONE: Self = Self {
        kinds:     [PipKind::Item; MAX_PIPS],
        count:     0,
        overflow:  false,
        placement: PipPlacement::Inside,
        arrange:   Arrange::Orbit,
    };

    /// How many leading pips are containers; these are ordered first.
    #[must_use]
    pub fn groups(&self) -> usize {
        self.kinds
            .iter()
            .take(self.count)
            .take_while(|kind| matches!(kind, PipKind::Group))
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Presentation {
    pub radius: f32,
    pub pips:   Pips,
}

fn contents(children: usize, groups: usize, arrange: Arrange, tuning: &Tuning) -> Pips {
    let cap = tuning.pip_cap.min(MAX_PIPS);
    let overflow = children > cap;
    // A grid fills its cells, so the marker saying there is more has to be
    // given one; an orbit's middle is already free.
    let shown = cap - usize::from(overflow && matches!(arrange, Arrange::Grid));
    let count = children.min(shown);
    let groups = groups.min(count);
    let mut kinds = [PipKind::Item; MAX_PIPS];
    for kind in kinds.iter_mut().take(groups) {
        *kind = PipKind::Group;
    }
    Pips {
        kinds,
        count,
        overflow,
        placement: PipPlacement::Inside,
        arrange,
    }
}

/// Depth is a count rather than a shape, so it circles the body whatever the
/// level below opens into.
fn depth_marks(depth: usize, tuning: &Tuning) -> Pips {
    let cap = tuning.pip_cap.min(MAX_PIPS);
    Pips {
        kinds:     [PipKind::Item; MAX_PIPS],
        count:     depth.min(cap),
        overflow:  depth > cap,
        placement: PipPlacement::Around,
        arrange:   Arrange::Orbit,
    }
}

#[must_use]
pub fn present(spec: &MoteSpec, attention: Attention, tuning: &Tuning) -> Presentation {
    let attention_scale = match attention {
        Attention::Engaged => tuning.seize_scale,
        Attention::Attended => tuning.attend_scale,
        Attention::Idle | Attention::Near => 1.0,
    };
    let role_scale = match spec.role {
        Role::Group { .. } => tuning.group_scale,
        Role::Action | Role::Item { .. } | Role::Cast => tuning.action_scale,
        Role::Parent { .. } => tuning.parent_scale,
    };

    Presentation {
        radius: tuning.mote_radius * role_scale * attention_scale,
        pips:   match spec.role {
            Role::Group {
                children,
                groups,
                arrange,
            } => contents(children, groups, arrange, tuning),
            Role::Parent { depth } => depth_marks(depth, tuning),
            Role::Action | Role::Item { .. } | Role::Cast => Pips::NONE,
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

    const fn group(children: usize, groups: usize) -> Role {
        Role::Group {
            children,
            groups,
            arrange: Arrange::Orbit,
        }
    }

    fn present_at(role: Role, attention: Attention) -> Presentation {
        present(&spec(role), attention, &tuning())
    }

    #[test]
    fn pips_report_the_real_child_count() {
        let pips = present_at(group(3, 0), Attention::Idle).pips;
        assert_eq!(pips.count, 3);
        assert!(!pips.overflow);
        assert_eq!(pips.groups(), 0);
    }

    #[test]
    fn container_children_are_marked_so_they_can_be_drawn_see_through() {
        let pips = present_at(group(5, 2), Attention::Idle).pips;
        assert_eq!(pips.groups(), 2);
        assert_eq!(pips.kinds[0], PipKind::Group);
        assert_eq!(pips.kinds[2], PipKind::Item);
    }

    #[test]
    fn an_oversized_group_reports_overflow_rather_than_lying() {
        let cap = tuning().pip_cap;
        let pips = present_at(group(cap + 5, 0), Attention::Idle).pips;
        assert_eq!(pips.count, cap);
        assert!(pips.overflow);
    }

    #[test]
    fn more_groups_than_shown_pips_does_not_overrun() {
        let pips = present_at(group(40, 40), Attention::Idle).pips;
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
        for role in [
            Role::Action,
            Role::Item { unique: true },
            Role::Item { unique: false },
            Role::Cast,
        ] {
            assert_eq!(present_at(role, Attention::Idle).pips.count, 0);
        }
    }

    #[test]
    fn a_source_and_the_one_of_a_thing_are_both_items() {
        assert!(Role::Item { unique: false }.is_source());
        assert!(!Role::Item { unique: true }.is_source());
        assert!(!Role::Action.is_source());
    }

    #[test]
    fn only_an_item_leaves_its_slot_when_dragged() {
        assert!(Role::Item { unique: true }.is_takeable());
        assert!(Role::Item { unique: false }.is_takeable());
        for button in [
            Role::Action,
            Role::Cast,
            Role::Parent { depth: 1 },
            group(0, 0),
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
        let shown = present_at(group(3, 0), Attention::Idle);
        assert!(shown.radius > present_at(Role::Action, Attention::Idle).radius);
        assert!(present_at(Role::Parent { depth: 1 }, Attention::Idle).radius < shown.radius);
    }

    #[test]
    fn a_container_previews_its_contents_in_the_shape_they_will_open_into() {
        for arrange in [Arrange::Orbit, Arrange::Grid] {
            let pips = present_at(
                Role::Group {
                    children: 4,
                    groups: 0,
                    arrange,
                },
                Attention::Idle,
            )
            .pips;
            assert_eq!(
                pips.arrange, arrange,
                "pips promise the shape the level will open into"
            );
        }
    }
}
