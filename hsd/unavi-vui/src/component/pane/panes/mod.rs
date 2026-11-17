//! Panes a script has mounted: each a pane, the prims that draw it, and
//! the body that carries its pose and takes its input.

use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    component::pane::{
        Event,
        Pane,
        Refused,
        block::Block,
        draw::Runs,
        prims::PanePrims,
    },
    machinery::float::Float,
    part::{
        draw,
        shadow::Shadow,
        tether::prims::TetherPrims,
    },
    tokens::tuning::Tuning,
    wired::{
        input::{
            device,
            types::{
                InputSubscription,
                PointerKind,
            },
        },
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::Property,
        },
    },
};

mod input;
mod pose;

/// A pane [`Panes`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneId(usize);

/// Events a pane holds for a consumer that has not asked.
const EVENT_CAPACITY: usize = 64;

/// Where a pane is and how it gets there. `at` is in the script's document
/// frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mount {
    /// Stands with its bottom centre on `at`, turned about the vertical
    /// axis to face the viewer.
    Standing { at: Vec3 },
    /// A weightless body centred on `at` that a grip carries, which once
    /// let go drifts to a stop and turns to face the viewer.
    Floating { at: Vec3 },
}

/// A drag that started on a pane and ends where the trigger is released.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Drag {
    Scrub {
        id:         u32,
        per_column: f32,
        last:       Option<f32>,
    },
    Slide {
        id:   u32,
        left: f32,
        wide: f32,
        last: f32,
    },
    Rail {
        block:  u32,
        top:    f32,
        height: f32,
    },
}

struct Mounted {
    /// Carries the pose and the collider, at the pane's middle.
    body:       (u64, u64),
    prims:      PanePrims,
    tether:     TetherPrims,
    /// Where the tether ends, in the script's document frame.
    tether_to:  Option<Vec3>,
    /// What the tether is coloured, or none for the neutral one.
    hue:        Option<Color>,
    input:      InputSubscription,
    pane:       Pane,
    mount:      Option<Mount>,
    shown:      bool,
    events:     Vec<Event>,
    /// What the pane draws, until something changes it.
    runs:       Option<Runs>,
    /// The body as last written or read, which input is read against.
    pose:       Option<Transform>,
    /// Where the standing pose is eased from.
    drawn:      Option<Transform>,
    written:    Shadow<Transform>,
    /// Whether the body is in the simulation, which it is while floating.
    simulated:  bool,
    float:      Float,
    /// Where the script last put a floating pane, until it is moved there.
    put:        Option<Vec3>,
    collider:   Shadow<Vec2>,
    /// Whether the body has been posed and given its collider.
    placed:     bool,
    /// Seconds of bobbing, for its phase.
    bob:        f32,
    /// How far the pane is drawn above its body, which input is read
    /// against.
    bob_offset: f32,
    drag:       Option<(PointerKind, Drag)>,
}

/// Every pane a script holds, as a free list like the panels.
#[derive(Default)]
pub struct Panes {
    slots: Vec<Option<Mounted>>,
    free:  Vec<usize>,
    /// Where the viewer's eye was last drawn from, which floating panes
    /// turn to.
    eye:   Option<Vec3>,
    /// Every key pressed, for the pane that has the keyboard.
    keys:  Option<InputSubscription>,
    /// The pane that has the keyboard.
    focus: Option<usize>,
    shift: bool,
}

impl Panes {
    pub fn add(&mut self, doc: &Document, pane: Pane) -> anyhow::Result<PaneId> {
        if self.keys.is_none() {
            self.keys = Some(device::listen().map_err(|err| anyhow::anyhow!("{err:?}"))?);
        }
        let body = doc.create_prim(Layer::Local, None)?;
        doc.local()
            .set(body, Property::Transform(draw::hidden()))
            .flush()?;
        let mounted = Mounted {
            body,
            prims: PanePrims::new(doc, body)?,
            tether: TetherPrims::new(doc)?,
            tether_to: None,
            hue: None,
            input: crate::wired::input::targeted::listen(doc, body)?,
            pane,
            mount: None,
            shown: false,
            events: Vec::new(),
            runs: None,
            pose: None,
            drawn: None,
            written: Shadow::new(),
            simulated: false,
            float: Float::default(),
            put: None,
            collider: Shadow::new(),
            placed: false,
            bob: 0.0,
            bob_offset: 0.0,
            drag: None,
        };
        let index = if let Some(index) = self.free.pop() {
            self.slots[index] = Some(mounted);
            index
        } else {
            self.slots.push(Some(mounted));
            self.slots.len() - 1
        };
        Ok(PaneId(index))
    }

