//! Node logging: one-line field rendering, and dropping the sync warnings a
//! node cannot avoid.

use std::fmt;

use tracing::{
    Event,
    Level,
    Metadata,
    Subscriber,
    field::{
        Field,
        Visit,
    },
};
use tracing_subscriber::{
    EnvFilter,
    Layer,
    field::RecordFields,
    fmt::{
        format::{
            FormatFields,
            Writer,
        },
        writer::MakeWriterExt,
    },
    layer::{
        Context,
        Filter,
        SubscriberExt,
    },
    util::SubscriberInitExt,
};

/// The iroh-docs module that logs a warning per failed sync.
const DOCS_LIVE: &str = "iroh_docs::engine::live";

pub fn init(debug: bool) {
    let level = if debug { Level::DEBUG } else { Level::INFO };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .map_writer(|w| w.with_max_level(level))
        .fmt_fields(OneLineFields)
        .with_filter(DropRejectedDials);

    let registry = tracing_subscriber::registry().with(fmt_layer);

    #[cfg(feature = "devtools-console")]
    let registry = registry.with(console_subscriber::spawn());

    registry
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();
}

/// Drops `sync failed` warnings for dials this node made.
///
/// The node answers peers and never dials, so a `Connect` failure only reports
/// the rejected dial. A failure on a connection a peer opened still surfaces.
#[derive(Debug, Clone, Copy)]
struct DropRejectedDials;

impl<S: Subscriber> Filter<S> for DropRejectedDials {
    fn enabled(&self, _: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _: &Context<'_, S>) -> bool {
        !is_rejected_dial(event)
    }
}

fn is_rejected_dial(event: &Event<'_>) -> bool {
    event.metadata().target() == DOCS_LIVE
        && *event.metadata().level() == Level::WARN
        && Origin::of(event).is_connect()
}

/// The `origin` field the docs engine records on a sync failure.
struct Origin(bool);

impl Origin {
    fn of(event: &Event<'_>) -> Self {
        let mut origin = Self(false);
        event.record(&mut origin);
        origin
    }

    const fn is_connect(&self) -> bool {
        self.0
    }
}

impl Visit for Origin {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "origin" && format!("{value:?}").starts_with("Connect") {
            self.0 = true;
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

    #[derive(Debug)]
    enum Origin {
        Connect,
        Accept,
    }

    fn emitted(event: impl FnOnce()) -> String {
        let buf = Capture::default();
        let subscriber = Registry::default().with(
            tracing_subscriber::fmt::layer()
                .with_writer(buf.clone())
                .fmt_fields(OneLineFields)
                .with_filter(DropRejectedDials),
        );

        tracing::subscriber::with_default(subscriber, event);
        let captured = buf.0.lock().expect("poisoned").clone();
        String::from_utf8(captured).expect("utf-8")
    }

    #[test]
    fn connect_origin_sync_failures_are_dropped() {
        let out = emitted(|| {
            tracing::warn!(
                target: DOCS_LIVE,
                origin = ?Origin::Connect,
                err = %"rejected",
                "sync failed"
            );
        });

        assert!(out.is_empty(), "expected no output, got {out:?}");
    }

    #[test]
    fn accept_origin_sync_failures_survive() {
        let out = emitted(|| {
            tracing::warn!(
                target: DOCS_LIVE,
                origin = ?Origin::Accept,
                err = %"rejected",
                "sync failed"
            );
        });

        assert!(out.contains("sync failed"), "got {out:?}");
    }

    #[test]
    fn warnings_from_other_targets_survive() {
        let out = emitted(|| {
            tracing::warn!(
                target: "iroh_docs::engine::other",
                origin = ?Origin::Connect,
                "sync failed"
            );
        });

        assert!(out.contains("sync failed"), "got {out:?}");
    }

    #[test]
    fn an_error_chain_renders_on_one_line() {
        let err = anyhow::anyhow!("Connection was rejected locally")
            .context("Failed to establish connection");
        let out = emitted(|| {
            tracing::warn!(
                target: DOCS_LIVE,
                origin = ?Origin::Accept,
                err = ?err,
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
