use std::marker::PhantomData;

use bevy::{
    asset::uuid_handle,
    pbr::{
        ExtendedMaterial,
        MaterialExtension,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use bevy_vrm::mtoon::MtoonMaterial;

pub const SEAM_CLIP_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("9a4065b6-b09f-4091-ae05-2630fcec10bb");
pub const SEAM_CLIP_STANDARD_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("4946577f-5b0d-480a-80fa-7a57ad6d8d3b");
pub const SEAM_CLIP_MTOON_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("8ef2d136-e292-4b14-9add-c773eeef0073");

/// Base material that can be clipped at a world-space seam plane.
pub trait Clippable: Material {
    /// Fragment shader reproducing this material's shading with the seam clip,
    /// binding the clip plane at `@binding(100)` (see `assets/seam_clip.wgsl`).
    const CLIP_SHADER: Handle<Shader>;

    /// Render both faces, so the cap exposed at the cut is drawn.
    fn make_double_sided(&mut self);
}

impl Clippable for StandardMaterial {
    const CLIP_SHADER: Handle<Shader> = SEAM_CLIP_STANDARD_SHADER_HANDLE;

    fn make_double_sided(&mut self) {
        self.cull_mode = None;
        self.double_sided = true;
    }
}

impl Clippable for MtoonMaterial {
    const CLIP_SHADER: Handle<Shader> = SEAM_CLIP_MTOON_SHADER_HANDLE;

    fn make_double_sided(&mut self) {
        self.double_sided = true;
    }
}

/// Material extension discarding fragments behind a world-space plane, so a
/// mesh straddling a seam does not protrude out its back side.
#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct SeamClip<M: Clippable> {
    #[uniform(100)]
    pub plane: Vec4,
    _base:     PhantomData<fn() -> M>,
}

impl<M: Clippable> MaterialExtension for SeamClip<M> {
    fn fragment_shader() -> ShaderRef {
        M::CLIP_SHADER.into()
    }
}

pub type Clipped<M> = ExtendedMaterial<M, SeamClip<M>>;
pub type ClippedStandardMaterial = Clipped<StandardMaterial>;
pub type ClippedMtoonMaterial = Clipped<MtoonMaterial>;

/// World-space clip plane keeping the half-space where
/// `dot(n, p) + w >= 0`.
#[must_use]
pub fn clip_plane(plane_transform: &GlobalTransform, side: f32) -> Vec4 {
    let normal = plane_transform
        .affine()
        .transform_vector3(Vec3::Z)
        .normalize_or_zero()
        * side;
    normal.extend(-normal.dot(plane_transform.translation()))
}

/// Clipped variant of a material, rendered double-sided so the shader can shade
/// exposed back faces as a flat cap.
#[must_use]
pub fn clipped_variant<M: Clippable>(mut base: M, plane: Vec4) -> Clipped<M> {
    base.make_double_sided();
    ExtendedMaterial {
        base,
        extension: SeamClip {
            plane,
            _base: PhantomData,
        },
    }
}

/// Original material of a mesh node whose material is temporarily swapped for
/// a clipped variant while its body straddles a seam.
#[derive(Component)]
pub struct UnclippedMaterial<M: Material>(pub Handle<M>);

/// Marker on a body whose subtree materials are currently clipped, recording
/// the seam it straddles.
#[derive(Component)]
pub struct ClippedBody {
    pub seam:  Entity,
    pub plane: Vec4,
}

pub fn subtree(world: &World, root: Entity) -> Vec<Entity> {
    let mut nodes = vec![root];
    let mut i = 0;
    while i < nodes.len() {
        if let Some(children) = world.get::<Children>(nodes[i]) {
            nodes.extend(children.iter());
        }
        i += 1;
    }
    nodes
}

/// Swap a node's `M` material for a clipped variant, stashing the original.
fn clip_node<M: Clippable>(world: &mut World, node: Entity, plane: Vec4) {
    let Some(handle) = world.get::<MeshMaterial3d<M>>(node).map(|m| m.0.clone()) else {
        return;
    };
    let Some(base) = world.resource::<Assets<M>>().get(handle.id()).cloned() else {
        return;
    };
    let clipped = world
        .resource_mut::<Assets<Clipped<M>>>()
        .add(clipped_variant(base, plane));
    world
        .entity_mut(node)
        .insert((MeshMaterial3d(clipped), UnclippedMaterial(handle)))
        .remove::<MeshMaterial3d<M>>();
}

/// Restore a node's stashed `M` material.
fn unclip_node<M: Clippable>(world: &mut World, node: Entity) {
    let Some(original) = world.get::<UnclippedMaterial<M>>(node).map(|m| m.0.clone()) else {
        return;
    };
    world
        .entity_mut(node)
        .insert(MeshMaterial3d(original))
        .remove::<(MeshMaterial3d<Clipped<M>>, UnclippedMaterial<M>)>();
}

/// Repoint a node's clipped `M` material at a new plane.
fn update_node<M: Clippable>(world: &mut World, node: Entity, plane: Vec4) {
    let Some(id) = world
        .get::<MeshMaterial3d<Clipped<M>>>(node)
        .map(|m| m.0.id())
    else {
        return;
    };
    if let Some(mut material) = world.resource_mut::<Assets<Clipped<M>>>().get_mut(id) {
        material.extension.plane = plane;
    }
}

/// Clones `source`'s `M` material onto `clone` as a clipped variant, resolving
/// to the unclipped handle whether the source is live or straddling.
#[must_use]
pub fn clone_clipped_node<M: Clippable>(
    world: &mut World,
    source: Entity,
    clone: Entity,
    plane: Vec4,
) -> bool {
    let Some(handle) = world
        .get::<MeshMaterial3d<M>>(source)
        .map(|m| m.0.clone())
        .or_else(|| {
            world
                .get::<UnclippedMaterial<M>>(source)
                .map(|m| m.0.clone())
        })
    else {
        return false;
    };
    let Some(base) = world.resource::<Assets<M>>().get(handle.id()).cloned() else {
        return false;
    };
    let clipped = world
        .resource_mut::<Assets<Clipped<M>>>()
        .add(clipped_variant(base, plane));
    world.entity_mut(clone).insert(MeshMaterial3d(clipped));
    true
}

pub fn clip_body(world: &mut World, body: Entity, seam: Entity, plane: Vec4) {
    for node in subtree(world, body) {
        clip_node::<StandardMaterial>(world, node, plane);
        clip_node::<MtoonMaterial>(world, node, plane);
    }
    world.entity_mut(body).insert(ClippedBody { seam, plane });
}

pub fn unclip_body(world: &mut World, body: Entity) {
    for node in subtree(world, body) {
        unclip_node::<StandardMaterial>(world, node);
        unclip_node::<MtoonMaterial>(world, node);
    }
    world.entity_mut(body).remove::<ClippedBody>();
}

pub fn update_body_clip_plane(world: &mut World, body: Entity, seam: Entity, plane: Vec4) {
    for node in subtree(world, body) {
        update_node::<StandardMaterial>(world, node, plane);
        update_node::<MtoonMaterial>(world, node, plane);
    }
    world.entity_mut(body).insert(ClippedBody { seam, plane });
}
