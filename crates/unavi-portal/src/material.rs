use bevy::{
    asset::uuid_handle,
    prelude::*,
    render::render_resource::{AsBindGroup, Face, SpecializedMeshPipelineError},
};

pub const PORTAL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("339faa2e-314e-45fc-b310-34b31639fcd7");

#[derive(Asset, AsBindGroup, Clone, TypePath)]
#[bind_group_data(PortalMaterialKey)]
pub struct PortalMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    pub cull_mode: Option<Face>,
}

impl Material for PortalMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        PORTAL_SHADER_HANDLE.into()
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
pub struct PortalMaterialKey {
    cull_mode: Option<Face>,
}

impl From<&PortalMaterial> for PortalMaterialKey {
    fn from(material: &PortalMaterial) -> Self {
        Self {
            cull_mode: material.cull_mode,
        }
    }
}
