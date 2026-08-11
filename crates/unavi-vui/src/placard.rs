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

/// Lines a placard may draw, title included. A placard that needs more is the
/// wrong tool, and the content belongs in a planted station instead.
pub const MAX_LINES: usize = 8;

/// Mounted text: what this mote is, and how to use it.
///
/// Deliberately only that. It first carried a kind readout, child counts,
/// meters and dividers, and every one was noise — the pips already say how
/// much a group holds, the hint already implies what sort of thing it is, and
/// the bars read as two stray lines under everything. What is left is what a
/// reader wanted: a name, an explanation, and the gesture.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Placard {
    pub title: SmolStr,
    /// Prose. Written plainly, without line breaks — [`view`] wraps it.
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

    /// Paragraphs past [`MAX_LINES`] are dropped rather than panicking: a
    /// placard is chrome, and chrome must not be able to take a script down.
    #[must_use]
    pub fn line(mut self, line: impl Into<SmolStr>) -> Self {
        let _ = self.body.try_push(line.into());
        self
    }

    /// The standard placard for a mote: its name, what it does, and the
    /// gesture that does it.
    ///
    /// Derived rather than authored so every mote in a tree explains itself
    /// consistently, and a consumer supplies only the one line it alone knows
    /// — [`MoteSpec::description`].
    #[must_use]
    pub fn describing(spec: &MoteSpec) -> Self {
        let mut placard = Self::new(spec.label.clone());
        if let Some(description) = &spec.description {
            placard = placard.line(description.clone());
        }
        placard.line(hint(spec.role))
    }
}

/// How to operate this mote, named for the gesture that does it rather than
/// the abstraction behind it — desktop and VR both grab, and nothing in this
/// interface taps.
const fn hint(role: Role) -> &'static str {
    match role {
        Role::Group { .. } => "Grab to open",
        Role::Parent { .. } => "Grab to go back up a level",
        Role::Cast => "Hold to confirm",
        Role::Item => "Grab and drag to take",
        Role::Action => "Grab to activate",
    }
}

/// How loudly a line is set. The binding maps these onto the palette, so a
/// placard holds no colours of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    Title,
    Body,
    /// The operating hint: present, but not the point.
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

/// Everything a renderer needs to draw one placard, in concrete values.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacardView {
    /// Orbit-local, at the placard's top centre.
    pub position: Vec3,
    /// 0 draws nothing. Fades rather than toggling.
    pub opacity:  f32,
    /// Backing panel extents, sized to the lines actually laid out.
    pub size:     Vec2,
    pub lines:    ArrayVec<PlacardLine, MAX_LINES>,
}

/// Graded reveal.
///
/// Zero until the dwell delay has passed, then in over
/// [`Tuning::placard_fade`] — the surface itself reacts instantly, so nothing
/// feels laggy while the text still never flickers during a sweep.
#[must_use]
pub fn opacity(dwell: f32, tuning: &Tuning) -> f32 {
    if tuning.placard_fade <= f32::EPSILON {
        return f32::from(u8::from(dwell >= tuning.placard_delay));
    }
    ((dwell - tuning.placard_delay) / tuning.placard_fade).clamp(0.0, 1.0)
}

/// Mounts a placard on the body at `mote`.
///
/// Above rather than beside: a side offset has to pick a side, and whichever
/// it picks covers a sibling on half the orbit. It sits only just clear of the
/// body — a placard floating well in front of the dial reads as belonging to
/// the room rather than to the mote.
fn mount(mote: Vec3, radius: f32, tuning: &Tuning) -> Vec3 {
    Vec3::new(
        mote.x,
        mote.y + radius + tuning.placard_gap,
        mote.z + tuning.placard_lift,
    )
}

/// Longest line, in characters, that fits `width` at `size`.
///
/// An estimate, because only the renderer knows what a string measures. It is
/// deliberately a *pessimistic* one: breaking a line early costs a card that
/// is wider than its text, while breaking late costs text hanging off the
/// backdrop. [`Tuning::advance_estimate`] is the average glyph width the guess
/// assumes.
fn budget(width: f32, size: f32, tuning: &Tuning) -> usize {
    let glyph = size * tuning.advance_estimate;
    if glyph <= f32::EPSILON {
        return usize::MAX;
    }
    ((width / glyph) as usize).max(1)
}

/// Greedy wrap at `budget` characters, breaking on spaces.
///
/// A word longer than the whole budget is split where it ran out of room,
/// which is the only alternative to letting it overhang. Newlines in the
/// source break too, so an author who *wants* a break can still have one.
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

/// Lays a placard out, top-centre origin, y descending.
///
/// Wrapping happens **here**, not in the renderer. The renderer is the only
/// thing that knows what a string measures, so letting it wrap would add lines
/// this never sized the panel for — which is exactly how the first cut spilled
/// text off its own backdrop. Wrapping to an estimate instead keeps the height
/// exact and keeps line breaks out of the author's hands.
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
                // The hint is always last, and is the one line a reader who
                // already knows the gesture should be able to skip.
                emphasis: if index == last {
                    Emphasis::Dim
                } else {
                    Emphasis::Body
                },
            });
        }
    }

    PlacardView {
        position: mount(mote, radius, tuning),
        opacity,
        size: Vec2::new(tuning.placard_width, -y + tuning.placard_pad),
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn spec(role: Role) -> MoteSpec {
        MoteSpec {
            role,
            label: SmolStr::new_static("Citrus"),
            description: Some(SmolStr::new_static("Sharp and bright")),
        }
    }

    fn group() -> MoteSpec {
        spec(Role::Group {
            children: 4,
            groups:   1,
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
    fn it_mounts_just_above_the_body_not_out_in_the_room() {
        let tuning = tuning();
        let mounted = mount(Vec3::new(0.1, 0.2, 0.0), 0.03, &tuning);
        assert!((mounted.x - 0.1).abs() < 1.0e-5, "centred on the mote");
        assert!(mounted.y > 0.2 + 0.03, "clear of the body's surface");
        assert!(
            mounted.z < tuning.mote_radius,
            "a placard further out than the mote is wide stops reading as its"
        );
    }

    #[test]
    fn a_mote_explains_itself_without_being_told_to() {
        let view = view_of(&Placard::describing(&group()));
        assert_eq!(view.lines[0].text, "Citrus");
        assert_eq!(view.lines[0].emphasis, Emphasis::Title);
        assert!(texts(&view).contains(&"Sharp and bright"));
        assert!(texts(&view).contains(&"Grab to open"));
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
    fn the_hint_names_the_gesture_that_actually_works() {
        assert_eq!(hint(Role::Action), "Grab to activate");
        assert_eq!(hint(Role::Item), "Grab and drag to take");
        assert_eq!(
            hint(Role::Parent { depth: 1 }),
            "Grab to go back up a level",
            "the way back is the one control nobody is told about"
        );
    }

    #[test]
    fn the_hint_is_the_quietest_line() {
        let view = view_of(&Placard::describing(&group()));
        let last = view.lines.last().expect("lines");
        assert_eq!(last.text, "Grab to open");
        assert_eq!(last.emphasis, Emphasis::Dim);
        assert_eq!(view.lines[1].emphasis, Emphasis::Body);
    }

    #[test]
    fn a_mote_with_nothing_to_add_still_explains_how_to_use_it() {
        let view = view_of(&Placard::describing(&MoteSpec {
            description: None,
            ..spec(Role::Action)
        }));
        assert!(texts(&view).contains(&"Grab to activate"));
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
