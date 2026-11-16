//! Turns `tokens.css` and `tokens.toml` into a module of `pub const`s.
//!
//! Included by path from each `build.rs` that consumes the tokens. Pure
//! string handling, so the output is plain constants with no clock or I/O.

use std::{
    collections::HashSet,
    fmt,
};

#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
    /// A value that should be `#RRGGBB` is not.
    Colour { name: String, value: String },
    /// A `tokens.toml` line that is not a section, a key or a comment.
    Syntax { line: usize, text: String },
    /// A `$name` that is not a colour in `tokens.css`.
    Reference { name: String },
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Colour { name, value } => write!(f, "{name}: {value:?} is not #RRGGBB"),
            Self::Syntax { line, text } => write!(f, "tokens.toml line {line}: {text:?}"),
            Self::Reference { name } => write!(f, "${name} is not a colour in tokens.css"),
        }
    }
}

impl std::error::Error for TokenError {}

enum Value {
    Text(String),
    Int(u32),
    Float(f64),
    Ints(Vec<u32>),
}

struct Entry {
    section: String,
    key:     String,
    value:   Value,
}

/// The sRGB channel `value` in `0.0..=1.0` as a linear one.
#[must_use]
pub fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// A module body holding one constant per colour and duration in `css` and
/// one per entry of `toml`. Colours come out as linear `Color`s, durations as
/// seconds.
pub fn generate(css: &str, toml: &str) -> Result<String, TokenError> {
    let mut out = String::new();
    let mut colours = HashSet::new();

    for (name, value) in custom_properties(css) {
        if let Some(rgb) = parse_hex(&value) {
            out.push_str(&colour_const(&ident(&[&name]), rgb));
            colours.insert(name);
        } else if let Some(seconds) = parse_millis(&value) {
            out.push_str(&float_const(&ident(&[&name]), seconds));
        }
    }

    for Entry {
        section,
        key,
        value,
    } in entries(toml)?
    {
        let seconds = key.strip_suffix("_ms");
        let name = ident(&[&section, seconds.unwrap_or(&key)]);
        let line = match (value, seconds) {
            (Value::Text(text), _) => text_const(&name, &text, &colours)?,
            (Value::Int(value), Some(_)) => float_const(&name, f64::from(value) / 1000.0),
            (Value::Float(value), Some(_)) => float_const(&name, value / 1000.0),
            (Value::Int(value), None) => format!("pub const {name}: u32 = {value};\n"),
            (Value::Float(value), None) => float_const(&name, value),
            (Value::Ints(values), _) => {
                let list = values.iter().map(u32::to_string).collect::<Vec<_>>();
                format!(
                    "pub const {name}: [u32; {}] = [{}];\n",
                    values.len(),
                    list.join(", ")
                )
            }
        };
        out.push_str(&line);
    }

    Ok(out)
}

fn ident(parts: &[&str]) -> String {
    parts.join("_").replace('-', "_").to_uppercase()
}

/// A float literal with its fractional digits in groups of three, which is
/// how clippy wants a long one written.
fn literal(value: f32) -> String {
    let text = format!("{value:?}");
    let (mantissa, exponent) = match text.split_once('e') {
        Some((mantissa, exponent)) => (mantissa, format!("e{exponent}")),
        None => (text.as_str(), String::new()),
    };
    let Some((whole, fraction)) = mantissa.split_once('.') else {
        return text;
    };
    if fraction.len() <= 4 {
        return text;
    }
    let groups = fraction
        .as_bytes()
        .chunks(3)
        .map(|digits| String::from_utf8_lossy(digits).into_owned())
        .collect::<Vec<_>>();
    format!("{whole}.{}{exponent}", groups.join("_"))
}

fn float_const(name: &str, value: f64) -> String {
    format!("pub const {name}: f32 = {};\n", literal(value as f32))
}

fn colour_const(name: &str, [r, g, b]: [f64; 3]) -> String {
    let [r, g, b] = [r, g, b].map(|channel| literal(srgb_to_linear(channel) as f32));
    format!("pub const {name}: Color = Color::rgb({r}, {g}, {b});\n")
}

fn text_const(
    name: &str,
    text: &str,
    colours: &HashSet<String>,
) -> Result<String, TokenError> {
    if let Some(reference) = text.strip_prefix('$') {
        if !colours.contains(reference) {
            return Err(TokenError::Reference {
                name: reference.to_string(),
            });
        }
        return Ok(format!(
            "pub const {name}: Color = {};\n",
            ident(&[reference])
        ));
    }
    let rgb = parse_hex(text).ok_or_else(|| TokenError::Colour {
        name: name.to_string(),
        value: text.to_string(),
    })?;
    Ok(colour_const(name, rgb))
}

fn parse_hex(text: &str) -> Option<[f64; 3]> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let channel = |at: usize| {
        let pair = digits.get(at..at + 2)?;
        Some(f64::from(u8::from_str_radix(pair, 16).ok()?) / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn parse_millis(text: &str) -> Option<f64> {
    let millis = text.strip_suffix("ms")?.trim().parse::<f64>().ok()?;
    Some(millis / 1000.0)
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some((before, after)) = rest.split_once("/*") {
        out.push_str(before);
        rest = after.split_once("*/").map_or("", |(_, tail)| tail);
    }
    out.push_str(rest);
    out
}

/// Every `--name: value` declaration, keeping the first of a repeated name so
/// a media query's override does not replace the base value.
fn custom_properties(css: &str) -> Vec<(String, String)> {
    let css = strip_comments(css);
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for declaration in css.split(';') {
        let Some((_, rest)) = declaration.split_once("--") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        let valid = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if valid && seen.insert(name.to_string()) {
            found.push((name.to_string(), value.to_string()));
        }
    }
    found
}

fn strip_line_comment(line: &str) -> &str {
    let mut quoted = false;
    for (at, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..at],
            _ => {}
        }
    }
    line
}

