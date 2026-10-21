//! Drops a cube on whatever the player is pointing at.
//!
//! A cube is a body, so it is a document of its own. In a space the user can
//! write it is kept: a reference site in the space document places it,
//! committed through an undoable edit. Every kept cube of one colour in one
//! space transcludes the same template document, and each site still opens its
//! own instance with its own body. Anywhere else a cube is minted and placed
//! directly, and lasts while someone present holds it.

use std::cell::RefCell;

use wired_guest::math::{
    Color,
    Ray,
    Transform,
    Vec3,
};

use crate::{
    palette,
    unavi::{
        author::api::Edit,
        shapes::api::Cuboid,
    },
    wired::{
        core::ids::DocumentId,
        physics::simulation::{
            RayFilter,
            raycast,
        },
        portal::portals::current_space,
        scene::{
            document::{
                Anchor,
                Document,
                Layer,
                create_document,
                open_document,
                script_document,
            },
            properties::{
                Property,
                RigidBody,
            },
        },
    },
};

const SIZE: f32 = 0.3;
const HALF: f32 = SIZE * 0.5;

/// Furthest ahead a cube lands when the ray finds nothing.
const REACH: f32 = 6.0;
/// Nearest it can land, so pointing at a wall an arm's length away does not
/// drop one inside the player.
const MIN_DIST: f32 = 1.2;

/// The template cube documents kept cubes transclude, one per colour and
/// space, minted on first use in a run. A minted document belongs to the
/// space it was minted in, so another space gets its own.
#[derive(Default)]
pub struct Templates(RefCell<Vec<(DocumentId, Color, DocumentId)>>);

impl Templates {
    fn get_or_build(&self, space: DocumentId, color: Color) -> anyhow::Result<DocumentId> {
        let known = self
            .0
            .borrow()
            .iter()
            .find(|(s, c, _)| *s == space && *c == color)
            .map(|(_, _, id)| *id);
        if let Some(id) = known
            && open_document(id)?.is_some()
        {
            return Ok(id);
        }
        let id = build(color, Transform::IDENTITY)?.id();
        let mut templates = self.0.borrow_mut();
        templates.retain(|(s, c, _)| *s != space || *c != color);
        templates.push((space, color, id));
        Ok(id)
    }
}

/// Puts a cube of `color` where `aim` points.
pub fn spawn(templates: &Templates, color: Color, aim: &Transform) -> anyhow::Result<()> {
    let pose = Transform {
        translation: landing(aim),
        rotation:    aim.rotation,
        scale:       Vec3::ONE,
    };
    writable_space()?.map_or_else(
        || drop_loose(color, &pose),
        |space| keep(templates, color, &pose, &space),
    )
}

/// The space the user stands in, when they may write it.
fn writable_space() -> anyhow::Result<Option<Document>> {
    let Some(id) = current_space()? else {
        return Ok(None);
    };
    Ok(open_document(id)?.filter(Document::may_write))
}

/// Keeps a cube of `color` in `space` under a reference site.
fn keep(
    templates: &Templates,
    color: Color,
    pose: &Transform,
    space: &Document,
) -> anyhow::Result<()> {
    let cube = templates.get_or_build(space.id(), color)?;

    let to_space = script_document()?
        .offset_to(space)
        .ok_or_else(|| anyhow::anyhow!("the space is not placed"))?
        .inverse();
    let site = space.create_prim(Layer::Local, None)?;
    space
        .local()
        .set(site, Property::Name("cube".to_owned()))
        .set(site, Property::Reference(cube))
        .set(site, Property::Transform(to_space * *pose))
        .flush()?;

    let edit = Edit::new("Spawn cube")?;
    edit.track_creation(space, site)?;
    edit.commit()?;
    Ok(())
}

/// Places the cube directly, where it lasts while someone present pins it.
fn drop_loose(color: Color, pose: &Transform) -> anyhow::Result<()> {
    let doc = create_document()?;
    // Place before writing: a prim transform is stated in its document's
    // frame, and `pose` is measured in the spawner's. The minted document's
    // frame must exist before the two can be compared.
    doc.place(Anchor::Space, Transform::IDENTITY)?;
    let to_world = script_document()?
        .world_frame()
        .unwrap_or(Transform::IDENTITY);
    fill(&doc, color, to_world * *pose)
}

/// Mints a cube document of `color` with its body at `at`, unplaced.
fn build(color: Color, at: Transform) -> anyhow::Result<Document> {
    let doc = create_document()?;
    fill(&doc, color, at)?;
    Ok(doc)
}

/// Writes the cube's mesh, collider, body and material into `doc` and makes
/// them durable.
fn fill(doc: &Document, color: Color, at: Transform) -> anyhow::Result<()> {
    // `set-doc` takes the document by value; a fresh handle is spent on it
    // (rather than `doc` itself) so the mesh lands in the minted document
    // and `doc` stays usable for the edits and commit below. Same pattern as
    // `example-unavi-vui/fruit.rs`.
    let cuboid = Cuboid::new(Vec3::splat(SIZE));
    cuboid.set_doc(reopen(doc.id())?);
    let cube = cuboid.mesh()?;

    doc.local()
        .set(cube, Property::Collider(cuboid.collider()))
        .set(cube, Property::RigidBody(RigidBody::dynamic()))
        .set(cube, Property::Material(palette::cube(color)))
        .set(cube, Property::Transform(at))
        .flush()?;

    // `local` never leaves this peer, so the cube reaches others only once
    // committed. Committing every key, `parent` included, makes the locally
    // created prims durable along with the mesh `Cuboid::mesh` wrote.
    let keys: Vec<_> = doc
        .prims()
        .into_iter()
        .flat_map(|prim| doc.keys(prim).into_iter().map(move |key| (prim, key)))
        .collect();
    doc.commit(&keys)?;
    Ok(())
}

/// Opens a fresh handle onto `id`, for an API that takes a document by
/// value while the caller keeps using its own handle afterward.
fn reopen(id: DocumentId) -> anyhow::Result<Document> {
    open_document(id)?.ok_or_else(|| anyhow::anyhow!("the minted document is no longer loaded"))
}

/// Where a cube goes: resting against whatever `aim` points at, out to
/// [`REACH`] when nothing is, and never nearer than [`MIN_DIST`].
///
/// A miss is not a failure — it means open air, which is a legitimate place to
/// put something — so it lands at arm's length rather than not at all.
fn landing(aim: &Transform) -> Vec3 {
    let dir = aim.forward();

    let at = match raycast(
        Ray {
            origin:    aim.translation,
            direction: dir,
        },
        REACH,
        &RayFilter {
            exclude_local_agent: true,
            exclude_documents:   Vec::new(),
        },
    ) {
        Ok(Some(hit)) => hit.point + hit.normal * HALF,
        Ok(None) => aim.translation + dir * REACH,
        Err(err) => {
            eprintln!("spawner: raycast failed, placing ahead instead: {err:?}");
            aim.translation + dir * REACH
        }
    };

    if (at - aim.translation).length() < MIN_DIST {
        return aim.translation + dir * MIN_DIST;
    }
    at
}
