use std::time::Duration;

use bevy::{
    asset::load_internal_asset,
    camera::visibility::VisibilitySystems,
    prelude::*,
};
use blake3::Hash;
use loro::TreeID;

use crate::material::{
    SEAM_SHADER_HANDLE,
    SeamMaterial,
};

pub mod develop;
pub mod horizon;
pub mod material;
pub mod resolver;
pub mod transition;
pub mod visuals;

pub struct ManifoldPlugin;

impl Plugin for ManifoldPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SEAM_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/seam.wgsl"),
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<SeamMaterial>::default())
            .init_resource::<DevelopmentHorizon>()
            .add_systems(
                Update,
                (
                    material::update_seam_time,
                    resolver::resolve_target_doc,
                    resolver::resolve_target_receptor,
                    visuals::ensure_seam_mesh,
                    visuals::update_seam_state,
                    horizon::select_developed_seams,
                    visuals::apply_active_material,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    (
                        develop::update_develop_image_sizes,
                        develop::update_develop_camera_transforms,
                        develop::update_develop_camera_clip_planes,
                    )
                        .chain()
                        .after(TransformSystems::Propagate)
                        .before(VisibilitySystems::UpdateFrusta),
                    transition::apply_seam_crossings.after(TransformSystems::Propagate),
                ),
            )
            .add_observer(transition::carry_momentum);
    }
}

#[derive(Component, Default)]
#[require(SeamState, SeamSize)]
pub struct Seam;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct SeamSize {
    pub width:  f32,
    pub height: f32,
}

impl Default for SeamSize {
    fn default() -> Self {
        Self {
            width:  1.0,
            height: 1.0,
        }
    }
}

pub const SEAM_DEPTH: f32 = 0.05;

#[derive(Component, Clone, Copy)]
pub struct SeamTargetDoc(pub Hash);

#[derive(Component, Clone, PartialEq, Eq)]
pub struct SeamTargetReceptor {
    pub document: Hash,
    pub prim:     TreeID,
}

#[derive(Component, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SeamState {
    #[default]
    Closed,
    Loading,
    Open,
}

#[derive(Component, Default)]
#[relationship_target(relationship = GluedTo)]
pub struct GluedFrom(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = GluedFrom)]
pub struct GluedTo(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = DevelopCamera)]
pub struct DevelopCameras(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = DevelopCameras)]
#[require(Transform)]
pub struct DevelopCamera {
    pub seam: Entity,
}

#[derive(Component)]
pub struct TrackedCamera(pub Entity);

#[derive(Component)]
pub struct SeamActiveRender;

/// Marker for the camera whose position drives seam render-budget selection.
#[derive(Component)]
pub struct ManifoldViewer;

#[derive(Resource)]
pub struct DevelopmentHorizon {
    pub max_active:   usize,
    pub max_distance: f32,
}

impl Default for DevelopmentHorizon {
    fn default() -> Self {
        Self {
            max_active:   8,
            max_distance: 64.0,
        }
    }
}

#[derive(Component)]
#[require(TransitionCooldown, PrevTranslation)]
pub struct ManifoldBody;

#[derive(Component)]
pub struct TransitionCooldown {
    pub last_travel: Option<Duration>,
    pub duration:    Duration,
}

impl Default for TransitionCooldown {
    fn default() -> Self {
        Self {
            last_travel: None,
            duration:    Duration::from_millis(100),
        }
    }
}

#[derive(Component, Default)]
pub struct PrevTranslation(pub Vec3);
