use wired_math::types::{
    Vec2,
    Vec3,
};

use crate::{
    grasp::Outcome,
    layout::Layout,
    mote::MoteSpec,
    palette::Palette,
    placard::PlacardView,
    surface::Surface,
    tuning::Tuning,
    view::{
        Frame,
        PageView,
        SlotView,
        Style,
    },
};

/// A rack's cell arrangement. A row, a column and a grid differ only in their
/// counts, so all three are this.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shelf {
    pub columns: usize,
    pub rows:    usize,
    pub pitch:   Vec2,
}

impl Shelf {
    #[must_use]
    pub const fn grid(columns: usize, rows: usize, pitch: Vec2) -> Self {
        Self {
            columns,
            rows,
            pitch,
        }
    }

    #[must_use]
    pub const fn row(count: usize, pitch: Vec2) -> Self {
        Self::grid(count, 1, pitch)
    }

    #[must_use]
    pub const fn column(count: usize, pitch: Vec2) -> Self {
        Self::grid(1, count, pitch)
    }

    #[must_use]
    pub const fn cells(&self) -> usize {
        self.columns * self.rows
    }

    #[must_use]
    pub const fn layout(&self) -> Layout {
        Layout::grid(self.columns, self.rows, self.pitch)
    }
}

/// A bounded volume holding motes on a shelf: the inventory, the result set,
/// and the space browser.
///
/// Unlike an orbit a rack is a *destination* — it has real extents, so a mote
/// released over it files into it.
pub struct Rack {
    surface: Surface,
    shelf:   Shelf,
}

impl Rack {
    #[must_use]
    pub fn new(shelf: Shelf, tuning: Tuning, palette: Palette) -> Self {
        Self {
            surface: Surface::new(shelf.cells(), tuning, palette),
            shelf,
        }
    }

    #[must_use]
    pub const fn shelf(&self) -> Shelf {
        self.shelf
    }

    #[must_use]
    pub const fn tuning(&self) -> &Tuning {
        &self.surface.tuning
    }

    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.surface.palette
    }

    /// Half-extents of the housing, which is also the region the rack answers
    /// for.
    #[must_use]
    pub fn extents(&self) -> Vec2 {
        self.shelf.layout().extents(&self.surface.tuning)
    }

    /// Whether a release at `local` files into this rack.
    #[must_use]
    pub fn accepts(&self, local: Vec2) -> bool {
        let extents = self.extents();
        local.x.abs() <= extents.x && local.y.abs() <= extents.y
    }

    pub fn update(&mut self, specs: &[MoteSpec], frame: &Frame) {
        self.surface.update(specs, self.shelf.layout(), 0, frame);
    }

    #[must_use]
    pub fn views(&self) -> &[SlotView] {
        self.surface.views()
    }

    /// The collection index each drawn slot stands for, parallel to
    /// [`Rack::views`].
    #[must_use]
    pub fn drawn(&self) -> &[usize] {
        self.surface.drawn()
    }

    #[must_use]
    pub fn spec_index(&self, slot: usize) -> Option<usize> {
        self.surface.spec_index(slot)
    }

    #[must_use]
    pub const fn page(&self) -> PageView {
        self.surface.page()
    }

    pub const fn turn_by(&mut self, delta: isize) {
        self.surface.turn_by(delta);
    }

    pub const fn turn_to(&mut self, page: usize) {
        self.surface.turn_to(page);
    }

    #[must_use]
    pub const fn attended(&self) -> Option<usize> {
        self.surface.attended()
    }

    #[must_use]
    pub const fn is_seized(&self) -> bool {
        self.surface.is_seized()
    }

    #[must_use]
    pub fn displaced(&self) -> Option<usize> {
        self.surface.displaced()
    }

    #[must_use]
    pub const fn placard(&self) -> Option<&PlacardView> {
        self.surface.placard()
    }

    #[must_use]
    pub const fn resting_style(&self, spec: &MoteSpec) -> Style {
        self.surface.resting_style(spec)
    }

    pub fn press(&mut self, at: Vec3) {
        self.surface.press(at);
    }

    pub fn release(&mut self) -> Option<Outcome> {
        self.surface.release()
    }
}

#[cfg(test)]
mod tests {
    use smol_str::SmolStr;
    use wired_math::types::{
        Quat,
        Transform,
    };

    use super::*;
    use crate::{
        mote::Role,
        view::Aim,
    };

    const PITCH: Vec2 = Vec2::new(0.08, 0.08);

    fn item(label: &str) -> MoteSpec {
        MoteSpec {
            role:        Role::Item,
            label:       SmolStr::new(label),
            description: None,
        }
    }

    fn items(count: usize) -> Vec<MoteSpec> {
        (0..count).map(|i| item(&format!("i{i}"))).collect()
    }

