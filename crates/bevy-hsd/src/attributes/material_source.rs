//! Decides which backend renders a prim, so that exactly one does.
//!
//! The format has one binding concept — USD's `material:binding`, naming
//! another prim rather than a backend — and this crate has two
//! implementations of it. Without a single decision point a prim can end up
//! carrying both `MeshMaterial3d<StandardMaterial>` and
//! `MeshMaterial3d<ShaderGraphMaterial>`, which renders it twice rather than
//! failing.

use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::attributes::material;

use crate::{
    HsdChild,
    HsdPrimIndex,
    HsdRelationships,
    attributes::{
        material::{
            HsdMaterial,
            MaterialData,
        },
        material_graph::{
            HsdMaterialGraphSlot,
            HsdShaderGraphMaterial,
            ShaderGraphMaterial,
        },
    },
};

/// Which backend renders a prim, and whose definition it uses.
///
/// The inner entity is the prim the definition comes from — itself, or the
/// target of its `material:binding`. A bound prim follows whatever its
/// target resolved to, so a graph and a PBR material are interchangeable
/// from the binder's point of view.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialSource {
    /// Renders the source prim's compiled graph, parameterized by *this*
    /// prim's own overrides — a graph binding shares a program, not a
    /// finished look, which is what per-instance public inputs are for.
    Graph(Entity),
    /// Shares the source prim's built `StandardMaterial` outright, since a
    /// `MaterialAttr` has no per-instance parameters to keep local.
    Pbr(Entity),
}

#[derive(SystemParam)]
pub struct SourceCtx<'w, 's> {
    children:      Query<'w, 's, &'static HsdChild>,
    indices:       Query<'w, 's, &'static HsdPrimIndex>,
    relationships: Query<'w, 's, &'static HsdRelationships>,
    graphs:        Query<'w, 's, (), With<HsdMaterialGraphSlot>>,
    materials:     Query<'w, 's, (), With<MaterialData>>,
}

impl SourceCtx<'_, '_> {
    /// The prim a `material:binding` names, if it resolves within the same
    /// document.
    fn binding_target(&self, prim: Entity) -> Option<Entity> {
        let rels = self.relationships.get(prim).ok()?;
        let target = rels.0.get(material::BINDING)?;
        let doc = self.children.get(prim).ok()?.0;
        let target = *self.indices.get(doc).ok()?.0.get(target)?;
        (target != prim).then_some(target)
    }

    fn own_source(&self, prim: Entity) -> Option<MaterialSource> {
        if self.graphs.contains(prim) {
            Some(MaterialSource::Graph(prim))
        } else if self.materials.contains(prim) {
            Some(MaterialSource::Pbr(prim))
        } else {
            None
        }
    }

    /// A prim's own definition wins over anything it binds to; a graph wins
    /// over an inline `MaterialAttr` on the same prim.
    fn resolve(&self, prim: Entity) -> Option<MaterialSource> {
        self.own_source(prim)
            .or_else(|| self.own_source(self.binding_target(prim)?))
    }
}

pub fn resolve_material_source(
    changed: Query<
        Entity,
        Or<(
            Changed<HsdMaterialGraphSlot>,
            Changed<MaterialData>,
            Changed<HsdRelationships>,
        )>,
    >,
    mut removed_graph: RemovedComponents<HsdMaterialGraphSlot>,
    mut removed_material: RemovedComponents<MaterialData>,
    binders: Query<Entity, With<HsdRelationships>>,
    existing: Query<&MaterialSource>,
    ctx: SourceCtx,
    mut commands: Commands,
) {
    let mut dirty = changed.iter().collect::<Vec<_>>();
    dirty.extend(removed_graph.read());
    dirty.extend(removed_material.read());
    if dirty.is_empty() {
        return;
    }

    // A prim bound to one that changed re-resolves too: its material is
    // whatever its target just became.
    let sources = dirty.clone();
    for ent in &binders {
        if !sources.contains(&ent)
            && ctx
                .binding_target(ent)
                .is_some_and(|target| sources.contains(&target))
        {
            dirty.push(ent);
        }
    }

    for prim in dirty {
        let next = ctx.resolve(prim);
        if next == existing.get(prim).ok().copied() {
            continue;
        }

        let mut entity = commands.entity(prim);
        match next {
            Some(source @ MaterialSource::Graph(_)) => {
                entity
                    .insert(source)
                    .remove::<(HsdMaterial, MeshMaterial3d<StandardMaterial>)>();
            }
            Some(source @ MaterialSource::Pbr(_)) => {
                entity
                    .insert(source)
                    .remove::<(HsdShaderGraphMaterial, MeshMaterial3d<ShaderGraphMaterial>)>();
            }
            None => {
                entity.remove::<MaterialSource>().remove::<(
                    HsdMaterial,
                    MeshMaterial3d<StandardMaterial>,
                    HsdShaderGraphMaterial,
                    MeshMaterial3d<ShaderGraphMaterial>,
                )>();
            }
        }
    }
}
