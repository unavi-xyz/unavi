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
//!
//! NOTE: `unavi:shapes` is not yet ported (it is owned by another
//! unit of this work), so the `Sphere`/`Capsule`/`Cuboid` calls below are a
//! best-effort translation against its published WIT and have not been
//! type-checked against real bindings. `set-doc` is typed as an owned
//! `document` rather than `borrow<document>` in that WIT; this file
//! re-opens a fresh handle for it rather than assuming `Document` is
//! `Clone`, which is unconfirmed.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

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
        core::ids::DocumentId,
        physics::simulation::set_velocity,
        scene::{
            document::{
                Anchor,
                Document,
                create_document,
                open_document,
            },
            properties::{
                Material,
                Property,
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
    /// The one fruit a unique mote stands for, once it has been made: the
    /// document it stands in, and its body. A source never fills this: each
    /// of its fruit is somebody else's now.
    placed:   Option<(DocumentId, (u64, u64))>,
}

impl Fruit {
    pub fn grow(variety: &Variety) -> anyhow::Result<Self> {
        let mote = Mote::new(Kind::Item, variety.label)?;
        if !variety.description.is_empty() {
            mote.set_description(variety.description)?;
        }
        mote.set_unique(variety.unique);
        mote.set_tint(Some(variety.color));

        let icon_doc = crate::wired::scene::document::script_document()?;
        let icon = icon(&icon_doc, variety.shape)?;
        dressed(&icon_doc, icon, ICON_GLASS)?;
        mote.set_icon(&icon_doc, Some(icon));

        Ok(Self {
            mote,
            shape: variety.shape,
            color: variety.color,
            size: variety.size,
            placed: None,
        })
    }

    /// Puts a fruit where the mote was let go.
    ///
    /// The position rides the body rather than the document's offset. A
    /// document stands where it was anchored and physics owns what is inside
    /// it, so moving the frame under a body that avian is already simulating
    /// moves nothing; writing the body's own transform teleports it.
    pub fn deliver(&mut self, landing: Landing) -> anyhow::Result<Throw> {
        if let Some((document_id, body)) = self.placed {
            let document = reopen(document_id)?;
            document
                .local()
                .set(body, Property::Transform(at(landing.at)))
                .flush()?;
            return Ok(Throw::new(document_id, body, landing.velocity));
        }

        // Built in full while the document is still held out of the scene, so
        // what the room sees is a fruit appearing where it was let go rather
        // than one arriving at the origin and moving.
        let document = create_document()?;
        let document_id = document.id();
        let body = self.body(&document)?;
        document
            .local()
            .set(body, Property::Transform(at(landing.at)))
            .set(
                body,
                Property::RigidBody(RigidBody {
                    kind:            RigidBodyKind::Dynamic,
                    angular_damping: None,
                    friction:        Some(0.6),
                    linear_damping:  None,
                    mass:            None,
                    restitution:     Some(0.2),
                }),
            )
            .flush()?;
        // Anchored to the space root with no offset of its own: the fruit's
        // place is the body's, and one frame of reference is enough.
        document.place(Anchor::Space, at(Vec3::ZERO))?;

        let throw = Throw::new(document_id, body, landing.velocity);
        if self.mote.unique() {
            self.placed = Some((document_id, body));
        }
        Ok(throw)
    }

    /// The fruit at the size it is in the world, with the collider that lets
    /// it be picked up again once it has landed.
    fn body(&self, document: &Document) -> anyhow::Result<(u64, u64)> {
        // `set-doc` takes the document by value; a fresh handle is spent on
        // it so `document` stays usable afterward for the collider write.
        let target = reopen(document.id())?;
        let prim = match self.shape {
            Shape::Round => {
                let sphere = Sphere::new(self.size);
                sphere.set_doc(target);
                let prim = sphere.mesh()?;
                document
                    .local()
                    .set(prim, Property::Collider(sphere.collider()))
                    .flush()?;
                prim
            }
            Shape::Long => {
                let capsule = Capsule::new(self.size * 0.6, self.size);
                capsule.set_doc(target);
                let prim = capsule.mesh()?;
                document
                    .local()
                    .set(prim, Property::Collider(capsule.collider()))
                    .flush()?;
                prim
            }
            Shape::Cube => {
                let cuboid = Cuboid::new(Vec3::splat(self.size * 1.6));
                cuboid.set_doc(target);
                let prim = cuboid.mesh()?;
                document
                    .local()
                    .set(prim, Property::Collider(cuboid.collider()))
                    .flush()?;
                prim
            }
        };
        dressed(document, prim, self.color)?;
        Ok(prim)
    }
}

/// The throw a placed fruit has not taken yet.
///
/// A document placed this tick has no bodies in the world until the placement
/// is committed, and velocity is set on a body rather than written into the
/// document, so the throw lands on a later tick than the fruit does.
pub struct Throw {
    document: DocumentId,
    body:     (u64, u64),
    velocity: Vec3,
    /// Ticks left to find the body before the throw is given up on.
    tries:    u32,
}

/// Generous: one tick is normally enough, and a fruit that lands still is a
/// worse failure than a fruit that lands late.
const THROW_TRIES: u32 = 8;

impl Throw {
    const fn new(document: DocumentId, body: (u64, u64), velocity: Vec3) -> Self {
        Self {
            document,
            body,
            velocity,
            tries: THROW_TRIES,
        }
    }

    /// Gives the fruit its momentum, reporting whether it still has to be
    /// tried again.
    pub fn apply(&mut self) -> bool {
        self.tries = self.tries.saturating_sub(1);
        let sent: anyhow::Result<()> = reopen(self.document).and_then(|document| {
            set_velocity(&document, self.body, Some(self.velocity), None)?;
            Ok(())
        });
        if sent.is_ok() {
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

fn reopen(id: DocumentId) -> anyhow::Result<Document> {
    open_document(id)?.ok_or_else(|| anyhow::anyhow!("the fruit's document is no longer loaded"))
}

const fn at(translation: Vec3) -> Transform {
    Transform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
}

fn icon(doc: &Document, shape: Shape) -> anyhow::Result<(u64, u64)> {
    let target = reopen(doc.id())?;
    Ok(match shape {
        Shape::Round => {
            let sphere = Sphere::new(ICON);
            sphere.set_doc(target);
            sphere.mesh()?
        }
        Shape::Long => {
            let capsule = Capsule::new(ICON * 0.6, ICON);
            capsule.set_doc(target);
            capsule.mesh()?
        }
        Shape::Cube => {
            let cuboid = Cuboid::new(Vec3::splat(ICON * 1.6));
            cuboid.set_doc(target);
            cuboid.mesh()?
        }
    })
}

/// Lit rather than lit-by-the-room: a mote hangs in mid-air, where there is
/// nothing to bounce light off.
fn dressed(document: &Document, prim: (u64, u64), color: Color) -> anyhow::Result<()> {
    document
        .local()
        .set(
            prim,
            Property::Material(Material {
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
            }),
        )
        .flush()?;
    Ok(())
}

pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}