    fn frame(aim: Option<Aim>) -> Frame {
        Frame {
            eye: Vec3::new(0.0, 0.0, 1.0),
            anchor: Transform {
                translation: Vec3::ZERO,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            },
            aim,
            hand: aim.map(|aim| aim.world),
            delta: 0.016,
        }
    }

    fn rack(shelf: Shelf) -> Rack {
        Rack::new(shelf, Tuning::DEFAULT, Palette::DEFAULT)
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-5
    }

    #[test]
    fn a_row_a_column_and_a_grid_differ_only_in_their_counts() {
        assert_eq!(Shelf::row(4, PITCH).cells(), 4);
        assert_eq!(Shelf::column(4, PITCH).cells(), 4);
        assert_eq!(Shelf::grid(2, 2, PITCH).cells(), 4);
        assert_eq!(Shelf::row(4, PITCH).rows, 1);
        assert_eq!(Shelf::column(4, PITCH).columns, 1);
    }

    #[test]
    fn a_row_lays_its_motes_out_sideways_and_a_column_downward() {
        let mut row = rack(Shelf::row(3, PITCH));
        row.update(&items(3), &frame(None));
        let across = row.views();
        assert!(across[0].position.x < across[2].position.x);
        assert!((across[0].position.y - across[2].position.y).abs() < 1.0e-5);

        let mut column = rack(Shelf::column(3, PITCH));
        column.update(&items(3), &frame(None));
        let down = column.views();
        assert!(down[0].position.y > down[2].position.y);
        assert!((down[0].position.x - down[2].position.x).abs() < 1.0e-5);
    }

    #[test]
    fn a_rack_has_real_extents() {
        let rack = rack(Shelf::grid(4, 2, PITCH));
        let extents = rack.extents();
        assert!(close(extents.x, PITCH.x * 2.0));
        assert!(close(extents.y, PITCH.y));
    }

    #[test]
    fn a_release_over_the_housing_files_into_it() {
        let rack = rack(Shelf::grid(4, 2, PITCH));
        assert!(rack.accepts(Vec2::ZERO));
        assert!(rack.accepts(Vec2::new(PITCH.x * 1.9, 0.0)));
        assert!(
            !rack.accepts(Vec2::new(PITCH.x * 2.5, 0.0)),
            "a rack is a destination with a size, not the whole room"
        );
        assert!(!rack.accepts(Vec2::new(0.0, PITCH.y * 1.5)));
    }

    #[test]
    fn a_full_rack_paginates_rather_than_dropping_what_it_cannot_show() {
        let mut rack = rack(Shelf::grid(2, 2, PITCH));
        let specs = items(9);

        rack.update(&specs, &frame(None));
        assert_eq!(rack.views().len(), 4);
        assert_eq!(rack.page().count, 3, "9 items over 4 cells");
        assert_eq!(rack.drawn(), vec![0, 1, 2, 3]);

        rack.turn_by(1);
        rack.update(&specs, &frame(None));
        assert_eq!(rack.drawn(), vec![4, 5, 6, 7]);

        rack.turn_by(1);
        rack.update(&specs, &frame(None));
        assert_eq!(rack.drawn(), vec![8], "the last page is as short as it is");
        assert!(!rack.page().has_next());
        assert!(rack.page().has_previous());
    }

    #[test]
    fn an_empty_rack_is_one_empty_page() {
        let mut rack = rack(Shelf::grid(2, 2, PITCH));
        rack.update(&[], &frame(None));
        assert!(rack.views().is_empty());
        assert_eq!(rack.page().count, 1);
        assert_eq!(rack.page().total, 0);
        assert!(!rack.page().is_paged());
    }

    #[test]
    fn a_cell_is_attended_by_pointing_at_it() {
        let mut rack = rack(Shelf::grid(2, 2, PITCH));
        let specs = items(4);
        let cell = rack.shelf().layout().slot(3).expect("cell");
        rack.update(
            &specs,
            &frame(Some(Aim {
                local: cell,
                world: Vec3::new(cell.x, cell.y, 0.0),
            })),
        );
        assert_eq!(rack.attended(), Some(3));
        assert_eq!(rack.spec_index(3), Some(3));
    }

    #[test]
    fn a_grab_on_a_later_page_reports_the_item_it_landed_on() {
        let mut rack = rack(Shelf::grid(2, 2, PITCH));
        let specs = items(9);
        let cell = rack.shelf().layout().slot(1).expect("cell");
        let aim = Aim {
            local: cell,
            world: Vec3::new(cell.x, cell.y, 0.0),
        };

        rack.turn_to(1);
        rack.update(&specs, &frame(Some(aim)));
        rack.press(aim.world);
        rack.update(&specs, &frame(Some(aim)));

        let Some(Outcome::Tap(slot)) = rack.release() else {
            panic!("a tap");
        };
        assert_eq!(rack.spec_index(slot), Some(5));
    }
}
