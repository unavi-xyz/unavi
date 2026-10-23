//! Drawing a slate's rows: each row's label and value, and the tabs row's
//! labels and the mark under the selected one.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use super::{
    LIFT,
    Slate,
};
use crate::{
    palette::Palette,
    render::draw,
    slate::{
        self,
        HEADER_SIZE,
        Kind,
        PADDING,
        ROW_HEIGHT,
        TEXT_SIZE,
        WIDTH,
    },
    wired::scene::properties::{
        Property,
        Text,
        TextAlign,
        TextAnchor,
    },
};

impl Slate {
    /// Writes each row's label and value into `batch`, and hides the rest.
    pub(super) fn draw_rows<'a>(
        &self,
        mut batch: crate::Batch<'a>,
        palette: &Palette,
        progress: f32,
    ) -> crate::Batch<'a> {
        let mut tabs_drawn = false;
        for (index, (label, value)) in self.labels.iter().zip(&self.values).enumerate() {
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
                batch = self.draw_tabs(batch, palette, progress, y, labels, *selected);
                tabs_drawn = true;
                continue;
            }
            let header = row.kind == Kind::Header;
            let (shown_label, shown_value) = row.fitted();
            let ink = if header {
                palette.muted_ink
            } else {
                palette.ink
            };
            batch = batch
                .set(
                    *label,
                    Property::Transform(Transform::from_translation(Vec3::new(
                        -WIDTH / 2.0 + PADDING,
                        y,
                        LIFT,
                    ))),
                )
                .set(
                    *label,
                    Property::Text(text(
                        &shown_label,
                        if header { HEADER_SIZE } else { TEXT_SIZE },
                        TextAlign::Left,
                        draw::with_alpha(ink, progress),
                    )),
                )
                .set(
                    *value,
                    Property::Transform(Transform::from_translation(Vec3::new(
                        WIDTH / 2.0 - PADDING,
                        y,
                        LIFT,
                    ))),
                )
                .set(
                    *value,
                    Property::Text(text(
                        &shown_value,
                        TEXT_SIZE,
                        TextAlign::Right,
                        draw::with_alpha(palette.muted_ink, progress),
                    )),
                );
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
            batch = batch
                .set(
                    *prim,
                    Property::Transform(Transform::from_translation(Vec3::new(x, y, LIFT))),
                )
                .set(
                    *prim,
                    Property::Text(text(
                        &slate::tab_label(tab, count),
                        TEXT_SIZE,
                        TextAlign::Center,
                        draw::with_alpha(ink, progress),
                    )),
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
