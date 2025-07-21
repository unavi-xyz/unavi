use std::sync::{Arc, LazyLock, Mutex};

use bevy::{
    log::{BoxedLayer, LogPlugin},
    prelude::*,
};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    Layer,
    field::MakeVisitor,
    fmt::format::{PrettyFields, Writer},
    layer::Context,
};
use unavi_script::{LoadScriptAsset, ScriptPlugin};

pub fn setup_test_app(package: &'static str) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../unavi/assets".to_string(),
            ..Default::default()
        },
        LogPlugin {
            custom_layer,
            ..Default::default()
        },
        ScriptPlugin,
    ))
    .add_systems(Startup, move |mut events: EventWriter<LoadScriptAsset>| {
        events.write(LoadScriptAsset {
            namespace: "test",
            package,
        });
    });
    app
}

pub static LOGS: LazyLock<VecStorageLayer> = LazyLock::new(VecStorageLayer::default);

fn custom_layer(_: &mut App) -> Option<BoxedLayer> {
    Some(LOGS.clone().boxed())
}

#[derive(Clone, Default)]
pub struct VecStorageLayer {
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl<S> Layer<S> for VecStorageLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut buf = String::new();
        let mut visitor = PrettyFields::default().make_visitor(Writer::new(&mut buf));
        event.record(&mut visitor);

        if let Ok(mut logs) = self.logs.lock() {
            logs.push(buf);
        }
    }
}
