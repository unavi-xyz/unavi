//! Slot geometry for a surface's own plane: where each slot sits, and which
//! one a pointer falls in.

use std::f32::consts::TAU;

use wired_guest::math::Vec2;

use crate::tuning::Tuning;

/// What, if anything, holds the middle of an orbit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Centre {
    /// Every mote takes a direction; a null deflection selects nothing.
    Open,
    /// The first mote sits in the middle and stays there on every page. The
    /// way back and a level's own subject are both this; which one it is
    /// belongs to whoever supplies the specs.
    Held,
}

/// How a surface's slots are arranged in its own plane.
///
/// Composition is spatial rather than hierarchical: a layout never contains
/// another layout, and a surface that wants more structure is a second surface
/// planted somewhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    /// `points` slots evenly spaced around a full turn, slot 0 at the top,
    /// advancing clockwise.
    Star { points: usize, radius: f32 },
    /// [`Layout::Star`] plus a centre slot at index 0.
    Centred { points: usize, radius: f32 },
    /// Row-major cells centred on the anchor, slot 0 at the top left. A
    /// `held` grid puts slot 0 in the centre cell instead and fills the rest
    /// row-major around it, so the way back sits on the anchor as it does in
    /// an orbit.
    Grid {
        columns: usize,
        rows:    usize,
        pitch:   Vec2,
        held:    bool,
    },
}

impl Layout {
    #[must_use]
    pub const fn star(points: usize, radius: f32) -> Self {
        Self::Star { points, radius }
    }

    #[must_use]
    pub const fn centred(points: usize, radius: f32) -> Self {
        Self::Centred { points, radius }
    }

    #[must_use]
    pub const fn grid(columns: usize, rows: usize, pitch: Vec2) -> Self {
        Self::Grid {
            columns,
            rows,
            pitch,
            held: false,
        }
    }

    /// [`Layout::grid`] with slot 0 held in the centre cell.
    ///
    /// Only an odd count of columns and rows has a cell on the anchor, so
    /// see [`Layout::odd`].
    #[must_use]
    pub const fn held_grid(columns: usize, rows: usize, pitch: Vec2) -> Self {
        debug_assert!(
            columns % 2 == 1 && rows % 2 == 1,
            "a held grid needs a cell on the anchor"
        );
        Self::Grid {
            columns,
            rows,
            pitch,
            held: true,
        }
    }

    /// `count` rounded up to the nearest odd number, which is what a held
    /// grid's dimensions must be.
    #[must_use]
    pub const fn odd(count: usize) -> usize {
        count | 1
    }

    /// The orbit `count` motes take under `centre`, shrunk to `capacity` and
    /// leaving no gaps; anything past it paginates. Returns the layout and the
    /// number of leading slots pinned on every page.
    #[must_use]
    pub fn orbit(count: usize, centre: Centre, capacity: usize, radius: f32) -> (Self, usize) {
        let held = usize::from(centre == Centre::Held).min(count);
        let points = (count - held).min(capacity.saturating_sub(held));
        let layout = match centre {
            Centre::Open => Self::star(points, radius),
            Centre::Held => Self::centred(points, radius),
        };
        (layout, held)
    }

