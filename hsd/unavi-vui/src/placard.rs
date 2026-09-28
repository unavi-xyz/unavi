use arrayvec::ArrayVec;
use smol_str::SmolStr;
use wired_math::types::{
    Vec2,
    Vec3,
};

use crate::{
    mote::{
        MoteSpec,
        Role,
    },
    tuning::Tuning,
};

/// Lines a placard may draw, title included.
pub const MAX_LINES: usize = 8;

/// Mounted text: what this mote is and how to use it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Placard {
    pub title: SmolStr,
    /// Prose, without line breaks; [`view`] wraps it.
    pub body:  ArrayVec<SmolStr, MAX_LINES>,
}

impl Placard {
    #[must_use]
    pub fn new(title: impl Into<SmolStr>) -> Self {
        Self {
            title: title.into(),
            body:  ArrayVec::new(),
        }
    }

    /// Paragraphs past [`MAX_LINES`] are dropped rather than panicking.
    #[must_use]
    pub fn line(mut self, line: impl Into<SmolStr>) -> Self {
        let _ = self.body.try_push(line.into());
        self
    }

    /// The standard placard for a mote: its name, its description, and what
    /// kind of thing it is.
    #[must_use]
    pub fn describing(spec: &MoteSpec) -> Self {
        let mut placard = Self::new(spec.label.clone());
        if let Some(description) = &spec.description {
            placard = placard.line(description.clone());
        }
        placard.line(kind(spec.role))
    }
}

/// What a mote is, in one word. The gestures are the same everywhere and are
/// learned once; what changes between motes is what they are, which is the
/// thing a placard can say and a body cannot.
const fn kind(role: Role) -> &'static str {
    match role {
        // How it opens is the group's own setting; a grid and an orbit are the
        // same kind of thing and read as one.
        Role::Group { .. } => "group",
        Role::Parent { .. } => "back",
        Role::Cast => "cast",
        Role::Item { unique: true } => "item",
        Role::Item { unique: false } => "source",
        Role::Action => "action",
        Role::Toggle => "toggle",
    }
}

/// How strongly a line is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    Title,
    Body,
    /// The operating hint.
    Dim,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacardLine {
    pub text:     SmolStr,
    /// Placard-local, from the left inner edge, y descending.
    pub offset:   Vec2,
    pub size:     f32,
    pub emphasis: Emphasis,
}

/// Everything a renderer needs to draw one placard.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacardView {
    /// Orbit-local, at the placard's top centre.
    pub position: Vec3,
    /// 0 draws nothing.
    pub opacity:  f32,
    /// Backing panel extents, sized to the laid-out lines.
    pub size:     Vec2,
    pub lines:    ArrayVec<PlacardLine, MAX_LINES>,
}

/// Zero until the dwell delay has passed, then ramps in over
/// [`Tuning::placard_fade`].
#[must_use]
pub fn opacity(dwell: f32, tuning: &Tuning) -> f32 {
    if tuning.placard_fade <= f32::EPSILON {
        return f32::from(u8::from(dwell >= tuning.placard_delay));
    }
    ((dwell - tuning.placard_delay) / tuning.placard_fade).clamp(0.0, 1.0)
}

/// Mounts a placard clear of the body at `mote`.
///
/// The panel hangs down from its own top edge, so the mount is a whole
/// `height` above the mote rather than resting on it — the two would otherwise
/// overlap for every placard that is more than a line long. Above rather than
/// beside, because the mote's own name is already below it and a placard that
/// picked a side would leave the surface at the edge slots.
fn mount(mote: Vec3, radius: f32, height: f32, tuning: &Tuning) -> Vec3 {
    Vec3::new(
        mote.x,
        mote.y + radius + tuning.placard_gap + height,
        mote.z + tuning.placard_lift,
    )
}

/// Longest line, in characters, that fits `width` at `size`, estimated from
/// [`Tuning::advance_estimate`]. Only the renderer knows what a string
/// measures.
fn budget(width: f32, size: f32, tuning: &Tuning) -> usize {
    let glyph = size * tuning.advance_estimate;
    if glyph <= f32::EPSILON {
        return usize::MAX;
    }
    ((width / glyph) as usize).max(1)
}

/// Greedy wrap at `budget` characters, breaking on spaces and newlines.
///
/// A word longer than the budget is split where it runs out of room.
fn wrap(text: &str, budget: usize, out: &mut ArrayVec<SmolStr, MAX_LINES>) {
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut word = word;
            while word.chars().count() > budget {
                let (head, tail) = split_at_chars(word, budget);
                if !line.is_empty() {
                    push(out, &line);
                    line.clear();
                }
                push(out, head);
                word = tail;
            }
            let candidate =
                line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
            if !line.is_empty() && candidate > budget {
                push(out, &line);
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        push(out, &line);
    }
}

