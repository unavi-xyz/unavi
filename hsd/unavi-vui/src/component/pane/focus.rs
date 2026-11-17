//! Which control has the keyboard, and the order Tab visits them in.

use crate::component::pane::{
    Event,
    Pane,
    control::Press,
    layout::{
        Item,
        LaidRow,
    },
    scroll,
};

/// A control Tab stops on.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stop {
    id:    u32,
    block: usize,
    row:   usize,
    /// What Enter does on it.
    press: Press,
}

impl Pane {
    /// Every control in reading order, whether or not it is scrolled into
    /// view.
    fn stops(&self) -> Vec<Stop> {
        let mut stops = Vec::new();
        for (block_index, block) in self.laid().blocks.iter().enumerate() {
            for (row_index, row) in block.rows.iter().enumerate() {
                stops.extend(stops_in(row).into_iter().map(|(id, press)| Stop {
                    id,
                    block: block_index,
                    row: row_index,
                    press,
                }));
            }
        }
        stops
    }

    /// The ids Tab visits, in order.
    #[must_use]
    pub fn focus_order(&self) -> Vec<u32> {
        self.stops().into_iter().map(|stop| stop.id).collect()
    }

    /// Moves the keyboard to the next control, or the previous one, wrapping
    /// at the ends, and scrolls it into view. Returns where it landed.
    pub fn focus_step(&mut self, backwards: bool) -> Option<u32> {
        let stops = self.stops();
        if stops.is_empty() {
            self.set_focus(None);
            return None;
        }
        let at = self
            .focus()
            .and_then(|id| stops.iter().position(|stop| stop.id == id));
        let next = match (at, backwards) {
            (None, false) => 0,
            (None, true) => stops.len() - 1,
            (Some(at), false) => (at + 1) % stops.len(),
            (Some(at), true) => (at + stops.len() - 1) % stops.len(),
        };
        let stop = stops[next];
        self.set_focus(Some(stop.id));
        self.reveal(stop.block, stop.row);
        Some(stop.id)
    }

    /// What Enter reports on the focused control.
    #[must_use]
    pub fn activate(&self) -> Option<Event> {
        let id = self.focus()?;
        self.stops()
            .into_iter()
            .find(|stop| stop.id == id)
            .and_then(|stop| stop.press.event(id))
    }

    /// Scrolls `row` of `block` onto the lines the block shows.
    fn reveal(&mut self, block: usize, row: usize) {
        let Some(laid) = self.laid().blocks.get(block) else {
            return;
        };
        let Some(scrolling) = row.checked_sub(laid.pinned) else {
            return;
        };
        let (id, window, content) = (laid.id, laid.window() as usize, laid.content());
        let now = self.scroll_of(id) as usize;
        let wanted = if scrolling < now {
            scrolling
        } else if window > 0 && scrolling >= now + window {
            scrolling + 1 - window
        } else {
            now
        };
        let offset = scroll::clamp(wanted as u32, content, window as u32);
        self.view.scroll.insert(id, offset);
    }
}

/// What Tab stops on in `row`, and what Enter does there. A row a press
/// selects is one stop; any other has one per control.
fn stops_in(row: &LaidRow) -> Vec<(u32, Press)> {
    if let Some(select) = row.select {
        return vec![(select.id, Press::Select)];
    }
    row.items
        .iter()
        .filter_map(|item| match item {
            Item::Control { id, control, .. } if control.pressable() => {
                Some((*id, control.activation()))
            }
            Item::Text {
                link: Some(link), ..
            } => Some((*link, Press::Follow)),
            _ => None,
        })
        .collect()
}
