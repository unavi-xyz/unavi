use std::sync::Arc;

use bevy::prelude::*;
use loro::LoroDoc;
use rstest::fixture;

pub struct TestContext {
    pub app: App,
    pub doc: Arc<LoroDoc>,
}

impl Default for TestContext {
    fn default() -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy_hsd::HsdPlugin));

        let mut ctx = Self {
            app,
            doc: Arc::default(),
        };

        ctx.spawn_hsd();

        ctx
    }
}

impl TestContext {
    pub fn spawn_hsd(&mut self) {
        self.app
            .world_mut()
            .spawn(bevy_hsd::Hsd(Arc::clone(&self.doc)));
    }
}

#[fixture]
pub fn ctx() -> TestContext {
    TestContext::default()
}
