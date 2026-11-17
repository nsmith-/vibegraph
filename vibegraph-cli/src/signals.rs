//! How a run answers SIGTERM and SIGINT.
//!
//! A batch system ends a job with one SIGTERM and, some minutes later, a
//! SIGKILL; a terminal sends SIGINT on `^C`. For `integrate`, the first of
//! either is the same request as the display's stop key: finish the iteration
//! in flight, keep what is complete, write the artifact and emit the result.
//! The second takes the default action, so a run that will not stop in time can
//! still be ended. `q`, `^C` and the signals share one request, so one of each
//! counts as two.
//!
//! The request is only honoured once sampling has started, because the
//! integration reads it at its iteration boundaries and nowhere earlier: a
//! signal during enumeration, compilation or the channel survey takes the
//! default action at once rather than waiting on a read that is not coming.
//!
//! The signals are taken on a thread of their own rather than in a handler.
//! The default action is re-raised with the default disposition, so the exit
//! status is the one the kill would have produced.

use std::sync::atomic::{AtomicBool, Ordering};

static SAMPLING: AtomicBool = AtomicBool::new(false);

/// Mark the integration's iteration loop as about to start, from which point
/// the first SIGTERM or SIGINT asks for a stop instead of ending the process.
pub(crate) fn sampling_started() {
    SAMPLING.store(true, Ordering::SeqCst);
}

/// Whether the first SIGTERM or SIGINT is a request to stop, as opposed to the
/// default action.
fn is_stop_request(signal: i32, graceful: bool, sampling: bool, already_requested: bool) -> bool {
    #[cfg(unix)]
    let stoppable = matches!(
        signal,
        signal_hook::consts::SIGTERM | signal_hook::consts::SIGINT
    );
    #[cfg(not(unix))]
    let stoppable = {
        let _ = signal;
        false
    };
    stoppable && graceful && sampling && !already_requested
}

/// Take SIGTERM and SIGINT, and, for a run under the display, SIGHUP and
/// SIGQUIT too, which only give the terminal back before the default action.
///
/// `graceful` is whether the command reads the stop request; with it false
/// every signal takes the default action at once, after the terminal has been
/// handed back.
#[cfg(unix)]
pub(crate) fn install(graceful: bool, display: bool) {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
    use signal_hook::iterator::Signals;

    let signals = if display {
        Signals::new([SIGTERM, SIGHUP, SIGQUIT, SIGINT])
    } else {
        Signals::new([SIGTERM, SIGINT])
    };
    let Ok(mut signals) = signals else {
        return;
    };
    let stop = crate::tui::stop_signal();
    let _ = std::thread::Builder::new()
        .name("vibegraph-signals".to_string())
        .spawn(move || {
            for signal in signals.forever() {
                if is_stop_request(
                    signal,
                    graceful,
                    SAMPLING.load(Ordering::SeqCst),
                    stop.requested(),
                ) {
                    stop.request();
                    tracing::warn!(
                        "stopping: the run is finishing the iteration in flight; \
                         a second signal ends it at once"
                    );
                    continue;
                }
                crate::tui::release_terminal_on_kill();
                let _ = signal_hook::low_level::emulate_default_handler(signal);
            }
        });
}

#[cfg(not(unix))]
pub(crate) fn install(_graceful: bool, _display: bool) {}

#[cfg(all(test, unix))]
mod tests {
    use super::is_stop_request;
    use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};

    #[test]
    fn only_the_first_term_or_int_during_sampling_asks_for_a_stop() {
        for signal in [SIGTERM, SIGINT] {
            assert!(is_stop_request(signal, true, true, false));
            assert!(!is_stop_request(signal, true, true, true), "second");
            assert!(!is_stop_request(signal, true, false, false), "not sampling");
            assert!(!is_stop_request(signal, false, true, false), "not graceful");
        }
        for signal in [SIGHUP, SIGQUIT] {
            assert!(!is_stop_request(signal, true, true, false));
        }
    }
}
