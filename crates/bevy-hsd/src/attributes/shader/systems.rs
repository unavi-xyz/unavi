use bevy::{
    light::NotShadowCaster,
    pbr::MeshMaterial3d,
    platform::collections::HashSet,
    prelude::*,
};
use hsd::attributes::shader::{
    self,
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    overrides::{
        GraphOverridesAttr,
        validate_overrides,
    },
    value::GraphValue,
};

use crate::{
    attributes::{
        image::HsdImage,
        material::source::MaterialSource,
        shader::{
            ShaderGraphData,
            ShaderOverridesData,
            cache::ShaderGraphCache,
            material::{
                GraphParams,
                ShaderGraphMaterial,
            },
        },
    },
    prim::{
        HsdRelationships,
        PrimOf,
        PrimRelations,
    },
};

/// The cache hash a prim's material was last built from, so an overrides-only
/// edit can look up the graph's public inputs without recompiling.
#[derive(Component, Clone, Copy)]
pub struct BuiltFromGraph(blake3::Hash);

/// Writes changed overrides straight into the existing material's uniform
/// block, skipping decode, validation and codegen entirely.
pub fn apply_graph_overrides(
    changed: Query<
        (
            &MeshMaterial3d<ShaderGraphMaterial>,
            &BuiltFromGraph,
            Option<&ShaderOverridesData>,
        ),
        Changed<ShaderOverridesData>,
    >,
    cache: Res<ShaderGraphCache>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
) {
    for (material, built, overrides) in &changed {
        let Some(cached) = cache.get(built.0) else {
            continue;
        };
        let params = build_params_from(&cached.public_inputs, overrides.map(|o| &o.0));
        let Some(mut asset) = materials.get_mut(&material.0) else {
            continue;
        };
        // `get_mut` marks the asset changed regardless of whether this write
        // changes it, and a changed material re-uploads its whole bind group.
        if asset.params != params {
            asset.params = params;
        }
    }
}

pub fn rebuild_shader_material(
    changed: Query<
        Entity,
        Or<(
            Changed<ShaderGraphData>,
            Changed<HsdRelationships>,
            Changed<MaterialSource>,
        )>,
    >,
    changed_slots: Query<Entity, Changed<ShaderGraphData>>,
    changed_images: Query<Entity, Changed<HsdImage>>,
    sources: Query<&MaterialSource>,
    slots: Query<&ShaderGraphData>,
    overrides: Query<&ShaderOverridesData>,
    doc_of: Query<&PrimOf>,
    relations: PrimRelations,
    images: Query<&HsdImage>,
    mut cache: ResMut<ShaderGraphCache>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
    mut existing: Query<&mut MeshMaterial3d<ShaderGraphMaterial>>,
    mut commands: Commands,
) {
    let mut dirty: HashSet<Entity> = changed.iter().collect();
    mark_binders_of_changed_graphs(&mut dirty, &changed_slots, &sources, &relations);
    mark_graphs_with_changed_textures(&mut dirty, &changed_images, &sources, &relations);

    for prim in dirty {
        // The graph may live on another prim. `material/binding` names that
        // prim, and a bound prim renders the target's graph with its own
        // overrides.
        let Ok(&MaterialSource::Graph(source)) = sources.get(prim) else {
            continue;
        };
        let Ok(slot) = slots.get(source) else {
            continue;
        };
        let Ok(&PrimOf(doc)) = doc_of.get(prim) else {
            continue;
        };

        let Some(cached) = cache.get_or_build(slot.hash, &slot.bytes, doc, &mut shaders) else {
            continue;
        };

        let overrides_attr = overrides.get(prim).ok().map(|o| &o.0).filter(|o| {
            match validate_overrides(&cached.public_inputs, o) {
                Ok(()) => true,
                Err(err) => {
                    warn!(
                        ?err,
                        "shader graph overrides do not match the graph; using its defaults"
                    );
                    false
                }
            }
        });

        let params = build_params_from(&cached.public_inputs, overrides_attr);
        let [texture_0, texture_1, texture_2, texture_3] =
            resolve_textures(prim, &relations, &images);

        let material = ShaderGraphMaterial {
            params,
            texture_0,
            texture_1,
            texture_2,
            texture_3,
            fragment_shader: cached.fragment.clone(),
            vertex_shader: cached.vertex.clone(),
            alpha_mode: cached.alpha_mode,
            cull_mode: cached.cull_mode,
            reads_scene: cached.reads_scene,
            transmissive: cached.transmissive,
        };

        commands.entity(prim).insert(BuiltFromGraph(slot.hash));
        if cached.cast_shadows {
            commands.entity(prim).remove::<NotShadowCaster>();
        } else {
            commands.entity(prim).insert(NotShadowCaster);
        }

        if let Ok(mut existing) = existing.get_mut(prim) {
            if let Some(mut asset) = materials.get_mut(&existing.0) {
                *asset = material;
            } else {
                existing.0 = materials.add(material);
            }
        } else {
            let handle = materials.add(material);
            commands.entity(prim).insert(MeshMaterial3d(handle));
        }
    }
}

