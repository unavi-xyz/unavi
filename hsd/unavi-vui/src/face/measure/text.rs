//! Text measured in ems against Noto Sans Regular.

use std::mem::take;

use crate::face::measure::noto::ADVANCES;

const ELLIPSIS: char = '\u{2026}';

/// Slack for the sums of f32 advances, so a line that fits exactly does fit.
const EPSILON: f32 = 1.0e-4;

/// The em a wide character takes: CJK text is set on a square.
const WIDE: f32 = 1.0;

/// Advance of the widest character in the table, in ems.
const WIDEST: f32 = widest() as f32 / 1000.0;

/// Advance of `c` in ems. A wide character takes [`WIDE`], and a character
/// outside the table takes the widest advance in it, so text measures no
/// narrower than it draws.
#[must_use]
pub fn advance(c: char) -> f32 {
    if is_wide(c) {
        return WIDE;
    }
    ADVANCES
        .binary_search_by_key(&c, |&(ch, _)| ch)
        .map_or(WIDEST, |i| f32::from(ADVANCES[i].1) / 1000.0)
}

/// Width of `text` in ems on one line.
#[must_use]
pub fn width(text: &str) -> f32 {
    text.chars().map(advance).sum()
}

/// Shortens `text` to at most `max_em` by cutting its end and appending an
/// ellipsis. Text that fits is returned unchanged, and a limit narrower than
/// the ellipsis leaves nothing.
#[must_use]
pub fn truncate(text: &str, max_em: f32) -> String {
    if width(text) <= max_em + EPSILON {
        return text.to_owned();
    }
    let ellipsis = advance(ELLIPSIS);
    if ellipsis > max_em + EPSILON {
        return String::new();
    }

    let mut out = String::new();
    let mut used = ellipsis;
    for c in text.chars() {
        used += advance(c);
        if used > max_em + EPSILON {
            break;
        }
        out.push(c);
    }
    out.push(ELLIPSIS);
    out
}

/// Breaks `text` into lines of at most `max_em`. Lines break at whitespace,
/// and a word longer than a line breaks between characters. `\n` starts a new
/// line and a blank line survives.
#[must_use]
pub fn wrap(text: &str, max_em: f32) -> Vec<String> {
    let space = advance(' ');
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut used = 0.0;

        for word in paragraph.split_whitespace() {
            let word_width = width(word);
            if used > 0.0 && used + space + word_width <= max_em + EPSILON {
                line.push(' ');
                line.push_str(word);
                used += space + word_width;
                continue;
            }
            if used > 0.0 {
                lines.push(take(&mut line));
                used = 0.0;
            }
            for c in word.chars() {
                let step = advance(c);
                if used > 0.0 && used + step > max_em + EPSILON {
                    lines.push(take(&mut line));
                    used = 0.0;
                }
                line.push(c);
                used += step;
            }
        }
        lines.push(line);
    }

    lines
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

const fn is_wide(c: char) -> bool {
    matches!(
        c,
        '\u{1100}'..='\u{11FF}'
            | '\u{2E80}'..='\u{2FDF}'
            | '\u{3000}'..='\u{33FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF00}'..='\u{FF60}'
            | '\u{FFE0}'..='\u{FFE7}'
            | '\u{20000}'..='\u{3134F}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-4
    }

    #[test]
    fn the_table_is_sorted_for_lookup() {
        assert!(ADVANCES.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn width_sums_the_advances() {
        let expected = advance('A') + advance('i');
        assert!(near(width("Ai"), expected));
        assert!(near(width(""), 0.0));
    }

    #[test]
    fn a_proportional_face_makes_an_i_narrower_than_an_m() {
        assert!(width("iiii") < width("mmmm"));
    }

    #[test]
    fn a_wide_character_is_one_em_and_an_unknown_one_the_widest_advance() {
        assert!(near(advance('漢'), 1.0));
        assert!(near(advance('\u{0e01}'), WIDEST));
        assert!(ADVANCES.iter().all(|&(c, _)| advance(c) <= WIDEST));
    }

    #[test]
    fn text_that_fits_is_unchanged_and_long_text_ends_in_an_ellipsis_within_the_limit() {
        assert_eq!(truncate("Home", 10.0), "Home");
        let out = truncate("A very long space name", 5.0);
        assert!(out.ends_with(ELLIPSIS));
        assert!(width(&out) <= 5.0 + EPSILON, "{}", width(&out));
        assert_eq!(truncate("long", 0.1), "");
    }

    #[test]
    fn words_wrap_at_the_width() {
        let widest = width("the quick").max(width("brown fox"));
        assert_eq!(wrap("the quick brown fox", widest), ["the quick", "brown fox"]);
        assert_eq!(wrap("a b", 24.0), ["a b"]);
    }

    #[test]
    fn a_newline_starts_a_line_and_a_blank_line_survives() {
        assert_eq!(wrap("a\n\nb", 24.0), ["a", "", "b"]);
    }

    #[test]
    fn a_word_longer_than_the_line_breaks_between_characters() {
        let line = width("abc");
        assert_eq!(wrap("abcdefgh", line), ["abc", "def", "gh"]);
    }

    #[test]
    fn wrapped_lines_never_exceed_the_width() {
        for tenths in 12..60 {
            let max = tenths as f32 / 10.0;
            for line in wrap("漢字 and some ascii 한글 together, wrapping", max) {
                assert!(width(&line) <= max + EPSILON, "{line:?} in {max}");
            }
        }
    }

    #[test]
    fn a_character_wider_than_the_line_still_advances() {
        assert_eq!(wrap("漢字", 0.5), ["漢", "字"]);
    }
}
