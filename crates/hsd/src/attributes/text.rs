use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::{
    Attribute,
    material::ColorVec,
};

/// A string drawn in the world.
///
/// A property rather than a slot: a label costs a handful of bytes to sync
/// instead of a mesh upload.
///
/// The string fields follow [`super::material::MaterialAttr`]: a value a
/// newer client understands and this one does not still stores, syncs and
/// re-serves rather than failing to decode.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextAttr {
    pub value:         String,
    /// Em height in metres, like every other length in the format. 0.02 is
    /// body text read at arm's length.
    pub size:          Option<f64>,
    /// `left` | `center` | `right`.
    pub align:         Option<String>,
    /// `baseline` | `top` | `middle` | `bottom`.
    pub anchor:        Option<String>,
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
    /// `none` | `yaw` | `full`.
    pub billboard:     Option<String>,
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
            align:         Some("center".to_string()),
            anchor:        Some("middle".to_string()),
            wrap:          Some(0.4),
            line_height:   Some(1.2),
            color:         Some(ColorVec(vec![1.0, 1.0, 1.0, 1.0])),
            outline:       Some(ColorVec(vec![0.0, 0.0, 0.0, 1.0])),
            outline_width: Some(0.25),
            emissive:      Some(0.5),
            billboard:     Some("yaw".to_string()),
        };
        let decoded = TextAttr::decode(&attr.encode().expect("encode")).expect("decode");
        assert_eq!(decoded, attr);
    }

    #[test]
    fn an_alignment_this_build_does_not_know_still_decodes() {
        let attr = TextAttr {
            value: "hello".to_string(),
            align: Some("justify".to_string()),
            ..Default::default()
        };
        let decoded = TextAttr::decode(&attr.encode().expect("encode")).expect("decode");
        assert_eq!(decoded.align.as_deref(), Some("justify"));
    }
}
