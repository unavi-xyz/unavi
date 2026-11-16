//! Portals: glued openings between spaces. Bodies cross them, and echoes and
//! live views render through them.

// Bevy's `AsBindGroup` needs higher limit
#![recursion_limit = "256"]

use bevy::{
    app::AnimationSystems,
    asset::load_internal_asset,
    camera::visibility::VisibilitySystems,
    prelude::*,
};

use bevy_shader_mtoon::MtoonPlugin;

use crate::{
    clip::{
        ClippedMtoonMaterial,
        ClippedStandardMaterial,
        PORTAL_CLIP_MTOON_SHADER_HANDLE,
        PORTAL_CLIP_STANDARD_SHADER_HANDLE,
    },
    portal::PortalLimits,
    view::{
        ViewBudget,
        material::{
            PORTAL_SHADER_HANDLE,
            PortalMaterial,
        },
        still::distance::{
            STILL_DISTANCE_SHADER_HANDLE,
            StillDistancePlugin,
        },
    },
};

pub mod arrival;
pub mod body;
pub mod clip;
pub mod config;
pub mod crossing;
pub mod destination;
pub mod echo;
pub mod portal;
pub mod render_layers;
pub mod view;

/// Registers portal rendering, echoing and crossing.
pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        load_shaders(app);

        // A clipped MToon material extends MToon's own shader, which only
        // exists once its plugin registers the library.
        if !app.is_plugin_added::<MtoonPlugin>() {
            app.add_plugins(MtoonPlugin);
        }

        app.add_plugins((
            MaterialPlugin::<PortalMaterial>::default(),
            MaterialPlugin::<ClippedStandardMaterial>::default(),
            MaterialPlugin::<ClippedMtoonMaterial>::default(),
            StillDistancePlugin,
        ))
        .init_resource::<ViewBudget>()
        .init_resource::<PortalLimits>()
        .add_systems(
            Update,
            (
                config::admit_refused_portals,
                destination::resolve_destinations,
                arrival::update_arrivals,
                view::mesh::ensure_mesh,
                view::mesh::update_state,
                view::select::select_viewed_portals,
                view::hue::update_hues,
                view::still::maintain_stills,
                view::visual::choose_visual,
                view::visual::apply_visual,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                (
                    view::camera::update_view_camera_image_sizes,
                    view::camera::update_view_camera_transforms,
                    view::camera::update_view_camera_clip_planes,
                    view::camera::update_view_camera_layers,
                )
                    .chain()
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::UpdateFrusta),
                (
                    portal::update_portal_frames,
                    echo::update_echo_radius,
                    crossing::apply_crossings,
                    echo::maintain_echoes,
                    echo::sync_echo_nodes,
                )
                    .chain()
                    .after(AnimationSystems)
                    .before(TransformSystems::Propagate),
                view::material::update_portal_params.after(TransformSystems::Propagate),
            ),
        )
        .add_observer(crossing::carry_momentum)
        .add_observer(view::ring::start_rings)
        .add_observer(config::sync_portal_config)
        .add_observer(config::clear_portal_config);
    }
}

fn load_shaders(app: &mut App) {
    // The clip helpers are a WESL module other shaders import, so the asset
    // needs the import path its module name comes from.
    let _ = app
        .world_mut()
        .resource_mut::<Assets<Shader>>()
        .insert(
            clip::PORTAL_CLIP_SHADER_HANDLE.id(),
            Shader::from_wesl(
                include_str!("portal_clip.wesl"),
                "embedded://unavi_portal/portal_clip.wesl",
            ),
        );

    load_internal_asset!(
        app,
        PORTAL_SHADER_HANDLE,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/portal.wesl"),
        Shader::from_wesl
    );
    load_internal_asset!(
        app,
        STILL_DISTANCE_SHADER_HANDLE,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/still_distance.wesl"),
        Shader::from_wesl
    );
    load_internal_asset!(
        app,
        PORTAL_CLIP_STANDARD_SHADER_HANDLE,
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/portal_clip_standard.wesl"
        ),
        Shader::from_wesl
    );
    load_internal_asset!(
        app,
        PORTAL_CLIP_MTOON_SHADER_HANDLE,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/portal_clip_mtoon.wesl"),
        Shader::from_wesl
    );
}
