//! A point on a pane to the block, row and control under it.

use wired_guest::math::Vec2;

use crate::component::pane::{
    COLUMN,
    LINE,
    PAD_X,
    Pane,
    Rect,
    control::Press,
    layout::{
        Item,
        LaidBlock,
        LaidRow,
    },
};

/// What lies under a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// The block the point is in, which a wheel scrolls.
    pub block: u32,
    /// The id a press reports, when a press does anything.
    pub id:    Option<u32>,
    pub press: Press,
    /// What was hit: the control, the rail, or the block.
    pub area:  Rect,
}

impl Pane {
    /// What `point`, in ems from the pane's top left, is over.
    #[must_use]
    pub fn hit(&self, point: Vec2) -> Option<Hit> {
        let frame = self.frame();
        self.laid().blocks.iter().find_map(|block| {
            let top = LINE.mul_add(block.top as f32, frame.body_top);
            let height = LINE * block.lines as f32;
            if point.y < top || point.y >= top + height {
                return None;
            }
            let whole = Rect {
                x: PAD_X,
                y: top,
                w: 2.0f32.mul_add(-PAD_X, frame.width),
                h: height,
            };
            if block.rail {
                let rail = Rect {
                    x: frame.width - PAD_X - COLUMN,
                    y: top,
                    w: COLUMN,
                    h: height,
                };
                if rail.contains(point) {
                    return Some(Hit {
                        block: block.id,
                        id:    None,
                        press: Press::Rail((point.y - top) / height),
                        area:  rail,
                    });
                }
            }
            let line = ((point.y - top) / LINE) as u32;
            let row_top = LINE.mul_add(line as f32, top);
            let row = block.row_at(line, self.scroll_of(block.id));
            Some(row.map_or_else(
                || nothing(block, whole),
                |row| over(block, row, point.x, row_top, whole),
            ))
        })
    }
}

const fn nothing(block: &LaidBlock, area: Rect) -> Hit {
    Hit {
        block: block.id,
        id: None,
        press: Press::Nothing,
        area,
    }
}

fn over(block: &LaidBlock, row: &LaidRow, x: f32, row_top: f32, whole: Rect) -> Hit {
    let line = |left: f32, width: f32| Rect {
        x: left,
        y: row_top,
        w: width,
        h: LINE,
    };
    for item in &row.items {
        let hit = match item {
            Item::Control {
                id,
                x: left,
                width,
                control,
            } if x >= *left && x < left + width => {
                Some((*id, control.press(x - left, *width), line(*left, *width)))
            }
            Item::Text {
                link: Some(target),
                x: left,
                width,
                ..
            } if x >= *left && x < left + width => {
                Some((*target, Press::Follow, line(*left, *width)))
            }
            Item::Fold {
                id,
                x: left,
                width,
                open,
            } if x >= *left && x < left + width => {
                Some((*id, Press::Fold(!open), line(*left, *width)))
            }
            _ => None,
        };
        if let Some((id, press, area)) = hit.filter(|(_, press, _)| *press != Press::Nothing) {
            return Hit {
                block: block.id,
                id: Some(id),
                press,
                area,
            };
        }
    }
    row.select.map_or_else(
        || nothing(block, whole),
        |select| Hit {
            block: block.id,
            id:    Some(select.id),
            press: Press::Select,
            area:  line(whole.x, whole.w),
        },
    )
}
