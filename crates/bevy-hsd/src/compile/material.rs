use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};

use crate::stage::Attrs;

pub fn parse_material_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if attrs.material_base_color.is_some()
        || attrs.material_base_color_texture.is_some()
        || attrs.material_double_sided.is_some()
        || attrs.material_metallic.is_some()
        || attrs.material_metallic_roughness_texture.is_some()
        || attrs.material_normal_texture.is_some()
        || attrs.material_occlusion_texture.is_some()
        || attrs.material_roughness.is_some()
    {
        let base_color = attrs.material_base_color.map(Color::srgb_from_array);
        let double_sided = attrs.material_double_sided;
        let metallic = attrs.material_metallic;
        let roughness = attrs.material_roughness;

        let base_color_texture = attrs.material_base_color_texture.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });
        let metallic_roughness_texture = attrs.material_metallic_roughness_texture.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });
        let normal_texture = attrs.material_normal_texture.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });
        let occlusion_texture = attrs.material_occlusion_texture.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });

        commands.entity(node).insert(MaterialParams {
            base_color,
            double_sided,
            metallic,
            roughness,
            base_color_texture,
            metallic_roughness_texture,
            normal_texture,
            occlusion_texture,
        });
    } else {
        commands
            .entity(node)
            .remove::<MeshMaterial3d<StandardMaterial>>();
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
    metallic_roughness_texture: Option<Entity>,
    normal_texture: Option<Entity>,
    occlusion_texture: Option<Entity>,
}

pub fn compile_materials(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loaded: Query<(Entity, &MaterialParams), Added<BlobDepsLoaded>>,
    mut blobs: Query<&mut BlobResponse>,
) {
    for (ent, params) in loaded {
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
            let Ok(Some(bytes)) = blobs.get_mut(value).map(|mut b| b.0.take()) else {
                continue;
            };
            let image = Image::default();
            let handle = asset_server.add(image);
            material.base_color_texture = Some(handle);
        }

        let handle = asset_server.add(material);
        commands.entity(ent).insert(MeshMaterial3d(handle));
    }
}
