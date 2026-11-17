//! Writes a [`Drawing`] into the prims each slot owns.

use std::rc::Rc;

use wired_guest::math::{
    Color,
    Transform,
    Vec3,
};

use super::{
    Bodies,
    Drawing,
    face::{
        Look,
        Wear,
    },
    prims::{
        PipShape,
        Shell,
        SlotPrims,
    },
};
use crate::{
    material::{
        glyph::Glyph,
        graphs,
    },
    part::{
        draw,
        mesh,
        spec::{
            Arrange,
            MoteSpec,
            PipPlacement,
            Role,
        },
        view::{
            SlotView,
            Style,
        },
    },
    tokens::{
        palette::Palette,
        tuning::Tuning,
    },
    wired::{
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::{
                Property,
                Text,
                TextAlign,
                TextAnchor,
            },
        },
        shading::graph::{
            GraphValue,
            set_overrides,
        },
    },
};

/// Contents sit within the body; depth marks ride outside it.
const INSIDE_ORBIT: f32 = 0.52;

const AROUND_ORBIT: f32 = 1.35;

const PIP_RADIUS: f32 = 0.13;

const MARK_RADIUS: f32 = 0.09;

const OVERFLOW_RADIUS: f32 = 0.11;

/// A toggle's bar is lit when it is on and receded when it is off, in the two
/// levels of the neutral family the rest of the surface already uses.
const fn bar_color(palette: &Palette, active: bool) -> Color {
    if active { palette.base } else { palette.dim }
}

/// Where a slot's name stands, or nowhere for a slot that has none.
fn label_pose(offset: Option<Vec3>) -> Transform {
    offset.map_or_else(draw::hidden, |offset| draw::placed(offset, 1.0))
}

/// A slot's offset into the idle breath. The golden angle, so neighbours never
/// land near each other and a form reads as several bodies rather than one.
fn phase(slot: usize) -> f32 {
    slot as f32 * 2.399_963
}

/// `len` pips from `start` of `total`, laid out in `arrange` at `spread`.
#[derive(Clone, Copy)]
struct Run {
    start:   usize,
    len:     usize,
    total:   usize,
    arrange: Arrange,
    spread:  f32,
    radius:  f32,
}

