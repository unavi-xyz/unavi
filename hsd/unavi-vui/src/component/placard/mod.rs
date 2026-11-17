//! The pane beside a mote that says what it does, or what went wrong with
//! it.

use std::rc::Rc;

use smol_str::SmolStr;
use wired_guest::math::{
    Vec2,
    Vec3,
};

use crate::{
    component::pane::{
        Pane,
        block::{
            Block,
            Height,
            Kind,
            Span,
            Tone,
        },
        draw::Runs,
    },
    part::spec::MoteSpec,
    tokens::tuning::Tuning,
};

/// Columns a placard is laid out in.
const COLUMNS: u32 = 40;
/// The one block of a placard.
const BLOCK: u32 = 0;

/// What a placard says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// A message about a mote, set as a warning.
    Notice(SmolStr),
    /// A mote's name, over what it does when it has said.
    Describing {
        label:       SmolStr,
        description: Option<SmolStr>,
    },
    /// What a bed's page says while nothing is attended or kept: a title over
    /// paragraphs.
    Resting { title: SmolStr, lines: Vec<SmolStr> },
}

impl Content {
    /// The standard placard for a mote: its name and its description.
    #[must_use]
    pub fn describing(spec: &MoteSpec) -> Self {
        Self::Describing {
            label:       spec.label.clone(),
            description: spec.description.clone(),
        }
    }

    fn spans(&self) -> (Option<&str>, Vec<Span>) {
        let span = |text: &SmolStr, tone| Span {
            text: text.clone(),
            tone,
            link: None,
        };
        match self {
            Self::Notice(message) => (None, vec![span(message, Tone::Warning)]),
            Self::Describing {
                label,
                description: Some(description),
            } => (Some(label.as_str()), vec![span(description, Tone::Body)]),
            Self::Describing {
                label,
                description: None,
            } => (None, vec![span(label, Tone::Body)]),
            Self::Resting { title, lines } => (
                Some(title.as_str()),
                lines.iter().map(|line| span(line, Tone::Body)).collect(),
            ),
        }
    }
}

/// A laid-out placard: what its pane draws, and how big it is in ems.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub runs: Runs,
    pub size: Vec2,
}

impl Sheet {
    /// `None` when the pane refuses the text, which a mote's limits keep from
    /// happening.
    fn of(content: &Content) -> Option<Self> {
        let (title, spans) = content.spans();
        let built = Pane::new(COLUMNS, None).and_then(|mut pane| {
            pane.set_title(title.unwrap_or_default())?;
            pane.set_blocks(vec![Block {
                id:     BLOCK,
                height: Height::Share,
                kind:   Kind::Text(spans),
            }])?;
            Ok(pane)
        });
        match built {
            Ok(pane) => Some(Self {
                runs: pane.runs(),
                size: pane.size(),
            }),
            Err(why) => {
                eprintln!("vui: placard text: {why}");
                None
            }
        }
    }
}

/// The sheet last laid out, so a placard that says the same thing is not laid
/// out again every frame.
#[derive(Debug, Default)]
pub struct Cache {
    last: Option<(Content, Option<Rc<Sheet>>)>,
}

impl Cache {
    /// The sheet for `content`, laid out if it is not the last one asked for.
    pub fn sheet(&mut self, content: Content) -> Option<Rc<Sheet>> {
        if let Some((last, sheet)) = &self.last
            && *last == content
        {
            return sheet.clone();
        }
        let sheet = Sheet::of(&content).map(Rc::new);
        self.last = Some((content, sheet.clone()));
        sheet
    }
}

/// A placard on its mote's body.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacardView {
    /// Orbit-local, at the pane's top centre.
    pub position: Vec3,
    pub sheet:    Rc<Sheet>,
}

/// Whether attention has been held long enough to earn a placard. How it
/// arrives is the pane's: it grows in once this is true.
#[must_use]
pub fn earned(dwell: f32, tuning: &Tuning) -> bool {
    dwell >= tuning.placard_delay
}

/// Mounts `sheet` clear of the body at `mote`.
///
/// The pane hangs down from its top edge, so the mount is a whole height above
/// the mote rather than resting on it. Above rather than beside, because the
/// mote's own name is already below it and a placard that picked a side would
/// leave the surface at the edge slots.
#[must_use]
pub fn view(sheet: Rc<Sheet>, mote: Vec3, radius: f32, tuning: &Tuning) -> PlacardView {
    let height = sheet.size.y * tuning.placard_em;
    PlacardView {
        position: Vec3::new(
            mote.x,
            mote.y + radius + tuning.placard_gap + height,
            mote.z + tuning.placard_lift,
        ),
        sheet,
    }
}

