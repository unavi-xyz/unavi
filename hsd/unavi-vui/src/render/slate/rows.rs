//! Drawing a slate's rows: each row's label and value, a vector row's three
//! numbers, the dots before a label and after a value, the rule over a
//! section, and the tabs row's labels and the mark under the selected one.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use super::{
    Extras,
    LIFT,
    Slate,
};
use crate::{
    palette::Palette,
    render::draw,
    slate::{
        self,
        DOT,
        HEADER_SIZE,
        Kind,
        PADDING,
        ROW_HEIGHT,
        Row,
        TEXT_SIZE,
        WIDTH,
        format_number,
    },
    wired::scene::properties::{
        Property,
        Text,
        TextAlign,
        TextAnchor,
    },
};

/// Metres thick the rule over a section is, and how strongly it is inked.
const RULE: f32 = 0.0012;
const RULE_INK: f32 = 0.35;

impl Slate {
    /// Writes each row's label and value into `batch`, and hides the rest.
    /// Type is drawn at `shown` opacity, which follows the pane in and out.
    pub(super) fn draw_rows<'a>(
        &self,
        mut batch: crate::Batch<'a>,
        palette: &Palette,
        shown: f32,
    ) -> crate::Batch<'a> {
        let mut tabs_drawn = false;
        for (index, ((label, value), extras)) in self
            .labels
            .iter()
            .zip(&self.values)
            .zip(&self.extras)
            .enumerate()
        {
            batch = hide_extras(batch, extras);
            let Some(row) = self.rows.get(index) else {
                batch = batch
                    .set(*label, Property::Transform(draw::hidden()))
                    .set(*value, Property::Transform(draw::hidden()));
                continue;
            };
            let y = slate::row_centre(index);
            if let Kind::Tabs { labels, selected } = &row.kind {
                batch = batch
                    .set(*label, Property::Transform(draw::hidden()))
                    .set(*value, Property::Transform(draw::hidden()));
                batch = self.draw_tabs(batch, palette, shown, y, labels, *selected);
                tabs_drawn = true;
                continue;
            }

            let (label_text, value_text) = row.fitted();
            let (size, ink) = match row.kind {
                Kind::Header => (HEADER_SIZE, palette.muted_ink),
                Kind::Section { open: false, .. } => (TEXT_SIZE, palette.muted_ink),
                _ => (TEXT_SIZE, palette.ink),
            };
            batch = place_text(
                batch,
                *label,
                Vec3::new(row.label_left(), y, LIFT),
                text(
                    &label_text,
                    size,
                    TextAlign::Left,
                    draw::with_alpha(ink, shown),
                ),
            );
            let muted = draw::with_alpha(palette.muted_ink, shown);
            match &row.kind {
                Kind::Vector { values, .. } => {
                    let [first, second] = extras.parts;
                    for (part, prim) in [first, second, *value].into_iter().enumerate() {
                        batch = place_text(
                            batch,
                            prim,
                            Vec3::new(row.part_right(part), y, LIFT),
                            text(
                                &format_number(values[part]),
                                TEXT_SIZE,
                                TextAlign::Right,
                                muted,
                            ),
                        );
                    }
                }
                _ => {
                    batch = place_text(
                        batch,
                        *value,
                        Vec3::new(row.value_right(), y, LIFT),
                        text(&value_text, TEXT_SIZE, TextAlign::Right, muted),
                    );
                }
            }
            batch = draw_extras(batch, palette, row, extras, y, shown);
        }
        if !tabs_drawn {
            batch = batch.set(self.tab_mark, Property::Transform(draw::hidden()));
            for prim in &self.tab_labels {
                batch = batch.set(*prim, Property::Transform(draw::hidden()));
            }
        }
        batch
    }

    /// The tabs row at height `y`: a label per tab, and the accent bar under
    /// the selected one, the way a toggle that is on is lit.
    fn draw_tabs<'a>(
        &self,
        mut batch: crate::Batch<'a>,
        palette: &Palette,
        progress: f32,
        y: f32,
        labels: &[smol_str::SmolStr],
        selected: u32,
    ) -> crate::Batch<'a> {
        let count = labels.len();
        batch = batch.set(self.tab_mark, Property::Transform(draw::hidden()));
        for (index, prim) in self.tab_labels.iter().enumerate() {
            let Some(tab) = labels.get(index) else {
                batch = batch.set(*prim, Property::Transform(draw::hidden()));
                continue;
            };
            let on = index == selected as usize;
            let ink = if on { palette.ink } else { palette.muted_ink };
            let x = slate::tab_centre(index, count);
            batch = place_text(
                batch,
                *prim,
                Vec3::new(x, y, LIFT),
                text(
                    &slate::tab_label(tab, count),
                    TEXT_SIZE,
                    TextAlign::Center,
                    draw::with_alpha(ink, progress),
                ),
            );
            if on {
                batch = batch
                    .set(
                        self.tab_mark,
                        Property::Transform(Transform {
                            translation: Vec3::new(x, y, LIFT / 2.0),
                            rotation:    Quat::IDENTITY,
                            scale:       Vec3::new(
                                slate::tab_width(count) - PADDING / 2.0,
                                ROW_HEIGHT * 0.9,
                                LIFT / 2.0,
                            ),
                        }),
                    )
                    .set(
                        self.tab_mark,
                        Property::Material(draw::ink(
                            self.accent.unwrap_or(palette.accent),
                            0.3 * progress,
                        )),
                    );
            }
        }
        batch
    }
}

