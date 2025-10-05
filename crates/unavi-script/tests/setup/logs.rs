use std::{
    fmt::Display,
    sync::{Arc, LazyLock, Mutex},
};

use bevy::{log::BoxedLayer, prelude::*};
use tracing::{
    Event, Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{
    Layer,
    field::MakeVisitor,
    fmt::format::{PrettyFields, Writer},
    layer::Context,
    registry::LookupSpan,
};

pub static LOGS: LazyLock<VecStorageLayer> = LazyLock::new(VecStorageLayer::default);

pub fn custom_layer(_: &mut App) -> Option<BoxedLayer> {
    Some(LOGS.clone().boxed())
}

#[derive(Default)]
struct SpanFields(String);

impl Display for SpanFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Default)]
pub struct VecStorageLayer {
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl<S> Layer<S> for VecStorageLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut buf = String::new();
            let mut visitor = PrettyFields::default().make_visitor(Writer::new(&mut buf));
            attrs.record(&mut visitor);

            let mut ext = span.extensions_mut();
            ext.insert(SpanFields(buf));
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut buf = String::new();
        let mut visitor = PrettyFields::default().make_visitor(Writer::new(&mut buf));
        event.record(&mut visitor);

        let mut stack = Vec::new();
        if let Some(scope) = ctx.lookup_current().map(|c| c.scope()) {
            for span in scope.from_root() {
                let name = span.metadata().name();
                let fields = span
                    .extensions()
                    .get::<SpanFields>()
                    .map(|f| f.to_string())
                    .unwrap_or_default();

                if fields.is_empty() {
                    stack.push(name.to_string());
                } else {
                    stack.push(format!("{name}{{{fields}}}"));
                }
            }
        }

        let line = if stack.is_empty() {
            buf
        } else {
            format!("[{}] {}", stack.join("::"), buf)
        };

        if let Ok(mut logs) = self.logs.lock() {
            logs.push(line);
        }
    }
}

pub fn count_logs_with(value: &str) -> usize {
    LOGS.logs
        .lock()
        .unwrap()
        .iter()
        .filter(|line| line.to_lowercase().contains(value))
        .count()
}

pub fn has_log(value: &str) -> bool {
    LOGS.logs
        .lock()
        .unwrap()
        .iter()
        .any(|line| line.to_lowercase().contains(value))
}

pub fn has_error_log() -> bool {
    has_log("error")
}
