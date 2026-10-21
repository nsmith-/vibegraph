//! A `tracing` subscriber for tests that keeps every event it is handed.

use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// One captured event: its target, its level, and its fields as
/// `(name, rendered value)` in emission order. A string field is rendered
/// without quotes.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Captured {
    pub(crate) target: String,
    pub(crate) level: Level,
    pub(crate) fields: Vec<(String, String)>,
}

impl Captured {
    pub(crate) fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Default)]
struct Recorder(Vec<(String, String)>);

impl Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .push((field.name().to_string(), format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.push((field.name().to_string(), value.to_string()));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.push((field.name().to_string(), value.to_string()));
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.push((field.name().to_string(), value.to_string()));
    }
}

/// Enables everything, so a field an emitter never records is genuinely absent
/// rather than filtered away.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Captured>>>);

impl Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        let mut fields = Recorder::default();
        event.record(&mut fields);
        self.0.lock().expect("capture").push(Captured {
            target: event.metadata().target().to_string(),
            level: *event.metadata().level(),
            fields: fields.0,
        });
    }
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

/// Run `run` with a capturing subscriber installed on this thread, and return
/// its result with the events it emitted on this thread.
///
/// `tracing` caches each callsite's interest process-wide from whichever
/// thread reaches it first; under a parallel test run that is usually a test
/// with no subscriber, which stamps the callsite `never` so it is skipped before
/// `enabled` is consulted. The cache is rebuilt with this subscriber installed,
/// which heals a callsite stamped earlier. A callsite that is not yet registered
/// must be reached once from this thread first, or another thread can stamp it
/// between the rebuild and `run`: [`capture_primed`] takes that first hit.
pub(crate) fn capture<R>(run: impl FnOnce() -> R) -> (R, Vec<Captured>) {
    capture_primed(|| {}, run)
}

/// [`capture`], with `prime` run first inside the subscriber's scope. `prime`
/// emits one event from each callsite `run` is about to rely on; its events are
/// in the result, so the caller filters them out.
pub(crate) fn capture_primed<R>(
    prime: impl FnOnce(),
    run: impl FnOnce() -> R,
) -> (R, Vec<Captured>) {
    let capture = Capture::default();
    let result = tracing::subscriber::with_default(capture.clone(), || {
        prime();
        tracing::callsite::rebuild_interest_cache();
        run()
    });
    let events = capture.0.lock().expect("capture").clone();
    (result, events)
}
