//! A slate's rows and where each one stands on its pane: the layout a
//! renderer draws and pointer hits are read against.
//!
//! The pane hangs down from its top centre, which is the slate's origin; +X is
//! to the viewer's right and the rows run down -Y.

use smol_str::SmolStr;
use wired_guest::math::Vec2;

use crate::tuning::Tuning;

/// Most rows a slate holds.
pub const MAX_ROWS: usize = 24;
/// Longest label.
pub const MAX_LABEL_BYTES: usize = 64;
/// Longest field value.
pub const MAX_VALUE_BYTES: usize = 128;
/// Most tabs a tabs row holds, and the longest tab label.
pub const MAX_TABS: usize = 4;
pub const MAX_TAB_BYTES: usize = 16;

/// Metres across the pane.
pub const WIDTH: f32 = 0.46;
/// Metres per row.
pub const ROW_HEIGHT: f32 = 0.036;
/// Metres of pane around the rows.
pub const PADDING: f32 = 0.018;
/// Type height of a row's label and value, and of a header.
pub const TEXT_SIZE: f32 = 0.017;
pub const HEADER_SIZE: f32 = 0.02;

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Header,
    Field(SmolStr),
    Number {
        value:     f32,
        per_metre: f32,
    },
    Toggle(bool),
    Action,
    /// Up to [`MAX_TABS`] labels side by side, one of them `selected`.
    Tabs {
        labels:   Vec<SmolStr>,
        selected: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub id:    u32,
    pub label: SmolStr,
    pub kind:  Kind,
}

impl Row {
    /// What a pointer pressing on this row does.
    #[must_use]
    pub const fn press(&self) -> Press {
        match self.kind {
            Kind::Header | Kind::Field(_) => Press::Nothing,
            Kind::Number { per_metre, .. } => Press::Scrub(per_metre),
            Kind::Toggle(on) => Press::Toggle(!on),
            Kind::Action => Press::Act,
            Kind::Tabs { .. } => Press::Pick,
        }
    }

    /// The value drawn at the row's right edge, empty for a header or an
    /// action.
    #[must_use]
    pub fn value(&self) -> SmolStr {
        match &self.kind {
            Kind::Header | Kind::Action | Kind::Tabs { .. } => SmolStr::default(),
            Kind::Field(value) => value.clone(),
            Kind::Number { value, .. } => format_number(*value),
            Kind::Toggle(true) => SmolStr::new_static("on"),
            Kind::Toggle(false) => SmolStr::new_static("off"),
        }
    }

    /// The label and value as drawn, cut to share the row's width without
    /// touching. See [`fit`].
    #[must_use]
    pub fn fitted(&self) -> (SmolStr, SmolStr) {
        let size = if self.kind == Kind::Header {
            HEADER_SIZE
        } else {
            TEXT_SIZE
        };
        fit(
            &self.label,
            &self.value(),
            budget(2.0f32.mul_add(-PADDING, WIDTH), size),
        )
    }
}

/// Tab `label` cut to fit one of `count` tabs.
#[must_use]
pub fn tab_label(label: &str, count: usize) -> SmolStr {
    ellipsize(label, budget(tab_width(count) - PADDING, TEXT_SIZE))
}

/// Characters kept clear between a label and its value.
const GAP_CHARS: usize = 2;

/// Most characters of `size` type across `width`, estimated from the average
/// advance. Only the renderer knows what a string measures.
fn budget(width: f32, size: f32) -> usize {
    (width / (size * Tuning::DEFAULT.advance_estimate)) as usize
}

/// `label` and `value` cut to fit `chars` characters between them, with a gap.
/// The label keeps at least half the room when both are long, since it says
/// what the row is. A cut ends in an ellipsis.
#[must_use]
pub fn fit(label: &str, value: &str, chars: usize) -> (SmolStr, SmolStr) {
    let value_len = value.chars().count();
    if value_len == 0 {
        return (ellipsize(label, chars), SmolStr::default());
    }
    let room = chars.saturating_sub(GAP_CHARS);
    let label_room = label
        .chars()
        .count()
        .min(room.saturating_sub(value_len).max(room / 2));
    (
        ellipsize(label, label_room),
        ellipsize(value, room - label_room),
    )
}

