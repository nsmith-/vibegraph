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
//! A stage that runs once per group of a multi-group card (see [`Group`]) also
//! carries `group`, `groups`, `label` and, once the group's channels are built,
//! `channels`; a record from outside any group has none of the four.
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

use std::cell::RefCell;
use std::marker::PhantomData;

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
    /// Diagram enumeration, counting the candidate particle assignments of one
    /// process line's alias expansion — pruned and enumerated alike — out of the
    /// expansion's size, which is known before the walk starts. See
    /// [`enumeration`](super::enumeration) for the fields beside the count.
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

/// One group of a card whose integration is split into several: a final-state
/// multiplicity and set of outgoing masses, each integrated over channels of
/// its own.
///
/// While a [`GroupScope`] holds a group, every [`step`] on that thread carries
/// its four fields:
///
/// | field | meaning |
/// |---|---|
/// | `group` | the group's position, from 1 |
/// | `groups` | how many groups the card is split into |
/// | `label` | the group's process tag (`@1`, or `@1 @2` when several lines share its final state), or its outgoing-leg count when the tags do not tell the groups apart |
/// | `channels` | the group's channel count; absent while the group's channels are not yet built |
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub(crate) index: u64,
    pub(crate) count: u64,
    pub(crate) label: String,
    pub(crate) channels: Option<u64>,
}

thread_local! {
    static GROUP: RefCell<Option<Group>> = const { RefCell::new(None) };
}

/// Run `read` on the group this thread is in, if any.
fn in_group<T>(read: impl FnOnce(&Group) -> T) -> Option<T> {
    GROUP
        .try_with(|g| g.borrow().as_ref().map(read))
        .ok()
        .flatten()
}

/// The group a thread's progress is reported under, until dropped; the group it
/// replaced is restored then. It belongs to the thread that entered it, so the
/// work a stage hands to other threads reports outside any group.
#[must_use = "the group applies until the scope is dropped"]
pub struct GroupScope {
    previous: Option<Group>,
    not_send: PhantomData<*const ()>,
}

/// Report this thread's progress under `group` until the returned scope drops.
pub fn enter_group(group: Group) -> GroupScope {
    let previous = GROUP
        .try_with(|g| g.borrow_mut().replace(group))
        .ok()
        .flatten();
    GroupScope {
        previous,
        not_send: PhantomData,
    }
}

impl Drop for GroupScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        let _ = GROUP.try_with(|g| *g.borrow_mut() = previous);
    }
}

/// `done` of `total` units of `stage` finished, under the thread's [`Group`]
/// when it is in one.
#[inline]
pub fn step(stage: &'static str, done: u64, total: Option<u64>) {
    trace!(
        target: TARGET,
        stage,
        done,
        total,
        group = in_group(|g| g.index),
        groups = in_group(|g| g.count),
        label = in_group(|g| g.label.clone()).as_deref(),
        channels = in_group(|g| g.channels).flatten()
    );
}

/// One step through a process line's candidate assignments: `done` of `total`
/// candidates have been considered.
///
/// A line's alias expansion (`p p > e+ e- j j` is every particle `p` and `j`
/// stand for, leg by leg) is walked once, and its size is the product of the
/// legs' alternatives, so `total` is known when the walk starts. `done` counts
/// every candidate passed over, including those pruned as duplicates or for
/// violating charge, which cost almost nothing; the walk's time is in the
/// candidates that survive to topology assignment, and a report follows each
/// of those, so a bar over `done/total` advances unevenly. The walk of one line
/// restarts the count, as does each further walk a line needs: the automatic
/// coupling-order search repeats the expansion at a higher bound while no
/// subprocess has a diagram, and a decay chain walks each of its cores.
///
/// Beside `stage`, `done` and `total`, the event carries:
///
/// | field | meaning |
/// |---|---|
/// | `line` | the process line being walked, from 1 |
/// | `lines` | the process lines the card holds |
/// | `subprocesses` | concrete subprocesses this walk has enumerated so far |
/// | `diagrams` | diagrams the card has produced so far, earlier lines included |
#[inline]
pub fn enumeration(
    done: u64,
    total: u64,
    (line, lines): (u64, u64),
    subprocesses: u64,
    diagrams: u64,
) {
    trace!(
        target: TARGET,
        stage = stage::ENUMERATE,
        done,
        total = Some(total),
        line,
        lines,
        subprocesses,
        diagrams
    );
}