fn parse_value(text: &str) -> Option<Value> {
    if let Some(inner) = text.strip_prefix('"') {
        return Some(Value::Text(inner.strip_suffix('"')?.to_string()));
    }
    if let Some(inner) = text.strip_prefix('[') {
        let items = inner.strip_suffix(']')?.split(',');
        let values = items
            .map(|item| item.trim().parse().ok())
            .collect::<Option<Vec<u32>>>()?;
        return Some(Value::Ints(values));
    }
    if text.contains('.') {
        return text.parse().ok().map(Value::Float);
    }
    text.parse().ok().map(Value::Int)
}

fn entries(toml: &str) -> Result<Vec<Entry>, TokenError> {
    let mut section = String::new();
    let mut found = Vec::new();
    for (index, raw) in toml.lines().enumerate() {
        let line = strip_line_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let syntax = || TokenError::Syntax {
            line: index + 1,
            text: raw.to_string(),
        };
        if let Some(name) = line.strip_prefix('[') {
            section = name.strip_suffix(']').ok_or_else(syntax)?.trim().to_string();
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(syntax)?;
        if section.is_empty() {
            return Err(syntax());
        }
        found.push(Entry {
            section: section.clone(),
            key:     key.trim().to_string(),
            value:   parse_value(value.trim()).ok_or_else(syntax)?,
        });
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated(css: &str, toml: &str) -> String {
        match generate(css, toml) {
            Ok(out) => out,
            Err(error) => panic!("{error}"),
        }
    }

    const CSS: &str = "
        :root {
          --bg: #F8F8F5;  /* ground */
          --blue: #2563EB;
          --fg-rgb: 15, 15, 15;
          --text-md: 1rem;
          --dur-press: 100ms;
        }
        @media (prefers-reduced-motion: reduce) {
          :root { --dur-press: 0ms; }
        }
    ";

    #[test]
    fn the_endpoints_and_midpoint_convert_to_linear() {
        assert!(srgb_to_linear(0.0).abs() < 1.0e-12);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1.0e-12);
        assert!((srgb_to_linear(0.5) - 0.214_041).abs() < 1.0e-5);
        assert!((srgb_to_linear(0.02) - 0.02 / 12.92).abs() < 1.0e-12);
    }

    #[test]
    fn long_fractions_are_grouped_in_threes() {
        assert_eq!(literal(0.938_685_7), "0.938_685_7");
        assert_eq!(literal(0.06), "0.06");
        assert_eq!(literal(20.0), "20.0");
    }

    #[test]
    fn a_hex_colour_comes_out_linear() {
        let out = generated("--panel: #FFFFFF; --mid: #808080;", "");
        assert!(out.contains("pub const PANEL: Color = Color::rgb(1.0, 1.0, 1.0);"));
        let mid = literal(srgb_to_linear(128.0 / 255.0) as f32);
        assert!(out.contains(&format!("Color::rgb({mid}")), "{out}");
    }

    #[test]
    fn only_colours_and_durations_are_taken_from_the_css() {
        let out = generated(CSS, "");
        assert!(out.contains("pub const BG: Color"));
        assert!(out.contains("pub const BLUE: Color"));
        assert!(!out.contains("TEXT_MD"));
        assert!(!out.contains("FG_RGB"));
    }

    #[test]
    fn a_media_query_does_not_override_a_duration() {
        let out = generated(CSS, "");
        assert!(out.contains("pub const DUR_PRESS: f32 = 0.1;"), "{out}");
    }

    #[test]
    fn the_toml_names_follow_their_section_and_unit() {
        let toml = "
            [mrad]
            body = 20.0   # em height
            [cells]
            wake_radius = 7
            front_ms = 200
            [mono]
            columns = [24, 40]
            [field]
            cover = \"#e4d2df\"
        ";
        let out = generated("", toml);
        assert!(out.contains("pub const MRAD_BODY: f32 = 20.0;"));
        assert!(out.contains("pub const CELLS_WAKE_RADIUS: u32 = 7;"));
        assert!(out.contains("pub const CELLS_FRONT: f32 = 0.2;"));
        assert!(out.contains("pub const MONO_COLUMNS: [u32; 2] = [24, 40];"));
        assert!(out.contains("pub const FIELD_COVER: Color = Color::rgb("));
    }

    #[test]
    fn a_dollar_names_a_css_colour() {
        let out = generated("--fg: #0F0F0F;", "[cells]\ncolour = \"$fg\"");
        assert!(out.contains("pub const CELLS_COLOUR: Color = FG;"));
        assert_eq!(
            generate("", "[cells]\ncolour = \"$fg\""),
            Err(TokenError::Reference {
                name: "fg".to_string()
            })
        );
    }

    #[test]
    fn malformed_toml_is_refused() {
        assert!(matches!(
            generate("", "orphan = 1"),
            Err(TokenError::Syntax { line: 1, .. })
        ));
        assert!(matches!(
            generate("", "[a]\nb = \"nope\""),
            Err(TokenError::Colour { .. })
        ));
    }
}
