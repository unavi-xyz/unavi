use std::{sync::Arc, time::Duration};

use avian3d::{
    PhysicsPlugins, collision::CollisionDiagnostics, dynamics::solver::SolverDiagnostics,
    prelude::SpatialQueryDiagnostics,
};
use bevy::{prelude::*, scene::ScenePlugin, transform::TransformPlugin};
use bevy_hsd::{HsdDoc, HsdPlugin};
use loro::LoroDoc;

const TICK: Duration = Duration::from_millis(100);

pub struct TestHarness {
    pub app: App,
    pub doc_entity: Entity,
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
        .init_resource::<CollisionDiagnostics>()
        .init_resource::<SolverDiagnostics>()
        .init_resource::<SpatialQueryDiagnostics>()
        .insert_resource(Time::<Virtual>::from_max_delta(TICK))
        .insert_resource(Time::<Fixed>::from_duration(TICK));

        let doc = Arc::new(LoroDoc::new());
        let doc_entity = app.world_mut().spawn(HsdDoc(Arc::clone(&doc))).id();
        Self::tick(&mut app);

        Self {
            app,
            doc_entity,
            doc,
        }
    }

    /// Commit the doc and advance one frame, flushing all queued HSD changes.
    pub fn commit_and_update(&mut self) {
        self.doc.commit();
        Self::tick(&mut self.app);
    }

    fn tick(app: &mut App) {
        std::thread::sleep(TICK);
        app.update();
    }
}
