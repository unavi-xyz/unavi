//! Decides which backend renders a prim, so exactly one does.
//!
//! The format's `material/binding` names another prim, not a backend, and this
//! crate has two backends for it. Without a single decision point a prim can
//! carry both material components and render twice.

use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::attributes::material;

use crate::{
    HsdRelationships,
    attributes::{
        material::{
            HsdMaterial,
            MaterialData,
        },
        relations::PrimRelations,
        shader::{
            HsdMaterialGraphSlot,
            HsdShaderGraphMaterial,
            ShaderGraphMaterial,
        },
    },
};

/// Which backend renders a prim, and whose definition it uses.
///
/// The inner entity is the prim the definition comes from — itself, or the
/// target of its `material/binding`. A bound prim follows whatever its target
/// resolved to, so a graph and a PBR material are interchangeable.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialSource {
    /// Renders the source prim's compiled graph, parameterized by this prim's
    /// own overrides; a graph binding shares a program, not a finished look.
    Graph(Entity),
    /// Shares the source prim's built `StandardMaterial` outright;
    /// `MaterialAttr` has no per-instance parameters.
    Pbr(Entity),
}

#[derive(SystemParam)]
pub struct SourceCtx<'w, 's> {
    relations: PrimRelations<'w, 's>,
    graphs:    Query<'w, 's, (), With<HsdMaterialGraphSlot>>,
    materials: Query<'w, 's, (), With<MaterialData>>,
}

impl SourceCtx<'_, '_> {
    /// The prim a `material/binding` names, if it resolves within the same
    /// document.
    fn binding_target(&self, prim: Entity) -> Option<Entity> {
        let target = self.relations.target(prim, &material::BINDING)?;
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

    // A prim bound to a changed one re-resolves too: its material is whatever
    // its target just became.
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
