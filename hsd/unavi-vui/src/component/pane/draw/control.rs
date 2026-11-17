//! A control as runs.

use wired_guest::math::Vec2;

use crate::{
    component::pane::{
        BASELINE,
        LINE,
        Pane,
        Rect,
        block::Tone,
        control::{
            Control,
            format_number,
            segments,
        },
        draw::{
            Fill,
            Runs,
        },
    },
    face::measure::text,
};

/// Share of the line a track, a bead and an action's ground fill, so
/// neighbouring rows do not touch.
const INSET: f32 = 0.2;
/// Ems thick the line under a focused control is.
const FOCUS_LINE: f32 = 0.1;

impl Pane {
    /// Draws `control` in `area`, the row it sits on.
    pub(super) fn draw_control(&self, runs: &mut Runs, id: u32, control: &Control, area: Rect) {
        let view = self.view();
        let target = control.pressable().then_some(id);
        if target.is_some() {
            if view.pressed == Some(id) {
                runs.mark(area, Fill::Accent, target);
            } else if view.hovered == Some(id) {
                runs.mark(area, Fill::Tint, target);
            }
            if view.focus == Some(id) {
                let under = Rect {
                    y: area.y + area.h - FOCUS_LINE,
                    h: FOCUS_LINE,
                    ..area
                };
                runs.mark(under, Fill::Focus, target);
            }
        }

        let inset = Rect {
            y: LINE.mul_add(INSET, area.y),
            h: LINE * 2.0f32.mul_add(-INSET, 1.0),
            ..area
        };
        let baseline = |x: f32| Vec2::new(x, BASELINE + area.y);
        match control {
            Control::Value(value) => runs.text(value, baseline(area.x), 1.0, Tone::Body, None),
            Control::Link(label) => {
                runs.text(label, baseline(area.x), 1.0, Tone::Body, target);
                runs.rect(
                    Rect {
                        y: baseline(area.x).y + 0.14,
                        h: 0.08,
                        ..area
                    },
                    Fill::Underline,
                    target,
                );
            }
            Control::Number { value, .. } => {
                runs.text(
                    &format_number(*value),
                    baseline(area.x),
                    1.0,
                    Tone::Body,
                    target,
                );
            }
            Control::Action { label, primary } => {
                let fill = if *primary { Fill::Accent } else { Fill::Tint };
                runs.rect(inset, fill, target);
                let shown = text::truncate(label, area.w);
                let x = (area.w - text::width(&shown)).mul_add(0.5, area.x);
                runs.text(&shown, baseline(x), 1.0, Tone::Body, target);
            }
            Control::Toggle(on) => {
                runs.rect(inset, Fill::Track, target);
                let half = area.w / 2.0;
                runs.rect(
                    Rect {
                        x: if *on { area.x + half } else { area.x },
                        w: half,
                        ..inset
                    },
                    Fill::Lit,
                    target,
                );
            }
            Control::Slider(value) | Control::Meter(value) => {
                runs.rect(inset, Fill::Track, target);
                runs.rect(
                    Rect {
                        w: area.w * value.clamp(0.0, 1.0),
                        ..inset
                    },
                    Fill::Lit,
                    target,
                );
            }
            Control::Choice { options, selected } => {
                for (index, (option, (start, width))) in
                    options.iter().zip(segments(options)).enumerate()
                {
                    if start >= area.w {
                        break;
                    }
                    let fill = if index == *selected {
                        Fill::Accent
                    } else {
                        Fill::Tint
                    };
                    let cell = Rect {
                        x: area.x + start,
                        w: width,
                        ..inset
                    };
                    runs.rect(cell, fill, target);
                    let x = (width - text::width(option)).mul_add(0.5, cell.x);
                    runs.text(option, baseline(x), 1.0, Tone::Body, target);
                }
            }
        }
    }
}
