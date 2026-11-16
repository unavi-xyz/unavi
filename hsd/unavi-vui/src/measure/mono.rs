//! Monospace text measured in cells.

use std::mem::take;

/// Horizontal advance of one cell, in ems.
pub const ADVANCE: f32 = 0.6;
/// Height of one line, in ems.
pub const LINE: f32 = 1.5;

const ELLIPSIS: char = '\u{2026}';

/// Cells taken by `c`: two for the wide ranges of CJK text, one otherwise.
/// Half-width forms are narrow.
pub const fn cells(c: char) -> usize {
    if is_wide(c) { 2 } else { 1 }
}

/// Cells taken by `text` on one line.
pub fn width(text: &str) -> usize {
    text.chars().map(cells).sum()
}

/// Breaks `text` into lines of at most `columns` cells. Lines break at
/// whitespace, and a word longer than a line breaks between characters. A
/// wide character never straddles a line end. `\n` starts a new line.
pub fn wrap(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut used = 0;

        for word in paragraph.split_whitespace() {
            let word_width = width(word);
            if used > 0 && used + 1 + word_width <= columns {
                line.push(' ');
                line.push_str(word);
                used += 1 + word_width;
                continue;
            }
            if used > 0 {
                lines.push(take(&mut line));
                used = 0;
            }
            for c in word.chars() {
                let advance = cells(c);
                if used > 0 && used + advance > columns {
                    lines.push(take(&mut line));
                    used = 0;
                }
                line.push(c);
                used += advance;
            }
        }
        lines.push(line);
    }

    lines
}

/// Shortens an identifier to at most `columns` cells by replacing its middle
/// with an ellipsis, so both ends stay readable. A shorter identifier is
/// returned unchanged.
pub fn truncate_middle(text: &str, columns: usize) -> String {
    if width(text) <= columns {
        return text.to_owned();
    }
    if columns == 0 {
        return String::new();
    }

    let budget = columns - 1;
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let advance = cells(c);
        if used + advance > budget.div_ceil(2) {
            break;
        }
        out.push(c);
        used += advance;
    }

    out.push(ELLIPSIS);

    let mut tail = Vec::new();
    used = 0;
    for c in text.chars().rev() {
        let advance = cells(c);
        if used + advance > budget / 2 {
            break;
        }
        tail.push(c);
        used += advance;
    }
    out.extend(tail.into_iter().rev());

    out
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

    #[test]
    fn ascii_is_one_cell_per_character() {
        assert_eq!(width("hello world"), 11);
        assert_eq!(width(""), 0);
    }

    #[test]
    fn wide_characters_take_two_cells_and_half_width_forms_one() {
        assert_eq!(width("漢字"), 4);
        assert_eq!(width("한"), 2);
        assert_eq!(width("Ａ"), 2);
        assert_eq!(width("ｱ"), 1);
    }

    #[test]
    fn the_cell_metrics_are_the_mono_faces() {
        assert!((ADVANCE - 0.6).abs() < 1e-6);
        assert!((LINE - 1.5).abs() < 1e-6);
    }

    #[test]
    fn ascii_wraps_at_word_boundaries() {
        assert_eq!(wrap("the quick brown fox", 9), ["the quick", "brown fox"]);
    }

    #[test]
    fn a_line_that_fits_is_one_line() {
        assert_eq!(wrap("a b", 24), ["a b"]);
    }

    #[test]
    fn a_newline_starts_a_line_and_blank_lines_survive() {
        assert_eq!(wrap("a\n\nb", 24), ["a", "", "b"]);
    }

    #[test]
    fn a_wide_character_that_would_straddle_the_line_end_moves_down() {
        assert_eq!(wrap("abcd漢", 5), ["abcd", "漢"]);
    }

    #[test]
    fn a_word_longer_than_the_line_breaks_between_characters() {
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn a_word_after_a_broken_one_continues_its_last_line() {
        assert_eq!(wrap("abcdefgh i", 5), ["abcde", "fgh i"]);
    }

    #[test]
    fn a_wide_character_wider_than_the_line_still_advances() {
        assert_eq!(wrap("漢字", 1), ["漢", "字"]);
    }

    #[test]
    fn wrapped_lines_never_exceed_the_columns() {
        for columns in 2..12 {
            for line in wrap("漢字 and some ascii 한글 together", columns) {
                assert!(width(&line) <= columns, "{line:?} in {columns}");
            }
        }
    }

    #[test]
    fn an_identifier_shorter_than_the_limit_is_unchanged() {
        assert_eq!(truncate_middle("did:key:z6Mk", 24), "did:key:z6Mk");
        assert_eq!(truncate_middle("exact", 5), "exact");
    }

    #[test]
    fn a_long_identifier_keeps_both_ends() {
        let out = truncate_middle("did:key:z6MkhaXgBZDvotDkL5257", 12);
        assert_eq!(out, "did:ke…L5257");
        assert!(width(&out) <= 12);
    }

    #[test]
    fn truncation_fits_for_every_limit() {
        for columns in 0..20 {
            let out = truncate_middle("abc漢字defghijklmnop", columns);
            assert!(width(&out) <= columns, "{out:?} in {columns}");
        }
    }
}
