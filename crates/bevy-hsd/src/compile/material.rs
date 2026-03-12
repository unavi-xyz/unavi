use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};

use crate::{CompiledMaterial, data::HsdMaterial};

pub fn parse_material_data(
    mut commands: Commands,
    materials: Query<(Entity, &HsdMaterial), Changed<HsdMaterial>>,
) {
    for (ent, mat) in &materials {
        #[expect(clippy::cast_possible_truncation)]
        let base_color = mat.base_color.as_ref().and_then(|c| {
            if c.len() >= 3 {
                Some(Color::srgb(c[0] as f32, c[1] as f32, c[2] as f32))
            } else {
                None
            }
        });

        #[expect(clippy::cast_possible_truncation)]
        let metallic = mat.metallic.map(|v| v as f32);
        #[expect(clippy::cast_possible_truncation)]
        let roughness = mat.roughness.map(|v| v as f32);

        let base_color_texture = mat.base_color_texture.map(|hash| {
            commands
                .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                .id()
        });

        let metallic_roughness_texture = mat.metallic_roughness_texture.map(|hash| {
            commands
                .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                .id()
        });

        let normal_texture = mat.normal_texture.map(|hash| {
            commands
                .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                .id()
        });

        let occlusion_texture = mat.occlusion_texture.map(|hash| {
            commands
                .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                .id()
        });

        commands.entity(ent).insert(MaterialParams {
            base_color,
            double_sided: mat.double_sided,
            metallic,
            roughness,
            base_color_texture,
            _metallic_roughness_texture: metallic_roughness_texture,
            _normal_texture: normal_texture,
            _occlusion_texture: occlusion_texture,
        });
    }
}

#[derive(Component)]
#[require(BlobDeps)]
pub struct MaterialParams {
    base_color: Option<Color>,
    double_sided: Option<bool>,
    metallic: Option<f32>,
    roughness: Option<f32>,
    base_color_texture: Option<Entity>,
    _metallic_roughness_texture: Option<Entity>,
    _normal_texture: Option<Entity>,
    _occlusion_texture: Option<Entity>,
}

pub fn compile_materials(
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loaded: Query<(Entity, &MaterialParams, Option<&CompiledMaterial>), Added<BlobDepsLoaded>>,
    mut blobs: Query<&mut BlobResponse>,
) {
    for (ent, params, existing) in &loaded {
        let mut material = StandardMaterial::default();

        if let Some(value) = params.base_color {
            material.base_color = value;
        }
        if let Some(value) = params.double_sided {
            material.double_sided = value;
        }
        if let Some(value) = params.metallic {
            material.metallic = value;
        }
        if let Some(value) = params.roughness {
            material.perceptual_roughness = value;
        }

        if let Some(value) = params.base_color_texture {
            let Ok(Some(_bytes)) = blobs.get_mut(value).map(|mut b| b.0.take()) else {
                continue;
            };
            // TODO: load image details from HSD
            let image = Image::default();
            let handle = asset_server.add(image);
            material.base_color_texture = Some(handle);
        }

        // Update existing asset in-place to preserve handles in referencing nodes.
        if let Some(CompiledMaterial(handle)) = existing
            && let Some(asset) = mat_assets.get_mut(handle)
        {
            *asset = material;
            commands
                .entity(ent)
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
            continue;
        }

        let handle = asset_server.add(material);
        commands
            .entity(ent)
            .insert(CompiledMaterial(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}
