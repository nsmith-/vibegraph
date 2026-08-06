//! What the status display knows about the run, and how it learns it.
//!
//! Everything the footer draws lives in one [`UiState`], and two things write to
//! it: a `tracing` layer that folds each progress measurement in as it is
//! emitted, and a handful of setters the command calls with the facts it holds
//! itself — the model it loaded, the process it enumerated, how many phase-space
//! channels it built. Keeping the two apart is what lets the footer be a pure
//! function of the state: nothing in the drawing path reaches back into the run.

use std::sync::{Arc, Mutex, OnceLock};

use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use vibegraph::phasespace::GEV2_TO_PB;

use crate::logging::{LogLevel, Scope};

/// The model brief the footer shows: what was loaded, and enough of its contents
/// to tell two models with the same name apart.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ModelBrief {
    pub(crate) label: String,
    pub(crate) digest: String,
    pub(crate) particles: usize,
    pub(crate) vertices: usize,
    pub(crate) couplings: usize,
}

/// Everything the footer draws.
///
/// `done`/`total` are the position within whatever stage last reported, in that
/// stage's own unit; they are replaced wholesale by each measurement rather than
/// accumulated, because a stage that repeats a pass restarts its own count.
#[derive(Debug, Clone, Default)]
pub(crate) struct UiState {
    /// The stage that reported most recently, by its `vibegraph::progress` name.
    pub(crate) stage: Option<String>,
    pub(crate) done: u64,
    /// Absent while the reporting stage does not know its own extent.
    pub(crate) total: Option<u64>,
    /// The running cross section in picobarns, absent until an iteration has
    /// produced one.
    pub(crate) sigma_pb: Option<f64>,
    pub(crate) err_pb: f64,
    /// χ² per degree of freedom over the iterations combined so far; `0` until
    /// there are two of them.
    pub(crate) chi2: f64,
    pub(crate) ns_per_eval: Option<f64>,
    pub(crate) model: Option<ModelBrief>,
    pub(crate) process: Option<String>,
    pub(crate) channels: Option<usize>,
    /// The fraction of drawn events an accept/reject pass is expected to keep,
    /// as the frozen weight scan predicts it. Absent until a scan has run.
    pub(crate) efficiency: Option<f64>,
    /// Rotation of the logo's colour ramp, in characters.
    pub(crate) logo_phase: usize,
    /// What the log is currently showing, as the level keys have left it.
    pub(crate) level: LogLevel,
    pub(crate) scope: Scope,
    /// Whether a stop has been asked for and the run is finishing what it holds.
    pub(crate) stopping: bool,
}

/// The fields of one progress event, before they are folded into [`UiState`].
///
/// Collected separately so an event that records only some of them leaves the
/// rest of the state as it was.
#[derive(Default)]
struct Fields {
    stage: Option<String>,
    done: Option<u64>,
    total: Option<u64>,
    sigma: Option<f64>,
    err: Option<f64>,
    chi2: Option<f64>,
    ns_per_eval: Option<f64>,
}

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "stage" {
            self.stage = Some(value.to_string());
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "done" => self.done = Some(value),
            "total" => self.total = Some(value),
            _ => {}
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        match field.name() {
            "sigma" => self.sigma = Some(value),
            "err" => self.err = Some(value),
            "chi2" => self.chi2 = Some(value),
            "ns_per_eval" => self.ns_per_eval = Some(value),
            _ => {}
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

impl Fields {
    fn fold_into(self, state: &mut UiState) {
        if let Some(stage) = self.stage {
            state.stage = Some(stage);
        }
        state.done = self.done.unwrap_or(0);
        state.total = self.total;
        // The estimate is zero through the warm-up iterations that produce no
        // result, and a zero σ with a zero error would read as a measurement.
        if let Some(sigma) = self.sigma.filter(|s| *s != 0.0) {
            state.sigma_pb = Some(sigma * GEV2_TO_PB);
            state.err_pb = self.err.unwrap_or(0.0) * GEV2_TO_PB;
            state.chi2 = self.chi2.unwrap_or(0.0);
        }
        if let Some(ns) = self.ns_per_eval {
            state.ns_per_eval = Some(ns);
        }
    }
}

/// The `tracing` layer that keeps [`UiState`] current.
///
/// It is installed under a filter matching the progress target alone, so it sees
/// every measurement whatever level the visible log is set to, and sees nothing
/// else.
pub(crate) struct ProgressLayer {
    state: Arc<Mutex<UiState>>,
}

impl ProgressLayer {
    pub(crate) fn new(state: Arc<Mutex<UiState>>) -> Self {
        Self { state }
    }
}

impl<S: Subscriber> Layer<S> for ProgressLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        if let Ok(mut state) = self.state.lock() {
            fields.fold_into(&mut state);
        }
    }
}

/// The running display's state, once one is running.
///
/// A command describes itself through the setters below whether or not anything
/// is drawing; with no display installed they do nothing, which is what keeps
/// the call sites free of a mode test.
static DISPLAY: OnceLock<Arc<Mutex<UiState>>> = OnceLock::new();

pub(crate) fn install(state: &Arc<Mutex<UiState>>) {
    let _ = DISPLAY.set(Arc::clone(state));
}

fn update(edit: impl FnOnce(&mut UiState)) {
    let Some(state) = DISPLAY.get() else {
        return;
    };
    if let Ok(mut state) = state.lock() {
        edit(&mut state);
    }
}