/// Mounts `sheet` as the page of a bed whose slab reaches `top` above the
/// surface's centre.
///
/// The pane hangs down from its mount, so the mount is the pane's own height
/// above the clearance: a page grows upward from the bed and never moves a
/// key. It stands on the middle, where the bed itself is symmetric.
#[must_use]
pub fn page(sheet: Rc<Sheet>, top: f32, tuning: &Tuning) -> PlacardView {
    let height = sheet.size.y * tuning.placard_em;
    PlacardView {
        position: Vec3::new(0.0, top + tuning.placard_gap + height, tuning.placard_lift),
        sheet,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        part::spec::{
            Arrange,
            Role,
        },
        tokens::state::Condition,
    };

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    fn spec(description: Option<&'static str>) -> MoteSpec {
        MoteSpec {
            role:        Role::Group {
                children: 4,
                groups:   1,
                arrange:  Arrange::Orbit,
            },
            label:       SmolStr::new_static("Citrus"),
            description: description.map(SmolStr::new_static),
            condition:   Condition::default(),
            icon:        false,
            glyph:       None,
            tint:        None,
            film:        crate::tokens::palette::FILM,
            frost:       0.0,
            notice:      None,
        }
    }

    fn sheet(content: &Content) -> Rc<Sheet> {
        Cache::default().sheet(content.clone()).expect("a sheet")
    }

    fn texts(sheet: &Sheet) -> Vec<&str> {
        sheet
            .runs
            .text
            .iter()
            .map(|run| run.text.as_str())
            .collect()
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
        let sheet = sheet(&Content::describing(&spec(Some("Sharp and bright"))));
        let height = sheet.size.y * tuning.placard_em;
        let view = view(sheet, mote, radius, &tuning);

        assert!(
            (view.position.x - mote.x).abs() < 1.0e-5,
            "centred on the mote"
        );
        assert!(
            view.position.y - height >= mote.y + radius,
            "the pane hangs down from its mount, so its whole height has to \
             clear the body or it is drawn across it"
        );
        assert!(
            view.position.z < tuning.mote_radius,
            "a placard further out than the mote is wide stops reading as its"
        );
    }

    #[test]
    fn a_page_grows_upward_from_the_bed_whatever_its_height() {
        let tuning = tuning();
        let top = 0.165;
        let short = page(
            sheet(&Content::describing(&spec(Some("one")))),
            top,
            &tuning,
        );
        let tall = page(
            sheet(&Content::describing(&spec(Some(
                "a description long enough that it certainly has to break across \
                 more than a single line of this pane",
            )))),
            top,
            &tuning,
        );
        for view in [&short, &tall] {
            let height = view.sheet.size.y * tuning.placard_em;
            assert!(
                (view.position.y - height - top - tuning.placard_gap).abs() < 1.0e-5,
                "the lower edge stays a gap above the bed"
            );
            assert!(view.position.x.abs() < 1.0e-6, "centred on the bed");
        }
        assert!(
            tall.position.y > short.position.y,
            "the taller page reaches higher"
        );
    }

    #[test]
    fn a_resting_page_is_its_title_over_its_lines() {
        let sheet = sheet(&Content::Resting {
            title: SmolStr::new_static("Plaza"),
            lines: vec![
                SmolStr::new_static("14:07"),
                SmolStr::new_static("3 people"),
            ],
        });
        assert_eq!(texts(&sheet), vec!["Plaza", "14:07", "3 people"]);
    }

    #[test]
    fn a_mote_explains_itself_without_being_told_to() {
        let sheet = sheet(&Content::describing(&spec(Some("Sharp and bright"))));
        assert_eq!(texts(&sheet), vec!["Citrus", "Sharp and bright"]);
    }

    #[test]
    fn a_mote_with_nothing_to_add_is_just_its_name() {
        let sheet = sheet(&Content::describing(&spec(None)));
        assert_eq!(texts(&sheet), vec!["Citrus"]);
    }

    #[test]
    fn it_says_nothing_the_body_already_shows() {
        let sheet = sheet(&Content::describing(&spec(Some("Sharp and bright"))));
        for noise in ["Kind", "Group", "Holds", "4", "Groups", "1"] {
            assert!(!texts(&sheet).contains(&noise), "{noise:?} is noise");
        }
    }

    #[test]
    fn a_notice_is_its_message_set_as_a_warning() {
        let notice = sheet(&Content::Notice(SmolStr::new_static("No home.")));
        assert_eq!(texts(&notice), vec!["No home."]);
        assert_eq!(notice.runs.text[0].tone, Tone::Warning);
        let described = sheet(&Content::describing(&spec(None)));
        assert_ne!(described.runs.text[0].tone, Tone::Warning);
    }

    #[test]
    fn a_long_description_wraps_and_earns_height() {
        let short = sheet(&Content::describing(&spec(Some("one"))));
        let long = sheet(&Content::describing(&spec(Some(
            "a description long enough that it certainly has to break across \
             more than a single line of this pane",
        ))));
        assert!(long.runs.text.len() > short.runs.text.len());
        assert!(long.size.y > short.size.y);
    }

    #[test]
    fn the_same_content_is_laid_out_once() {
        let mut cache = Cache::default();
        let content = Content::describing(&spec(Some("Sharp")));
        let first = cache.sheet(content.clone()).expect("a sheet");
        let again = cache.sheet(content).expect("a sheet");
        assert!(Rc::ptr_eq(&first, &again));
    }
}
