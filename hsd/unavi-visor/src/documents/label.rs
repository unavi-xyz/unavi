//! The placard over the attended document.

use unavi_lens::DocumentId;
use wired_guest::{
    color::short_id,
    math::{
        Color,
        Vec3,
    },
};

use super::find::Kind;
use crate::unavi::vui::api::Placard;

/// A placard's size as a multiple of the size it is laid out at.
const SCALE: f32 = 0.5;

/// Characters of a DID kept at its end when it is shortened.
const DID_TAIL: usize = 6;

/// DIDs this short or shorter are shown whole.
const DID_WHOLE: usize = 12;

/// What the placard says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub title:  String,
    pub kicker: String,
}

/// What is known of a document, to be said.
pub struct Facts<'a> {
    pub id:          DocumentId,
    /// The name the document gives itself.
    pub name:        Option<&'a str>,
    /// The name its author gives themselves.
    pub author_name: Option<&'a str>,
    pub author:      Option<&'a str>,
    pub kind:        Kind,
}

#[must_use]
pub fn short_did(did: &str) -> String {
    let count = did.chars().count();
    if count <= DID_WHOLE {
        return did.to_owned();
    }
    let tail: String = did.chars().skip(count - DID_TAIL).collect();
    format!("…{tail}")
}

/// The document's name, else its short id; and who authors it and what it is
/// to the space.
#[must_use]
pub fn describe(facts: &Facts) -> Text {
    let author = facts
        .author_name
        .map(str::to_owned)
        .or_else(|| facts.author.map(short_did))
        .unwrap_or_else(|| "no author".to_owned());
    let kind = match facts.kind {
        Kind::Space => "this space",
        Kind::Referenced => "referenced",
    };
    Text {
        title:  facts.name.map_or_else(|| short_id(facts.id), str::to_owned),
        kicker: format!("{author} · {kind}"),
    }
}

pub struct Label {
    placard: Placard,
    /// The document it is over.
    pub key: Option<DocumentId>,
    text:    Option<Text>,
    accent:  Option<Color>,
    at:      Option<Vec3>,
    shown:   bool,
}

impl Label {
    pub fn new() -> anyhow::Result<Self> {
        let placard = Placard::new()?;
        placard.set_scale(SCALE);
        Ok(Self {
            placard,
            key: None,
            text: None,
            accent: None,
            at: None,
            shown: false,
        })
    }

    /// Moves the placard to another document, forgetting what it said.
    pub fn claim(&mut self, key: DocumentId) {
        if self.key == Some(key) {
            return;
        }
        self.hide();
        self.key = Some(key);
    }

    pub fn hide(&mut self) {
        self.key = None;
        self.text = None;
        self.accent = None;
        self.at = None;
        if self.shown {
            self.placard.set_shown(false);
            self.shown = false;
        }
    }

    /// Sets what it says and the colour of its tab, changing only what differs.
    pub fn say(&mut self, text: Text, accent: Color) {
        if self.accent != Some(accent) {
            self.placard.set_accent(Some(accent));
            self.accent = Some(accent);
        }
        if self.text.as_ref() == Some(&text) {
            return;
        }
        match self.placard.set_text(&text.title, Some(&text.kicker), &[]) {
            Ok(()) => self.text = Some(text),
            Err(err) => eprintln!("visor: document placard text: {err:?}"),
        }
    }

    /// Stands it with its bottom centre at `at`, and shows it once it has text.
    pub fn stand(&mut self, at: Vec3) {
        if self.at != Some(at) {
            self.placard.set_position(at);
            self.at = Some(at);
        }
        if self.text.is_some() && !self.shown {
            self.placard.set_shown(true);
            self.shown = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: DocumentId = (
        u64::from_le_bytes([0xAB, 0x01, 0xFF, 0, 0, 0, 0, 0]),
        0,
        0,
        0,
    );

    fn facts<'a>(name: Option<&'a str>, author: Option<&'a str>) -> Facts<'a> {
        Facts {
            id: ID,
            name,
            author_name: None,
            author,
            kind: Kind::Referenced,
        }
    }

    #[test]
    fn an_unnamed_document_is_titled_by_its_short_id() {
        assert_eq!(describe(&facts(None, None)).title, "ab01ff");
        assert_eq!(describe(&facts(Some("Lamp"), None)).title, "Lamp");
    }

    #[test]
    fn the_kicker_names_author_and_kind() {
        let text = describe(&facts(None, Some("did:key:z6Mkabcdefgh123456")));
        assert_eq!(text.kicker, "…123456 · referenced");
        assert_eq!(
            describe(&facts(None, None)).kicker,
            "no author · referenced"
        );
        let mut space = facts(None, None);
        space.kind = Kind::Space;
        assert!(describe(&space).kicker.ends_with("this space"));
    }

    #[test]
    fn an_authors_own_name_stands_in_for_their_did() {
        let mut named = facts(None, Some("did:key:z6Mkabcdefgh123456"));
        named.author_name = Some("Mira");
        assert_eq!(describe(&named).kicker, "Mira · referenced");
    }

    #[test]
    fn a_short_did_is_kept_whole() {
        assert_eq!(short_did("did:web:a.b"), "did:web:a.b");
    }
}
