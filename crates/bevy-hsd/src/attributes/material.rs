use bevy::{
    ecs::system::SystemParam,
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        hydrate_attr,
        material::{
            ColorVec,
            MaterialAttr,
        },
    },
};
use loro::{
    ContainerID,
    Index,
    TreeID,
    ValueOrContainer,
    event::Diff,
};

use crate::{
    HsdChild,
    HsdPrimIndex,
    HsdRelationships,
    attributes::{
        ApplyEvent,
        AttrDataEvent,
        AttributeParser,
        DocContext,
        ParseError,
        image::HsdImage,
        util::shallow_map_updated_keys,
    },
    diff::HsdDiffEvent,
};

const METALLIC_DEFAULT: f32 = 0.5;
const ROUGHNESS_DEFAULT: f32 = 0.5;

#[derive(Debug)]
pub enum MaterialEvent {
    Rebuild(MaterialAttr),
}

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
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert((
                HsdMaterial::default(),
                MeshMaterial3d::<StandardMaterial>::default(),
            ));
        } else {
            commands
                .entity(prim)
                .remove::<HsdMaterial>()
                .remove::<MaterialData>()
                .remove::<MaterialTextureRefs>()
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: MaterialAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        if keys.is_empty() {
            return Ok(());
        }

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Material(MaterialEvent::Rebuild(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
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

pub fn apply_material(
    trigger: On<ApplyEvent<MaterialEvent>>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let ent = trigger.entity;
    let MaterialEvent::Rebuild(attr) = &trigger.value;

    commands.entity(ent).insert(MaterialData(attr.clone()));
    rebuild_material(ent, Some(attr), &ctx, &mut assets, &mut commands);
}

pub fn propagate_material_relationship(
    changed: Query<(Entity, Option<&MaterialData>), Changed<HsdRelationships>>,
    ctx: MaterialCtx,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (ent, data) in &changed {
        rebuild_material(ent, data.map(|d| &d.0), &ctx, &mut assets, &mut commands);
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
                rebuild_material(mat_ent, Some(&data.0), &ctx, &mut assets, &mut commands);
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
            let Some(target_tree_id) = rels.0.get(MaterialAttr::KEY) else {
                continue;
            };
            let Ok(index) = indices.get(doc_child.0) else {
                continue;
            };
            if index.0.get(target_tree_id) == Some(&src_ent) && dep_ent != src_ent {
                rebuild_material(
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

fn rebuild_material(
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

    if let Ok(rels) = ctx.relationships.get(ent)
        && let Some(target_tree_id) = rels.0.get(MaterialAttr::KEY)
        && let Some(&target_ent) = index.0.get(target_tree_id)
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

    let texture_refs = MaterialTextureRefs {
        base_color:         lookup_image(attr.base_color_texture.as_ref(), index),
        emissive:           lookup_image(attr.emissive_texture.as_ref(), index),
        metallic_roughness: lookup_image(attr.metallic_roughness_texture.as_ref(), index),
        normal:             lookup_image(attr.normal_texture.as_ref(), index),
        occlusion:          lookup_image(attr.occlusion_texture.as_ref(), index),
    };

    let mut material = StandardMaterial::default();
    apply_attr_to_material(&mut material, attr, &texture_refs, &ctx.images);

    let handle = assets.add(material);

    commands.entity(ent).insert((
        HsdMaterial(handle.clone()),
        MeshMaterial3d(handle),
        texture_refs,
    ));
}

fn lookup_image(field: Option<&String>, index: &HsdPrimIndex) -> Option<Entity> {
    let target = TreeID::try_from(field?.as_str()).ok()?;
    index.0.get(&target).copied()
}

fn apply_attr_to_material(
    material: &mut StandardMaterial,
    attr: &MaterialAttr,
    refs: &MaterialTextureRefs,
    images: &Query<&HsdImage>,
) {
    material.base_color = color_from_color_vec(attr.base_color.as_ref()).unwrap_or(Color::WHITE);

    material.alpha_mode = match attr.alpha_mode.as_deref() {
        Some("Add") => AlphaMode::Add,
        Some("Blend") => AlphaMode::Blend,
        Some("Mask") => AlphaMode::Mask(attr.alpha_cutoff.as_ref().copied().unwrap_or(0.5) as f32),
        Some("Multiply") => AlphaMode::Multiply,
        Some("Opaque") => AlphaMode::Opaque,
        Some("Premultiplied") => AlphaMode::Premultiplied,
        _ => {
            if material.base_color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            }
        }
    };

    material.double_sided = attr.double_sided.as_ref().copied().unwrap_or_default();
    material.emissive =
        color_from_color_vec(attr.emissive.as_ref()).map_or(LinearRgba::BLACK, LinearRgba::from);
    material.metallic = attr
        .metallic
        .as_ref()
        .map_or(METALLIC_DEFAULT, |v| *v as f32);
    material.perceptual_roughness = attr
        .roughness
        .as_ref()
        .map_or(ROUGHNESS_DEFAULT, |v| *v as f32);

    material.base_color_texture = refs.base_color.and_then(|e| handle_for(images, e));
    material.emissive_texture = refs.emissive.and_then(|e| handle_for(images, e));
    material.metallic_roughness_texture =
        refs.metallic_roughness.and_then(|e| handle_for(images, e));
    material.normal_map_texture = refs.normal.and_then(|e| handle_for(images, e));
    material.occlusion_texture = refs.occlusion.and_then(|e| handle_for(images, e));
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
