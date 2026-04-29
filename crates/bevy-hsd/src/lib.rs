use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use avian3d::schedule::PhysicsSystems;
use bevy::prelude::*;
use blake3::Hash;
use loro::LoroDoc;
use smol_str::SmolStr;

pub mod asset;
pub mod hydrate;
pub mod instance;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::BlobLoader)
            .register_asset_loader(asset::HsdLoader)
            .init_asset::<asset::BlobAsset>()
            .init_asset::<asset::HsdAsset>()
            .init_resource::<DocRegistryMap>()
            .add_observer(hydrate::compile::node::handle_hsd_doc_transform_set)
            .add_observer(hydrate::compile::collider::on_collider_blobs_loaded)
            .add_observer(hydrate::compile::image::handle_hsd_image_despawned)
            .add_observer(hydrate::compile::image::handle_hsd_image_spawned)
            .add_observer(hydrate::compile::image::on_image_blobs_loaded)
            .add_observer(hydrate::compile::image::on_image_compiled)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_cutoff_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_mode_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_base_color_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_base_color_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_despawned)
            .add_observer(hydrate::compile::material::handle_hsd_material_double_sided_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_emissive_texture_set)
            .add_observer(
                hydrate::compile::material::handle_hsd_material_metallic_roughness_texture_set,
            )
            .add_observer(hydrate::compile::material::handle_hsd_material_metallic_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_name_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_normal_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_occlusion_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_roughness_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_spawned)
            .add_observer(hydrate::compile::material::handle_hsd_material_unlit_set)
            .add_observer(hydrate::compile::material::on_material_blobs_loaded)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_despawned)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_geometry_set)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_spawned)
            .add_observer(hydrate::compile::mesh::on_mesh_blobs_loaded)
            .add_observer(hydrate::compile::node::handle_hsd_node_collider_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_despawned)
            .add_observer(hydrate::compile::node::handle_hsd_node_material_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_mesh_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_name_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_parent_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_rigid_body_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_scripts_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_spawned)
            .add_observer(hydrate::compile::node::handle_hsd_node_transform_set)
            .add_observer(hydrate::compile::node::on_material_compiled)
            .add_observer(hydrate::compile::node::on_material_ref_set)
            .add_observer(hydrate::compile::node::on_mesh_compiled)
            .add_observer(hydrate::compile::node::on_mesh_ref_removed)
            .add_observer(hydrate::compile::node::on_mesh_ref_set)
            .add_systems(
                FixedPostUpdate,
                (hydrate::compile::node::guard_physics_scale, ApplyDeferred)
                    .chain()
                    .before(PhysicsSystems::StepSimulation),
            )
            .add_systems(
                FixedUpdate,
                (
                    (
                        hydrate::init::init_hsd_doc,
                        hydrate::queue::process_hsd_queue,
                        hydrate::compile::material::recompile_changed_materials,
                    )
                        .chain(),
                    instance::instance_hsd,
                ),
            );
    }
}

#[derive(Component)]
#[require(HsdChildren, Transform, Visibility)]
pub struct HsdDoc(pub Arc<LoroDoc>);

#[derive(Component, Default)]
#[relationship_target(relationship = HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HsdChildren)]
pub struct HsdChild {
    #[relationship]
    pub doc: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct NodeId(pub SmolStr);

#[derive(Component, Clone, Debug)]
pub struct MeshId(pub SmolStr);

#[derive(Component, Clone, Debug)]
pub struct MaterialId(pub SmolStr);

#[derive(Component, Clone, Debug, Default)]
#[relationship_target(relationship = ScriptNode, linked_spawn)]
pub struct NodeScripts(Vec<Entity>);

#[derive(Component, Clone, Debug)]
#[relationship(relationship_target = NodeScripts)]
pub struct ScriptNode(pub Entity);

#[derive(Component, Clone, Debug)]
pub struct HsdScript(pub Hash);

#[derive(Component, Clone, Debug, Default)]
pub struct HsdNodePhysics {
    pub collider: Option<hsd::HsdCollider>,
    pub rigid_body: Option<hsd::HsdRigidBody>,
}

#[derive(Component, Clone, Copy)]
pub struct HsdRecordId(pub blake3::Hash);

#[derive(Component)]
pub struct HsdSubscription(pub loro::Subscription);

#[derive(Component, Clone, Debug, Default)]
pub struct HsdAssets(pub BTreeMap<SmolStr, blake3::Hash>);

#[derive(Resource, Default)]
pub struct DocRegistryMap(pub HashMap<blake3::Hash, Entity>);

impl DocRegistryMap {
    #[must_use]
    pub fn get_entity(&self, doc_id: &blake3::Hash) -> Option<Entity> {
        self.0.get(doc_id).copied()
    }
}

#[derive(Component, Default)]
pub struct HsdEntityMaps {
    pub nodes: HashMap<SmolStr, Entity>,
    pub meshes: HashMap<SmolStr, Entity>,
    pub materials: HashMap<SmolStr, Entity>,
    pub images: HashMap<SmolStr, Entity>,
}