/// `text` in at most `room` characters, the last an ellipsis when cut.
fn ellipsize(text: &str, room: usize) -> SmolStr {
    if text.chars().count() <= room {
        return SmolStr::new(text);
    }
    let Some(kept) = room.checked_sub(1) else {
        return SmolStr::default();
    };
    let mut cut: String = text.chars().take(kept).collect();
    cut.push('…');
    SmolStr::new(cut)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Press {
    Nothing,
    /// Dragging sideways changes the number by this much per metre.
    Scrub(f32),
    /// Flips the toggle to this.
    Toggle(bool),
    Act,
    /// Reports the tab under the pointer.
    Pick,
}

/// Why rows were refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    Rows(usize),
    Label(usize),
    Value(usize),
    /// A tabs row of more than [`MAX_TABS`] tabs.
    Tabs(usize),
    TabLabel(usize),
    /// A second tabs row: a slate holds one.
    TabsRows,
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rows(count) => write!(f, "{count} rows, over the limit of {MAX_ROWS}"),
            Self::Label(len) => write!(f, "a label of {len} bytes, over {MAX_LABEL_BYTES}"),
            Self::Value(len) => write!(f, "a value of {len} bytes, over {MAX_VALUE_BYTES}"),
            Self::Tabs(count) => write!(f, "{count} tabs, over the limit of {MAX_TABS}"),
            Self::TabLabel(len) => write!(f, "a tab label of {len} bytes, over {MAX_TAB_BYTES}"),
            Self::TabsRows => write!(f, "more than one tabs row"),
        }
    }
}

