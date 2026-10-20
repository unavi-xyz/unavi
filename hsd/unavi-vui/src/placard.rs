//! A mote's expanded text, laid out as lines a renderer can place directly.

use arrayvec::ArrayVec;
use smol_str::SmolStr;
use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use crate::{
    mote::MoteSpec,
    tuning::Tuning,
};

/// Lines a placard may draw, kicker and title included.
pub const MAX_LINES: usize = 9;

/// Longest title a script may set on a standing placard.
pub const MAX_TITLE_BYTES: usize = 256;
/// Longest kicker a script may set on a standing placard.
pub const MAX_KICKER_BYTES: usize = 256;
/// Most paragraphs a script may set under the title.
pub const MAX_PARAGRAPHS: usize = 7;
/// Longest paragraph a script may set on a standing placard.
pub const MAX_PARAGRAPH_BYTES: usize = 1024;

/// How a paragraph is set.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Body,
    /// Ink at reduced contrast.
    Muted,
    Warning,
}

/// One paragraph under the title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paragraph {
    /// Prose, without line breaks; [`view`] wraps it.
    pub text: SmolStr,
    pub tone: Tone,
}

/// Why a script's text was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    Title(usize),
    Kicker(usize),
    Paragraphs(usize),
    Paragraph(usize),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Title(len) => {
                write!(
                    f,
                    "title is {len} bytes, over the limit of {MAX_TITLE_BYTES}"
                )
            }
            Self::Kicker(len) => {
                write!(
                    f,
                    "kicker is {len} bytes, over the limit of {MAX_KICKER_BYTES}"
                )
            }
            Self::Paragraphs(count) => {
                write!(f, "{count} lines, over the limit of {MAX_PARAGRAPHS}")
            }
            Self::Paragraph(len) => {
                write!(
                    f,
                    "a line is {len} bytes, over the limit of {MAX_PARAGRAPH_BYTES}"
                )
            }
        }
    }
}

/// Mounted text: a title and what there is to say under it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Placard {
    /// A small muted line above the title: an id, a category.
    pub kicker:  Option<SmolStr>,
    pub title:   SmolStr,
    pub body:    ArrayVec<Paragraph, MAX_LINES>,
    /// Whether this is a notice, which sets its title as a warning.
    pub warning: bool,
}

impl Placard {
    #[must_use]
    pub fn new(title: impl Into<SmolStr>) -> Self {
        Self {
            kicker:  None,
            title:   title.into(),
            body:    ArrayVec::new(),
            warning: false,
        }
    }

    /// A notice: `message` set as a warning title, with nothing under it.
    #[must_use]
    pub fn notice(message: impl Into<SmolStr>) -> Self {
        Self {
            warning: true,
            ..Self::new(message)
        }
    }

    /// Paragraphs past [`MAX_LINES`] are dropped rather than panicking.
    #[must_use]
    pub fn line(self, line: impl Into<SmolStr>) -> Self {
        self.toned(line, Tone::Body)
    }

    /// [`Placard::line`], set in `tone`.
    #[must_use]
    pub fn toned(mut self, line: impl Into<SmolStr>, tone: Tone) -> Self {
        let _ = self.body.try_push(Paragraph {
            text: line.into(),
            tone,
        });
        self
    }

    #[must_use]
    pub fn kicker(mut self, kicker: impl Into<SmolStr>) -> Self {
        self.kicker = Some(kicker.into());
        self
    }

