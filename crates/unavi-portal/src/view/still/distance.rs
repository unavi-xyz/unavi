//! The distance atlas: for every pixel of a still face, how far the surface it
//! shows is from the face's eye. `portal.wgsl` ray-marches it for parallax.
//!
//! A still camera renders its depth into a texture, then this pass converts
//! that depth to a linear distance and writes it into the same tile of a
//! second atlas, which has the colour atlas's layout.

use bevy::{
    asset::uuid_handle,
    core_pipeline::{
        Core3d,
        Core3dSystems,
        FullscreenShader,
    },
    prelude::*,
    render::{
        RenderApp,
        RenderStartup,
        extract_component::{
            ExtractComponent,
            ExtractComponentPlugin,
        },
        render_asset::RenderAssets,
        render_resource::{
            BindGroupEntries,
            BindGroupLayoutDescriptor,
            BindGroupLayoutEntries,
            CachedRenderPipelineId,
            ColorTargetState,
            ColorWrites,
            FragmentState,
            LoadOp,
            Operations,
            PipelineCache,
            RenderPassColorAttachment,
            RenderPassDescriptor,
            RenderPipelineDescriptor,
            ShaderStages,
            StoreOp,
            TextureFormat,
            binding_types::texture_depth_2d,
        },
        renderer::{
            RenderContext,
            ViewQuery,
        },
        texture::GpuImage,
        view::{
            ExtractedView,
            ViewDepthTexture,
        },
    },
};

pub const STILL_DISTANCE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("5a1f3c6e-0b7d-4c1e-9a52-7d8e4f2b6c13");

/// The distance atlas's texel format. 32-bit so distances keep their
/// precision out to the far end of a space; it is read unfiltered.
pub const DISTANCE_FORMAT: TextureFormat = TextureFormat::R32Float;

/// A still face camera, and the distance atlas it writes its tile of.
#[derive(Component, Clone, ExtractComponent)]
pub struct StillFace {
    pub distance: Handle<Image>,
}

/// Adds the distance pass to the render world's 3D schedule.
pub struct StillDistancePlugin;

impl Plugin for StillDistancePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<StillFace>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(
                Core3d,
                write_distance.in_set(Core3dSystems::EarlyPostProcess),
            );
    }
}

#[derive(Resource)]
struct DistancePipeline {
    layout: BindGroupLayoutDescriptor,
    id:     CachedRenderPipelineId,
}

fn init_pipeline(
    mut commands: Commands,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "still_distance_bind_group_layout",
        &BindGroupLayoutEntries::single(ShaderStages::FRAGMENT, texture_depth_2d()),
    );
    let id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("still_distance_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: STILL_DISTANCE_SHADER_HANDLE,
            targets: vec![Some(ColorTargetState {
                format:     DISTANCE_FORMAT,
                blend:      None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(DistancePipeline { layout, id });
}

/// Runs once per still face camera, after its main pass. The view's depth
/// texture is the size of the whole atlas and its viewport sits on the tile,
/// so the pixel the pass reads is the pixel it writes.
fn write_distance(
    view: ViewQuery<(&ExtractedView, &ViewDepthTexture, &StillFace)>,
    pipeline: Res<DistancePipeline>,
    pipeline_cache: Res<PipelineCache>,
    images: Res<RenderAssets<GpuImage>>,
    mut ctx: RenderContext,
) {
    let (extracted_view, depth, face) = view.into_inner();

    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.id) else {
        return;
    };
    let Some(target) = images.get(&face.distance) else {
        return;
    };

    let bind_group = ctx.render_device().create_bind_group(
        "still_distance_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::single(depth.view()),
    );

    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label:                    Some("still_distance"),
            color_attachments:        &[Some(RenderPassColorAttachment {
                view:           &target.texture_view,
                depth_slice:    None,
                resolve_target: None,
                // The other faces' tiles are already written.
                ops:            Operations {
                    load:  LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes:         None,
            occlusion_query_set:      None,
            multiview_mask:           None,
        });

    let tile = extracted_view.viewport;
    pass.set_viewport(
        tile.x as f32,
        tile.y as f32,
        tile.z as f32,
        tile.w as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(tile.x, tile.y, tile.z, tile.w);
    pass.set_pipeline(render_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..3, 0..1);
}
