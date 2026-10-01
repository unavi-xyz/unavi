use bevy::{
    asset::uuid_handle,
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        Face,
        ShaderType,
        SpecializedMeshPipelineError,
    },
};

use crate::{
    Seam,
    SeamSize,
};

pub const SEAM_SHADER_HANDLE: Handle<Shader> = uuid_handle!("339faa2e-314e-45fc-b310-34b31639fcd7");

#[derive(Asset, AsBindGroup, Clone, TypePath)]
#[bind_group_data(SeamMaterialKey)]
pub struct SeamMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture:   Option<Handle<Image>>,
    pub cull_mode: Option<Face>,
    #[uniform(2)]
    pub params:    SeamParams,
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct SeamParams {
    pub world_from_seam: Mat4,
    pub half_size:       Vec2,
}

impl Material for SeamMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        SEAM_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        SEAM_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SeamMaterialKey {
    cull_mode: Option<Face>,
}

impl From<&SeamMaterial> for SeamMaterialKey {
    fn from(material: &SeamMaterial) -> Self {
        Self {
            cull_mode: material.cull_mode,
        }
    }
}

pub fn update_seam_params(
    seams: Query<(&GlobalTransform, &SeamSize, &MeshMaterial3d<SeamMaterial>), With<Seam>>,
    mut materials: ResMut<Assets<SeamMaterial>>,
) {
    for (transform, size, handle) in &seams {
        let next = SeamParams {
            world_from_seam: transform.to_matrix(),
            half_size:       Vec2::new(size.width / 2.0, size.height / 2.0),
        };
        // Skip the no-op `get_mut`: it flags the asset as modified and forces a
        // GPU re-upload every frame.
        if materials
            .get(handle.0.id())
            .is_none_or(|m| m.params == next)
        {
            continue;
        }
        if let Some(mut material) = materials.get_mut(handle.0.id()) {
            material.params = next;
        }
    }
}
