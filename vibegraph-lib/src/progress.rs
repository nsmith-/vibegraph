//! The machine-readable side of a run's own commentary: which stage is working,
//! how far into it the run is, and the handful of numbers a live display shows.
//!
//! Progress is a stream of *measurements*, not of prose, and it is separated from
//! the human-facing log twice over. It carries the dedicated target [`TARGET`], so
//! a consumer subscribes to it by target rather than by parsing formatted lines;
//! and it is emitted at `TRACE`, so a sink that prints lines never prints it at
//! any verbosity a person runs at. A display attaches its own layer filtered on
//! the target alone and then receives every measurement whatever level the visible
//! log is set to.
//!
//! # Field contract
//!
//! Every event on [`TARGET`] carries three fields:
//!
//! | field | type | meaning |
//! |---|---|---|
//! | `stage` | `&'static str` | which stage is reporting — one of the [`stage`] constants |
//! | `done` | `u64` | units of work finished so far |
//! | `total` | `u64` | units of work the stage expects, when it knows |
//!
//! `total` is optional the way a `tracing` field is optional: a stage that cannot
//! yet know its own extent records no value, and a visitor sees the field *absent*
//! rather than set to zero. The unit `done` and `total` count is whatever the
//! stage names — subprocesses, channels, iterations, events — so the pair is a
//! fraction within one stage and is not comparable across stages.
//!
//! `done` rises through a stage, but not always monotonically across a whole run
//! of one: a stage that repeats its own pass restarts the count — enumeration
//! walks the alias expansion once per candidate coupling order — and a stage whose
//! work is spread over threads counts completions rather than positions. A display
//! shows the latest value rather than assuming the next one is larger.
//!
//! Stage-specific fields ride alongside those three and are documented on the
//! function that emits them: [`vegas_iteration`] carries the running estimate,
//! [`unweighting`] the accept/reject counts, [`eval_rate`] the measured cost of an
//! integrand evaluation.
//!
//! # Emitting
//!
//! Call these functions rather than writing the target and field names out at a
//! call site: the names *are* the interface a display is written against, and a
//! misspelling at one call site is a stage that silently never reports. They are
//! cheap when nothing is listening — a disabled `TRACE` callsite is an atomic load
//! and a branch — so a loop may call one per item.

use tracing::trace;

/// The target every progress event is emitted on, and the only thing a consumer
/// needs to match to receive all of them.
pub const TARGET: &str = "vibegraph::progress";

/// The `stage` field's vocabulary. A stage name is stable: a display keys its
/// layout off it.
pub mod stage {
    /// Reading and digesting the model. It is one indivisible unit of work, so it
    /// reports `0 of 1` when it starts and `1 of 1` when the model is resolved.
    pub const UFO_LOAD: &str = "ufo_load";
    /// Diagram enumeration, counting concrete subprocesses. The total is unknown
    /// until the alias expansion has been walked, so it is absent.
    pub const ENUMERATE: &str = "enumerate";
    /// Amplitude compilation, counting compiled subprocesses out of the enumerated
    /// ones.
    pub const COMPILE: &str = "compile";
    /// The Kleiss–Pittau survey that sets the channel selection weights, counting
    /// survey iterations.
    pub const ALPHA_SURVEY: &str = "alpha_survey";
    /// The VEGAS integration, counting adaptation iterations. Under a fixed
    /// budget the total is the plan. Under a convergence target it is the
    /// iteration the target is projected to be met at — re-estimated as the
    /// error contracts, capped where the run would give up, and absent through
    /// the warm-up, when no estimate exists to project from.
    pub const VEGAS: &str = "vegas";
    /// The frozen scan for each channel's maximum weight, counting channels.
    pub const WEIGHT_SCAN: &str = "weight_scan";
    /// The accept/reject pass, counting accepted events out of the requested ones.
    pub const UNWEIGHT: &str = "unweight";
}

/// `done` of `total` units of `stage` finished.
#[inline]
pub fn step(stage: &'static str, done: u64, total: Option<u64>) {
    trace!(target: TARGET, stage, done, total);
}

/// One completed VEGAS iteration, with the estimate as it stands after it.
///
/// `sigma` and `err` are **in the integrand's own units**, which for every
/// cross-section integrand in this crate is `GeV⁻²`: a consumer that displays
/// picobarns applies [`GEV2_TO_PB`](crate::phasespace::GEV2_TO_PB) itself, exactly
/// as the result line does. `chi2` is χ² per degree of freedom over the iterations
/// combined so far, and is `0` until there are two of them.
#[inline]
pub fn vegas_iteration(done: u64, total: Option<u64>, sigma: f64, err: f64, chi2: f64) {
    trace!(target: TARGET, stage = stage::VEGAS, done, total, sigma, err, chi2);
}