    /// Whether a release at `local` — in this layout's own plane — lands
    /// inside the region it answers for.
    #[must_use]
    pub fn accepts(&self, local: Vec2, tuning: &Tuning) -> bool {
        let extents = self.extents(tuning);
        local.x.abs() <= extents.x && local.y.abs() <= extents.y
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        match *self {
            Self::Star { points, .. } => points,
            Self::Centred { points, .. } => points + 1,
            Self::Grid { columns, rows, .. } => columns.saturating_mul(rows),
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn has_centre(&self) -> bool {
        matches!(*self, Self::Centred { .. })
    }

    /// Slots arranged radially, excluding any centre slot.
    #[must_use]
    pub const fn orbit_len(&self) -> usize {
        match *self {
            Self::Star { points, .. } | Self::Centred { points, .. } => points,
            Self::Grid { .. } => 0,
        }
    }

    /// Half-extents of the region this layout's slots answer for, which is
    /// what a drop is accepted within. A hit surface needs
    /// [`Layout::hit_extents`] instead, to take in the page tabs.
    #[must_use]
    pub fn extents(&self, tuning: &Tuning) -> Vec2 {
        match *self {
            Self::Star { radius, .. } | Self::Centred { radius, .. } => {
                Vec2::splat(radius * tuning.reach_frac)
            }
            Self::Grid {
                columns,
                rows,
                pitch,
                ..
            } => Vec2::new(columns as f32 * pitch.x, rows as f32 * pitch.y) * 0.5,
        }
    }

    /// Half-extents a hit surface must cover: the region the slots answer
    /// for, widened to take in the page tabs standing outside it. A shelf is
    /// this and no more; reaching past it would be a wall nobody can see.
    #[must_use]
    pub fn hit_extents(&self, tuning: &Tuning) -> Vec2 {
        let slots = self.extents(tuning);
        Vec2::new(
            slots.x.max(self.tab(true, tuning).x + tuning.tab_hit),
            slots.y,
        )
    }

    /// Radius of the disc a level is pointed at: everything
    /// [`Layout::hit_extents`] covers but the corners.
    #[must_use]
    pub fn hit_radius(&self, tuning: &Tuning) -> f32 {
        let tab = self.tab(true, tuning);
        let extents = self.extents(tuning);
        let slots = match *self {
            Self::Star { .. } | Self::Centred { .. } => extents.x,
            Self::Grid { .. } => extents.length(),
        };
        slots.max(tab.length() + tuning.tab_hit)
    }

    /// Where a page tab sits: on the horizontal axis, just past the outermost
    /// slot, `forward` to the right and back to the left.
    #[must_use]
    pub fn tab(&self, forward: bool, tuning: &Tuning) -> Vec2 {
        let outermost = match *self {
            Self::Star { radius, .. } | Self::Centred { radius, .. } => radius,
            Self::Grid { columns, pitch, .. } => (columns.saturating_sub(1)) as f32 * 0.5 * pitch.x,
        };
        let x = outermost + tuning.tab_gap;
        Vec2::new(if forward { x } else { -x }, 0.0)
    }

    #[must_use]
    pub fn slot(&self, index: usize) -> Option<Vec2> {
        if index >= self.len() {
            return None;
        }
        match *self {
            Self::Star { points, radius } => Some(orbit_position(index, points, radius)),
            Self::Centred { points, radius } => match index {
                0 => Some(Vec2::ZERO),
                _ => Some(orbit_position(index - 1, points, radius)),
            },
            Self::Grid {
                columns,
                rows,
                pitch,
                ..
            } => {
                let (column, row) = (self.cell_of(index) % columns, self.cell_of(index) / columns);
                Some(Vec2::new(
                    ((columns - 1) as f32).mul_add(-0.5, column as f32) * pitch.x,
                    ((rows - 1) as f32).mul_add(0.5, -(row as f32)) * pitch.y,
                ))
            }
        }
    }

    /// The slot a local-plane point falls in.
    ///
    /// `current` is the slot already holding attention; its target is widened
    /// so the pointer must move meaningfully into a neighbour to switch.
    #[must_use]
    pub fn resolve(&self, local: Vec2, current: Option<usize>, tuning: &Tuning) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        match *self {
            Self::Star { .. } | Self::Centred { .. } => self.resolve_radial(local, current, tuning),
            Self::Grid { .. } => self.resolve_grid(local, current, tuning),
        }
    }

    /// The grid cell, row-major from the top left, a slot occupies. Only a
    /// held grid reorders: slot 0 takes the centre cell and the slots after
    /// it fill the others in order.
    const fn cell_of(&self, slot: usize) -> usize {
        let Self::Grid {
            columns,
            rows,
            held: true,
            ..
        } = *self
        else {
            return slot;
        };
        let centre = centre_cell(columns, rows);
        match slot {
            0 => centre,
            _ if slot - 1 < centre => slot - 1,
            _ => slot,
        }
    }

    /// The inverse of [`Layout::cell_of`].
    const fn slot_of(&self, cell: usize) -> usize {
        let Self::Grid {
            columns,
            rows,
            held: true,
            ..
        } = *self
        else {
            return cell;
        };
        let centre = centre_cell(columns, rows);
        if cell == centre {
            0
        } else if cell < centre {
            cell + 1
        } else {
            cell
        }
    }

    fn resolve_grid(&self, local: Vec2, current: Option<usize>, tuning: &Tuning) -> Option<usize> {
        let Self::Grid {
            columns,
            rows,
            pitch,
            ..
        } = *self
        else {
            return None;
        };
        if pitch.x <= f32::EPSILON || pitch.y <= f32::EPSILON {
            return None;
        }
        let cell = |index: usize| -> Vec2 {
            Vec2::new(
                ((columns - 1) as f32).mul_add(-0.5, (index % columns) as f32),
                ((rows - 1) as f32).mul_add(0.5, -((index / columns) as f32)),
            )
        };
        let unit = Vec2::new(local.x / pitch.x, local.y / pitch.y);

        // The attended cell keeps the pointer until it is well inside a
        // neighbour, the same hysteresis a ring gets from `Tuning::stick`.
        if let Some(index) = current.filter(|index| *index < columns * rows) {
            let held = cell(self.cell_of(index));
            let stuck = 0.5 + tuning.grid_stick;
            if (unit.x - held.x).abs() <= stuck && (unit.y - held.y).abs() <= stuck {
                return Some(index);
            }
        }

        let column = ((columns - 1) as f32).mul_add(0.5, unit.x).round();
        let row = ((rows - 1) as f32).mul_add(0.5, -unit.y).round();
        if column < 0.0 || column >= columns as f32 || row < 0.0 || row >= rows as f32 {
            return None;
        }
        Some(self.slot_of(row as usize * columns + column as usize))
    }

    fn resolve_radial(
        &self,
        local: Vec2,
        current: Option<usize>,
        tuning: &Tuning,
    ) -> Option<usize> {
        let radius = self.radius()?;
        let distance = local.length();
        // The middle of an open orbit is a dead zone rather than the nearest
        // direction: a pointer resting there has not deflected toward any
        // mote, and the readout drawn there is something to look at, not a
        // way to pick whichever slot happens to be closest in angle.
        if distance <= radius * tuning.centre_frac {
            return self.has_centre().then_some(0);
        }
        if distance > radius * tuning.reach_frac {
            return None;
        }

        let points = self.orbit_len();
        if points == 0 {
            return None;
        }

        let offset = usize::from(self.has_centre());
        let angle = local.x.atan2(local.y);
        (0..points)
            .map(|i| {
                let slot = i + offset;
                let mut delta = angular_delta(angle, i as f32 * TAU / points as f32);
                if current == Some(slot) {
                    delta -= tuning.stick;
                }
                (slot, delta)
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(slot, _)| slot)
    }

    const fn radius(&self) -> Option<f32> {
        match *self {
            Self::Star { radius, .. } | Self::Centred { radius, .. } => Some(radius),
            Self::Grid { .. } => None,
        }
    }
}

/// Angle 0 is up, advancing clockwise, matching slot 0 of an orbit.
fn polar(angle: f32, radius: f32) -> Vec2 {
    Vec2::new(radius * angle.sin(), radius * angle.cos())
}

fn orbit_position(index: usize, points: usize, radius: f32) -> Vec2 {
    polar(index as f32 * TAU / points as f32, radius)
}

/// The middle cell of a grid; exactly central only for odd dimensions.
const fn centre_cell(columns: usize, rows: usize) -> usize {
    rows / 2 * columns + columns / 2
}

/// Shortest absolute angle between two directions, in `0..=PI`.
fn angular_delta(a: f32, b: f32) -> f32 {
    let delta = (a - b).rem_euclid(TAU);
    delta.min(TAU - delta)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    const R: f32 = 0.2;
    const PITCH: Vec2 = Vec2::new(0.08, 0.08);

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-5
    }

    #[test]
    fn star_slot_zero_is_up() {
        let layout = Layout::star(5, R);
        let slot = layout.slot(0).expect("slot 0");
        assert!((slot.x - 0.0).abs() < 1.0e-5);
        assert!((slot.y - R).abs() < 1.0e-5);
    }

    #[test]
    fn star_advances_clockwise() {
        let layout = Layout::star(4, R);
        assert!(layout.slot(1).expect("slot 1").x > 0.0, "slot 1 is right");
        assert!(layout.slot(2).expect("slot 2").y < 0.0, "slot 2 is down");
        assert!(layout.slot(3).expect("slot 3").x < 0.0, "slot 3 is left");
    }

    #[test]
    fn centred_puts_the_centre_first_and_shifts_the_ring() {
        let centred = Layout::centred(4, R);
        let star = Layout::star(4, R);
        assert_eq!(centred.len(), 5);
        assert_eq!(centred.slot(0), Some(Vec2::ZERO));
        assert_eq!(centred.slot(1), star.slot(0));
        assert_eq!(centred.slot(4), star.slot(3));
        assert_eq!(centred.slot(5), None);
    }

    #[test]
    fn resolve_picks_the_nearest_direction() {
        let layout = Layout::star(4, R);
        assert_eq!(layout.resolve(Vec2::new(0.0, R), None, &tuning()), Some(0));
        assert_eq!(layout.resolve(Vec2::new(R, 0.0), None, &tuning()), Some(1));
        assert_eq!(layout.resolve(Vec2::new(0.0, -R), None, &tuning()), Some(2));
        assert_eq!(layout.resolve(Vec2::new(-R, 0.0), None, &tuning()), Some(3));
    }

    #[test]
    fn centre_wins_inside_its_radius() {
        let layout = Layout::centred(4, R);
        let inside = R * tuning().centre_frac * 0.5;
        assert_eq!(
            layout.resolve(Vec2::new(inside, 0.0), None, &tuning()),
            Some(0)
        );
    }

    #[test]
    fn a_star_selects_nothing_at_its_centre() {
        let layout = Layout::star(5, R);
        let resolved = layout.resolve(Vec2::new(0.0, 0.0), None, &tuning());
        assert_eq!(resolved, None, "a null deflection picks no direction");
        let edge = R * tuning().centre_frac * 1.1;
        assert_eq!(
            layout.resolve(Vec2::new(0.0, edge), None, &tuning()),
            Some(0)
        );
    }

    #[test]
    fn an_open_orbit_has_a_slot_for_each_mote_and_none_for_its_centre() {
        // A readout stands in the middle of an open orbit, and the layout
        // knows nothing of it: whatever the pointer is over, only a mote's
        // own slot can come back.
        let (layout, pinned) = Layout::orbit(4, Centre::Open, 4, R);
        assert_eq!((layout.len(), pinned), (4, 0));
        for x in -10..=10 {
            for y in -10..=10 {
                let point = Vec2::new(x as f32, y as f32) * (R / 5.0);
                let slot = layout.resolve(point, None, &tuning());
                assert!(
                    slot.is_none_or(|slot| slot < 4),
                    "{point:?} resolved {slot:?}"
                );
            }
        }
    }

    #[test]
    fn nothing_is_attended_past_reach() {
        let layout = Layout::star(4, R);
        let far = R * tuning().reach_frac * 1.1;
        assert_eq!(layout.resolve(Vec2::new(0.0, far), None, &tuning()), None);
    }

    #[test]
    fn stick_holds_the_current_slot_across_the_boundary() {
        let layout = Layout::star(4, R);
        // Just past halfway from slot 0 toward slot 1.
        let past = PI / 4.0 + 0.08;
        let point = Vec2::new(R * past.sin(), R * past.cos());

        assert_eq!(layout.resolve(point, None, &tuning()), Some(1));
        assert_eq!(
            layout.resolve(point, Some(0), &tuning()),
            Some(0),
            "attention sticks until the pointer moves well into the neighbour"
        );
    }

    #[test]
    fn stick_yields_once_the_pointer_commits() {
        let layout = Layout::star(4, R);
        let well_past = PI / 4.0 + 0.4;
        let point = Vec2::new(R * well_past.sin(), R * well_past.cos());
        assert_eq!(layout.resolve(point, Some(0), &tuning()), Some(1));
    }

    #[test]
    fn an_empty_layout_resolves_to_nothing() {
        let layout = Layout::star(0, R);
        assert!(layout.is_empty());
        assert_eq!(layout.resolve(Vec2::new(0.0, R), None, &tuning()), None);
    }

    #[test]
    fn every_slot_resolves_to_itself() {
        for points in [3_usize, 4, 5, 7, 12] {
            let layout = Layout::centred(points, R);
            for index in 0..layout.len() {
                let position = layout.slot(index).expect("slot");
                assert_eq!(
                    layout.resolve(position, None, &tuning()),
                    Some(index),
                    "points={points} index={index}"
                );
            }
        }
    }

    #[test]
    fn a_grid_runs_row_major_from_the_top_left() {
        let layout = Layout::grid(3, 2, PITCH);
        assert_eq!(layout.len(), 6);
        let first = layout.slot(0).expect("first");
        let last = layout.slot(5).expect("last");
        assert!(first.x < 0.0 && first.y > 0.0, "slot 0 is the top left");
        assert!(
            last.x > 0.0 && last.y < 0.0,
            "the last slot is bottom right"
        );
        assert!(
            (layout.slot(1).expect("slot 1").x - 0.0).abs() < 1.0e-5,
            "an odd column count centres its middle column"
        );
    }

    #[test]
    fn every_grid_cell_resolves_to_itself() {
        for (columns, rows) in [(1_usize, 4_usize), (3, 3), (4, 2), (5, 4)] {
            let layout = Layout::grid(columns, rows, PITCH);
            for index in 0..layout.len() {
                let position = layout.slot(index).expect("cell");
                assert_eq!(
                    layout.resolve(position, None, &tuning()),
                    Some(index),
                    "{columns}x{rows} index={index}"
                );
            }
        }
    }

    #[test]
    fn nothing_resolves_outside_the_grid() {
        let layout = Layout::grid(3, 2, PITCH);
        assert_eq!(
            layout.resolve(Vec2::new(PITCH.x * 3.0, 0.0), None, &tuning()),
            None
        );
        assert_eq!(
            layout.resolve(Vec2::new(0.0, PITCH.y * 3.0), None, &tuning()),
            None
        );
    }

    #[test]
    fn grid_attention_sticks_across_a_cell_boundary() {
        let layout = Layout::grid(3, 1, PITCH);
        let just_past = PITCH.x * tuning().grid_stick.mul_add(0.5, 0.5);
        let point = Vec2::new(just_past - PITCH.x, 0.0);
        assert_eq!(layout.resolve(point, None, &tuning()), Some(1));
        assert_eq!(
            layout.resolve(point, Some(0), &tuning()),
            Some(0),
            "a cell keeps the pointer until it is well inside its neighbour"
        );
    }

    #[test]
    fn grid_stick_yields_once_the_pointer_commits() {
        let layout = Layout::grid(3, 1, PITCH);
        let point = Vec2::new(0.0, 0.0);
        assert_eq!(layout.resolve(point, Some(0), &tuning()), Some(1));
    }

    #[test]
    fn a_degenerate_grid_pitch_resolves_to_nothing() {
        let layout = Layout::grid(2, 2, Vec2::ZERO);
        assert_eq!(layout.resolve(Vec2::ZERO, None, &tuning()), None);
    }

    #[test]
    fn extents_bound_what_each_layout_answers_for() {
        let radial = Layout::star(5, R).extents(&tuning());
        assert!(close(radial.x, R * tuning().reach_frac));
        assert!(close(radial.y, radial.x));

        let grid = Layout::grid(4, 2, PITCH).extents(&tuning());
        assert!(close(grid.x, PITCH.x * 2.0));
        assert!(close(grid.y, PITCH.y));
    }

    #[test]
    fn a_ring_is_centred_or_not_and_reports_its_pin() {
        let (open, pinned) = Layout::orbit(4, Centre::Open, 12, R);
        assert!(!open.has_centre());
        assert_eq!(pinned, 0);

        let (held, pinned) = Layout::orbit(5, Centre::Held, 12, R);
        assert!(held.has_centre());
        assert_eq!(held.len(), 5, "the held centre plus 4 around it");
        assert_eq!(pinned, 1);
    }

    #[test]
    fn a_ring_shrinks_to_capacity_without_leaving_gaps() {
        let (layout, pinned) = Layout::orbit(20, Centre::Open, 4, R);
        assert_eq!(layout.len(), 4);
        assert_eq!(pinned, 0);

        let (layout, pinned) = Layout::orbit(20, Centre::Held, 4, R);
        assert_eq!(layout.len(), 4, "the held centre takes a slot of its own");
        assert_eq!(pinned, 1);
    }

    #[test]
    fn a_row_is_one_high_and_a_column_is_one_wide() {
        let row = Layout::grid(4, 1, PITCH);
        let column = Layout::grid(1, 4, PITCH);
        assert_eq!(row.len(), 4);
        assert_eq!(column.len(), 4);

        let across = (0..4)
            .map(|i| row.slot(i).expect("cell"))
            .collect::<Vec<_>>();
        assert!(across[0].x < across[3].x, "a row lays sideways");
        assert!((across[0].y - across[3].y).abs() < 1.0e-5);

        let down = (0..4)
            .map(|i| column.slot(i).expect("cell"))
            .collect::<Vec<_>>();
        assert!(down[0].y > down[3].y, "a column lays downward");
        assert!((down[0].x - down[3].x).abs() < 1.0e-5);
    }

    #[test]
    fn a_held_grid_keeps_slot_zero_on_the_anchor() {
        let layout = Layout::held_grid(3, 3, PITCH);
        assert_eq!(layout.len(), 9);
        assert_eq!(layout.slot(0), Some(Vec2::ZERO));
        assert_eq!(
            layout.slot(1),
            Layout::grid(3, 3, PITCH).slot(0),
            "content starts at the top left"
        );
        assert_eq!(
            layout.slot(5),
            Layout::grid(3, 3, PITCH).slot(5),
            "and carries on row-major past the middle without a gap"
        );
        assert_eq!(layout.slot(9), None);

        for (columns, rows) in [(5_usize, 3_usize), (3, 5), (1, 1)] {
            let layout = Layout::held_grid(columns, rows, PITCH);
            assert_eq!(layout.slot(0), Some(Vec2::ZERO), "{columns}x{rows}");
        }
    }

    #[test]
    fn a_held_grid_rounds_its_dimensions_up_to_odd() {
        assert_eq!(Layout::odd(3), 3);
        assert_eq!(Layout::odd(4), 5);
        assert_eq!(Layout::odd(0), 1);
    }

    #[test]
    fn a_held_grid_fills_every_cell_exactly_once() {
        let layout = Layout::held_grid(3, 3, PITCH);
        let plain = Layout::grid(3, 3, PITCH);
        for cell in 0..plain.len() {
            let position = plain.slot(cell).expect("cell");
            assert!(
                (0..layout.len()).any(|slot| layout.slot(slot) == Some(position)),
                "cell {cell} is held by some slot"
            );
        }
    }

    #[test]
    fn every_held_grid_cell_resolves_to_its_slot() {
        for (columns, rows) in [(3_usize, 3_usize), (5, 3), (5, 5)] {
            let layout = Layout::held_grid(columns, rows, PITCH);
            for index in 0..layout.len() {
                let position = layout.slot(index).expect("cell");
                assert_eq!(
                    layout.resolve(position, None, &tuning()),
                    Some(index),
                    "{columns}x{rows} index={index}"
                );
            }
        }
    }

    #[test]
    fn tabs_stand_outside_the_slots_on_the_horizontal_axis() {
        for layout in [
            Layout::centred(8, R),
            Layout::star(5, R),
            Layout::held_grid(3, 3, PITCH),
        ] {
            let (back, forward) = (layout.tab(false, &tuning()), layout.tab(true, &tuning()));
            assert!(back.x < 0.0 && forward.x > 0.0);
            assert!(close(back.y, 0.0) && close(forward.y, 0.0));
            assert!(close(back.x, -forward.x));
            let outermost = (0..layout.len())
                .filter_map(|slot| layout.slot(slot))
                .map(|at| at.x.abs())
                .fold(0.0_f32, f32::max);
            assert!(forward.x > outermost, "{layout:?}: past the last slot");
        }
    }

    #[test]
    fn the_hit_surface_covers_the_tabs() {
        let tuning = tuning();
        let grid = Layout::held_grid(3, 3, PITCH);
        let tab = grid.tab(true, &tuning);
        assert!(grid.hit_extents(&tuning).x >= tab.x + tuning.tab_hit);
        assert!(
            !grid.accepts(Vec2::new(tab.x, 0.0), &tuning),
            "a shelf's drop region is unchanged by its tabs"
        );

        let ring = Layout::centred(8, R);
        assert!(ring.hit_extents(&tuning).x >= ring.tab(true, &tuning).x + tuning.tab_hit);
    }

    /// The worst case on a tab's side: the biggest slot body, attended and
    /// leaned out toward it, against the tab body at its own largest.
    #[test]
    fn a_tab_clears_the_largest_slot_body_beside_it() {
        let tuning = tuning();
        let slot_reach = tuning
            .mote_radius
            .mul_add(tuning.group_scale * tuning.seize_scale, tuning.lean_dist);
        let tab_body = tuning.mote_radius * tuning.tab_scale * tuning.seize_scale;
        let margin = 0.015;

        for layout in [
            Layout::centred(8, tuning.orbit_radius),
            Layout::held_grid(3, 3, Vec2::splat(tuning.grid_pitch)),
        ] {
            let outermost = (0..layout.len())
                .filter_map(|slot| layout.slot(slot))
                .map(|at| at.x)
                .fold(0.0_f32, f32::max);
            let gap = layout.tab(true, &tuning).x - outermost;
            assert!(
                gap >= slot_reach + tab_body + margin,
                "{layout:?}: {gap} between centres"
            );
        }
    }

    #[test]
    fn the_hit_disc_covers_the_page_tabs_and_a_shelf_is_no_taller_than_its_slots() {
        let tuning = tuning();
        for layout in [
            Layout::centred(8, R),
            Layout::held_grid(3, 3, PITCH),
            Layout::grid(3, 3, PITCH),
        ] {
            let extents = layout.hit_extents(&tuning);
            let radius = layout.hit_radius(&tuning);
            assert!(
                close(extents.y, layout.extents(&tuning).y),
                "{layout:?}: no taller than its slots"
            );
            assert!(radius >= layout.tab(true, &tuning).x + tuning.tab_hit - 1.0e-6);
        }
    }

    #[test]
    fn a_grid_answers_for_exactly_its_extents() {
        let grid = Layout::grid(4, 2, PITCH);
        assert!(grid.accepts(Vec2::ZERO, &tuning()));
        assert!(grid.accepts(Vec2::new(PITCH.x * 1.9, 0.0), &tuning()));
        assert!(
            !grid.accepts(Vec2::new(PITCH.x * 2.5, 0.0), &tuning()),
            "a grid is a destination with a size, not the whole room"
        );
        assert!(!grid.accepts(Vec2::new(0.0, PITCH.y * 1.5), &tuning()));
    }
}