/// A prim bound to a source whose graph slot just changed renders that
/// source's program, and must rebuild with it.
fn mark_binders_of_changed_graphs(
    dirty: &mut HashSet<Entity>,
    changed_slots: &Query<Entity, Changed<ShaderGraphData>>,
    sources: &Query<&MaterialSource>,
    relations: &PrimRelations,
) {
    for slot_ent in changed_slots {
        for binder in relations.sources(slot_ent) {
            if sources.get(binder).ok() == Some(&MaterialSource::Graph(slot_ent)) {
                dirty.insert(binder);
            }
        }
    }
}

/// A graph prim naming a texture prim whose `HsdImage` just changed rebuilds
/// too, since the handle it samples moved.
fn mark_graphs_with_changed_textures(
    dirty: &mut HashSet<Entity>,
    changed_images: &Query<Entity, Changed<HsdImage>>,
    sources: &Query<&MaterialSource>,
    relations: &PrimRelations,
) {
    for img_ent in changed_images {
        for prim in relations.sources(img_ent) {
            if sources
                .get(prim)
                .is_ok_and(|src| matches!(src, MaterialSource::Graph(_)))
            {
                dirty.insert(prim);
            }
        }
    }
}

/// Resolves up to [`MAX_TEXTURE_SAMPLES`] fixed texture slots by relationship.
/// The referenced image prim's own pipeline already loads the image. This
/// only reads the resulting handle.
fn resolve_textures(
    prim: Entity,
    relations: &PrimRelations,
    images: &Query<&HsdImage>,
) -> [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] {
    let mut out: [Option<Handle<Image>>; MAX_TEXTURE_SAMPLES] = Default::default();

    for (slot, handle) in out.iter_mut().enumerate() {
        let Some(name) = shader::texture(slot as u8) else {
            continue;
        };
        *handle = relations
            .target(prim, &name)
            .and_then(|ent| images.get(ent).ok())
            .map(|img| img.0.clone());
    }

    out
}

const fn pack_input(value: GraphValue) -> Vec4 {
    match value {
        GraphValue::Float(v) => Vec4::new(v, 0.0, 0.0, 0.0),
        GraphValue::Vec2([x, y]) => Vec4::new(x, y, 0.0, 0.0),
        GraphValue::Vec3([x, y, z]) => Vec4::new(x, y, z, 0.0),
        GraphValue::Color([r, g, b, a]) => Vec4::new(r, g, b, a),
    }
}

fn build_params_from(
    defaults: &[GraphValue],
    overrides: Option<&GraphOverridesAttr>,
) -> GraphParams {
    let mut inputs = [Vec4::ZERO; MAX_PUBLIC_INPUTS];
    for (index, default) in defaults.iter().enumerate().take(MAX_PUBLIC_INPUTS) {
        let value = overrides
            .and_then(|o| o.overrides.get(&(index as u16)))
            .filter(|v| v.kind() == default.kind())
            .copied()
            .unwrap_or(*default);
        inputs[index] = pack_input(value);
    }
    GraphParams { inputs }
}