/// One completed VEGAS iteration, with the estimate as it stands after it.
///
/// `sigma` and `err` are **in the integrand's own units**, which for every
/// cross-section integrand in this crate is `GeV⁻²`: a consumer that displays
/// picobarns applies [`GEV2_TO_PB`](crate::phasespace::GEV2_TO_PB) itself, exactly
/// as the result line does. `chi2` is χ² per degree of freedom over the iterations
/// combined so far, and is `None`, and left out of the event, until there are two
/// of them.
#[inline]
pub fn vegas_iteration(done: u64, total: Option<u64>, sigma: f64, err: f64, chi2: Option<f64>) {
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

    use crate::test_capture::{capture, Captured};
    use tracing::Level;

    fn captured(emit: impl FnOnce()) -> Vec<Captured> {
        capture(emit).1
    }

    /// A group scope puts its four fields on every step taken inside it, a
    /// scope inside a scope shadows it until it drops, and nothing is left
    /// behind after the outermost drops.
    #[test]
    fn a_step_in_a_group_carries_the_groups_fields() {
        use super::{enter_group, Group};
        let group = |index, label: &str, channels| Group {
            index,
            count: 2,
            label: label.to_string(),
            channels,
        };
        let events = captured(|| {
            super::step(stage::COMPILE, 0, Some(1));
            let outer = enter_group(group(1, "@1", None));
            super::step(stage::COMPILE, 1, Some(1));
            {
                let _inner = enter_group(group(2, "@2", Some(9)));
                super::step(stage::ALPHA_SURVEY, 1, Some(6));
            }
            super::step(stage::COMPILE, 1, Some(1));
            drop(outer);
            super::step(stage::COMPILE, 1, Some(1));
        });
        assert_eq!(events.len(), 5);
        fn key(e: &Captured) -> [Option<&str>; 4] {
            [
                e.field("group"),
                e.field("groups"),
                e.field("label"),
                e.field("channels"),
            ]
        }
        assert_eq!(key(&events[0]), [None; 4]);
        assert_eq!(key(&events[1]), [Some("1"), Some("2"), Some("@1"), None]);
        assert_eq!(
            key(&events[2]),
            [Some("2"), Some("2"), Some("@2"), Some("9")]
        );
        assert_eq!(key(&events[3]), key(&events[1]));
        assert_eq!(key(&events[4]), [None; 4]);
        assert_eq!(events[4].fields.len(), 3);
    }

    /// The group belongs to the thread that entered it.
    #[test]
    fn a_group_does_not_follow_work_onto_other_threads() {
        use super::{enter_group, Group};
        let _scope = enter_group(Group {
            index: 1,
            count: 1,
            label: "@1".to_string(),
            channels: None,
        });
        assert_eq!(super::in_group(|g| g.index), Some(1));
        let elsewhere = std::thread::spawn(|| super::in_group(|g| g.index))
            .join()
            .unwrap();
        assert_eq!(elsewhere, None);
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
            vegas_iteration(4, Some(12), 802.94, 3.11, Some(1.02));
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
            vegas_iteration(4, Some(12), 802.94, 3.11, Some(1.02));
            unweighting(250, 1_000);
            eval_rate(stage::VEGAS, 4, Some(12), 212.5);
        });
        assert_eq!(events[0].field("stage"), Some("vegas"));
        assert_eq!(events[0].field("sigma"), Some("802.94"));
        assert_eq!(events[0].field("err"), Some("3.11"));
        assert_eq!(events[0].field("chi2"), Some("1.02"));
        let undefined = captured(|| vegas_iteration(3, Some(12), 802.94, 3.11, None));
        assert_eq!(undefined[0].field("chi2"), None);

        assert_eq!(events[1].field("stage"), Some("unweight"));
        assert_eq!(events[1].field("accepted"), Some("250"));
        assert_eq!(events[1].field("requested"), Some("1000"));
        assert_eq!(events[1].field("done"), Some("250"));
        assert_eq!(events[1].field("total"), Some("1000"));

        assert_eq!(events[2].field("ns_per_eval"), Some("212.5"));
    }

    /// Enumeration knows its extent before the walk starts, so `total` is
    /// always recorded, beside the line it belongs to and the running counts.
    #[test]
    fn enumeration_carries_a_total_its_line_and_the_running_counts() {
        let events = captured(|| super::enumeration(40, 81, (2, 3), 5, 120));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, TARGET);
        assert_eq!(events[0].field("stage"), Some("enumerate"));
        assert_eq!(events[0].field("done"), Some("40"));
        assert_eq!(events[0].field("total"), Some("81"));
        assert_eq!(events[0].field("line"), Some("2"));
        assert_eq!(events[0].field("lines"), Some("3"));
        assert_eq!(events[0].field("subprocesses"), Some("5"));
        assert_eq!(events[0].field("diagrams"), Some("120"));
    }
}
