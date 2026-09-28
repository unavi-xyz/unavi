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
    property::{
        Payload,
        Property,
        name::PropName,
    },
};

use crate::{
    HsdRelationships,
    attributes::{
        ParseError,
        color::from_color_vec,
        image::HsdImage,
        material_source::MaterialSource,
        relations::PrimRelations,
    },
};

const METALLIC_DEFAULT: f32 = 0.5;
const ROUGHNESS_DEFAULT: f32 = 0.5;
const ALPHA_CUTOFF_DEFAULT: f32 = 0.5;

/// A prim's built PBR material.
///
/// The second field is `true` when this prim minted the handle and may
/// mutate the asset in place; `false` when it shares another prim's handle
/// by a `material/binding` and must never write through it.
#[derive(Component, Clone)]
pub struct HsdMaterial(pub Handle<StandardMaterial>, bool);

#[derive(Component, Clone)]
pub(crate) struct MaterialData(pub MaterialAttr);

#[derive(Component, Default, Debug, Clone)]
pub(crate) struct MaterialTextureRefs {
    pub base_color:         Option<Entity>,
    pub emissive:           Option<Entity>,
    pub metallic_roughness: Option<Entity>,
    pub normal:             Option<Entity>,
    pub occlusion:          Option<Entity>,
}

impl MaterialTextureRefs {
    #[must_use]
    pub fn references(&self, ent: Entity) -> bool {
        [
            self.base_color,
            self.emissive,
            self.metallic_roughness,
            self.normal,
            self.occlusion,
        ]
        .into_iter()
        .any(|e| e == Some(ent))
    }
}

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    if *name != MaterialAttr::NAME {
        return Ok(());
    }
    match payload {
        // Only the decoded definition: whether this prim renders as PBR at
        // all is `MaterialSource`'s call, and `build` inserts the render
        // components once it is. Inserting them here would give a
        // graph-backed prim a competing material for a frame.
        Some(payload) => {
            commands
                .entity(prim)
                .insert(MaterialData(MaterialAttr::decode(payload)?));
        }
        None => {
            commands
                .entity(prim)
                .remove::<HsdMaterial>()
                .remove::<MaterialData>()
                .remove::<MaterialTextureRefs>()
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
    }
    Ok(())
}

#[derive(SystemParam)]
pub(crate) struct MaterialCtx<'w, 's> {
    pub relations: PrimRelations<'w, 's>,
    pub images:    Query<'w, 's, &'static HsdImage>,
    pub materials: Query<'w, 's, &'static HsdMaterial>,
    pub sources:   Query<'w, 's, &'static MaterialSource>,
}

/// Rebuilds on the definition, the relationships, or the resolved source.
///
/// [`MaterialSource`] is in that list because a binding only becomes a PBR
/// material once the resolver has decided the target is not a shader graph.
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
    dependents: Query<(Entity, &MaterialTextureRefs, &MaterialData)>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for img_ent in &changed {
        for (mat_ent, refs, data) in &dependents {
            if refs.references(img_ent) {
                build(mat_ent, Some(&data.0), &ctx, &mut assets, &mut commands);
            }
        }
    }
}

/// A prim bound to one whose `StandardMaterial` handle just changed picks it
/// up.
///
/// `HsdMaterial` only changes on this signal when its owner mints or
/// replaces the handle: an in-place asset mutation never re-inserts it, so a
/// stable binding is never re-copied for nothing.
pub(crate) fn propagate_material_to_dependents(
    changed: Query<Entity, Changed<HsdMaterial>>,
    dependents: Query<(Entity, &MaterialSource, Option<&MaterialData>)>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for src_ent in &changed {
        for (dep_ent, source, data) in &dependents {
            if *source == MaterialSource::Pbr(src_ent) && dep_ent != src_ent {
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
}

fn build(
    ent: Entity,
    attr: Option<&MaterialAttr>,
    ctx: &MaterialCtx,
    assets: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    // `MaterialSource` decides the backend; a prim resolved to a shader graph
    // is not built here and must not get a competing `MeshMaterial3d`.
    let Ok(&MaterialSource::Pbr(source)) = ctx.sources.get(ent) else {
        return;
    };

    if source != ent {
        let Ok(target_mat) = ctx.materials.get(source) else {
            return;
        };
        let handle = target_mat.0.clone();
        commands.entity(ent).insert((
            HsdMaterial(handle.clone(), false),
            MeshMaterial3d(handle),
            MaterialTextureRefs::default(),
        ));
        return;
    }

    let Some(attr) = attr else {
        return;
    };

    let texture = |name: &PropName| ctx.relations.target(ent, name);
    let texture_refs = MaterialTextureRefs {
        base_color:         texture(&material::BASE_COLOR_TEXTURE),
        emissive:           texture(&material::EMISSIVE_TEXTURE),
        metallic_roughness: texture(&material::METALLIC_ROUGHNESS_TEXTURE),
        normal:             texture(&material::NORMAL_TEXTURE),
        occlusion:          texture(&material::OCCLUSION_TEXTURE),
    };

    let mut standard = StandardMaterial::default();
    apply_attr(&mut standard, attr, &texture_refs, &ctx.images);

    // An owned handle is mutated in place: every dependent already holding it
    // sees the new definition without the handle itself ever changing.
    if let Some(existing) = ctx.materials.get(ent).ok().filter(|m| m.1) {
        if let Some(mut asset) = assets.get_mut(&existing.0) {
            *asset = standard;
        }
        commands.entity(ent).insert(texture_refs);
        return;
    }

    let handle = assets.add(standard);
    commands.entity(ent).insert((
        HsdMaterial(handle.clone(), true),
        MeshMaterial3d(handle),
        texture_refs,
    ));
}

fn apply_attr(
    standard: &mut StandardMaterial,
    attr: &MaterialAttr,
    refs: &MaterialTextureRefs,
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

    standard.base_color_texture = refs.base_color.and_then(|e| handle_for(images, e));
    standard.emissive_texture = refs.emissive.and_then(|e| handle_for(images, e));
    standard.metallic_roughness_texture =
        refs.metallic_roughness.and_then(|e| handle_for(images, e));
    standard.normal_map_texture = refs.normal.and_then(|e| handle_for(images, e));
    standard.occlusion_texture = refs.occlusion.and_then(|e| handle_for(images, e));
}

fn handle_for(images: &Query<&HsdImage>, ent: Entity) -> Option<Handle<Image>> {
    images.get(ent).ok().map(|i| i.0.clone())
}

/// A factor a peer wrote, held to `range`. A document may carry any double at
/// all, and a non-finite one would reach the material as a `NaN` uniform.
fn clamped(value: Option<f64>, fallback: f32, range: std::ops::RangeInclusive<f32>) -> f32 {
    value
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(*range.start(), *range.end())
}
