use std::time::Duration;

use bevy::{
    asset::load_internal_asset,
    camera::visibility::VisibilitySystems,
    prelude::*,
};
use blake3::Hash;
use loro::TreeID;

use crate::material::{
    PORTAL_SHADER_HANDLE,
    PortalMaterial,
};

pub mod bridge;
pub mod discovery;
pub mod material;
pub mod render_budget;
pub mod resolver;
pub mod teleport;
pub mod tracking;
pub mod visuals;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PORTAL_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/portal.wgsl"),
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PortalMaterial>::default())
            .init_resource::<PortalRenderBudget>()
            .add_observer(bridge::sync_portal_config)
            .add_observer(bridge::clear_portal_config)
            .add_observer(discovery::on_hsd_ready)
            .add_systems(
                Update,
                (
                    material::update_portal_time,
                    resolver::resolve_target_doc,
                    resolver::resolve_target_receptor,
                    visuals::ensure_portal_mesh,
                    visuals::update_portal_state,
                    render_budget::select_active_portals,
                    visuals::apply_active_material,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    (
                        tracking::update_portal_image_sizes,
                        tracking::update_portal_camera_transforms,
                    )
                        .chain()
                        .after(TransformSystems::Propagate)
                        .before(VisibilitySystems::UpdateFrusta),
                    teleport::handle_traveler_teleport.after(TransformSystems::Propagate),
                    tracking::update_portal_camera_frustums.after(VisibilitySystems::UpdateFrusta),
                ),
            );
    }
}

#[derive(Component, Default)]
#[require(PortalState, PortalSize, PortalAllowIncoming)]
pub struct Portal;

#[derive(Component, Clone, Copy)]
pub struct PortalSize {
    pub width:  f32,
    pub height: f32,
}

impl Default for PortalSize {
    fn default() -> Self {
        Self {
            width:  1.0,
            height: 1.0,
        }
    }
}

pub const PORTAL_DEPTH: f32 = 0.05;

#[derive(Component, Default, Clone, Copy)]
pub struct PortalAllowIncoming(pub bool);

#[derive(Component, Clone, Copy)]
pub struct PortalTargetDoc(pub Hash);

#[derive(Component, Clone)]
pub struct PortalTargetReceptor {
    pub document: Hash,
    pub prim:     TreeID,
}

#[derive(Component, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PortalState {
    #[default]
    Closed,
    Loading,
    Open,
}

#[derive(Component, Default)]
#[relationship_target(relationship = PortalDestination)]
pub struct IncomingPortals(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = IncomingPortals)]
pub struct PortalDestination(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = PortalCamera)]
pub struct PortalCameras(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = PortalCameras)]
#[require(Transform)]
pub struct PortalCamera {
    pub portal: Entity,
}

#[derive(Component)]
pub struct TrackedCamera(pub Entity);

#[derive(Component)]
pub struct PortalActiveRender;

#[derive(Resource)]
pub struct PortalRenderBudget {
    pub max_active:   usize,
    pub max_distance: f32,
}

impl Default for PortalRenderBudget {
    fn default() -> Self {
        Self {
            max_active:   8,
            max_distance: 64.0,
        }
    }
}

#[derive(Component)]
#[require(TravelCooldown, PrevTranslation)]
pub struct PortalTraveler;

#[derive(Component)]
pub struct TravelCooldown {
    pub last_travel: Option<Duration>,
    pub duration:    Duration,
}

impl Default for TravelCooldown {
    fn default() -> Self {
        Self {
            last_travel: None,
            duration:    Duration::from_millis(100),
        }
    }
}

#[derive(Component, Default)]
pub struct PrevTranslation(pub Vec3);