/// The accept/reject pass's position: `accepted` events secured of `requested`.
///
/// "Secured" is the pass's own numerator — accepted points for a strategy that
/// writes one event per accepted point, events written for one that writes an
/// accepted point several times — so the two numbers are always the fraction of
/// the requested sample that exists. `done`/`total` repeat them, so a display
/// driving a bar off the generic pair needs no special case for this stage.
#[inline]
pub fn unweighting(accepted: u64, requested: u64) {
    trace!(
        target: TARGET,
        stage = stage::UNWEIGHT,
        done = accepted,
        total = Some(requested),
        accepted,
        requested
    );
}

/// What one integrand evaluation cost, measured over a stage's own last block of
/// work rather than over the run.
#[inline]
pub fn eval_rate(stage: &'static str, done: u64, total: Option<u64>, ns_per_eval: f64) {
    trace!(target: TARGET, stage, done, total, ns_per_eval);
}

#[cfg(test)]
mod tests {
    use super::{eval_rate, stage, unweighting, vegas_iteration, TARGET};

    use std::sync::{Arc, Mutex};

    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id, Record};
    use tracing::{Event, Level, Metadata, Subscriber};

    /// One captured event: its target, its level, and its fields as
    /// `(name, rendered value)` in emission order.
    #[derive(Debug, Clone, PartialEq)]
    struct Captured {
        target: String,
        level: Level,
        fields: Vec<(String, String)>,
    }

    impl Captured {
        fn field(&self, name: &str) -> Option<&str> {
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

    /// A subscriber that keeps every event it is handed. It enables everything, so
    /// a field the emitter never records is a field genuinely absent rather than
    /// one filtered away.
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

    fn captured(emit: impl FnOnce()) -> Vec<Captured> {
        let capture = Capture::default();
        tracing::subscriber::with_default(capture.clone(), emit);
        let events = capture.0.lock().expect("capture").clone();
        events
    }

    /// The target and the level are the whole reason this stream can be consumed
    /// separately from the log, so both are pinned: at any level above `TRACE` a
    /// line sink would print these as text, and on any other target a display
    /// would have to match on message content instead.
    #[test]
    fn progress_is_trace_on_its_own_target() {
        let events = captured(|| super::step(stage::COMPILE, 3, Some(7)));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, TARGET);
        assert_eq!(events[0].level, Level::TRACE);
    }

    #[test]
    fn every_event_carries_stage_done_and_total() {
        let events = captured(|| {
            super::step(stage::COMPILE, 3, Some(7));
            vegas_iteration(4, Some(12), 802.94, 3.11, 1.02);
            unweighting(250, 1_000);
            eval_rate(stage::VEGAS, 4, Some(12), 212.5);
        });
        assert_eq!(events.len(), 4);
        for event in &events {
            assert!(event.field("stage").is_some(), "{event:?}");
            assert!(event.field("done").is_some(), "{event:?}");
            assert!(event.field("total").is_some(), "{event:?}");
        }
        assert_eq!(events[0].field("stage"), Some("compile"));
        assert_eq!(events[0].field("done"), Some("3"));
        assert_eq!(events[0].field("total"), Some("7"));
    }

    /// An unknown total is the field's *absence*, not a zero — a display that read
    /// a missing total as `0` would draw a full bar over an unbounded stage.
    #[test]
    fn an_unknown_total_records_no_value() {
        let events = captured(|| super::step(stage::ENUMERATE, 12, None));
        assert_eq!(events[0].field("done"), Some("12"));
        assert_eq!(events[0].field("total"), None);
        assert_eq!(events[0].fields.len(), 2);
    }

    /// The stage-specific fields a display reads by name, each pinned on the
    /// function that promises it.
    #[test]
    fn the_stage_specific_fields_are_named_as_documented() {
        let events = captured(|| {
            vegas_iteration(4, Some(12), 802.94, 3.11, 1.02);
            unweighting(250, 1_000);
            eval_rate(stage::VEGAS, 4, Some(12), 212.5);
        });
        assert_eq!(events[0].field("stage"), Some("vegas"));
        assert_eq!(events[0].field("sigma"), Some("802.94"));
        assert_eq!(events[0].field("err"), Some("3.11"));
        assert_eq!(events[0].field("chi2"), Some("1.02"));

        assert_eq!(events[1].field("stage"), Some("unweight"));
        assert_eq!(events[1].field("accepted"), Some("250"));
        assert_eq!(events[1].field("requested"), Some("1000"));
        assert_eq!(events[1].field("done"), Some("250"));
        assert_eq!(events[1].field("total"), Some("1000"));

        assert_eq!(events[2].field("ns_per_eval"), Some("212.5"));
    }
}
