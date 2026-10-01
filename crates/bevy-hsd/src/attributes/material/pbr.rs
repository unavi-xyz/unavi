use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::{
    attributes::material::{
        self,
        MaterialAttr,
    },
    property::name::PropName,
};

use crate::{
    attributes::{
        image::HsdImage,
        material::{
            MaterialData,
            source::MaterialSource,
        },
        values::{
            clamped,
            from_color_vec,
        },
    },
    prim::{
        HsdRelationships,
        PrimRelations,
    },
};

const METALLIC_DEFAULT: f32 = 0.5;
const ROUGHNESS_DEFAULT: f32 = 0.5;
const ALPHA_CUTOFF_DEFAULT: f32 = 0.5;

/// A PBR material this prim owns. A bound prim has only the shared
/// `MeshMaterial3d`.
#[derive(Component, Clone)]
pub struct HsdMaterial(pub Handle<StandardMaterial>);

#[derive(SystemParam)]
pub(crate) struct MaterialCtx<'w, 's> {
    pub relations: PrimRelations<'w, 's>,
    pub images:    Query<'w, 's, &'static HsdImage>,
    pub materials: Query<'w, 's, &'static HsdMaterial>,
    pub sources:   Query<'w, 's, &'static MaterialSource>,
}

pub(crate) fn rebuild_material(
    changed: Query<
        (Entity, Option<&MaterialData>),
        Or<(
            Changed<MaterialData>,
            Changed<HsdRelationships>,
            Changed<MaterialSource>,
        )>,
    >,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (ent, data) in &changed {
        build(ent, data.map(|d| &d.0), &ctx, &mut assets, &mut commands);
    }
}

pub(crate) fn propagate_image_to_material(
    changed: Query<Entity, Changed<HsdImage>>,
    dependents: Query<&MaterialData>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for img_ent in &changed {
        for mat_ent in ctx.relations.sources(img_ent) {
            if let Ok(data) = dependents.get(mat_ent) {
                build(mat_ent, Some(&data.0), &ctx, &mut assets, &mut commands);
            }
        }
    }
}

/// `HsdMaterial` changes only when its handle is replaced, not when its asset
/// is mutated in place.
pub(crate) fn propagate_material_to_dependents(
    changed: Query<Entity, Changed<HsdMaterial>>,
    dependents: Query<Option<&MaterialData>>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for src_ent in &changed {
        for dep_ent in ctx.relations.sources(src_ent) {
            if dep_ent == src_ent {
                continue;
            }
            if ctx.sources.get(dep_ent).ok() != Some(&MaterialSource::Pbr(src_ent)) {
                continue;
            }
            let data = dependents.get(dep_ent).ok().flatten();
            build(
                dep_ent,
                data.map(|d| &d.0),
                &ctx,
                &mut assets,
                &mut commands,
            );
        }
    }
}

fn build(
    ent: Entity,
    attr: Option<&MaterialAttr>,
    ctx: &MaterialCtx,
    assets: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    let Ok(&MaterialSource::Pbr(source)) = ctx.sources.get(ent) else {
        return;
    };

    if source != ent {
        let Ok(target_mat) = ctx.materials.get(source) else {
            return;
        };
        commands
            .entity(ent)
            .insert(MeshMaterial3d(target_mat.0.clone()))
            .remove::<HsdMaterial>();
        return;
    }

    let Some(attr) = attr else {
        return;
    };

    let mut standard = StandardMaterial::default();
    apply_attr(&mut standard, attr, ent, &ctx.relations, &ctx.images);

    // Mutated in place so bound prims keep the same handle.
    if let Ok(existing) = ctx.materials.get(ent) {
        if let Some(mut asset) = assets.get_mut(&existing.0) {
            *asset = standard;
        }
        return;
    }

    let handle = assets.add(standard);
    commands
        .entity(ent)
        .insert((HsdMaterial(handle.clone()), MeshMaterial3d(handle)));
}

fn apply_attr(
    standard: &mut StandardMaterial,
    attr: &MaterialAttr,
    ent: Entity,
    relations: &PrimRelations,
    images: &Query<&HsdImage>,
) {
    standard.base_color = from_color_vec(attr.base_color.as_ref(), Color::WHITE);

    standard.alpha_mode = match attr.alpha_mode {
        Some(material::AlphaMode::Add) => AlphaMode::Add,
        Some(material::AlphaMode::Blend) => AlphaMode::Blend,
        Some(material::AlphaMode::Mask) => {
            AlphaMode::Mask(clamped(attr.alpha_cutoff, ALPHA_CUTOFF_DEFAULT, 0.0..=1.0))
        }
        Some(material::AlphaMode::Multiply) => AlphaMode::Multiply,
        Some(material::AlphaMode::Opaque) => AlphaMode::Opaque,
        Some(material::AlphaMode::Premultiplied) => AlphaMode::Premultiplied,
        None => {
            if standard.base_color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
    };

    standard.double_sided = attr.double_sided.unwrap_or_default();
    standard.emissive = from_color_vec(attr.emissive.as_ref(), Color::BLACK).into();
    standard.metallic = clamped(attr.metallic, METALLIC_DEFAULT, 0.0..=1.0);
    standard.perceptual_roughness = clamped(attr.roughness, ROUGHNESS_DEFAULT, 0.0..=1.0);

    standard.base_color_texture = texture(relations, ent, &material::BASE_COLOR_TEXTURE, images);
    standard.emissive_texture = texture(relations, ent, &material::EMISSIVE_TEXTURE, images);
    standard.metallic_roughness_texture = texture(
        relations,
        ent,
        &material::METALLIC_ROUGHNESS_TEXTURE,
        images,
    );
    standard.normal_map_texture = texture(relations, ent, &material::NORMAL_TEXTURE, images);
    standard.occlusion_texture = texture(relations, ent, &material::OCCLUSION_TEXTURE, images);
}

fn texture(
    relations: &PrimRelations,
    ent: Entity,
    name: &PropName,
    images: &Query<&HsdImage>,
) -> Option<Handle<Image>> {
    relations
        .target(ent, name)
        .and_then(|e| images.get(e).ok())
        .map(|i| i.0.clone())
}
