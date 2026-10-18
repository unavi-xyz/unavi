//! One-line rendering for the redial warnings the app cannot avoid.

use std::fmt;

use bevy::{
    app::App,
    log::{
        BoxedFmtLayer,
        BoxedLayer,
    },
};
use tracing::{
    Event,
    Metadata,
    Subscriber,
    field::{
        Field,
        Visit,
    },
};
use tracing_subscriber::{
    Layer,
    field::RecordFields,
    fmt::{
        self as tfmt,
        format::{
            FormatFields,
            Writer,
        },
    },
    layer::{
        Context,
        Filter,
    },
};

/// Collapses the `Caused by:` chain of a redial warning onto one line.
///
/// `iroh-docs` retries every peer it remembers when a namespace is served, and
/// each rejected dial renders its `ConnectError` across four lines. This keeps
/// those to one; everything else keeps the default rendering.
pub fn collapsing_layer(_: &mut App) -> Option<BoxedLayer> {
    if cfg!(target_family = "wasm") {
        return None;
    }

    Some(Box::new(
        tfmt::Layer::default()
            .with_writer(std::io::stderr)
            .fmt_fields(OneLineFields)
            .with_filter(Redial),
    ))
}

/// The default field rendering, for every event that is not a redial warning.
pub fn plain_layer(_: &mut App) -> Option<BoxedFmtLayer> {
    Some(Box::new(
        tfmt::Layer::default()
            .with_writer(std::io::stderr)
            .with_filter(Other),
    ))
}

/// Passes only the redial warnings.
#[derive(Debug, Clone, Copy)]
struct Redial;

impl<S: Subscriber> Filter<S> for Redial {
    fn enabled(&self, _: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _: &Context<'_, S>) -> bool {
        is_redial_warning(event)
    }
}

/// Passes everything the [`Redial`] filter does not.
#[derive(Debug, Clone, Copy)]
struct Other;

impl<S: Subscriber> Filter<S> for Other {
    fn enabled(&self, _: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _: &Context<'_, S>) -> bool {
        !is_redial_warning(event)
    }
}

/// A dial a redialing engine made that failed before reaching the peer.
fn is_redial_warning(event: &Event<'_>) -> bool {
    let message = Message::of(event);
    match event.metadata().target() {
        "iroh_gossip::net" => message.starts_with("dial"),
        "iroh_docs::engine::live" => message.is("sync failed"),
        "unavi_space::link::dial" => message.is("dial failed"),
        _ => false,
    }
}

/// The `message` field an event recorded.
struct Message(String);

impl Message {
    fn of(event: &Event<'_>) -> Self {
        let mut message = Self(String::new());
        event.record(&mut message);
        message
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix)
    }

    fn is(&self, text: &str) -> bool {
        self.0 == text
    }
}

impl Visit for Message {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_owned();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

/// Renders event fields, collapsing an error's `Caused by:` source chain onto
/// the same line so one failure stays one log line.
#[derive(Debug, Clone, Copy)]
struct OneLineFields;

impl<'writer> FormatFields<'writer> for OneLineFields {
    fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> fmt::Result {
        let mut visitor = Fields::new(writer);
        fields.record(&mut visitor);
        visitor.result
    }
}

struct Fields<'a> {
    writer: Writer<'a>,
    empty:  bool,
    result: fmt::Result,
}

impl<'a> Fields<'a> {
    const fn new(writer: Writer<'a>) -> Self {
        Self {
            writer,
            empty: true,
            result: Ok(()),
        }
    }

    fn value(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_ok() {
            self.result = self.write_value(field, value);
        }
    }

    fn write_value(&mut self, field: &Field, value: &dyn fmt::Debug) -> fmt::Result {
        if !self.empty {
            self.writer.write_str(" ")?;
        }
        self.empty = false;

        let rendered = collapse(format!("{value:?}"));
        let name = field.name();
        match name.strip_prefix("r#").unwrap_or(name) {
            "message" => self.writer.write_str(&rendered),
            name => write!(self.writer, "{name}={rendered}"),
        }
    }
}

