//! The tag over the attended document.

use unavi_lens::DocumentId;
use wired_guest::{
    color::short_id,
    math::Vec3,
};

use crate::tag::Tag;

/// Columns the tag is laid out in.
const COLUMNS: u32 = 24;

/// Characters of a DID kept at its end when it is shortened.
const DID_TAIL: usize = 6;

/// DIDs this short or shorter are shown whole.
const DID_WHOLE: usize = 12;

/// What the tag says.
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

/// The document's name, else its short id; and who authors it.
#[must_use]
pub fn describe(facts: &Facts) -> Text {
    let author = facts
        .author_name
        .map(str::to_owned)
        .or_else(|| facts.author.map(short_did))
        .unwrap_or_else(|| "no author".to_owned());
    Text {
        title:  facts.name.map_or_else(|| short_id(facts.id), str::to_owned),
        kicker: author,
    }
}

pub struct Label {
    tag:     Tag,
    /// The document it is over.
    pub key: Option<DocumentId>,
    text:    Option<Text>,
    at:      Option<Vec3>,
    shown:   bool,
}

impl Label {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            tag:   Tag::new(COLUMNS)?,
            key:   None,
            text:  None,
            at:    None,
            shown: false,
        })
    }

    /// Moves the tag to another document, forgetting what it said.
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
        self.at = None;
        if self.shown {
            self.tag.set_shown(false);
            self.shown = false;
        }
    }

    /// Sets what it says, changing only what differs.
    pub fn say(&mut self, text: Text) {
        if self.text.as_ref() == Some(&text) {
            return;
        }
        match self.tag.set_text(&text.title, Some(&text.kicker), &[]) {
            Ok(()) => self.text = Some(text),
            Err(err) => eprintln!("visor: document tag text: {err:?}"),
        }
    }

    /// Stands it with its bottom centre at `at`, and shows it once it has text.
    pub fn stand(&mut self, at: Vec3) {
        if self.at != Some(at) {
            self.tag.stand(at, None);
            self.at = Some(at);
        }
        if self.text.is_some() && !self.shown {
            self.tag.set_shown(true);
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
        }
    }

    #[test]
    fn an_unnamed_document_is_titled_by_its_short_id() {
        assert_eq!(describe(&facts(None, None)).title, "ab01ff");
        assert_eq!(describe(&facts(Some("Lamp"), None)).title, "Lamp");
    }

    #[test]
    fn the_kicker_names_the_author() {
        let text = describe(&facts(None, Some("did:key:z6Mkabcdefgh123456")));
        assert_eq!(text.kicker, "…123456");
        assert_eq!(describe(&facts(None, None)).kicker, "no author");
    }

    #[test]
    fn an_authors_own_name_stands_in_for_their_did() {
        let mut named = facts(None, Some("did:key:z6Mkabcdefgh123456"));
        named.author_name = Some("Mira");
        assert_eq!(describe(&named).kicker, "Mira");
    }

    #[test]
    fn a_short_did_is_kept_whole() {
        assert_eq!(short_did("did:web:a.b"), "did:web:a.b");
    }
}
