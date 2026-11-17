//! The lines under a tag's title.

use super::content::Content;
use crate::{
    tag::Line,
    unavi::vui::api::Tone,
};

/// The lines under a tag's title: the head count, who made it, and a warning
/// when there is no way back. All but the warning are muted.
pub fn tag_lines(text: &Content) -> Vec<Line> {
    let mut lines = Vec::new();
    if let Some(count) = text.occupants {
        lines.push(Line {
            text: match count {
                1 => String::from("1 person"),
                count => format!("{count} people"),
            },
            tone: Tone::Muted,
        });
    }
    if let Some(rung) = &text.rung {
        lines.push(Line {
            text: rung.clone(),
            tone: Tone::Muted,
        });
    }
    if text.one_way {
        lines.push(Line {
            text: String::from("One way"),
            tone: Tone::Warning,
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
        assert_eq!(lines[0].text, "3 people");
        assert_eq!(lines[1].text, "Trusted");
    }

    #[test]
    fn one_person_is_not_people() {
        let lines = tag_lines(&Content {
            kicker:    String::new(),
            title:     String::new(),
            occupants: Some(1),
            rung:      None,
            one_way:   false,
        });
        assert_eq!(lines[0].text, "1 person");
    }
}
