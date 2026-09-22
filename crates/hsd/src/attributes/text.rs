use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::{
    Attribute,
    material::ColorVec,
};

/// Horizontal placement of a label's lines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Which point of the text box sits at the prim's origin.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnchor {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

/// How a label turns to face the viewer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextBillboard {
    #[default]
    None,
    Yaw,
    Full,
}

/// A text string drawn in the world.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextAttr {
    pub value:         String,
    /// Em height in metres, like every other length in the format. 0.02 is
    /// body text read at arm's length.
    pub size:          Option<f64>,
    pub align:         Option<TextAlign>,
    pub anchor:        Option<TextAnchor>,
    /// Wrap width in metres. Absent breaks only on newlines.
    pub wrap:          Option<f64>,
    /// Multiple of the font's own baseline-to-baseline distance.
    pub line_height:   Option<f64>,
    pub color:         Option<ColorVec>,
    pub outline:       Option<ColorVec>,
    /// Fraction of the font's baked distance range the outline reaches out
    /// to. Past roughly 0.4 the field runs out of gradient.
    pub outline_width: Option<f64>,
    pub emissive:      Option<f64>,
    pub billboard:     Option<TextBillboard>,
}

impl Attribute for TextAttr {
    const KEY: &'static str = "text";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_string_carries_no_settings() {
        let attr = TextAttr {
            value: "hello".to_string(),
            ..Default::default()
        };
        let decoded = TextAttr::decode(&attr.encode().expect("encode")).expect("decode");
        assert_eq!(decoded, attr);
    }

    #[test]
    fn a_fully_specified_label_survives_a_round_trip() {
        let attr = TextAttr {
            value:         "hello".to_string(),
            size:          Some(0.02),
            align:         Some(TextAlign::Center),
            anchor:        Some(TextAnchor::Middle),
            wrap:          Some(0.4),
            line_height:   Some(1.2),
            color:         Some(ColorVec(vec![1.0, 1.0, 1.0, 1.0])),
            outline:       Some(ColorVec(vec![0.0, 0.0, 0.0, 1.0])),
            outline_width: Some(0.25),
            emissive:      Some(0.5),
            billboard:     Some(TextBillboard::Yaw),
        };
        let decoded = TextAttr::decode(&attr.encode().expect("encode")).expect("decode");
        assert_eq!(decoded, attr);
    }
}