fn push(out: &mut ArrayVec<SmolStr, MAX_LINES>, text: &str) {
    let _ = out.try_push(SmolStr::new(text));
}

fn split_at_chars(text: &str, count: usize) -> (&str, &str) {
    let at = text
        .char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| index);
    text.split_at(at)
}

/// Lays a placard out, top-centre origin, y descending. Wrapping happens
/// here, not in the renderer.
#[must_use]
pub fn view(
    placard: &Placard,
    mote: Vec3,
    radius: f32,
    opacity: f32,
    tuning: &Tuning,
) -> PlacardView {
    let mut lines = ArrayVec::new();
    let inner = tuning.placard_pad.mul_add(-2.0, tuning.placard_width);
    let mut y = -tuning.placard_pad;

    if !placard.title.is_empty() {
        let mut wrapped = ArrayVec::new();
        wrap(
            &placard.title,
            budget(inner, tuning.placard_title, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= tuning.placard_title;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_title,
                emphasis: Emphasis::Title,
            });
        }
    }

    let step = tuning.placard_row * tuning.placard_line;
    let last = placard.body.len().saturating_sub(1);
    for (index, paragraph) in placard.body.iter().enumerate() {
        let mut wrapped = ArrayVec::new();
        wrap(
            paragraph,
            budget(inner, tuning.placard_row, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= step;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_row,
                emphasis: if index == last {
                    Emphasis::Dim
                } else {
                    Emphasis::Body
                },
            });
        }
    }

    let size = Vec2::new(tuning.placard_width, -y + tuning.placard_pad);
    PlacardView {
        position: mount(mote, radius, size.y, tuning),
        opacity,
        size,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mote::Arrange;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn spec(role: Role) -> MoteSpec {
        MoteSpec {
            role,
            label: SmolStr::new_static("Citrus"),
            description: Some(SmolStr::new_static("Sharp and bright")),
            active: false,
            icon: false,
            tint: None,
            film: crate::palette::FILM,
            frost: 0.0,
        }
    }

    fn group() -> MoteSpec {
        spec(Role::Group {
            children: 4,
            groups:   1,
            arrange:  Arrange::Orbit,
        })
    }

    fn view_of(placard: &Placard) -> PlacardView {
        view(placard, Vec3::ZERO, 0.03, 1.0, &tuning())
    }

    fn texts(view: &PlacardView) -> Vec<&str> {
        view.lines.iter().map(|line| line.text.as_str()).collect()
    }

    fn wrapped(text: &str, budget: usize) -> Vec<String> {
        let mut out = ArrayVec::new();
        wrap(text, budget, &mut out);
        out.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_placard_waits_then_fades_rather_than_popping() {
        let tuning = tuning();
        assert!(opacity(0.0, &tuning).abs() < 1.0e-5);
        assert!(opacity(tuning.placard_delay, &tuning).abs() < 1.0e-5);

        let midway = opacity(tuning.placard_delay + tuning.placard_fade / 2.0, &tuning);
        assert!(midway > 0.0 && midway < 1.0, "it arrives over time");
        assert!((opacity(10.0, &tuning) - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn it_mounts_clear_of_the_body_not_out_in_the_room() {
        let tuning = tuning();
        let radius = 0.03;
        let mote = Vec3::new(0.1, 0.2, 0.0);
        let view = view(&Placard::describing(&group()), mote, radius, 1.0, &tuning);

        assert!(
            (view.position.x - mote.x).abs() < 1.0e-5,
            "centred on the mote"
        );
        assert!(
            view.position.y - view.size.y >= mote.y + radius,
            "the panel hangs down from its mount, so its whole height has to \
             clear the body or it is drawn across it"
        );
        assert!(
            view.position.z < tuning.mote_radius,
            "a placard further out than the mote is wide stops reading as its"
        );
    }

    #[test]
    fn a_mote_explains_itself_without_being_told_to() {
        let view = view_of(&Placard::describing(&group()));
        assert_eq!(view.lines[0].text, "Citrus");
        assert_eq!(view.lines[0].emphasis, Emphasis::Title);
        assert!(texts(&view).contains(&"Sharp and bright"));
        assert!(texts(&view).contains(&"group"));
    }

    /// The counts and the kind readout were removed on purpose: the pips
    /// already show how much a group holds.
    #[test]
    fn it_says_nothing_the_body_already_shows() {
        let view = view_of(&Placard::describing(&group()));
        for noise in ["Kind", "Group", "Holds", "4", "Groups", "1"] {
            assert!(!texts(&view).contains(&noise), "{noise:?} is noise");
        }
    }

    #[test]
    fn the_last_line_names_what_the_mote_is_rather_than_a_gesture() {
        assert_eq!(kind(Role::Action), "action");
        assert_eq!(kind(Role::Item { unique: true }), "item");
        assert_eq!(kind(Role::Item { unique: false }), "source");
        assert_eq!(
            kind(Role::Group {
                children: 2,
                groups:   0,
                arrange:  Arrange::Grid,
            }),
            "group",
            "a grid is a group that opens as one, not another kind of mote"
        );
        for role in [
            Role::Action,
            Role::Cast,
            Role::Item { unique: true },
            Role::Parent { depth: 1 },
        ] {
            let word = kind(role);
            assert!(
                !word.contains(' ') && word.chars().all(char::is_lowercase),
                "{word:?} is an instruction, not a name"
            );
        }
    }

    #[test]
    fn what_a_mote_is_reads_quieter_than_what_it_says() {
        let view = view_of(&Placard::describing(&group()));
        let last = view.lines.last().expect("lines");
        assert_eq!(last.text, "group");
        assert_eq!(last.emphasis, Emphasis::Dim);
        assert_eq!(view.lines[1].emphasis, Emphasis::Body);
    }

    #[test]
    fn a_mote_with_nothing_to_add_still_says_what_it_is() {
        let view = view_of(&Placard::describing(&MoteSpec {
            description: None,
            ..spec(Role::Action)
        }));
        assert!(texts(&view).contains(&"action"));
    }

    #[test]
    fn prose_wraps_without_the_author_asking() {
        assert_eq!(
            wrapped("one two three four", 9),
            vec!["one two", "three", "four"]
        );
    }

    #[test]
    fn a_word_longer_than_the_line_is_split_rather_than_left_hanging() {
        let lines = wrapped("a supercalifragilistic word", 8);
        assert!(
            lines.iter().all(|line| line.chars().count() <= 8),
            "{lines:?}"
        );
        assert_eq!(lines.concat().replace(' ', ""), "asupercalifragilisticword");
    }

    #[test]
    fn an_author_may_still_break_a_line_by_hand() {
        assert_eq!(wrapped("a\nb", 40), vec!["a", "b"]);
    }

    #[test]
    fn the_budget_errs_toward_breaking_early() {
        let tuning = tuning();
        let inner = tuning.placard_pad.mul_add(-2.0, tuning.placard_width);
        let chars = budget(inner, tuning.placard_row, &tuning);
        assert!(
            (chars as f32) * tuning.placard_row * 0.5 < inner,
            "a line of average lowercase must not fill the card, or the \
             estimate has no headroom for wider text"
        );
    }

    #[test]
    fn a_degenerate_size_does_not_hang() {
        assert_eq!(budget(0.2, 0.0, &tuning()), usize::MAX);
    }

    #[test]
    fn every_wrapped_line_earns_panel_height() {
        let short = view_of(&Placard::new("t").line("one"));
        let long = view_of(&Placard::new("t").line(
            "a description long enough that it certainly has to break across \
             more than a single line of this card",
        ));
        assert!(long.lines.len() > short.lines.len());
        assert!(
            long.size.y > short.size.y,
            "the backdrop grows with what wrapping produced, which is the \
             whole reason wrapping happens here and not in the renderer"
        );
    }

    #[test]
    fn every_line_stays_within_the_panel() {
        let view = view_of(&Placard::describing(&group()));
        let half = view.size.x / 2.0;
        for line in &view.lines {
            assert!(line.offset.x.abs() <= half, "{:?} escapes", line.text);
            assert!(-line.offset.y <= view.size.y);
        }
    }

    #[test]
    fn everything_shares_a_left_edge() {
        let view = view_of(&Placard::describing(&group()));
        let left = view.lines[0].offset.x;
        for line in &view.lines {
            assert!(
                (line.offset.x - left).abs() < 1.0e-6,
                "{:?} does not line up with the title",
                line.text
            );
        }
    }

    #[test]
    fn lines_descend() {
        let view = view_of(&Placard::describing(&group()));
        for pair in view.lines.windows(2) {
            assert!(pair[0].offset.y > pair[1].offset.y);
        }
    }

    #[test]
    fn a_titleless_placard_starts_with_its_first_line() {
        let view = view_of(&Placard::default().line("bare"));
        assert_eq!(view.lines.len(), 1);
        assert_ne!(view.lines[0].emphasis, Emphasis::Title);
    }

    #[test]
    fn more_lines_than_the_budget_are_dropped_rather_than_panicking() {
        let mut placard = Placard::new("t");
        for index in 0..MAX_LINES + 4 {
            placard = placard.line(index.to_string());
        }
        assert_eq!(placard.body.len(), MAX_LINES);
        assert!(view_of(&placard).lines.len() <= MAX_LINES);
    }

    #[test]
    fn a_paragraph_that_wraps_past_the_budget_is_truncated_not_overrun() {
        let placard = Placard::new("t").line("word ".repeat(200));
        assert!(view_of(&placard).lines.len() <= MAX_LINES);
    }
}
