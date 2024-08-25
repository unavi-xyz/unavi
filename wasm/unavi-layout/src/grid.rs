use std::cell::RefCell;

use crate::{
    bindings::{
        exports::unavi::layout::{
            container::GuestContainer,
            grid::{Guest, GuestGrid},
        },
        wired::math::types::Vec3,
    },
    container::Container,
    GuestImpl,
};

impl Guest for GuestImpl {
    type Grid = Grid;
}

pub struct Grid {
    root: Container,
    rows: RefCell<Vec3>,
}

impl GuestGrid for Grid {
    fn new(size: Vec3, rows: Vec3) -> Self {
        Self {
            root: Container::new(size),
            rows: RefCell::new(rows),
        }
    }

    fn rows(&self) -> Vec3 {
        *self.rows.borrow()
    }
    fn set_rows(&self, value: Vec3) {
        self.rows.replace(value);
        // TODO: Adjust cells
    }

    fn root(&self) -> crate::bindings::exports::unavi::layout::grid::Container {
        todo!()
    }

    fn cells(&self) -> Vec<crate::bindings::exports::unavi::layout::grid::Container> {
        todo!()
    }

    fn cell(
        &self,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<crate::bindings::exports::unavi::layout::grid::Container> {
        todo!()
    }
}