/// The dot before the label, the mark after the value and the rule over a
/// section, for `row` at height `y`.
fn draw_extras<'a>(
    mut batch: crate::Batch<'a>,
    palette: &Palette,
    row: &Row,
    extras: &Extras,
    y: f32,
    shown: f32,
) -> crate::Batch<'a> {
    let dots = [
        (extras.dot, row.dot, -WIDTH / 2.0 + PADDING + DOT / 2.0),
        (extras.mark, row.mark, WIDTH / 2.0 - PADDING - DOT / 2.0),
    ];
    for (prim, color, x) in dots {
        if let Some(color) = color {
            batch = batch
                .set(
                    prim,
                    Property::Transform(Transform {
                        translation: Vec3::new(x, y, LIFT),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::new(DOT / 2.0, DOT / 2.0, LIFT / 2.0),
                    }),
                )
                .set(prim, Property::Material(draw::ink(color, shown)));
        }
    }
    if matches!(row.kind, Kind::Section { .. }) {
        batch = batch
            .set(
                extras.rule,
                Property::Transform(Transform {
                    translation: Vec3::new(0.0, y + ROW_HEIGHT / 2.0, LIFT / 2.0),
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::new(2.0f32.mul_add(-PADDING, WIDTH), RULE, LIFT / 2.0),
                }),
            )
            .set(
                extras.rule,
                Property::Material(draw::ink(palette.muted_ink, RULE_INK * shown)),
            );
    }
    batch
}

fn hide_extras<'a>(mut batch: crate::Batch<'a>, extras: &Extras) -> crate::Batch<'a> {
    for prim in extras
        .parts
        .iter()
        .chain([&extras.dot, &extras.mark, &extras.rule])
    {
        batch = batch.set(*prim, Property::Transform(draw::hidden()));
    }
    batch
}

fn place_text(batch: crate::Batch<'_>, prim: (u64, u64), at: Vec3, text: Text) -> crate::Batch<'_> {
    batch
        .set(prim, Property::Transform(Transform::from_translation(at)))
        .set(prim, Property::Text(text))
}

fn text(value: &str, size: f32, align: TextAlign, color: Color) -> Text {
    Text {
        value:         value.to_owned(),
        size:          Some(size),
        align:         Some(align),
        anchor:        Some(TextAnchor::Middle),
        wrap:          None,
        line_height:   None,
        color:         Some(color),
        // Dark type on a light pane needs no outline, and ink does not glow.
        outline:       None,
        outline_width: None,
        emissive:      Some(0.0),
        billboard:     None,
    }
}