/// The model this run resolved, and the size of what it resolved to.
pub(crate) fn describe_model(
    label: &str,
    digest: &str,
    particles: usize,
    vertices: usize,
    couplings: usize,
) {
    update(|state| {
        state.model = Some(ModelBrief {
            label: label.to_string(),
            digest: digest.to_string(),
            particles,
            vertices,
            couplings,
        });
    });
}

/// The process being integrated, as the proc card spells it.
pub(crate) fn describe_process(process: &str) {
    update(|state| state.process = Some(process.to_string()));
}

/// How many phase-space channels the integration was built over.
pub(crate) fn note_channels(channels: usize) {
    update(|state| state.channels = Some(channels));
}

/// The accept/reject efficiency the frozen weight scan predicts, as a fraction.
pub(crate) fn note_unweighting_efficiency(efficiency: f64) {
    update(|state| state.efficiency = Some(efficiency));
}

#[cfg(test)]
mod tests {
    use super::{ProgressLayer, UiState};

    use std::sync::{Arc, Mutex};

    use tracing_subscriber::layer::{Layer, SubscriberExt};
    use tracing_subscriber::Registry;
    use vibegraph::phasespace::GEV2_TO_PB;
    use vibegraph::progress;

    /// Emit through the library's own progress functions with nothing but the
    /// display's layer listening, and hand back what the display learned.
    fn folded(emit: impl FnOnce()) -> UiState {
        let state = Arc::new(Mutex::new(UiState::default()));
        let subscriber =
            Registry::default().with(ProgressLayer::new(Arc::clone(&state)).with_filter(
                tracing_subscriber::filter::filter_fn(|meta| meta.target() == progress::TARGET),
            ));
        tracing::subscriber::with_default(subscriber, emit);
        let state = state.lock().expect("state").clone();
        state
    }

    #[test]
    fn a_step_records_its_stage_and_position() {
        let state = folded(|| progress::step(progress::stage::COMPILE, 3, Some(7)));
        assert_eq!(state.stage.as_deref(), Some("compile"));
        assert_eq!(state.done, 3);
        assert_eq!(state.total, Some(7));
    }

    /// An absent total must stay absent: read as a zero it would be a division
    /// by zero, and read as `done` it would draw a finished bar over a stage
    /// that has no end in sight.
    #[test]
    fn an_unknown_total_stays_unknown() {
        let state = folded(|| progress::step(progress::stage::ENUMERATE, 12, None));
        assert_eq!(state.done, 12);
        assert_eq!(state.total, None);
    }

    /// A later measurement without a total clears the total an earlier stage
    /// reported, rather than leaving the bar showing the previous stage's extent.
    #[test]
    fn a_stage_that_knows_no_total_clears_the_previous_one() {
        let state = folded(|| {
            progress::step(progress::stage::COMPILE, 3, Some(7));
            progress::step(progress::stage::ENUMERATE, 1, None);
        });
        assert_eq!(state.stage.as_deref(), Some("enumerate"));
        assert_eq!(state.total, None);
    }

    /// The integrand reports GeV⁻²; the display is picobarns, and the
    /// conversion belongs on the ingestion side so exactly one place performs it.
    #[test]
    fn the_cross_section_is_converted_out_of_inverse_gev_squared() {
        let state = folded(|| progress::vegas_iteration(4, Some(12), 2.0e-6, 8.0e-9, 1.02));
        let sigma = state.sigma_pb.expect("a cross section");
        assert!(
            (sigma - 2.0e-6 * GEV2_TO_PB).abs() < 1e-9 * sigma.abs(),
            "{sigma}"
        );
        assert!((state.err_pb - 8.0e-9 * GEV2_TO_PB).abs() < 1e-9 * state.err_pb.abs());
        assert_eq!(state.chi2, 1.02);
    }

    /// The warm-up iterations report a zero estimate. Showing it as "0 ± 0 pb"
    /// would state a measurement nobody made.
    #[test]
    fn a_warm_up_iteration_leaves_the_cross_section_unset() {
        let state = folded(|| progress::vegas_iteration(1, Some(12), 0.0, 0.0, 0.0));
        assert_eq!(state.sigma_pb, None);
        assert_eq!(state.done, 1);
    }

    /// A result already shown is not erased by a later measurement that carries
    /// no estimate of its own.
    #[test]
    fn a_later_stage_does_not_erase_the_cross_section() {
        let state = folded(|| {
            progress::vegas_iteration(4, Some(12), 2.0e-6, 8.0e-9, 1.02);
            progress::unweighting(250, 1_000);
        });
        assert!(state.sigma_pb.is_some());
        assert_eq!(state.stage.as_deref(), Some("unweight"));
        assert_eq!(state.done, 250);
        assert_eq!(state.total, Some(1_000));
    }

    #[test]
    fn the_evaluation_cost_is_taken_from_its_own_field() {
        let state = folded(|| progress::eval_rate(progress::stage::VEGAS, 4, Some(12), 212.5));
        assert_eq!(state.ns_per_eval, Some(212.5));
    }

    /// The layer's filter is by target, so an ordinary log event at any level
    /// must leave the state untouched — otherwise a stray `done` field
    /// somewhere in the library would move the progress bar.
    #[test]
    fn an_event_on_another_target_is_ignored() {
        let state = folded(|| {
            tracing::trace!(stage = "not a stage", done = 99u64, "an ordinary event");
        });
        assert_eq!(state.stage, None);
        assert_eq!(state.done, 0);
    }
}
