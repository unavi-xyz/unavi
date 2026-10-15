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

use crate::portal::{
    Portal,
    PortalSize,
};

pub const PORTAL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("339faa2e-314e-45fc-b310-34b31639fcd7");

#[derive(Asset, AsBindGroup, Clone, TypePath)]
#[bind_group_data(PortalMaterialKey)]
pub struct PortalMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture:   Option<Handle<Image>>,
    pub cull_mode: Option<Face>,
    #[uniform(2)]
    pub params:    PortalParams,
}

#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct PortalParams {
    pub world_from_portal: Mat4,
    pub half_size:         Vec2,
}

impl Material for PortalMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        PORTAL_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        PORTAL_SHADER_HANDLE.into()
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

pub fn update_portal_params(
    portals: Query<
        (
            &GlobalTransform,
            &PortalSize,
            &MeshMaterial3d<PortalMaterial>,
        ),
        With<Portal>,
    >,
    mut materials: ResMut<Assets<PortalMaterial>>,
) {
    for (transform, size, handle) in &portals {
        let next = PortalParams {
            world_from_portal: transform.to_matrix(),
            half_size:         Vec2::new(size.width / 2.0, size.height / 2.0),
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

/// Shared fallback materials for a portal's loading/open-but-not-live states,
/// created once at startup instead of a fresh [`StandardMaterial`] asset on
/// every state change.
#[derive(Resource, Clone)]
pub struct PortalFallbacks {
    pub loading: Handle<StandardMaterial>,
    pub open:    Handle<StandardMaterial>,
}

const LOADING_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const OPEN_FALLBACK_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

impl FromWorld for PortalFallbacks {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            loading: materials.add(fallback_material(LOADING_COLOR)),
            open:    materials.add(fallback_material(OPEN_FALLBACK_COLOR)),
        }
    }
}

fn fallback_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}
