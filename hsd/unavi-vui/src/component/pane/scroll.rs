//! Scroll offsets in rows: how far a block can go, and the bead on its rail.

/// Shortest the bead is drawn, as a fraction of the rail, so a long block's
/// bead stays something to point at.
const MIN_THUMB: f32 = 0.08;

/// The bead on a rail, as fractions of its length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thumb {
    pub start: f32,
    pub len:   f32,
}

/// The furthest a block of `content` rows can scroll through `window` rows.
#[must_use]
pub const fn max_offset(content: usize, window: u32) -> u32 {
    content.saturating_sub(window as usize) as u32
}

#[must_use]
pub const fn clamp(offset: u32, content: usize, window: u32) -> u32 {
    let max = max_offset(content, window);
    if offset > max { max } else { offset }
}

/// `offset` moved by `rows`, held to the block.
#[must_use]
pub fn by(offset: u32, rows: i32, content: usize, window: u32) -> u32 {
    let moved = i64::from(offset) + i64::from(rows);
    clamp(moved.max(0) as u32, content, window)
}

/// The offset a press at `fraction` of the rail's length asks for.
#[must_use]
pub fn at(fraction: f32, content: usize, window: u32) -> u32 {
    let max = max_offset(content, window);
    clamp(
        (fraction.clamp(0.0, 1.0) * max as f32).round() as u32,
        content,
        window,
    )
}

#[must_use]
pub fn thumb(offset: u32, content: usize, window: u32) -> Thumb {
    let max = max_offset(content, window);
    if max == 0 || content == 0 {
        return Thumb {
            start: 0.0,
            len:   1.0,
        };
    }
    let len = (window as f32 / content as f32).clamp(MIN_THUMB, 1.0);
    Thumb {
        start: offset.min(max) as f32 / max as f32 * (1.0 - len),
        len,
    }
}