fn apply_run(batch: crate::Batch<'_>, prim: (u64, u64), run: Run) -> crate::Batch<'_> {
    let Run {
        start,
        len,
        total,
        arrange,
        spread,
        radius,
    } = run;
    if len == 0 {
        return batch;
    }
    draw::mesh(&mesh::cluster(start, len, total, arrange, spread, radius))
        .into_iter()
        .fold(batch, |batch, property| batch.set(prim, property))
}

impl Bodies {
    pub fn apply(&self, doc: &Document, drawing: &Drawing<'_>) -> anyhow::Result<()> {
        let Drawing {
            motes,
            delta,
            readout,
            readout_bloom,
            views,
            specs,
            drawn,
            placard,
            palette,
            held,
            eye,
        } = *drawing;
        if let Some(view) = placard {
            self.placard_at.set(view.position);
        }
        let tuning = Tuning {
            pane_em: self.tuning.placard_em,
            ..self.tuning
        };
        self.placard.borrow_mut().update(
            doc,
            placard.map(|view| (&view.sheet.runs, view.sheet.size)),
            self.placard_at.get(),
            palette,
            &tuning,
            delta,
        )?;

        self.ensure(doc, views.len())?;
        let slots = self.slots.borrow();

        for (index, (slot, view)) in slots.iter().zip(views).enumerate() {
            let mut batch = doc.local();
            if held == Some(index) {
                // The solver owns this slot's transform until it comes back;
                // forgetting what we last wrote makes the first frame back
                // write for sure, rather than trusting it matches.
                slot.root_xform.clear();
            } else if let Some(xform) = slot
                .root_xform
                .diff(draw::placed(view.position, view.bloom))
            {
                batch = batch.set(slot.root, Property::Transform(xform));
            }
            if let Some(xform) = slot.body_xform.diff(draw::placed(Vec3::ZERO, view.radius)) {
                batch = batch.set(slot.body, Property::Transform(xform));
            }

            let spec = drawn.get(index).and_then(|index| specs.get(*index));
            batch = Self::apply_label(batch, slot, view, spec, palette);

            // A page tab has no spec, so the chevrons go by role.
            let glyph = spec
                .and_then(|spec| spec.glyph.as_ref())
                .or_else(|| self.chevrons.of(view.role));
            let icon = slot.icon.borrow().is_some() || glyph.is_some();
            let shell = Shell {
                style:  view.style,
                active: spec.is_some_and(|spec| spec.condition.active),
                film:   spec.map_or(0.0, |spec| spec.film),
                frost:  spec.map_or(0.0, |spec| spec.frost),
                bloom:  view.bloom,
            };
            // Not `Option::map_or`: the closure would need to move `batch`
            // out of this scope, and a `Batch` is not `Clone`.
            #[expect(clippy::option_if_let_else)]
            let overrides = if let Some(shell) = slot.shell.diff(shell) {
                batch = batch
                    .set(slot.nested, Property::Material(draw::pip(view.style, true)))
                    .set(slot.plain, Property::Material(draw::pip(view.style, false)))
                    .set(
                        slot.overflow,
                        Property::Material(draw::pip(view.style, false)),
                    );
                batch = Self::mark_materials(batch, slot, view, shell.active, palette);
                Some((index, shell))
            } else {
                None
            };

            batch = Self::apply_pips(batch, slot, view);
            batch = Self::apply_marks(batch, slot, view, icon);
            // Every shadow above was committed as it was diffed, so a flush
            // that fails must take them back or its edits are never retried.
            if let Err(err) = batch.flush() {
                slot.forget();
                return Err(err.into());
            }

            if let Some((index, shell)) = overrides {
                Self::apply_shell(doc, slot, view, index, shell)?;
            }

            let owner = drawn.get(index).and_then(|index| motes.get(*index));
            // A held slot stands wherever the solver put it, which `view`
            // does not know, so its face keeps to the pole rather than aiming
            // from a position the slot has left.
            let seen_from = if held == Some(index) {
                view.position + Vec3::Z
            } else {
                eye
            };
            let look = Look::of_slot(view, seen_from, palette);
            self.apply_face(doc, &slot.face, look, Wear { owner, glyph }, delta)?;
        }

        let mut batch = doc.local();
        for slot in slots.iter().skip(views.len()) {
            slot.face.unseen();
            batch = batch.set(slot.root, Property::Transform(draw::hidden()));
        }
        if !self.shown.get() {
            // A dismissed surface stops being stepped once it has left, so a
            // glyph changed while it is down should appear on return rather
            // than flip.
            for slot in slots.iter() {
                slot.face.unseen();
            }
        }
        batch.flush()?;
        self.apply_readout(doc, readout, palette, readout_bloom, delta)
    }

    /// Draws the centre's text, or hides it. Gone the moment the surface is
    /// sent away, since a dismissed surface is not stepped once it has left
    /// and would otherwise leave it hanging in the room.
    fn apply_readout(
        &self,
        doc: &Document,
        text: Option<&Rc<Glyph>>,
        palette: &Palette,
        bloom: f32,
        delta: f32,
    ) -> anyhow::Result<()> {
        let up = self.shown.get();
        let look = Look {
            pose: draw::placed(Vec3::ZERO, 2.0 * self.tuning.readout_radius),
            color: palette.glyph,
            heat: 0.0,
            bloom,
        };
        let wear = Wear {
            owner: None,
            glyph: text.filter(|_| up),
        };
        self.apply_face(doc, &self.readout, look, wear, delta)?;
        if !up {
            self.readout.unseen();
        }
        Ok(())
    }

    /// Hands the shell graph what differs between one mote and the next.
    ///
    /// The palette still decides colour and the graph decides light: it is
    /// given a finished tint rather than a heat to interpret, so every rule
    /// about what a mote's colour means stays where it is written and tested.
    /// Heat goes over only because a rim is per-fragment and cannot be
    /// computed here.
    fn apply_shell(
        doc: &Document,
        slot: &SlotPrims,
        view: &SlotView,
        index: usize,
        shell: Shell,
    ) -> anyhow::Result<()> {
        set_overrides(
            doc,
            slot.body,
            Layer::Local,
            &[
                (
                    graphs::SHELL_TINT,
                    GraphValue::Color(draw::with_alpha(shell.style.color, shell.style.alpha)),
                ),
                (
                    graphs::SHELL_EMISSIVE,
                    GraphValue::Float(shell.style.emissive),
                ),
                (graphs::SHELL_HEAT, GraphValue::Float(view.heat)),
                (graphs::SHELL_PHASE, GraphValue::Float(phase(index))),
                (graphs::SHELL_FILM, GraphValue::Float(shell.film)),
                (graphs::SHELL_FROST, GraphValue::Float(shell.frost)),
                (graphs::SHELL_BLOOM, GraphValue::Float(shell.bloom)),
            ],
        )?;
        Ok(())
    }

    /// Drawn always; only the attended name's brightness changes with
    /// attention. A slot with no spec is a page tab, which has no name, and
    /// whatever mote last used the slot must not leave its name behind.
    fn apply_label<'a>(
        mut batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
        spec: Option<&MoteSpec>,
        palette: &Palette,
    ) -> crate::Batch<'a> {
        let offset = spec.map(|_| view.label_offset);
        if let Some(offset) = slot.label_xform.diff(offset) {
            batch = batch.set(slot.label, Property::Transform(label_pose(offset)));
        }
        let Some(spec) = spec else {
            return batch;
        };

        let Some((label, attention)) = slot.written.diff((spec.label.clone(), view.attention))
        else {
            return batch;
        };
        let color = palette.tint(attention);

        batch.set(
            slot.label,
            Property::Text(Text {
                value:         label.to_string(),
                size:          Some(view.label_size),
                align:         Some(TextAlign::Center),
                anchor:        Some(TextAnchor::Top),
                wrap:          None,
                line_height:   None,
                color:         Some(color),
                // A surface does not control its surroundings, so its names
                // carry their own contrast.
                outline:       Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.85,
                }),
                outline_width: Some(0.22),
                emissive:      Some(if attention.is_active() { 0.5 } else { 0.1 }),
                billboard:     None,
            }),
        )
    }

    /// Colours the marks: the bar by whether its toggle is on, the rest in
    /// the receded level so a kind is legible without competing with the
    /// state.
    fn mark_materials<'a>(
        batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
        active: bool,
        palette: &Palette,
    ) -> crate::Batch<'a> {
        let styled = |color| {
            Property::Material(draw::pip(
                Style {
                    color,
                    alpha: 1.0,
                    emissive: view.style.emissive,
                },
                false,
            ))
        };
        // The cast mark wears the mote's own hue rather than the receded
        // neutral, so it reads as part of the mote instead of a grey band
        // around it.
        batch
            .set(slot.marks.bar.prim, styled(bar_color(palette, active)))
            .set(slot.marks.ring.prim, styled(view.style.color))
    }

    /// Marks the kind of mote the body is, scaled with the body.
    ///
    /// A toggle's bar is drawn only when the slot shows no face or icon: with
    /// one it would cut across the picture, and the colour already says
    /// whether the toggle is on. `icon` is whether the slot shows one.
    fn apply_marks<'a>(
        batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
        icon: bool,
    ) -> crate::Batch<'a> {
        let on = |wanted: bool| wanted.then(|| draw::placed(Vec3::ZERO, view.radius));
        let marks = &slot.marks;
        let batch = marks.bar.apply(
            batch,
            mesh::toggle_bar,
            on(matches!(view.role, Role::Toggle) && !icon),
        );
        marks
            .ring
            .apply(batch, mesh::cast_ring, on(matches!(view.role, Role::Cast)))
    }

    /// Drawn unconditionally: how much a container holds is structural.
    fn apply_pips<'a>(
        mut batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
    ) -> crate::Batch<'a> {
        let groups = view.pips.groups();
        let shape = PipShape {
            count: view.pips.count,
            groups,
            overflow: view.pips.overflow,
            placement: view.pips.placement,
            arrange: view.pips.arrange,
        };

        let arrange = view.pips.arrange;
        let total = view.pips.count;
        let (spread, radius) = match view.pips.placement {
            PipPlacement::Inside => (INSIDE_ORBIT, PIP_RADIUS),
            PipPlacement::Around => (AROUND_ORBIT, MARK_RADIUS),
        };

        if slot.shape.diff(shape).is_some() {
            let run = |start, len| Run {
                start,
                len,
                total,
                arrange,
                spread,
                radius,
            };
            batch = apply_run(batch, slot.nested, run(0, groups));
            batch = apply_run(batch, slot.plain, run(groups, total - groups));
        }

        if view.pips.overflow && !slot.marked.get() {
            slot.marked.set(true);
            batch = draw::mesh(&mesh::overflow_marker(OVERFLOW_RADIUS))
                .into_iter()
                .fold(batch, |batch, property| batch.set(slot.overflow, property));
        }

        let visible = draw::placed(Vec3::ZERO, view.radius);
        // The pips are unit-space and scaled by the body; the marker's mesh is
        // centred, so its own offset is scaled the same way by hand. Asked for
        // only when there is a marker: an empty level has no cell after its
        // last pip.
        let marker = view.pips.overflow.then(|| {
            let at = Vec3::from_array(mesh::overflow_at(arrange, total, spread));
            draw::placed(at * view.radius, view.radius)
        });

        for (prim, placed) in [
            (slot.nested, (groups > 0).then_some(visible)),
            (slot.plain, (view.pips.count > groups).then_some(visible)),
            (slot.overflow, marker),
        ] {
            batch = batch.set(
                prim,
                Property::Transform(placed.unwrap_or_else(draw::hidden)),
            );
        }
        batch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_toggle_s_bar_is_lit_when_on_and_receded_when_off() {
        let palette = Palette::DEFAULT;
        assert_eq!(bar_color(&palette, true), palette.base);
        assert_eq!(bar_color(&palette, false), palette.dim);
    }

    #[test]
    fn a_slot_with_no_name_hides_the_label_it_inherited() {
        let named = label_pose(Some(Vec3::new(0.0, -0.04, 0.004)));
        assert_eq!(named.scale, Vec3::ONE);
        assert_eq!(label_pose(None), draw::hidden());
    }
}