impl Visit for Fields<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.value(field, &format_args!("{value}"));
        } else {
            self.value(field, &value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.value(field, value);
    }
}

/// anyhow renders a source chain as a `Caused by:` block. Rejoin it with `: `
/// so a field carrying one stays on one line.
fn collapse(rendered: String) -> String {
    const MARKER: &str = "\n\nCaused by:";

    let Some(start) = rendered.find(MARKER) else {
        return rendered;
    };
    let (message, chain) = rendered.split_at(start);
    let chain = &chain[MARKER.len()..];

    let mut line = message.trim_end().to_owned();
    for cause in chain.lines() {
        let cause = cause.trim();
        if cause.is_empty() {
            continue;
        }
        let cause = match cause.split_once(": ") {
            Some((number, rest)) if number.chars().all(|c| c.is_ascii_digit()) => rest,
            _ => cause,
        };
        line.push_str(": ");
        line.push_str(cause);
    }
    line
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{
            Arc,
            Mutex,
        },
    };

    use tracing_subscriber::{
        Registry,
        fmt::writer::MakeWriter,
        layer::{
            Filter,
            SubscriberExt,
        },
    };

    use super::*;

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("poisoned").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Capture {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn emitted<F: Filter<Registry> + Send + Sync + 'static>(
        filter: F,
        event: impl FnOnce(),
    ) -> String {
        let buf = Capture::default();
        let subscriber = Registry::default().with(
            tfmt::layer()
                .with_writer(buf.clone())
                .fmt_fields(OneLineFields)
                .with_filter(filter),
        );

        tracing::subscriber::with_default(subscriber, event);
        let captured = buf.0.lock().expect("poisoned").clone();
        String::from_utf8(captured).expect("utf-8")
    }

    fn chain() -> anyhow::Error {
        anyhow::anyhow!("Connection was rejected locally").context("Failed to establish connection")
    }

    #[test]
    fn a_docs_redial_warning_renders_on_one_line() {
        let out = emitted(Redial, || {
            tracing::warn!(
                target: "iroh_docs::engine::live",
                origin = ?"Connect",
                err = ?chain(),
                "sync failed"
            );
        });

        assert_eq!(out.lines().count(), 1, "got {out:?}");
        assert!(
            out.contains("Failed to establish connection: Connection was rejected locally"),
            "got {out:?}"
        );
    }

    #[test]
    fn a_gossip_redial_warning_is_recognized() {
        let out = emitted(Redial, || {
            tracing::warn!(
                target: "iroh_gossip::net",
                peer = "ae2d9dd413",
                "dial failed: Connection was rejected locally"
            );
        });

        assert!(out.contains("dial failed: Connection was rejected locally"));
    }

    #[test]
    fn the_redial_filter_ignores_other_warnings() {
        let out = emitted(Redial, || {
            tracing::warn!(
                target: "iroh_docs::engine::live",
                err = ?chain(),
                "sync actor error"
            );
        });

        assert!(out.is_empty(), "expected no output, got {out:?}");
    }

    #[test]
    fn the_other_filter_takes_the_rest() {
        let out = emitted(Other, || {
            tracing::warn!(
                target: "iroh_docs::engine::live",
                err = ?chain(),
                "sync actor error"
            );
        });

        assert!(out.contains("sync actor error"), "got {out:?}");
    }

    #[test]
    fn a_source_chain_collapses_onto_one_line() {
        let rendered = "Failed to establish connection\n\nCaused by:\n    0: Connection was rejected locally\n    1: Connection was rejected locally";

        assert_eq!(
            collapse(rendered.to_owned()),
            "Failed to establish connection: Connection was rejected locally: Connection was rejected locally"
        );
    }

    #[test]
    fn a_single_cause_collapses_onto_one_line() {
        let rendered = "node failed\n\nCaused by:\n    nope";

        assert_eq!(collapse(rendered.to_owned()), "node failed: nope");
    }

    #[test]
    fn a_value_without_a_chain_is_unchanged() {
        assert_eq!(
            collapse("Connection was rejected locally".to_owned()),
            "Connection was rejected locally"
        );
    }
}
