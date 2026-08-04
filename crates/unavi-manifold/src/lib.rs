use bevy::{
    app::AnimationSystems,
    asset::load_internal_asset,
    camera::visibility::VisibilitySystems,
    math::Affine3A,
    prelude::*,
};
use hsd::id::{
    DocId,
    PrimId,
};

use crate::{
    clip::{
        ClippedMtoonMaterial,
        ClippedStandardMaterial,
        SEAM_CLIP_MTOON_SHADER_HANDLE,
        SEAM_CLIP_SHADER_HANDLE,
        SEAM_CLIP_STANDARD_SHADER_HANDLE,
    },
    material::{
        SEAM_SHADER_HANDLE,
        SeamMaterial,
    },
};

pub mod clip;
pub mod develop;
pub mod echo;
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
        load_internal_asset!(
            app,
            SEAM_CLIP_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/seam_clip.wgsl"),
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SEAM_CLIP_STANDARD_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/seam_clip_standard.wgsl"
            ),
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SEAM_CLIP_MTOON_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/seam_clip_mtoon.wgsl"),
            Shader::from_wgsl
        );

        app.add_plugins((
            MaterialPlugin::<SeamMaterial>::default(),
            MaterialPlugin::<ClippedStandardMaterial>::default(),
            MaterialPlugin::<ClippedMtoonMaterial>::default(),
        ))
        .init_resource::<DevelopmentHorizon>()
        .add_systems(
            Update,
            (
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
                    visuals::update_develop_camera_layers,
                )
                    .chain()
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::UpdateFrusta),
                (
                    transition::apply_seam_crossings,
                    echo::maintain_seam_echoes,
                    echo::sync_echo_nodes,
                )
                    .chain()
                    .after(AnimationSystems)
                    .before(TransformSystems::Propagate),
                material::update_seam_params.after(TransformSystems::Propagate),
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

/// Depth of the latch slab around the seam plane; suppresses an immediate
/// re-crossing after a body lands near the destination plane.
pub const SEAM_DEPTH: f32 = 0.05;

#[derive(Component, Clone, Copy)]
pub struct SeamTargetDoc(pub DocId);

#[derive(Component, Clone, PartialEq, Eq)]
pub struct SeamTargetReceptor {
    pub document: DocId,
    pub prim:     PrimId,
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

/// Mirrored stand-in on the far side of a seam, spawned while its body
/// straddles the plane so a half-inserted object protrudes from both faces.
#[derive(Component, Clone, Copy)]
pub struct SeamEcho {
    pub body: Entity,
    pub seam: Entity,
}

/// Node of an echo subtree, cloned from `source` in the body's hierarchy.
#[derive(Component, Clone, Copy)]
pub struct EchoNode {
    pub source: Entity,
}

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
#[require(SeamLatch, PrevTranslation, EchoBody)]
pub struct ManifoldBody;

/// Casts seam echoes but is never locally teleported across a seam. Every
/// [`ManifoldBody`] is one; bodies whose crossings are driven externally, such
/// as network-replicated avatars, carry it alone.
#[derive(Component, Default)]
pub struct EchoBody;

/// Set after a crossing, cleared once the body leaves every slab; stops it
/// teleporting straight back out of the slab it lands in.
#[derive(Component, Default)]
pub struct SeamLatch(pub bool);

#[derive(Component, Default)]
pub struct PrevTranslation(pub Vec3);

/// Affine map carrying poses through a seam: into the source seam's space,
/// through the half-turn aligning the two faces, then out at the destination.
#[must_use]
pub fn seam_transfer(source: &GlobalTransform, destination: &GlobalTransform) -> Affine3A {
    let flip = Affine3A::from_quat(Quat::from_rotation_y(std::f32::consts::PI));
    destination.affine() * flip * source.affine().inverse()
}
