use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::attributes::{
    Attribute,
    material::{
        self,
        ColorVec,
        MaterialAttr,
    },
};

use crate::{
    HsdChild,
    HsdPrimIndex,
    HsdRelationships,
    attributes::{
        AttributeParser,
        ParseError,
        image::HsdImage,
    },
};

const METALLIC_DEFAULT: f32 = 0.5;
const ROUGHNESS_DEFAULT: f32 = 0.5;

#[derive(Component, Default, Clone)]
pub struct HsdMaterial(pub Handle<StandardMaterial>);

#[derive(Component, Clone)]
pub struct MaterialData(pub MaterialAttr);

#[derive(Component, Default, Debug, Clone)]
pub struct MaterialTextureRefs {
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

pub struct MaterialParser;

impl AttributeParser for MaterialParser {
    fn key(&self) -> &'static str {
        MaterialAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands.entity(prim).insert((
                    MaterialData(MaterialAttr::decode(payload)?),
                    HsdMaterial::default(),
                    MeshMaterial3d::<StandardMaterial>::default(),
                ));
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
}

#[derive(SystemParam)]
pub struct MaterialCtx<'w, 's> {
    pub children:      Query<'w, 's, &'static HsdChild>,
    pub indices:       Query<'w, 's, &'static HsdPrimIndex>,
    pub relationships: Query<'w, 's, &'static HsdRelationships>,
    pub images:        Query<'w, 's, &'static HsdImage>,
    pub materials:     Query<'w, 's, &'static HsdMaterial>,
}

/// Rebuilds on either the inline definition or the relationships changing: a
/// prim can carry both `material/` and `material:binding/`, and which wins is
/// this renderer's rule, not the format's.
pub fn rebuild_material(
    changed: Query<
        (Entity, Option<&MaterialData>),
        Or<(Changed<MaterialData>, Changed<HsdRelationships>)>,
    >,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (ent, data) in &changed {
        build(ent, data.map(|d| &d.0), &ctx, &mut assets, &mut commands);
    }
}

pub fn propagate_image_to_material(
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

pub fn propagate_material_to_dependents(
    changed: Query<Entity, Changed<HsdMaterial>>,
    dependents: Query<(Entity, &HsdRelationships, Option<&MaterialData>, &HsdChild)>,
    indices: Query<&HsdPrimIndex>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for src_ent in &changed {
        for (dep_ent, rels, data, doc_child) in &dependents {
            let Some(target) = rels.0.get(material::BINDING) else {
                continue;
            };
            let Ok(index) = indices.get(doc_child.0) else {
                continue;
            };
            if index.0.get(target) == Some(&src_ent) && dep_ent != src_ent {
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
    let Ok(doc_child) = ctx.children.get(ent) else {
        warn!("material prim has no HsdChild");
        return;
    };
    let Ok(index) = ctx.indices.get(doc_child.0) else {
        warn!("doc has no HsdPrimIndex");
        return;
    };

    let rels = ctx.relationships.get(ent).ok();

    if let Some(rels) = rels
        && let Some(target) = rels.0.get(material::BINDING)
        && let Some(&target_ent) = index.0.get(target)
        && target_ent != ent
        && let Ok(target_mat) = ctx.materials.get(target_ent)
    {
        let handle = target_mat.0.clone();
        commands.entity(ent).insert((
            HsdMaterial(handle.clone()),
            MeshMaterial3d(handle),
            MaterialTextureRefs::default(),
        ));
        return;
    }

    let Some(attr) = attr else {
        return;
    };

    let texture = |name: &str| {
        rels.and_then(|rels| rels.0.get(name))
            .and_then(|target| index.0.get(target).copied())
    };

    let texture_refs = MaterialTextureRefs {
        base_color:         texture(material::BASE_COLOR_TEXTURE),
        emissive:           texture(material::EMISSIVE_TEXTURE),
        metallic_roughness: texture(material::METALLIC_ROUGHNESS_TEXTURE),
        normal:             texture(material::NORMAL_TEXTURE),
        occlusion:          texture(material::OCCLUSION_TEXTURE),
    };

    let mut standard = StandardMaterial::default();
    apply_attr(&mut standard, attr, &texture_refs, &ctx.images);

    let handle = assets.add(standard);

    commands.entity(ent).insert((
        HsdMaterial(handle.clone()),
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
    standard.base_color = color_from_color_vec(attr.base_color.as_ref()).unwrap_or(Color::WHITE);

    standard.alpha_mode = match attr.alpha_mode.as_deref() {
        Some("Add") => AlphaMode::Add,
        Some("Blend") => AlphaMode::Blend,
        Some("Mask") => AlphaMode::Mask(attr.alpha_cutoff.unwrap_or(0.5) as f32),
        Some("Multiply") => AlphaMode::Multiply,
        Some("Opaque") => AlphaMode::Opaque,
        Some("Premultiplied") => AlphaMode::Premultiplied,
        _ => {
            if standard.base_color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
    };

    standard.double_sided = attr.double_sided.unwrap_or_default();
    standard.emissive =
        color_from_color_vec(attr.emissive.as_ref()).map_or(LinearRgba::BLACK, LinearRgba::from);
    standard.metallic = attr.metallic.map_or(METALLIC_DEFAULT, |v| v as f32);
    standard.perceptual_roughness = attr.roughness.map_or(ROUGHNESS_DEFAULT, |v| v as f32);

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

fn color_from_color_vec(vec: Option<&ColorVec>) -> Option<Color> {
    match vec?.0.as_slice() {
        [r, g, b, a] => Some(Color::linear_rgba(
            *r as f32, *g as f32, *b as f32, *a as f32,
        )),
        [r, g, b] => Some(Color::linear_rgb(*r as f32, *g as f32, *b as f32)),
        _ => None,
    }
}

/// A prim can receive a material purely by relationship, with no inline
/// definition of its own, so the slots have to exist before
/// `propagate_material_to_dependents` can fill them.
pub fn prepare_bound_material(
    changed: Query<(Entity, Option<&MaterialData>, &HsdRelationships), Changed<HsdRelationships>>,
    mut commands: Commands,
) {
    for (ent, data, rels) in &changed {
        if rels.0.contains_key(material::BINDING) {
            commands.entity(ent).insert((
                HsdMaterial::default(),
                MeshMaterial3d::<StandardMaterial>::default(),
            ));
        } else if data.is_none() {
            commands
                .entity(ent)
                .remove::<HsdMaterial>()
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
    }
}
