//! Montserrat 700 titles measured in ems.

use crate::measure::montserrat::ADVANCES;

/// Letter spacing of capitals signage, in ems after every character.
pub const TRACKING: f32 = 0.14;

const ELLIPSIS: char = '\u{2026}';

/// Advance of the widest character in the table, in ems.
const WIDEST: f32 = widest() as f32 / 1000.0;

/// Width of `text` in ems, tracking included. Signage is set in capitals, so
/// a caller passes the text already upper-cased. A character outside the
/// table takes the widest advance in it, so a title measures no narrower than
/// it draws.
pub fn width(text: &str) -> f32 {
    text.chars().map(|c| advance(c) + TRACKING).sum()
}

/// Shortens `text` to at most `max_em` by cutting its end and appending an
/// ellipsis. A title that fits is returned unchanged, and one too narrow for
/// the ellipsis is empty.
pub fn truncate(text: &str, max_em: f32) -> String {
    if width(text) <= max_em {
        return text.to_owned();
    }

    let ellipsis = advance(ELLIPSIS) + TRACKING;
    if ellipsis > max_em {
        return String::new();
    }

    let mut out = String::new();
    let mut used = ellipsis;
    for c in text.chars() {
        used += advance(c) + TRACKING;
        if used > max_em {
            break;
        }
        out.push(c);
    }
    out.push(ELLIPSIS);

    out
}

fn advance(c: char) -> f32 {
    ADVANCES
        .binary_search_by_key(&c, |&(ch, _)| ch)
        .map_or(WIDEST, |i| f32::from(ADVANCES[i].1) / 1000.0)
}

const fn widest() -> u16 {
    let mut max = 0;
    let mut i = 0;
    while i < ADVANCES.len() {
        if ADVANCES[i].1 > max {
            max = ADVANCES[i].1;
        }
        i += 1;
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn ascii_sums_the_advances_and_one_tracking_per_character() {
        let expected = 0.766 + 0.301 + TRACKING + TRACKING;
        assert!(near(width("Ai"), expected), "{}", width("Ai"));
    }

    #[test]
    fn the_table_is_sorted_for_lookup() {
        assert!(ADVANCES.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn a_title_with_a_character_outside_the_table_is_never_narrower_than_the_widest() {
        let expected = 0.766 + WIDEST + TRACKING + TRACKING;
        assert!(near(width("A漢"), expected), "{}", width("A漢"));
        assert!(ADVANCES.iter().all(|&(c, _)| advance(c) <= WIDEST));
    }

    #[test]
    fn a_title_that_fits_is_unchanged() {
        assert_eq!(truncate("HOME", 10.0), "HOME");
    }

    #[test]
    fn a_long_title_ends_in_an_ellipsis_within_the_limit() {
        let out = truncate("A VERY LONG SPACE NAME", 5.0);
        assert!(out.ends_with(ELLIPSIS));
        assert!(width(&out) <= 5.0, "{}", width(&out));
        assert!(out.len() > ELLIPSIS.len_utf8());
    }

    #[test]
    fn truncating_unknown_characters_charges_the_widest_advance() {
        let out = truncate("漢字漢字漢字", 3.0);
        assert!(width(&out) <= 3.0, "{}", width(&out));
        assert_eq!(out.chars().filter(|&c| c == '漢' || c == '字').count(), 1);
    }

    #[test]
    fn a_limit_narrower_than_the_ellipsis_leaves_nothing() {
        assert_eq!(truncate("LONG", 0.1), "");
    }
}