    /// A script's text, held to the limits a standing placard has.
    pub fn from_script(
        title: &str,
        kicker: Option<&str>,
        lines: &[(String, Tone)],
    ) -> Result<Self, Refused> {
        if title.len() > MAX_TITLE_BYTES {
            return Err(Refused::Title(title.len()));
        }
        if let Some(kicker) = kicker.filter(|kicker| kicker.len() > MAX_KICKER_BYTES) {
            return Err(Refused::Kicker(kicker.len()));
        }
        if lines.len() > MAX_PARAGRAPHS {
            return Err(Refused::Paragraphs(lines.len()));
        }
        if let Some(long) = lines
            .iter()
            .map(|(text, _)| text.len())
            .find(|&len| len > MAX_PARAGRAPH_BYTES)
        {
            return Err(Refused::Paragraph(long));
        }
        let placard = lines
            .iter()
            .fold(Self::new(title), |placard, (text, tone)| {
                placard.toned(text.as_str(), *tone)
            });
        Ok(match kicker {
            Some(kicker) => placard.kicker(kicker),
            None => placard,
        })
    }

    /// The standard placard for a mote: its name and its description.
    #[must_use]
    pub fn describing(spec: &MoteSpec) -> Self {
        let placard = Self::new(spec.label.clone());
        match &spec.description {
            Some(description) => placard.line(description.clone()),
            None => placard,
        }
    }
}

/// How strongly a line is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    Title,
    /// The small line above the title.
    Kicker,
    /// A notice's message, or a warning line, which wears a colour of its own.
    Warning,
    Body,
    /// A line set in ink at reduced contrast.
    Muted,
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
    /// Backing panel extents, sized to the laid-out lines.
    pub size:     Vec2,
    pub lines:    ArrayVec<PlacardLine, MAX_LINES>,
    /// What the pane's corner tab is drawn in, if it has one.
    pub accent:   Option<Color>,
    /// How many times larger than a mote's this is laid out, which the pane's
    /// shape follows.
    pub unit:     f32,
}

