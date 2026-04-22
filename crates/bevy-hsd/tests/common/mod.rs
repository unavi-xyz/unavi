#![allow(unused)]

use std::{sync::Arc, time::Duration};

use avian3d::{
    PhysicsPlugins, collider_tree::ColliderTreeDiagnostics, collision::CollisionDiagnostics,
    dynamics::solver::SolverDiagnostics, prelude::SpatialQueryDiagnostics,
};
use bevy::{prelude::*, scene::ScenePlugin, transform::TransformPlugin};
use bevy_hsd::{
    HsdDoc, HsdPlugin, HsdRecordId,
    cache::{MeshState, SceneRegistry},
    hydrate::compile::mesh::{HsdMeshGeometrySet, MeshGeometrySource},
};
use loro::{LoroDoc, LoroMap};

const TICK: Duration = Duration::from_millis(100);

pub struct TestHarness {
    pub app: App,
    pub doc_entity: Entity,
    pub doc_id: blake3::Hash,
    pub doc: Arc<LoroDoc>,
}

impl TestHarness {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            ScenePlugin,
            PhysicsPlugins::default(),
            HsdPlugin,
        ))
        .init_asset::<StandardMaterial>()
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_resource::<ColliderTreeDiagnostics>()
        .init_resource::<CollisionDiagnostics>()
        .init_resource::<SolverDiagnostics>()
        .init_resource::<SpatialQueryDiagnostics>()
        .insert_resource(Time::<Virtual>::from_max_delta(TICK))
        .insert_resource(Time::<Fixed>::from_duration(TICK));

        let doc = Arc::new(LoroDoc::new());
        let doc_id = blake3::hash(b"test-doc");
        let doc_entity = app
            .world_mut()
            .spawn((HsdDoc(Arc::clone(&doc)), HsdRecordId(doc_id)))
            .id();
        Self::tick(&mut app);

        Self {
            app,
            doc_entity,
            doc_id,
            doc,
        }
    }

    /// Commit the doc and advance one frame, flushing all queued HSD changes.
    pub fn commit_and_update(&mut self) {
        self.doc.commit();
        Self::tick(&mut self.app);
    }

    /// Add a mesh to the doc, set its inline geometry state, and compile it.
    ///
    /// After this call the mesh entity will have `CompiledMesh`.
    pub fn attach_inline_mesh(&mut self, id: &str, state: MeshState) {
        self.doc
            .get_map("hsd")
            .get_or_create_container("meshes", LoroMap::new())
            .expect("meshes map")
            .get_or_create_container(id, LoroMap::new())
            .expect("mesh map entry");
        self.commit_and_update();

        let registry = self
            .app
            .world()
            .get::<SceneRegistry>(self.doc_entity)
            .expect("SceneRegistry")
            .clone();
        let mesh_inner = registry
            .0
            .meshes
            .lock()
            .expect("meshes lock")
            .get(id)
            .cloned()
            .expect("mesh inner");
        *mesh_inner.state.lock().expect("mesh state lock") = state;

        self.app.world_mut().trigger(HsdMeshGeometrySet {
            doc_id: self.doc_id,
            id: id.into(),
            source: MeshGeometrySource::Inline,
        });
        self.app.world_mut().flush();
        Self::tick(&mut self.app);
    }

    fn tick(app: &mut App) {
        std::thread::sleep(TICK);
        app.update();
    }
}