/// Checks rows against the limits.
pub fn check(rows: &[Row]) -> Result<(), Refused> {
    if rows.len() > MAX_ROWS {
        return Err(Refused::Rows(rows.len()));
    }
    let mut tabs_rows = 0;
    for row in rows {
        if row.label.len() > MAX_LABEL_BYTES {
            return Err(Refused::Label(row.label.len()));
        }
        match &row.kind {
            Kind::Field(value) if value.len() > MAX_VALUE_BYTES => {
                return Err(Refused::Value(value.len()));
            }
            Kind::Tabs { labels, .. } => {
                tabs_rows += 1;
                if tabs_rows > 1 {
                    return Err(Refused::TabsRows);
                }
                if labels.len() > MAX_TABS {
                    return Err(Refused::Tabs(labels.len()));
                }
                if let Some(long) = labels.iter().find(|label| label.len() > MAX_TAB_BYTES) {
                    return Err(Refused::TabLabel(long.len()));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The pane's width and height for `rows` rows.
#[must_use]
pub fn size(rows: usize) -> Vec2 {
    Vec2::new(WIDTH, (rows as f32).mul_add(ROW_HEIGHT, 2.0 * PADDING))
}

/// The height of row `index`'s centre below the top edge, as a negative Y.
#[must_use]
pub fn row_centre(index: usize) -> f32 {
    -((index as f32 + 0.5).mul_add(ROW_HEIGHT, PADDING))
}

/// The row at `local`, a point in the slate's frame, among `rows` rows.
#[must_use]
pub fn row_at(local: Vec2, rows: usize) -> Option<usize> {
    if local.x.abs() > WIDTH / 2.0 {
        return None;
    }
    let down = -local.y - PADDING;
    if down < 0.0 {
        return None;
    }
    let index = (down / ROW_HEIGHT) as usize;
    (index < rows).then_some(index)
}

/// The width of one of `count` tabs, which share the pane between its
/// paddings.
#[must_use]
pub fn tab_width(count: usize) -> f32 {
    2.0_f32.mul_add(-PADDING, WIDTH) / count.max(1) as f32
}

/// The centre across the pane of tab `index` of `count`.
#[must_use]
pub fn tab_centre(index: usize, count: usize) -> f32 {
    let width = tab_width(count);
    (index as f32 + 0.5).mul_add(width, -(WIDTH / 2.0 - PADDING))
}

/// The tab of `count` at `x` across the pane.
#[must_use]
pub fn tab_at(x: f32, count: usize) -> Option<usize> {
    let across = x + WIDTH / 2.0 - PADDING;
    if across < 0.0 {
        return None;
    }
    let index = (across / tab_width(count)) as usize;
    (index < count).then_some(index)
}

/// A number as a slate draws it: two decimals, no negative zero.
#[must_use]
pub fn format_number(value: f32) -> SmolStr {
    let shown = if value.abs() < 0.005 { 0.0 } else { value };
    smol_str::format_smolstr!("{shown:.2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_on_a_row_finds_that_row() {
        for index in 0..5 {
            let at = Vec2::new(0.0, row_centre(index));
            assert_eq!(row_at(at, 5), Some(index), "the centre of row {index}");
        }
    }

    #[test]
    fn the_padding_and_the_space_past_the_last_row_are_no_row() {
        assert_eq!(row_at(Vec2::new(0.0, -PADDING / 2.0), 5), None);
        assert_eq!(row_at(Vec2::new(0.0, row_centre(5)), 5), None);
        assert_eq!(row_at(Vec2::new(WIDTH, row_centre(0)), 5), None);
    }

    #[test]
    fn pressing_a_toggle_asks_for_the_other_state() {
        let row = Row {
            id:    1,
            label: SmolStr::new_static("Frozen"),
            kind:  Kind::Toggle(true),
        };
        assert_eq!(row.press(), Press::Toggle(false));
    }

    #[test]
    fn a_number_near_zero_reads_as_zero() {
        assert_eq!(format_number(-0.001), "0.00");
        assert_eq!(format_number(1.234), "1.23");
    }

    fn tabs(labels: &[&str]) -> Row {
        Row {
            id:    7,
            label: SmolStr::default(),
            kind:  Kind::Tabs {
                labels:   labels.iter().map(SmolStr::new).collect(),
                selected: 0,
            },
        }
    }

    #[test]
    fn a_point_across_the_tabs_finds_its_tab() {
        for index in 0..3 {
            assert_eq!(tab_at(tab_centre(index, 3), 3), Some(index));
        }
        assert_eq!(tab_at(-WIDTH / 2.0, 3), None);
        assert_eq!(tab_at(WIDTH / 2.0, 3), None);
    }

    #[test]
    fn pressing_tabs_picks() {
        assert_eq!(tabs(&["A", "B"]).press(), Press::Pick);
    }

    #[test]
    fn tabs_over_their_limits_are_refused() {
        assert_eq!(check(&[tabs(&["A", "B", "C", "D"])]), Ok(()));
        assert_eq!(
            check(&[tabs(&["A", "B", "C", "D", "E"])]),
            Err(Refused::Tabs(5))
        );
        assert_eq!(
            check(&[tabs(&["a label far too long"])]),
            Err(Refused::TabLabel(20))
        );
        assert_eq!(check(&[tabs(&["A"]), tabs(&["B"])]), Err(Refused::TabsRows));
    }

    #[test]
    fn a_label_and_value_that_fit_are_drawn_whole() {
        assert_eq!(fit("Mass", "2.00", 20), ("Mass".into(), "2.00".into()));
    }

    #[test]
    fn a_long_label_and_value_split_the_row_and_never_meet() {
        let (label, value) = fit(&"l".repeat(40), &"v".repeat(40), 20);
        assert_eq!(label.chars().count(), 9);
        assert!(label.ends_with('…') && value.ends_with('…'));
        assert!(label.chars().count() + value.chars().count() + GAP_CHARS <= 20);
    }

    #[test]
    fn a_short_label_leaves_a_long_value_the_rest_of_the_row() {
        let (label, value) = fit("Text", &"v".repeat(40), 20);
        assert_eq!(label, "Text");
        assert_eq!(value.chars().count(), 20 - GAP_CHARS - 4);
    }

    #[test]
    fn a_long_label_keeps_what_a_short_value_leaves() {
        let (label, value) = fit(&"l".repeat(40), "on", 20);
        assert_eq!(value, "on");
        assert_eq!(label.chars().count(), 20 - GAP_CHARS - 2);
    }

    #[test]
    fn a_row_holds_some_forty_characters() {
        let chars = budget(2.0f32.mul_add(-PADDING, WIDTH), TEXT_SIZE);
        assert!((30..50).contains(&chars), "{chars}");
    }

    #[test]
    fn too_many_rows_are_refused() {
        let row = Row {
            id:    0,
            label: SmolStr::default(),
            kind:  Kind::Action,
        };
        assert_eq!(
            check(&vec![row; MAX_ROWS + 1]),
            Err(Refused::Rows(MAX_ROWS + 1))
        );
    }
}
