//! The bound each automatic WEIGHTED search accepts, read from the search
//! loop's own `debug!` events (see `tests/weighted_search.rs` for the model).

mod common;

use vibegraph::diagrams::{
    generate_from_proc_card_in, parse_proc_card, EnumerationPool, ParsingOptions,
};
use vibegraph::ufo::UFOModel;

/// The loop's own record of the bounds it tried, in order: one `no diagrams at
/// WEIGHTED w` per failed trial, then either `lowest WEIGHTED order with
/// diagrams: w` (a trial succeeded below the cap) or `no diagrams below the
/// WEIGHTED cap c; generating at w` (the loop left at the cap). These are
/// MadGraph's `Trying coupling order WEIGHTED<=w` lines, which it prints before
/// each trial, so its successful trial is one of them. The search runs on the
/// calling thread (`EnumerationPool::Ambient`), where the scoped subscriber sees
/// it.
///
/// This file must stay the only test binary that installs a scoped subscriber:
/// tracing caches each callsite's interest process-wide on its first hit, so a
/// callsite first reached by a concurrent test with no subscriber is cached as
/// uninterested and this subscriber never sees it.
fn search_record(process: &str, model: &UFOModel) -> Vec<String> {
    use std::sync::{Arc, Mutex};
    use tracing::field::{Field, Visit};
    use tracing::level_filters::LevelFilter;
    use tracing::span::{Attributes, Id, Record};
    use tracing::{Event, Metadata, Subscriber};

    struct Message<'a>(&'a mut String);
    impl Visit for Message<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                *self.0 = format!("{value:?}");
            }
        }
    }
    struct Collect(Arc<Mutex<Vec<String>>>);
    impl Subscriber for Collect {
        fn enabled(&self, _: &Metadata<'_>) -> bool {
            true
        }
        fn max_level_hint(&self) -> Option<LevelFilter> {
            Some(LevelFilter::DEBUG)
        }
        fn new_span(&self, _: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }
        fn record(&self, _: &Id, _: &Record<'_>) {}
        fn record_follows_from(&self, _: &Id, _: &Id) {}
        fn event(&self, event: &Event<'_>) {
            let mut message = String::new();
            event.record(&mut Message(&mut message));
            if message.contains("WEIGHTED") {
                self.0.lock().unwrap().push(message);
            }
        }
        fn enter(&self, _: &Id) {}
        fn exit(&self, _: &Id) {}
    }

    let record = Arc::new(Mutex::new(Vec::new()));
    tracing::subscriber::with_default(Collect(record.clone()), || {
        let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
            .expect("the process card parses");
        generate_from_proc_card_in(&card, model, EnumerationPool::Ambient)
            .expect("the process generates");
    });
    let record = record.lock().unwrap().clone();
    record
}

/// The bound each toy row's search accepts, not only the diagram count: the
/// diagram set at a bound is the set of diagrams weighing at most that bound,
/// so a stop at 8 and a generation at the cap 9 give the same 6 diagrams and
/// only the loop's own record tells them apart.
///
/// MadGraph prints `Trying coupling order WEIGHTED<=w` before each trial, so
/// these rows read, on MadGraph's side:
///
/// | row                  | `Trying` lines | then                              |
/// |----------------------|----------------|-----------------------------------|
/// | `fw~ fw > xw xw`     | 4, 5           | loop leaves at the cap 6, generates there |
/// | `fw~ fw > xw xw vw`  | 6, 7, 8        | the trial at 8 succeeds (cap 9)   |
/// | `xw xw > xw xw`      | 4              | the trial at 4 succeeds (cap 6)   |
/// | SM `z > u u~ d d~`   | none           | a top-level decay runs no search  |
#[test]
fn the_search_accepts_the_bound_madgraph_does() {
    let model = common::toy_weighted_model();
    assert_eq!(
        search_record("fw~ fw > xw xw", &model),
        [
            "no diagrams at WEIGHTED 4, raising the bound",
            "no diagrams at WEIGHTED 5, raising the bound",
            "no diagrams below the WEIGHTED cap 6; generating at 6",
        ]
    );
    assert_eq!(
        search_record("fw~ fw > xw xw vw", &model),
        [
            "no diagrams at WEIGHTED 6, raising the bound",
            "no diagrams at WEIGHTED 7, raising the bound",
            "lowest WEIGHTED order with diagrams: 8",
        ]
    );
    assert_eq!(
        search_record("xw xw > xw xw", &model),
        ["lowest WEIGHTED order with diagrams: 4"]
    );

    // A one-particle process outside a chain runs no search at all.
    assert!(search_record("z > u u~ d d~", &common::sm_model()).is_empty());
}
