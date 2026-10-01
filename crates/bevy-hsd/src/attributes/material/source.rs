use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    platform::collections::HashSet,
    prelude::*,
};
use hsd::attributes::material;

use crate::{
    attributes::{
        material::{
            MaterialData,
            pbr::HsdMaterial,
        },
        shader::{
            ShaderGraphData,
            material::ShaderGraphMaterial,
        },
    },
    prim::{
        HsdRelationships,
        PrimRelations,
    },
};

/// Which material backend renders a prim. The entity holds the definition,
/// either the prim itself or its `material/binding` target.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialSource {
    /// The source's graph, with this prim's own overrides.
    Graph(Entity),
    /// The source's `StandardMaterial`, shared as is.
    Pbr(Entity),
}

#[derive(SystemParam)]
pub struct SourceCtx<'w, 's> {
    relations: PrimRelations<'w, 's>,
    graphs:    Query<'w, 's, (), With<ShaderGraphData>>,
    materials: Query<'w, 's, (), With<MaterialData>>,
}

impl SourceCtx<'_, '_> {
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

    /// A prim's own definition wins over anything it binds to. A graph wins
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
            Changed<ShaderGraphData>,
            Changed<MaterialData>,
            Changed<HsdRelationships>,
        )>,
    >,
    mut removed_graph: RemovedComponents<ShaderGraphData>,
    mut removed_material: RemovedComponents<MaterialData>,
    existing: Query<&MaterialSource>,
    ctx: SourceCtx,
    mut commands: Commands,
) {
    let mut dirty: HashSet<Entity> = changed.iter().collect();
    dirty.extend(removed_graph.read());
    dirty.extend(removed_material.read());
    if dirty.is_empty() {
        return;
    }

    let seeds: Vec<Entity> = dirty.iter().copied().collect();
    for prim in seeds {
        dirty.extend(ctx.relations.sources(prim));
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
                    .remove::<MeshMaterial3d<ShaderGraphMaterial>>();
            }
            None => {
                entity.remove::<MaterialSource>().remove::<(
                    HsdMaterial,
                    MeshMaterial3d<StandardMaterial>,
                    MeshMaterial3d<ShaderGraphMaterial>,
                )>();
            }
        }
    }
}
