//! The lines under a tag's title.

use wired_guest::glyph::{
    PERSON,
    pack,
};

use super::content::Content;
use crate::unavi::vui::api::{
    Glyph,
    PlacardLine,
    Tone,
};

/// The lines under a tag's title: the head count after a person, who made it,
/// and a warning when there is no way back. All but the warning are muted.
pub fn tag_lines(text: &Content) -> Vec<PlacardLine> {
    let mut lines = Vec::new();
    if let Some(count) = text.occupants {
        let (size, bits) = pack(&PERSON);
        lines.push(PlacardLine {
            text: count.to_string(),
            tone: Tone::Muted,
            icon: Some(Glyph { size, bits }),
        });
    }
    if let Some(rung) = &text.rung {
        lines.push(PlacardLine {
            text: rung.clone(),
            tone: Tone::Muted,
            icon: None,
        });
    }
    if text.one_way {
        lines.push(PlacardLine {
            text: String::from("One way"),
            tone: Tone::Warning,
            icon: None,
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_and_the_rung_are_separate_lines() {
        let lines = tag_lines(&Content {
            kicker:    String::new(),
            title:     String::new(),
            occupants: Some(3),
            rung:      Some(String::from("Trusted")),
            one_way:   false,
        });
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "3");
        assert!(lines[0].icon.is_some());
        assert_eq!(lines[1].text, "Trusted");
        assert!(lines[1].icon.is_none());
    }
}