/// Whether attention has been held long enough to earn a placard. How it
/// arrives is the renderer's: it eases in once this is true.
#[must_use]
pub fn earned(dwell: f32, tuning: &Tuning) -> bool {
    dwell >= tuning.placard_delay
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
pub fn view(placard: &Placard, mote: Vec3, radius: f32, tuning: &Tuning) -> PlacardView {
    let (size, lines) = lay_out(placard, tuning);
    PlacardView {
        position: mount(mote, radius, size.y, tuning),
        size,
        lines,
        accent: None,
        unit: 1.0,
    }
}

/// Lays a placard out standing on the origin by its bottom centre, for one
/// that stands where it is put rather than on a mote.
///
/// Laid out as a mote's is and then scaled as a whole by
/// [`Tuning::stand_scale`], so the wrapping and the proportions are the
/// same and only the reading distance differs.
#[must_use]
pub fn standing(placard: &Placard, tuning: &Tuning) -> PlacardView {
    let (size, mut lines) = lay_out(placard, tuning);
    let scale = tuning.stand_scale;
    for line in &mut lines {
        line.offset *= scale;
        line.size *= scale;
    }
    let size = size * scale;
    PlacardView {
        position: Vec3::new(0.0, size.y, 0.0),
        size,
        lines,
        accent: None,
        unit: scale,
    }
}

fn lay_out(placard: &Placard, tuning: &Tuning) -> (Vec2, ArrayVec<PlacardLine, MAX_LINES>) {
    let mut lines = ArrayVec::new();
    let inner = tuning.placard_pad.mul_add(-2.0, tuning.placard_width);
    let mut y = -tuning.placard_pad;

    if let Some(kicker) = placard
        .kicker
        .as_deref()
        .filter(|kicker| !kicker.is_empty())
    {
        let mut wrapped = ArrayVec::new();
        wrap(
            kicker,
            budget(inner, tuning.placard_kicker, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= tuning.placard_kicker;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_kicker,
                emphasis: Emphasis::Kicker,
            });
        }
    }

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
                emphasis: if placard.warning {
                    Emphasis::Warning
                } else {
                    Emphasis::Title
                },
            });
        }
    }

    let step = tuning.placard_row * tuning.placard_line;
    for paragraph in &placard.body {
        let mut wrapped = ArrayVec::new();
        wrap(
            &paragraph.text,
            budget(inner, tuning.placard_row, tuning),
            &mut wrapped,
        );
        for text in wrapped {
            y -= step;
            let _ = lines.try_push(PlacardLine {
                text,
                offset: Vec2::new(-inner / 2.0, y),
                size: tuning.placard_row,
                emphasis: match paragraph.tone {
                    Tone::Body => Emphasis::Body,
                    Tone::Muted => Emphasis::Muted,
                    Tone::Warning => Emphasis::Warning,
                },
            });
        }
    }

    (
        Vec2::new(tuning.placard_width, -y + tuning.placard_pad),
        lines,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mote::{
        Arrange,
        Role,
    };

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
            glyph: None,
            tint: None,
            film: crate::palette::FILM,
            frost: 0.0,
            notice: None,
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
        view(placard, Vec3::ZERO, 0.03, &tuning())
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
    fn the_placard_waits_out_the_delay() {
        let tuning = tuning();
        assert!(!earned(0.0, &tuning));
        assert!(!earned(tuning.placard_delay - 0.01, &tuning));
        assert!(earned(tuning.placard_delay, &tuning));
        assert!(earned(10.0, &tuning));
    }

    #[test]
    fn it_mounts_clear_of_the_body_not_out_in_the_room() {
        let tuning = tuning();
        let radius = 0.03;
        let mote = Vec3::new(0.1, 0.2, 0.0);
        let view = view(&Placard::describing(&group()), mote, radius, &tuning);

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

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-5
    }

    #[test]
    fn a_standing_placard_stands_on_the_origin_and_lays_out_like_a_mote_s_but_larger() {
        let tuning = tuning();
        let placard = Placard::describing(&group());
        let standing = standing(&placard, &tuning);
        let on_mote = view_of(&placard);
        let scale = tuning.stand_scale;

        assert_eq!(standing.position, Vec3::new(0.0, standing.size.y, 0.0));
        assert!(near(standing.size.x, on_mote.size.x * scale));
        assert!(near(standing.size.y, on_mote.size.y * scale));
        assert_eq!(standing.lines.len(), on_mote.lines.len());
        for (large, small) in standing.lines.iter().zip(&on_mote.lines) {
            assert_eq!(large.text, small.text, "wrapped the same");
            assert_eq!(large.emphasis, small.emphasis);
            assert!(near(large.size, small.size * scale));
            assert!(near(large.offset.x, small.offset.x * scale));
            assert!(near(large.offset.y, small.offset.y * scale));
        }
        assert!(near(standing.unit, scale), "the pane's shape scales too");
        assert!(near(on_mote.unit, 1.0));
    }

    #[test]
    fn a_standing_title_reads_at_the_size_the_plan_asks_for() {
        let title = standing(&Placard::new("t"), &tuning()).lines[0].size;
        assert!((0.05..0.06).contains(&title), "{title} m");
    }

    #[test]
    fn a_standing_placard_starts_with_no_accent() {
        assert_eq!(standing(&Placard::new("t"), &tuning()).accent, None);
    }

    #[test]
    fn the_kicker_is_a_smaller_muted_line_above_the_title() {
        let view = view_of(&Placard::new("Title").kicker("A3F9").line("body"));
        assert_eq!(texts(&view), vec!["A3F9", "Title", "body"]);
        assert_eq!(view.lines[0].emphasis, Emphasis::Kicker);
        assert!(view.lines[0].size < view.lines[2].size);
        assert!(view.lines[0].size < view.lines[1].size);
    }

    #[test]
    fn a_kicker_earns_panel_height_and_none_costs_nothing() {
        let bare = view_of(&Placard::new("Title"));
        let kicked = view_of(&Placard::new("Title").kicker("A3F9"));
        assert!(kicked.size.y > bare.size.y);
        assert_eq!(bare, view_of(&Placard::new("Title")));
        assert_eq!(
            view_of(&Placard::new("Title").kicker("")).size,
            bare.size,
            "an empty kicker is no line"
        );
    }

    #[test]
    fn each_line_is_set_in_its_own_tone() {
        let view = view_of(
            &Placard::new("Title")
                .line("plain")
                .toned("quiet", Tone::Muted)
                .toned("careful", Tone::Warning),
        );
        let emphasis = view
            .lines
            .iter()
            .map(|line| line.emphasis)
            .collect::<Vec<_>>();
        assert_eq!(
            emphasis,
            vec![
                Emphasis::Title,
                Emphasis::Body,
                Emphasis::Muted,
                Emphasis::Warning
            ]
        );
    }

    #[test]
    fn a_kicker_and_a_full_body_still_fit_the_line_budget() {
        let placard = Placard::from_script(
            "t",
            Some("k"),
            &vec![(String::from("x"), Tone::Body); MAX_PARAGRAPHS],
        )
        .unwrap();
        assert_eq!(view_of(&placard).lines.len(), MAX_LINES);
    }

    #[test]
    fn script_text_at_the_limits_is_taken() {
        let lines = vec![("x".repeat(MAX_PARAGRAPH_BYTES), Tone::Muted); MAX_PARAGRAPHS];
        let placard = Placard::from_script(
            &"t".repeat(MAX_TITLE_BYTES),
            Some(&"k".repeat(MAX_KICKER_BYTES)),
            &lines,
        )
        .unwrap();
        assert_eq!(placard.body.len(), MAX_PARAGRAPHS);
        assert_eq!(placard.body[0].tone, Tone::Muted);
        assert_eq!(
            placard.kicker.map(|kicker| kicker.len()),
            Some(MAX_KICKER_BYTES)
        );
    }

    #[test]
    fn script_text_over_a_limit_is_refused() {
        let one = vec![(String::from("x"), Tone::Body)];
        assert_eq!(
            Placard::from_script(&"t".repeat(MAX_TITLE_BYTES + 1), None, &one),
            Err(Refused::Title(MAX_TITLE_BYTES + 1))
        );
        assert_eq!(
            Placard::from_script("t", Some(&"k".repeat(MAX_KICKER_BYTES + 1)), &one),
            Err(Refused::Kicker(MAX_KICKER_BYTES + 1))
        );
        assert_eq!(
            Placard::from_script(
                "t",
                None,
                &vec![(String::new(), Tone::Body); MAX_PARAGRAPHS + 1]
            ),
            Err(Refused::Paragraphs(MAX_PARAGRAPHS + 1))
        );
        assert_eq!(
            Placard::from_script(
                "t",
                None,
                &[("x".repeat(MAX_PARAGRAPH_BYTES + 1), Tone::Body)]
            ),
            Err(Refused::Paragraph(MAX_PARAGRAPH_BYTES + 1))
        );
    }

    #[test]
    fn a_notice_is_its_message_set_as_a_warning_with_nothing_under_it() {
        let view = view_of(&Placard::notice("No home."));
        assert_eq!(texts(&view), vec!["No home."]);
        assert_eq!(view.lines[0].emphasis, Emphasis::Warning);
        assert_ne!(
            view_of(&Placard::new("Citrus")).lines[0].emphasis,
            Emphasis::Warning
        );
    }

    #[test]
    fn a_mote_explains_itself_without_being_told_to() {
        let view = view_of(&Placard::describing(&group()));
        assert_eq!(view.lines[0].text, "Citrus");
        assert_eq!(view.lines[0].emphasis, Emphasis::Title);
        assert_eq!(texts(&view), vec!["Citrus", "Sharp and bright"]);
    }

    #[test]
    fn it_says_nothing_the_body_already_shows() {
        let view = view_of(&Placard::describing(&group()));
        for noise in ["Kind", "Group", "Holds", "4", "Groups", "1"] {
            assert!(!texts(&view).contains(&noise), "{noise:?} is noise");
        }
    }

    #[test]
    fn a_mote_with_nothing_to_add_is_just_its_name() {
        let view = view_of(&Placard::describing(&MoteSpec {
            description: None,
            ..spec(Role::Action)
        }));
        assert_eq!(texts(&view), vec!["Citrus"]);
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
