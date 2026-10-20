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
    portal::{
        Portal,
        PortalSize,
    },
    view::{
        hue::PortalHue,
        ring::PortalRing,
        visual::PortalVisual,
    },
};

pub const PORTAL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("339faa2e-314e-45fc-b310-34b31639fcd7");

/// Every visible portal's surface.
///
/// `still` is the arrival still's face atlas, laid out by
/// [`crate::view::still::face_tile`], and `still_distance` the same layout
/// holding each texel's distance from its face's eye.
#[derive(Asset, AsBindGroup, Clone, TypePath, Default)]
#[bind_group_data(PortalMaterialKey)]
pub struct PortalMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub live:           Option<Handle<Image>>,
    #[uniform(2)]
    pub params:         PortalParams,
    #[texture(3)]
    #[sampler(4)]
    pub still:          Option<Handle<Image>>,
    #[texture(5, sample_type = "float", filterable = false)]
    pub still_distance: Option<Handle<Image>>,
    pub cull_mode:      Option<Face>,
}

/// Field order matches `PortalParams` in `portal.wgsl`.
#[derive(Clone, Copy, ShaderType, Debug, Default, PartialEq)]
pub struct PortalParams {
    pub world_from_portal: Mat4,
    /// Linear RGB. Alpha is unused.
    pub rim_color:         Vec4,
    /// Plane-local centre in metres, start time in shader-time seconds, and
    /// 1 while a ring is set.
    pub ring:              Vec4,
    pub half_size:         Vec2,
    pub mode:              u32,
    /// 1 when `still_distance` is bound, so the still can be ray-marched.
    pub parallax:          u32,
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
            &PortalVisual,
            Option<&PortalHue>,
            Option<&PortalRing>,
            &MeshMaterial3d<PortalMaterial>,
        ),
        With<Portal>,
    >,
    mut materials: ResMut<Assets<PortalMaterial>>,
) {
    for (transform, size, visual, hue, ring, handle) in &portals {
        let Some(current) = materials.get(handle.0.id()) else {
            continue;
        };
        let next = PortalParams {
            world_from_portal: transform.to_matrix(),
            rim_color:         hue.map_or(PortalHue::NEUTRAL, |h| h.0).to_vec4(),
            ring:              ring.map_or(Vec4::ZERO, |r| r.point.extend(r.start).extend(1.0)),
            half_size:         Vec2::new(size.width / 2.0, size.height / 2.0),
            mode:              visual.mode(),
            parallax:          u32::from(current.still_distance.is_some()),
        };
        // Skip the no-op `get_mut`: it flags the asset as modified and forces a
        // GPU re-upload every frame.
        if current.params == next {
            continue;
        }
        if let Some(mut material) = materials.get_mut(handle.0.id()) {
            material.params = next;
        }
    }
}
