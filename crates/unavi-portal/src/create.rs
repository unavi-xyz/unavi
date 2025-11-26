use bevy::{
    camera::{Exposure, RenderTarget, visibility::RenderLayers},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::ColorGrading,
    },
    window::{PrimaryWindow, WindowRef},
};
use unavi_constants::PORTAL_RENDER_LAYER;

use crate::{
    Portal, PortalBounds, PortalCamera, PortalDestination, TrackedCamera, material::PortalMaterial,
};

pub struct CreatePortal {
    pub destination: Option<Entity>,
    pub tracked_camera: Option<Entity>,
    pub height: f32,
    pub width: f32,
    pub depth: f32,
}

impl Default for CreatePortal {
    fn default() -> Self {
        Self {
            destination: None,
            tracked_camera: None,
            height: 1.0,
            width: 1.0,
            depth: 0.4,
        }
    }
}

impl EntityCommand for CreatePortal {
    #[allow(clippy::too_many_lines)]
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(dest) = self.destination {
            entity.insert(PortalDestination(dest));
        }

        let id = entity.id();

        entity.world_scope(|world| {
            let tracked_camera_ent = self.tracked_camera.unwrap_or_else(|| {
                world
                    .query_filtered::<Entity, With<Camera3d>>()
                    .iter(world)
                    .next()
                    .expect("no camera found")
            });

            let primary_window_size = world
                .query_filtered::<&Window, With<PrimaryWindow>>()
                .single(world)
                .ok()
                .map(Window::physical_size);

            let camera = world
                .get::<Camera>(tracked_camera_ent)
                .expect("camera component not found");

            let viewport_size = camera
                .viewport
                .as_ref()
                .map_or_else(
                    || match &camera.target {
                        RenderTarget::Image(image) => world
                            .resource::<Assets<Image>>()
                            .get(image.handle.id())
                            .map(Image::size),
                        RenderTarget::None { size } => Some(*size),
                        RenderTarget::TextureView(view) => world
                            .resource::<ManualTextureViews>()
                            .get(view)
                            .map(|v| v.size),
                        RenderTarget::Window(window) => match window {
                            WindowRef::Primary => primary_window_size,
                            WindowRef::Entity(window_ent) => {
                                world.get::<Window>(*window_ent).map(Window::physical_size)
                            }
                        },
                    },
                    |v| Some(v.physical_size),
                )
                .unwrap_or_else(|| UVec2::splat(128));

            let size = Extent3d {
                width: viewport_size.x,
                height: viewport_size.y,
                ..default()
            };

            let mut image = Image {
                texture_descriptor: TextureDescriptor {
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    label: Some("PortalImage"),
                    mip_level_count: 1,
                    sample_count: 1,
                    size,
                    usage: TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT
                        | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                ..default()
            };
            image.resize(size);
            let image_handle = world.resource_mut::<Assets<Image>>().add(image);

            let material = PortalMaterial {
                texture: Some(image_handle.clone()),
                cull_mode: None,
            };
            let material_handle = world.resource_mut::<Assets<PortalMaterial>>().add(material);

            let mesh = Plane3d::default()
                .mesh()
                .normal(Dir3::Z)
                .size(self.width, self.height)
                .build();
            let mesh_handle = world.resource_mut::<Assets<Mesh>>().add(mesh);

            let portal_ent = world
                .entity_mut(id)
                .insert((
                    Portal,
                    PortalBounds {
                        depth: self.depth,
                        height: self.height,
                        width: self.width,
                    },
                    RenderLayers::layer(PORTAL_RENDER_LAYER),
                    MeshMaterial3d(material_handle),
                    Mesh3d(mesh_handle),
                ))
                .id();

            let camera_3d = world
                .get::<Camera3d>(tracked_camera_ent)
                .cloned()
                .unwrap_or_default();

            let portal_camera_ent = world
                .spawn((
                    PortalCamera { portal: portal_ent },
                    TrackedCamera(tracked_camera_ent),
                    Camera {
                        order: -1,
                        target: RenderTarget::Image(image_handle.into()),
                        ..default()
                    },
                    camera_3d,
                ))
                .id();

            if let Some(value) = world.get::<ColorGrading>(tracked_camera_ent).cloned() {
                world.entity_mut(portal_camera_ent).insert(value);
            }
            if let Some(value) = world.get::<DebandDither>(tracked_camera_ent).copied() {
                world.entity_mut(portal_camera_ent).insert(value);
            }
            if let Some(value) = world.get::<Exposure>(tracked_camera_ent).copied() {
                world.entity_mut(portal_camera_ent).insert(value);
            }
            if let Some(value) = world.get::<Projection>(tracked_camera_ent).cloned() {
                world.entity_mut(portal_camera_ent).insert(value);
            }
            if let Some(value) = world.get::<Tonemapping>(tracked_camera_ent).copied() {
                world.entity_mut(portal_camera_ent).insert(value);
            }
        });
    }
}
