//! A slate's rows and where each one stands on its pane: the layout a
//! renderer draws and pointer hits are read against.
//!
//! The pane hangs down from its top centre, which is the slate's origin; +X is
//! to the viewer's right and the rows run down -Y.

use smol_str::SmolStr;
use wired_guest::math::Vec2;

/// Most rows a slate holds.
pub const MAX_ROWS: usize = 24;
/// Longest label.
pub const MAX_LABEL_BYTES: usize = 64;
/// Longest field value.
pub const MAX_VALUE_BYTES: usize = 128;

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
    Number { value: f32, per_metre: f32 },
    Toggle(bool),
    Action,
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
        }
    }

    /// The value drawn at the row's right edge, empty for a header or an
    /// action.
    #[must_use]
    pub fn value(&self) -> SmolStr {
        match &self.kind {
            Kind::Header | Kind::Action => SmolStr::default(),
            Kind::Field(value) => value.clone(),
            Kind::Number { value, .. } => format_number(*value),
            Kind::Toggle(true) => SmolStr::new_static("on"),
            Kind::Toggle(false) => SmolStr::new_static("off"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Press {
    Nothing,
    /// Dragging sideways changes the number by this much per metre.
    Scrub(f32),
    /// Flips the toggle to this.
    Toggle(bool),
    Act,
}

/// Why rows were refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    Rows(usize),
    Label(usize),
    Value(usize),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rows(count) => write!(f, "{count} rows, over the limit of {MAX_ROWS}"),
            Self::Label(len) => write!(f, "a label of {len} bytes, over {MAX_LABEL_BYTES}"),
            Self::Value(len) => write!(f, "a value of {len} bytes, over {MAX_VALUE_BYTES}"),
        }
    }
}

/// Checks rows against the limits.
pub fn check(rows: &[Row]) -> Result<(), Refused> {
    if rows.len() > MAX_ROWS {
        return Err(Refused::Rows(rows.len()));
    }
    for row in rows {
        if row.label.len() > MAX_LABEL_BYTES {
            return Err(Refused::Label(row.label.len()));
        }
        if let Kind::Field(value) = &row.kind
            && value.len() > MAX_VALUE_BYTES
        {
            return Err(Refused::Value(value.len()));
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
