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

#[derive(Clone, Copy, ShaderType, Debug, Default)]
pub struct SeamParams {
    pub time: f32,
}

impl Material for SeamMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SEAM_SHADER_HANDLE.into()
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

pub fn update_seam_time(time: Res<Time>, mut materials: ResMut<Assets<SeamMaterial>>) {
    let t = time.elapsed_secs();

    for (_, mat) in materials.iter_mut() {
        mat.params.time = t;
    }
}
