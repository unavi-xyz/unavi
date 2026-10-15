//! A 3 by 5 dot font, and the readout laid out in it.
//!
//! A readout is text a consumer wants shown as a display rather than as a
//! label, so it is set in the same dots a glyph is: the font renders into a
//! [`Glyph`], and everything that draws or flips a glyph draws and flips the
//! readout.

use crate::glyph::{
    Glyph,
    MAX_SIZE,
};

/// Dots to a side of the readout's grid.
pub const GRID: u8 = MAX_SIZE as u8;

const WIDTH: usize = 3;
const HEIGHT: usize = 5;
/// Dots between two characters, and between two lines.
const GAP: usize = 1;

/// Most characters on a line when the readout is one line. Three fit across
/// the middle of the circle, where the clip is widest.
const ONE_LINE: usize = 3;
/// Most characters on a line when the readout is two. Three would put the
/// outer corners of the top and bottom rows past the clip circle, so a third
/// character would lose dots it was drawn with.
const TWO_LINES: usize = 2;
const MAX_LINES: usize = 2;

/// Rows of a character, top first, `#` for a lit dot. Lower case reads as
/// upper; anything the font lacks is blank.
const fn rows(c: char) -> [&'static str; HEIGHT] {
    match c.to_ascii_uppercase() {
        '0' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#.", "##.", ".#.", ".#.", "###"],
        '2' => ["###", "..#", "###", "#..", "###"],
        '3' => ["###", "..#", "###", "..#", "###"],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' => ["###", "#..", "###", "..#", "###"],
        '6' => ["###", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", ".#.", ".#.", ".#."],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "###"],
        ':' => ["...", ".#.", "...", ".#.", "..."],
        '-' => ["...", "...", "###", "...", "..."],
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'C' => [".##", "#..", "#..", "#..", ".##"],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => [".##", "#..", "#.#", "#.#", ".##"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'J' => ["..#", "..#", "..#", "#.#", ".#."],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["##.", "#.#", "#.#", "#.#", "#.#"],
        'O' => [".#.", "#.#", "#.#", "#.#", ".#."],
        'P' => ["##.", "#.#", "##.", "#..", "#.."],
        'Q' => [".#.", "#.#", "#.#", "##.", ".##"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => ["#.#", "#.#", "###", "###", "#.#"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        _ => ["...", "...", "...", "...", "..."],
    }
}

/// The lit dots of a readout's grid, row by row.
type Dots = [[bool; GRID as usize]; GRID as usize];

/// Where `text` puts its dots.
///
/// Lines split at `\n`; past two lines is dropped, and so is whatever a line
/// has more characters than fit. Each line is centred across the grid and the
/// block of lines down it, a dot to the left of centre when the room left is
/// odd.
fn lay_out(text: &str) -> Dots {
    let lines = text.split('\n').take(MAX_LINES).collect::<Vec<_>>();
    let per_line = if lines.len() > 1 { TWO_LINES } else { ONE_LINE };
    let height = lines.len() * HEIGHT + (lines.len() - 1) * GAP;
    let top = (usize::from(GRID) - height) / 2;

    let mut dots = [[false; GRID as usize]; GRID as usize];
    for (line, content) in lines.iter().enumerate() {
        let characters = content.chars().take(per_line).collect::<Vec<_>>();
        let width = (characters.len() * (WIDTH + GAP)).saturating_sub(GAP);
        let left = (usize::from(GRID) - width) / 2;
        let line_top = top + line * (HEIGHT + GAP);
        for (index, character) in characters.into_iter().enumerate() {
            for (down, pattern) in rows(character).into_iter().enumerate() {
                for (across, dot) in pattern.chars().enumerate() {
                    if dot == '#' {
                        let column = left + index * (WIDTH + GAP) + across;
                        dots[line_top + down][column] = true;
                    }
                }
            }
        }
    }
    dots
}

/// `text` as a glyph on the readout's grid.
#[must_use]
pub fn readout(text: &str) -> Glyph {
    let dots = lay_out(text);
    let size = usize::from(GRID);
    let mut bits = vec![0_u8; (size * size).div_ceil(8)];
    for (row, line) in dots.iter().enumerate() {
        for (column, lit) in line.iter().enumerate() {
            if *lit {
                let cell = row * size + column;
                bits[cell / 8] |= 1 << (cell % 8);
            }
        }
    }
    // The grid is a valid size and `bits` is exactly long enough.
    Glyph::new(GRID, &bits).expect("a 13 grid with bits for every cell is always valid")
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::{
        flip::Flip,
        glyph::inside,
    };

    const ALPHABET: &str = "0123456789:- ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    fn lit(dots: &Dots) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (row, line) in dots.iter().enumerate() {
            for (column, on) in line.iter().enumerate() {
                if *on {
                    out.push((column, row));
                }
            }
        }
        out
    }

    fn within_circle(dots: &Dots) -> bool {
        lit(dots)
            .iter()
            .all(|(column, row)| inside(usize::from(GRID), *column, *row))
    }

    #[test]
    fn every_character_draws_inside_the_circle_wherever_it_can_stand() {
        // Each character at each place a layout can put one, the other places
        // held by a wide character so the line is as wide as it can be.
        let wide = 'W';
        for character in ALPHABET.chars() {
            let one_line = (0..ONE_LINE).map(|place| {
                (0..ONE_LINE)
                    .map(|at| if at == place { character } else { wide })
                    .collect::<String>()
            });
            let two_lines = (0..2).flat_map(|line| {
                (0..TWO_LINES).map(move |place| {
                    let build = |is_line: bool| {
                        (0..TWO_LINES)
                            .map(|at| {
                                if is_line && at == place {
                                    character
                                } else {
                                    wide
                                }
                            })
                            .collect::<String>()
                    };
                    if line == 0 {
                        format!("{}\n{}", build(true), build(false))
                    } else {
                        format!("{}\n{}", build(false), build(true))
                    }
                })
            });
            for text in one_line.chain(two_lines) {
                assert!(
                    within_circle(&lay_out(&text)),
                    "{text:?} puts a dot outside the circle"
                );
            }
        }
    }

    #[test]
    fn what_is_drawn_is_what_the_glyph_keeps() {
        // The clip would silently drop a stray dot, so a readout is checked
        // to lose none: the glyph is the same as one built from the dots.
        for text in ["12\n34", "WWW", "88\n88", "-", ""] {
            let dots = lay_out(text);
            let size = usize::from(GRID);
            let mut bits = vec![0_u8; (size * size).div_ceil(8)];
            for (column, row) in lit(&dots) {
                let cell = row * size + column;
                bits[cell / 8] |= 1 << (cell % 8);
            }
            let raw_lit = lit(&dots).len();
            let kept = (0..size * size)
                .filter(|cell| inside(size, cell % size, cell / size))
                .filter(|cell| lit(&dots).contains(&(cell % size, cell / size)))
                .count();
            assert_eq!(raw_lit, kept, "{text:?}");
            assert_eq!(readout(text), Glyph::new(GRID, &bits).expect("valid"));
        }
    }

    #[test]
    fn a_line_is_centred_across_the_grid() {
        let columns = |text: &str| {
            let dots = lit(&lay_out(text));
            let first = dots.iter().map(|(column, _)| *column).min().unwrap_or(0);
            let last = dots.iter().map(|(column, _)| *column).max().unwrap_or(0);
            (first, last)
        };
        // "88" is seven wide with no blank edge: three dots of room each side.
        assert_eq!(columns("88"), (3, 9));
        // Three wide characters fill eleven, one dot of room each side.
        assert_eq!(columns("888"), (1, 11));
        // One character is three wide, five of room each side.
        assert_eq!(columns("8"), (5, 7));
    }

    #[test]
    fn two_lines_are_centred_down_the_grid_with_a_dot_between() {
        let rows_of = |text: &str| {
            let dots = lit(&lay_out(text));
            let first = dots.iter().map(|(_, row)| *row).min().unwrap_or(0);
            let last = dots.iter().map(|(_, row)| *row).max().unwrap_or(0);
            (first, last)
        };
        // Eleven rows of two lines and the gap, one of room above and below.
        assert_eq!(rows_of("88\n88"), (1, 11));
        // A single line has four above and four below.
        assert_eq!(rows_of("88"), (4, 8));
        let dots = lay_out("88\n88");
        assert!(dots[6].iter().all(|on| !on), "the gap between the lines");
    }

    #[test]
    fn an_hour_over_a_minute_reads_as_digits() {
        let dots = lay_out("12\n34");
        // The first line's first digit is a 1: the stem's foot is three wide.
        assert!(dots[5][3] && dots[5][4] && dots[5][5]);
        assert!(dots[1][4], "the stem of the 1");
    }

    #[test]
    fn unknown_characters_and_space_are_blank() {
        for text in ["é", "%", " ", "\u{1F600}"] {
            assert!(lit(&lay_out(text)).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn lower_case_reads_as_upper() {
        assert_eq!(lay_out("abc"), lay_out("ABC"));
    }

    #[test]
    fn what_does_not_fit_is_dropped_rather_than_clipped() {
        assert_eq!(lay_out("1234"), lay_out("123"));
        assert_eq!(lay_out("123\n456"), lay_out("12\n45"));
        assert_eq!(lay_out("12\n34\n56"), lay_out("12\n34"));
    }

    #[test]
    fn new_text_flips_over_from_the_old_on_the_same_face() {
        let mut flip = Flip::new();
        flip.show(0, Some(&Rc::new(readout("12\n34"))));
        assert!(!flip.is_flipping(), "the first text just appears");
        flip.show(0, Some(&Rc::new(readout("12\n35"))));
        assert!(flip.is_flipping(), "the minute ticking over flips");
        flip.show(0, Some(&Rc::new(readout("12\n35"))));
        assert!(
            flip.is_flipping(),
            "and the same text again changes nothing"
        );
        assert!(flip.progress().abs() < f32::EPSILON);
    }

    #[test]
    fn no_two_characters_look_alike() {
        let characters = ALPHABET.chars().collect::<Vec<_>>();
        for (index, first) in characters.iter().enumerate() {
            for second in &characters[index + 1..] {
                assert_ne!(rows(*first), rows(*second), "{first} and {second}");
            }
        }
    }
}
