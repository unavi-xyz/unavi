//! The pool of prims that draw corner brackets, in the visor's own document.
//!
//! A set is 24 thin bars, one document's brackets. Sets are made once and
//! handed from document to document.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use super::{
    cube::Cube,
    geometry::{
        BARS,
        Bar,
    },
};
use crate::wired::scene::{
    document::{
        Document,
        Edit,
        Layer,
        script_document,
    },
    properties::{
        Material,
        Property,
    },
};

/// Metres across a bar.
const THIN: f32 = 0.006;

/// Metres across a bar of an attended document.
const THICK: f32 = 0.010;

/// Sets made per call to [`Brackets::grow`].
const MAKE_PER_CALL: usize = 2;

/// How strongly a document's brackets show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// A document the local user may not write.
    Dim,
    /// A document the local user may write.
    Solid,
    /// The attended document.
    Raised,
}

impl Level {
    const fn thickness(self) -> f32 {
        match self {
            Self::Raised => THICK,
            Self::Dim | Self::Solid => THIN,
        }
    }

    /// Surface and glow strengths, as multiples of the colour.
    const fn strengths(self) -> (f32, f32) {
        match self {
            Self::Dim => (0.25, 0.5),
            Self::Solid => (0.6, 1.2),
            Self::Raised => (1.0, 2.2),
        }
    }
}

/// One document's brackets to draw.
pub struct Drawn {
    pub bars:     [Bar; BARS],
    /// The rotation of the document's frame, which every bar takes.
    pub rotation: Quat,
    pub color:    Color,
    pub level:    Level,
}

struct Set {
    prims:   [(u64, u64); BARS],
    /// What the bars last showed, `None` while put away.
    shown:   Option<[Transform; BARS]>,
    applied: Option<(Color, Level)>,
}

pub struct Brackets {
    doc:   Document,
    cube:  Cube,
    sets:  Vec<Set>,
    edits: Vec<Edit>,
}

const HIDDEN: Transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ZERO);

fn material(color: Color, level: Level) -> Material {
    let (surface, glow) = level.strengths();
    let scaled = |factor: f32| Color::rgb(color.r * factor, color.g * factor, color.b * factor);
    Material {
        base_color: Some(scaled(surface)),
        emissive: Some(scaled(glow)),
        ..Material::default()
    }
}

impl Brackets {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc:   script_document()?,
            cube:  Cube::new(),
            sets:  Vec::new(),
            edits: Vec::new(),
        })
    }

    #[must_use]
    pub const fn count(&self) -> usize {
        self.sets.len()
    }

    /// Makes sets until there are `want`, a couple at a time so a crowded
    /// space does not build hundreds of prims in one tick.
    pub fn grow(&mut self, want: usize) -> anyhow::Result<()> {
        for _ in 0..MAKE_PER_CALL {
            if self.sets.len() >= want {
                break;
            }
            let set = self.make()?;
            self.sets.push(set);
        }
        Ok(())
    }

    fn make(&self) -> anyhow::Result<Set> {
        let mut prims = [(0, 0); BARS];
        let mut edits = Vec::new();
        for (index, prim) in prims.iter_mut().enumerate() {
            match self.doc.create_prim(Layer::Local, None) {
                Ok(made) => *prim = made,
                Err(err) => {
                    self.remove(&prims[..index]);
                    return Err(err.into());
                }
            }
            for property in self.cube.properties() {
                edits.push(Edit::Set((*prim, property)));
            }
            edits.push(Edit::Set((*prim, Property::Transform(HIDDEN))));
        }
        if let Err(err) = self.doc.apply(Layer::Local, &edits) {
            self.remove(&prims);
            return Err(err.into());
        }
        Ok(Set {
            prims,
            shown: None,
            applied: None,
        })
    }

    fn remove(&self, prims: &[(u64, u64)]) {
        let edits: Vec<_> = prims.iter().map(|prim| Edit::Remove(*prim)).collect();
        if let Err(err) = self.doc.apply(Layer::Local, &edits) {
            eprintln!("visor: removing unwritten brackets: {err:?}");
        }
    }

    /// Queues drawing `drawn` on set `slot`, or putting the set away. Only
    /// what changed is queued; [`Brackets::flush`] sends it.
    pub fn stage(&mut self, slot: usize, drawn: Option<&Drawn>) {
        let Some(set) = self.sets.get_mut(slot) else {
            return;
        };
        let Some(drawn) = drawn else {
            if set.shown.take().is_some() {
                for prim in set.prims {
                    self.edits
                        .push(Edit::Set((prim, Property::Transform(HIDDEN))));
                }
            }
            return;
        };

        let look = (drawn.color, drawn.level);
        if set.applied != Some(look) {
            set.applied = Some(look);
            for prim in set.prims {
                let material = material(drawn.color, drawn.level);
                self.edits
                    .push(Edit::Set((prim, Property::Material(material))));
            }
            set.shown = None;
        }

        let thickness = drawn.level.thickness();
        let poses: [Transform; BARS] = std::array::from_fn(|index| {
            let bar = drawn.bars[index];
            let mut scale = Vec3::splat(thickness);
            scale[bar.axis] = bar.length;
            Transform::new(bar.centre, drawn.rotation, scale)
        });
        for (index, pose) in poses.iter().enumerate() {
            let moved = set.shown.is_none_or(|shown| shown[index] != *pose);
            if moved {
                self.edits
                    .push(Edit::Set((set.prims[index], Property::Transform(*pose))));
            }
        }
        set.shown = Some(poses);
    }

    /// Sends everything staged as one batch.
    pub fn flush(&mut self) {
        if self.edits.is_empty() {
            return;
        }
        if let Err(err) = self.doc.apply(Layer::Local, &self.edits) {
            eprintln!("visor: drawing brackets: {err:?}");
            for set in &mut self.sets {
                set.shown = None;
                set.applied = None;
            }
        }
        self.edits.clear();
    }
}
