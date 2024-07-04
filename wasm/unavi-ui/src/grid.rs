use std::cell::Cell;

use crate::{
    bindings::{
        exports::unavi::ui::{
            container::{ContainerBorrow, GuestContainer},
            grid::{Direction, Guest, GuestGrid},
        },
        wired::scene::node::{create_node, Node},
    },
    container::Container,
    GuestImpl,
};

impl Guest for GuestImpl {
    type Grid = Grid;
}

pub struct Grid {
    container: Container,
    node: Node,
    columns: Cell<u32>,
    rows: Cell<u32>,
    direction: Cell<Direction>,
}

impl GuestGrid for Grid {
    fn new() -> Self {
        Self {
            container: Container::new(),
            node: create_node(),
            columns: Cell::new(1),
            rows: Cell::new(1),
            direction: Cell::new(Direction::Xyz),
        }
    }

    fn root(&self) -> crate::bindings::exports::unavi::ui::grid::Container {
        todo!();
    }

    fn columns(&self) -> u32 {
        self.columns.get()
    }
    fn rows(&self) -> u32 {
        self.rows.get()
    }
    fn set_columns(&self, value: u32) {
        self.columns.set(value);
    }
    fn set_rows(&self, value: u32) {
        self.rows.set(value);
    }

    fn direction(&self) -> Direction {
        self.direction.get()
    }
    fn set_direction(&self, value: Direction) {
        self.direction.set(value);
    }

    fn add_item(&self, _item: ContainerBorrow) {
        todo!()
    }
    fn remove_item(&self, _item: ContainerBorrow) {
        todo!()
    }
    fn list_items(&self) -> Vec<crate::bindings::exports::unavi::ui::grid::Container> {
        todo!()
    }
}