    pub fn remove(&mut self, doc: &Document, id: PaneId) -> anyhow::Result<()> {
        let Some(mounted) = self.slots.get_mut(id.0).and_then(Option::take) else {
            return Ok(());
        };
        self.free.push(id.0);
        if self.focus == Some(id.0) {
            self.focus = None;
        }
        // Both go whichever fails, so a failed body does not strand the
        // tether.
        let body = doc.local().remove(mounted.body).flush();
        let tether = mounted.tether.remove(doc);
        body?;
        tether
    }

    pub fn set_title(&mut self, id: PaneId, title: &str) -> Result<(), Refused> {
        let Some(mounted) = self.get_mut(id) else {
            return Ok(());
        };
        mounted.pane.set_title(title)?;
        mounted.runs = None;
        Ok(())
    }

    pub fn set_status(&mut self, id: PaneId, status: Option<&str>) -> Result<(), Refused> {
        let Some(mounted) = self.get_mut(id) else {
            return Ok(());
        };
        mounted.pane.set_status(status)?;
        mounted.runs = None;
        Ok(())
    }

    pub fn set_blocks(&mut self, id: PaneId, blocks: Vec<Block>) -> Result<(), Refused> {
        let Some(mounted) = self.get_mut(id) else {
            return Ok(());
        };
        if mounted.pane.set_blocks(blocks)? {
            mounted.runs = None;
            mounted.end_drag_if_gone();
        }
        Ok(())
    }

    /// The pane, width by height in metres.
    #[must_use]
    pub fn size(&self, id: PaneId, tuning: &Tuning) -> Vec2 {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .map_or(Vec2::ZERO, |mounted| {
                mounted.pane.size() * mounted.em(tuning)
            })
    }

    pub fn set_mount(&mut self, id: PaneId, mount: Mount) {
        if let Some(mounted) = self.get_mut(id) {
            if let Mount::Floating { at } = mount {
                mounted.put = Some(at);
            }
            mounted.mount = Some(mount);
        }
    }

    pub fn set_tether(&mut self, id: PaneId, to: Option<Vec3>) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.tether_to = to;
        }
    }

    pub fn set_hue(&mut self, id: PaneId, hue: Option<Color>) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.hue = hue;
        }
    }

    pub fn set_shown(&mut self, id: PaneId, shown: bool) {
        if let Some(mounted) = self.get_mut(id) {
            mounted.shown = shown;
            if !shown {
                mounted.end_drag();
            }
        }
        if !shown && self.focus == Some(id.0) {
            self.focus = None;
        }
    }

    pub fn drain(&mut self, id: PaneId) -> Vec<Event> {
        self.get_mut(id)
            .map(|mounted| std::mem::take(&mut mounted.events))
            .unwrap_or_default()
    }

    fn get_mut(&mut self, id: PaneId) -> Option<&mut Mounted> {
        self.slots.get_mut(id.0)?.as_mut()
    }
}

impl Mounted {
    /// Metres in one em of the pane, which a standing pane has more of since
    /// it is read from further away.
    const fn em(&self, tuning: &Tuning) -> f32 {
        match self.mount {
            Some(Mount::Standing { .. }) => tuning.stand_em,
            Some(Mount::Floating { .. }) | None => tuning.pane_em,
        }
    }

    fn push(&mut self, event: Event) {
        self.events.push(event);
        let overrun = self.events.len().saturating_sub(EVENT_CAPACITY);
        self.events.drain(..overrun);
    }

    /// Ends a drag, telling the consumer, so a value it kept open is kept.
    fn end_drag(&mut self) {
        let Some((_, drag)) = self.drag.take() else {
            return;
        };
        match drag {
            Drag::Scrub { id, .. } | Drag::Slide { id, .. } => self.push(Event::Released(id)),
            Drag::Rail { .. } => {}
        }
        self.pane.set_pressed(None);
        self.runs = None;
    }

    /// Ends a drag on a control the new blocks no longer hold.
    fn end_drag_if_gone(&mut self) {
        let held = match self.drag {
            Some((_, Drag::Scrub { id, .. } | Drag::Slide { id, .. })) => Some(id),
            _ => None,
        };
        if held.is_some_and(|id| !self.pane.focus_order().contains(&id)) {
            self.end_drag();
        }
    }
}
