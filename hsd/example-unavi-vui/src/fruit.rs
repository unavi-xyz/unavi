//! A fruit in both of its bodies.
//!
//! Every item mote here wears two things that are deliberately not the same
//! object: an **icon**, a prim in this script's own document authored to fit a
//! unit sphere, which VUI shrinks into the mote's shell; and an **item**, a
//! document of its own at the size the thing really is, minted the moment the
//! mote is let go somewhere. Same recipe, two scales, two lifetimes — the icon
//! is drawn forever, the item is made on landing.
//!
//! A document is one thing, so what landing means depends on the mote. A
//! source mints another fruit every time. A unique one mints its fruit once
//! and moves that same document ever after: pick the pear up off the floor,
//! drop it somewhere else, and it is the pear that moved.

use wired_prelude::prelude::*;

use crate::{
    unavi::{
        shapes::api::{
            Capsule,
            Cuboid,
            Sphere,
        },
        vui::api::{
            Kind,
            Landing,
            Mote,
        },
    },
    wired::{
        physics::api::set_linear_velocity,
        scene::{
            api::create_document,
            types::{
                Document,
                Material,
                Prim,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

/// The icon fills half its mote, so the shell still reads as a shell.
const ICON: f32 = 0.5;
/// The glyph every icon wears: monochrome, because the shell carries the
/// hue. The delivered thing keeps `variety.color` — only the mote's icon is
/// neutral, or the two would read as clashing rather than as one form.
const ICON_GLASS: Color = rgb(0.92, 0.94, 0.96);

#[derive(Clone, Copy)]
pub enum Shape {
    Round,
    Long,
    Cube,
}

/// A kind of fruit, before there is a mote for it.
#[derive(Clone, Copy)]
pub struct Variety {
    pub label:       &'static str,
    pub description: &'static str,
    pub shape:       Shape,
    pub color:       Color,
    /// Metres across, as the thing itself rather than as a mote.
    pub size:        f32,
    /// Whether the shop has one of these or a crate of them.
    pub unique:      bool,
}

/// A mote, and what it takes to build what it delivers.
pub struct Fruit {
    pub mote: Mote,
    shape:    Shape,
    color:    Color,
    /// Metres across, as the thing itself rather than as a mote.
    size:     f32,
    /// The one fruit a unique mote stands for, once it has been made. A source
    /// never fills this: each of its fruit is somebody else's now.
    placed:   Option<Prim>,
}

impl Fruit {
    pub fn grow(variety: &Variety) -> Self {
        let mote = Mote::new(Kind::Item, variety.label);
        if !variety.description.is_empty() {
            mote.describe(variety.description);
        }
        mote.set_unique(variety.unique);
        mote.set_tint(Some(variety.color));
        mote.set_icon(Some(&dressed(icon(variety.shape), ICON_GLASS)));

        Self {
            mote,
            shape: variety.shape,
            color: variety.color,
            size: variety.size,
            placed: None,
        }
    }

    /// Puts a fruit where the mote was let go.
    ///
    /// The position rides the body rather than the document's offset. A
    /// document stands where it was anchored and physics owns what is inside
    /// it, so moving the frame under a body that avian is already simulating
    /// moves nothing; writing the body's own transform teleports it.
    pub fn deliver(&mut self, landing: Landing) -> anyhow::Result<Throw> {
        if let Some(body) = &self.placed {
            body.set_xform(Some(at(landing.at)))?;
            return Ok(Throw::new(body, landing.velocity));
        }

        // Built in full while the document is still held out of the scene, so
        // what the room sees is a fruit appearing where it was let go rather
        // than one arriving at the origin and moving.
        let document = create_document()?;
        let body = self.body(&document);
        body.set_xform(Some(at(landing.at)))?;
        body.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        Some(0.6),
            linear_damping:  None,
            mass:            None,
            restitution:     Some(0.2),
        }))?;
        // Anchored to the space root with no offset of its own: the fruit's
        // place is the body's, and one frame of reference is enough.
        document.set_offset(at(Vec3::ZERO))?;

        let throw = Throw::new(&body, landing.velocity);
        if self.mote.unique() {
            self.placed = Some(body);
        }
        Ok(throw)
    }

    /// The fruit at the size it is in the world, with the collider that lets
    /// it be picked up again once it has landed.
    fn body(&self, document: &Document) -> Prim {
        let prim = match self.shape {
            Shape::Round => {
                let sphere = Sphere::new(self.size);
                sphere.set_doc(document.clone());
                let prim = sphere.mesh();
                prim.set_collider(Some(sphere.collider())).ok();
                prim
            }
            Shape::Long => {
                let capsule = Capsule::new(self.size * 0.6, self.size);
                capsule.set_doc(document.clone());
                let prim = capsule.mesh();
                prim.set_collider(Some(capsule.collider())).ok();
                prim
            }
            Shape::Cube => {
                let cuboid = Cuboid::new(Vec3::splat(self.size * 1.6));
                cuboid.set_doc(document.clone());
                let prim = cuboid.mesh();
                prim.set_collider(Some(cuboid.collider())).ok();
                prim
            }
        };
        dressed(prim, self.color)
    }
}

/// The throw a placed fruit has not taken yet.
///
/// A document placed this tick has no bodies in the world until the placement
/// is committed, and velocity is set on a body rather than written into the
/// document, so the throw lands on a later tick than the fruit does.
pub struct Throw {
    body:     Prim,
    velocity: Vec3,
    /// Ticks left to find the body before the throw is given up on.
    tries:    u32,
}

/// Generous: one tick is normally enough, and a fruit that lands still is a
/// worse failure than a fruit that lands late.
const THROW_TRIES: u32 = 8;

impl Throw {
    fn new(body: &Prim, velocity: Vec3) -> Self {
        Self {
            body: body.clone(),
            velocity,
            tries: THROW_TRIES,
        }
    }

    /// Gives the fruit its momentum, reporting whether it still has to be
    /// tried again.
    pub fn apply(&mut self) -> bool {
        self.tries = self.tries.saturating_sub(1);
        if set_linear_velocity(&self.body, self.velocity).is_ok() {
            return false;
        }
        if self.tries == 0 {
            // Best-effort by nature: the fruit is already where it was let go,
            // and only the throw is lost.
            eprintln!("gave up on the throw for a fruit that never grew a body");
        }
        self.tries > 0
    }
}

const fn at(translation: Vec3) -> Transform {
    Transform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
}

fn icon(shape: Shape) -> Prim {
    match shape {
        Shape::Round => Sphere::new(ICON).mesh(),
        Shape::Long => Capsule::new(ICON * 0.6, ICON).mesh(),
        Shape::Cube => Cuboid::new(Vec3::splat(ICON * 1.6)).mesh(),
    }
}

/// Lit rather than lit-by-the-room: a mote hangs in mid-air, where there is
/// nothing to bounce light off.
fn dressed(prim: Prim, color: Color) -> Prim {
    prim.set_material(Some(Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        double_sided: None,
        emissive:     Some(Color {
            r: color.r * 0.25,
            g: color.g * 0.25,
            b: color.b * 0.25,
            a: 1.0,
        }),
        metallic:     Some(0.0),
        roughness:    Some(0.6),
    }))
    .ok();
    prim
}

pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}
