//! A tag: a standing pane that names a thing, with a muted line under the
//! title and a few lines beneath it.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use crate::{
    unavi::vui::api::{
        Block,
        BlockHeight,
        BlockKind,
        Pane,
        Span,
        Tone,
    },
    wired::core::error::Error,
};

/// The tag's one block.
const TEXT: u32 = 0;

/// One line under a tag's title.
pub struct Line {
    pub text: String,
    pub tone: Tone,
}

pub struct Tag {
    pane: Pane,
}

impl Tag {
    /// Makes a hidden tag `columns` wide, which grows to fit its text.
    pub fn new(columns: u32) -> anyhow::Result<Self> {
        let pane = Pane::new(columns, None)?;
        pane.set_standing(Vec3::ZERO);
        Ok(Self { pane })
    }

    /// Sets what the tag says. A tag with nothing under its title says the
    /// title as its one line.
    pub fn set_text(&self, title: &str, kicker: Option<&str>, lines: &[Line]) -> Result<(), Error> {
        let kicker = kicker
            .filter(|kicker| !kicker.is_empty())
            .map(|kicker| Line {
                text: kicker.to_owned(),
                tone: Tone::Muted,
            });
        let mut spans: Vec<_> = kicker
            .iter()
            .chain(lines)
            .map(|line| Span {
                text: line.text.clone(),
                tone: line.tone,
                link: None,
            })
            .collect();
        let title = if spans.is_empty() {
            spans.push(Span {
                text: title.to_owned(),
                tone: Tone::Body,
                link: None,
            });
            ""
        } else {
            title
        };
        self.pane.set_title(title)?;
        self.pane.set_blocks(&[Block {
            id:     TEXT,
            height: BlockHeight::Share,
            kind:   BlockKind::Text(spans),
        }])
    }

    /// The tag's width and height in metres, as it stands.
    pub fn size(&self) -> Vec2 {
        self.pane.size()
    }

    /// Stands it with its bottom centre on `at`, tied to `tether` by a line
    /// when there is one.
    pub fn stand(&self, at: Vec3, tether: Option<Vec3>) {
        self.pane.set_standing(at);
        self.pane.set_tether(tether);
    }

    /// Colours the tether.
    pub fn set_hue(&self, hue: Option<Color>) {
        self.pane.set_hue(hue);
    }

    pub fn set_shown(&self, shown: bool) {
        self.pane.set_shown(shown);
    }
}
