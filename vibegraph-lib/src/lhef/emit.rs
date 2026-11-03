//! Turning an accept/reject sample into the events a Les Houches file carries.
//!
//! The unweighting pass produces events whose weight is `1` almost always and
//! `w/w_max > 1` for a point that exceeded its channel's estimated maximum. A file
//! has to represent that tail somehow, and there is more than one honest way to do
//! it. [`UnweightStrategy`] is the choice, and the two implemented here differ in
//! what they put in `IDWTUP`:
//!
//! * [`Buffer`] keeps the weights, declaring `IDWTUP = -4` under the card's
//!   default `event_norm = average` — `XWGTUP` is a cross section in picobarns
//!   and the total is the *mean* of the event weights; `sum` declares `-3` and
//!   weights summing to the cross section, `unity` `-3` and weights of one. The
//!   overweight tail stays visible in the file, event by event, and each part of
//!   the sample is normalised to its integrated cross section.
//! * [`StochasticRounding`] keeps unit weights, declaring `IDWTUP = +3`, and
//!   writes an event `floor(w) + Bernoulli(frac(w))` times. The tail is
//!   represented as multiplicity instead of weight. Each part's share of the file
//!   is then the realised sample's, so it refuses a plan of several parts.
//!
//! # Why the `<init>` block decides the shape of this interface
//!
//! `<init>` precedes the first event in the file and carries `XSECUP`, `XERRUP`
//! and `XMAXUP`. A strategy whose `<init>` depends on the *realised* sample
//! therefore cannot stream: it must either hold the sample or produce it twice.
//! That is why a strategy owns the generation loop rather than filtering events
//! one at a time — the loop is where the difference lives.
//!
//! [`Buffer`] holds the sample. [`StochasticRounding`] needs nothing from it:
//! every `<init>` quantity is known before the first draw (`XSECUP` is the
//! integration's, `XMAXUP` is `1`, every weight is `1`), so it streams in a single
//! pass, needs no seekable sink and never allocates the sample.
//!
//! A third mode is possible and is not implemented here: streaming `IDWTUP = -4`
//! by *replaying* the source — measure the sample on one pass, restart, write on
//! the second. That is the only way to both stream and keep the overweight tail
//! visible in the file. [`EventSource::restart`] is the hook it needs, and the
//! same [`emit`](UnweightStrategy::emit) signature carries it.

use std::io::{self, Write};

use std::sync::Arc;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use crate::progress;

use crate::runcard::EventNorm;
use crate::systematics::plan::Writer as SystematicsWriter;
use crate::systematics::record::{LoRecord, RecordKinematics};

use super::record::{LheEvent, LheInit, LheProcess, WeightStrategy};
use super::write::{initrwgt_block, rwgt_block, LheWriter};

/// The RNG stream the stochastic-rounding draw runs on, so a strategy's coin
/// flips never share a stream with the event generation they are applied to.
const ROUNDING_STREAM: u64 = 0x0052_4E44;

/// One accepted event together with the dimensionless weight the accept/reject
/// pass gave it: `1` for an ordinary event, `w/w_max > 1` for one that exceeded
/// its channel's estimated maximum.
///
/// The record's own `XWGTUP` is not yet meaningful. The file's weight convention
/// is the strategy's to impose, and under `IDWTUP = -4` it is not even known until
/// the whole sample has been seen.
///
/// `reweights` carries the event's weight under each of the plan's reweighting
/// hypotheses as a ratio to its own ([`EmitPlan::reweights`]); it is empty when
/// the plan declares none.
#[derive(Clone, Debug)]
pub struct WeightedEvent {
    pub record: LheEvent,
    pub weight: f64,
    /// The index into [`EmitPlan::parts`] of the integrated part this event was
    /// drawn from: its phase space (final-state multiplicity, or set of
    /// outgoing masses) in a sum over several, `0` otherwise.
    pub part: usize,
    pub reweights: Vec<f64>,
    /// The event's `<mgrwt>` record, written after its particles; `None` where
    /// the run writes none. Its systematics weights are computed from it when
    /// the event is written.
    pub lo_record: Option<LoRecord>,
}

/// A deterministic, replayable sequence of accepted events.
pub trait EventSource {
    /// The next accepted event, or `None` when the source will produce no more —
    /// a trial budget exhausted, say.
    fn next_event(&mut self) -> Option<WeightedEvent>;

    /// Return the source to its initial state, so that the identical sequence
    /// follows.
    ///
    /// Implementors must reseed every stream they own and reset every accumulator
    /// they expose: a source that does not reproduce its sequence breaks the
    /// determinism the rest of the pipeline is built on, and silently invalidates
    /// any two-pass strategy.
    fn restart(&mut self);

    /// The cross section the source's own weighted estimator has accumulated over
    /// every trial it has spent — accepted *and* rejected — in picobarns.
    ///
    /// This is a property of the generation, not of the events handed out, so a
    /// strategy that declares the sample's own cross section reads it here rather
    /// than trying to rebuild it from the weights it was given.
    fn sigma_pb(&self) -> f64;
}

/// Everything about the emitted file that does not depend on which events come
/// out of the source.
#[derive(Clone, Debug)]
pub struct EmitPlan {
    /// Event records the file should carry. A strategy that writes an event more
    /// than once may overshoot this by less than one event's multiplicity, rather
    /// than truncate the last event's copies and bias it low.
    pub nevents: usize,
    /// The integration's cross section and its uncertainty, in picobarns.
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
    /// The integration's cross section per part of the sample, one entry per
    /// phase space of a sum over several, in the integrand's part
    /// order; every event names its entry by [`WeightedEvent::part`]. Empty
    /// means one part, the whole integration (`sigma_pb ± sigma_err_pb`).
    pub parts: Vec<PartSigma>,
    /// `IDBMUP`.
    pub beam_pdg: [i32; 2],
    /// `EBMUP`, in GeV.
    pub beam_energy: [f64; 2],
    /// `PDFGUP`, `0` for a beam with no parton densities.
    pub pdf_group: [i32; 2],
    /// `PDFSUP`.
    pub pdf_set: [i32; 2],
    /// `LPRUP` of every process entry, one per process number (`@N`) of the
    /// card, in the order the `<init>` block lists them. Every event's `IDPRUP`
    /// names one of them.
    pub process_ids: Vec<i32>,
    /// Lines after the process entry, `<generator>` among them.
    pub trailer: Vec<String>,
    /// Free-form provenance for the `<header>` block.
    pub header: Option<String>,
    /// Complete XML elements for the `<header>` block after the provenance
    /// comment, such as the `<MGRunCard>` a matched sample carries.
    pub header_blocks: Vec<String>,
    /// The reweighting hypotheses every event carries a weight for, as
    /// `(id, description)`: declared in `<initrwgt>`, and written per event as a
    /// `<rwgt>` block in `XWGTUP`'s own units.
    pub reweights: Vec<(String, String)>,
    /// The run card's `event_norm`: what the written `XWGTUP` are normalised to
    /// and the `IDWTUP` that says so. Only [`Buffer`] writes anything but
    /// [`EventNorm::Average`].
    pub event_norm: EventNorm,
    /// The systematics variations every event carries a weight for, ahead of
    /// the reweighting hypotheses in `<initrwgt>` and in each `<rwgt>`, as
    /// MadGraph's launch runs its systematics module before its reweighting.
    pub systematics: Option<Arc<SystematicsWriter>>,
}

/// One integrated part's cross section and its uncertainty, in picobarns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartSigma {
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
}

impl EmitPlan {
    /// The parts the sample is normalised over: [`EmitPlan::parts`], or the
    /// whole integration as the one part when that is empty.
    pub(crate) fn part_sigmas(&self) -> Vec<PartSigma> {
        if self.parts.is_empty() {
            vec![PartSigma {
                sigma_pb: self.sigma_pb,
                sigma_err_pb: self.sigma_err_pb,
            }]
        } else {
            self.parts.clone()
        }
    }
}

/// What an emission actually produced.
#[derive(Clone, Copy, Debug, Default)]
pub struct EmitSummary {
    /// Accepted events taken from the source.
    pub drawn: usize,
    /// Event records written. A strategy that duplicates overweight events writes
    /// more than it drew.
    pub written: u64,
    /// The `XSECUP` the file declares, summed over its processes, in picobarns.
    pub xsec_pb: f64,
    /// The cross section the accept/reject sample itself estimated
    /// ([`EventSource::sigma_pb`]) before any normalisation, and that estimate's
    /// statistical error (see [`sample_estimate_error`]).
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) sample_sigma_pb: f64,
    pub sample_sigma_err_pb: f64,
    /// The `XMAXUP` the file declares.
    pub xmax: f64,
    /// The mean generator weight over the events drawn — `1` when nothing went
    /// overweight, and the mean multiplicity a stochastic-rounding pass has to
    /// reproduce.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) mean_source_weight: f64,
}

#[derive(Debug)]
pub enum EmitError {
    Io(io::Error),
    /// The source ran out before the file was full.
    Exhausted {
        wanted: usize,
        drawn: usize,
    },
    /// An event names a process the plan does not declare.
    UndeclaredProcess(i32),
    /// An event names a part the plan does not declare.
    UndeclaredPart(usize),
    /// The plan normalises several integrated parts separately, which this
    /// strategy cannot do: it writes unit weights, so each part's share of the
    /// file would be the sample's own rather than its integration's.
    PartsUnsupported {
        strategy: &'static str,
        parts: usize,
    },
    /// The plan asks for an `event_norm` this strategy does not write: its
    /// `IDWTUP` is fixed by how it represents the overweight tail.
    EventNormUnsupported {
        strategy: &'static str,
        norm: EventNorm,
    },
    /// An event's systematics weights could not be computed.
    Systematics {
        event: usize,
        why: String,
    },
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitError::Io(e) => write!(f, "writing the event file: {e}"),
            EmitError::Exhausted { wanted, drawn } => write!(
                f,
                "the generator ran out of trials after {drawn} events, {wanted} were asked for"
            ),
            EmitError::UndeclaredProcess(id) => write!(
                f,
                "an event names process {id}, which the file's <init> block does not declare"
            ),
            EmitError::UndeclaredPart(k) => write!(
                f,
                "an event names integrated part {k}, which the emission plan does not declare"
            ),
            EmitError::EventNormUnsupported { strategy, norm } => write!(
                f,
                "{strategy} writes unit weights under IDWTUP = +3 and cannot honour \
                 event_norm = {}; use the buffered strategy",
                norm.as_str()
            ),
            EmitError::Systematics { event, why } => write!(
                f,
                "event {event} (counting from 1 in the order drawn): the systematics weights \
                 cannot be computed: {why}"
            ),
            EmitError::PartsUnsupported { strategy, parts } => write!(
                f,
                "{strategy} cannot normalise a sample of {parts} integrated parts (a sum over \
                 final-state phase spaces: multiplicities or sets of outgoing masses) to each \
                 part's own cross section; use the buffered strategy"
            ),
        }
    }
}

impl std::error::Error for EmitError {}

impl From<io::Error> for EmitError {
    fn from(e: io::Error) -> Self {
        EmitError::Io(e)
    }
}

/// How a weighted accept/reject sample becomes the events of a file.
pub trait UnweightStrategy {
    /// The `IDWTUP` a file written by this strategy declares under `norm`.
    fn weight_strategy(&self, norm: EventNorm) -> WeightStrategy;

    /// A one-line description of the mode, for a run's provenance header.
    fn describe(&self) -> String;

    /// Draw from `source` and write the whole file to `sink`.
    fn emit(
        &self,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
        sink: &mut dyn Write,
    ) -> Result<EmitSummary, EmitError>;
}

/// One process's share of a sample: the summed generator weight of its events
/// and the largest one among them.
#[derive(Clone, Copy, Debug, Default)]
struct ProcessShare {
    weight: f64,
    max_weight: f64,
}

/// Assemble the `<init>` block from the plan and the per-process entries a
/// strategy has resolved.
fn init_block(plan: &EmitPlan, strategy: WeightStrategy, processes: Vec<LheProcess>) -> LheInit {
    LheInit {
        beam_pdg: plan.beam_pdg,
        beam_energy: plan.beam_energy,
        pdf_group: plan.pdf_group,
        pdf_set: plan.pdf_set,
        weight_strategy: strategy,
        processes,
        trailer: plan.trailer.clone(),
        source: None,
    }
}

/// The process entries of a file whose total is `xsec_pb`, split by each process
/// id's share of the sample.
///
/// A single process takes the whole cross section and the integration's error.
/// With several, process `p` takes `XSECUP_p = σ·f_p`, `f_p` its share of the
/// sample's generator weight, and `XERRUP_p` the integration's relative error on
/// that plus the share's own sampling error over `n` events,
/// `√((f_p·Δσ)² + σ²·f_p(1 − f_p)/n)`: the process split is measured by the
/// sample, not by the integration, whose channels sum every process at once.
/// `XMAXUP_p` is `xmax` of the process's own largest weight.
fn shared_processes(
    plan: &EmitPlan,
    xsec_pb: f64,
    shares: &[ProcessShare],
    events: usize,
    xmax: impl Fn(f64) -> f64,
) -> Vec<LheProcess> {
    let total: f64 = shares.iter().map(|s| s.weight).sum();
    if plan.process_ids.len() <= 1 {
        let max_weight = shares.iter().map(|s| s.max_weight).fold(0.0f64, f64::max);
        return vec![LheProcess {
            xsec_pb,
            xerr_pb: plan.sigma_err_pb,
            xmax: xmax(max_weight),
            id: plan.process_ids.first().copied().unwrap_or(1),
        }];
    }
    plan.process_ids
        .iter()
        .zip(shares)
        .map(|(&id, share)| {
            let f = if total > 0.0 {
                share.weight / total
            } else {
                0.0
            };
            LheProcess {
                xsec_pb: xsec_pb * f,
                xerr_pb: share_variance(xsec_pb, plan.sigma_err_pb, f, events).sqrt(),
                xmax: xmax(share.max_weight),
                id,
            }
        })
        .collect()
}

/// The variance of the cross section `σ·f` a process takes as its share `f` of
/// `n` events integrated to `σ ± Δσ`: the integration's relative error on that
/// share and the share's own binomial sampling error, `(f·Δσ)² + σ²·f(1 − f)/n`.
/// A share of a sample of no events carries the integration's error alone.
fn share_variance(sigma: f64, sigma_err: f64, f: f64, events: usize) -> f64 {
    let sampling = if events > 0 {
        sigma * sigma * f * (1.0 - f) / events as f64
    } else {
        0.0
    };
    (f * sigma_err).powi(2) + sampling
}

/// The words [`Buffer`]'s header line opens with, before the sample's own
/// estimate and its error: `sample estimate before normalisation <σ̂> +- <err>`.
pub(crate) const SAMPLE_ESTIMATE_LINE: &str = "sample estimate before normalisation";

/// The sample's own estimate and its error, read back from the first header
/// line [`Buffer`] wrote them on, or `None` in a file without one.
pub fn sample_estimate_in(text: &str) -> Option<(f64, f64)> {
    let line = text
        .lines()
        .find_map(|l| l.strip_prefix(SAMPLE_ESTIMATE_LINE))?;
    let (sigma, err) = line.trim().split_once("+-")?;
    Some((sigma.trim().parse().ok()?, err.trim().parse().ok()?))
}

/// The statistical error of an accept/reject sample's own cross-section
/// estimate, from the generator weights `weights` of the events it accepted.
///
/// The estimate is `σ̂ = W·Σwᵢ/T` over `T` trials against summed maxima `W`
/// frozen before the first trial: each trial contributes `W·wᵢ` if accepted and
/// `0` if not. Its variance `(E[x²] − E[x]²)/T` is estimated by
/// `σ̂²·(Σwᵢ²/(Σwᵢ)² − 1/T)`. The `1/T` term is the unweighting efficiency times
/// smaller than the first and is dropped, which overstates the error by a
/// factor of about `1 + ε/2` — 1–2 % at the efficiencies these samples run at —
/// and leaves an expression of the accepted weights alone:
/// `σ̂·√(Σwᵢ²)/Σwᵢ`. The same holds for any subset of the events, a part's or a
/// process's, with `σ̂` that subset's share.
pub(crate) fn sample_estimate_error(sigma: f64, weights: impl IntoIterator<Item = f64>) -> f64 {
    let (sum, sum_sq) = weights
        .into_iter()
        .fold((0.0f64, 0.0f64), |(s, q), w| (s + w, q + w * w));
    estimate_error_from_sums(sigma, sum, sum_sq)
}

/// [`sample_estimate_error`] from the weights' sum `Σwᵢ` and sum of squares
/// `Σwᵢ²`, already accumulated: `σ̂·√(Σwᵢ²)/Σwᵢ`, and `0` with no weight.
fn estimate_error_from_sums(sigma: f64, sum: f64, sum_sq: f64) -> f64 {
    if sum > 0.0 {
        sigma.abs() * sum_sq.sqrt() / sum
    } else {
        0.0
    }
}

/// The `<init>` entry an event of `process_id` belongs to: the only one when the
/// plan declares a single process, else the position of its id. An id the plan
/// does not list is refused: the file would name a process its `<init>` block
/// does not declare.
fn process_slot(plan: &EmitPlan, process_id: i32) -> Result<usize, EmitError> {
    if plan.process_ids.len() <= 1 {
        return Ok(0);
    }
    plan.process_ids
        .iter()
        .position(|&id| id == process_id)
        .ok_or(EmitError::UndeclaredProcess(process_id))
}

/// Open the file: `header`, the plan's header blocks followed by its
/// `<initrwgt>` block if it declares reweighting hypotheses, and `init`.
fn begin<'w>(
    sink: &'w mut dyn Write,
    init: &LheInit,
    header: Option<&str>,
    plan: &EmitPlan,
) -> io::Result<LheWriter<&'w mut dyn Write>> {
    let mut blocks = plan.header_blocks.clone();
    let systematics = plan
        .systematics
        .as_ref()
        .map(|w| w.header_lines())
        .unwrap_or_default();
    if !plan.reweights.is_empty() || !systematics.is_empty() {
        blocks.extend(initrwgt_block(systematics, &plan.reweights));
    }
    LheWriter::begin_with_blocks(sink, init, header, &blocks)
}

/// `record` with its `XWGTUP` set to `xwgtup`, then its `<mgrwt>` record
/// where it carries one, then, when the plan declares systematics variations
/// or reweighting hypotheses, the `<rwgt>` block carrying the event's weight
/// under each: the variations first. `index` is the event's position in the
/// draw, for a refusal to name.
fn finished_record(
    event: &WeightedEvent,
    xwgtup: f64,
    plan: &EmitPlan,
    index: usize,
) -> Result<LheEvent, EmitError> {
    let mut record = event.record.clone();
    record.weight = xwgtup;
    if let Some(lo) = &event.lo_record {
        record.trailer.extend(lo.lines());
    }
    let mut ids: Vec<String> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();
    if let Some(writer) = &plan.systematics {
        let lo = event
            .lo_record
            .as_ref()
            .ok_or_else(|| EmitError::Systematics {
                event: index + 1,
                why: "the event carries no <mgrwt> record".to_string(),
            })?;
        let w = writer
            .weights(lo, &RecordKinematics::of(&record), xwgtup)
            .map_err(|why| EmitError::Systematics {
                event: index + 1,
                why,
            })?;
        ids.extend(writer.ids().iter().cloned());
        weights.extend(w);
    }
    if !plan.reweights.is_empty() {
        assert_eq!(
            event.reweights.len(),
            plan.reweights.len(),
            "an event carries one ratio per declared reweighting hypothesis"
        );
        ids.extend(plan.reweights.iter().map(|(id, _)| id.clone()));
        weights.extend(event.reweights.iter().map(|r| r * xwgtup));
    }
    record.trailer.extend(rwgt_block(&ids, &weights));
    Ok(record)
}

/// Events finished this many at a time, in parallel, before they are written
/// in order.
const FINISH_CHUNK: usize = 1024;

/// Take `n` accepted events from the source, or report how far it got.
fn draw_all(source: &mut dyn EventSource, n: usize) -> Result<Vec<WeightedEvent>, EmitError> {
    let _span = tracing::info_span!("unweight").entered();
    let mut events = Vec::with_capacity(n);
    progress::unweighting(0, n as u64);
    while events.len() < n {
        let Some(event) = source.next_event() else {
            return Err(EmitError::Exhausted {
                wanted: n,
                drawn: events.len(),
            });
        };
        events.push(event);
        progress::unweighting(events.len() as u64, n as u64);
    }
    Ok(events)
}

/// Generate the whole sample in memory, then write it with the weights it
/// carries (`IDWTUP = -4` under `event_norm = average`, the others as
/// [`EventNorm`] says), each part normalised to its integrated cross section.
///
/// The overweight events an accept/reject pass keeps above weight `1` stay in the
/// file as weights, so a consumer sees the tail rather than a smoothed version of
/// it, and `XMAXUP` is the largest weight actually written rather than a
/// prediction of it.
///
/// # Normalisation
///
/// Each part `k` of [`EmitPlan::part_sigmas`] — a final-state multiplicity of a
/// sum over several, or the whole run — has its events' weights scaled by one
/// common factor so that they contribute exactly its integrated `σₖ` to the
/// file's cross section: `Σ_{i∈k} XWGTUPᵢ / N = σₖ` over the file's `N` events,
/// which under `IDWTUP = -4` is the statement that the part's weights sum to its
/// cross section; under `sum` the `/ N` drops out. MadEvent pins each of its channels to its integral the same
/// way (`unwgt.f`, `xscale = xsecabs/xsum`); taking it at the multiplicity
/// level keeps every part normalised over at least the events of its share of
/// the sample rather than over a handful.
///
/// The rescale changes no event's weight relative to the others of its part, so
/// overweight events keep their weight (rescaled) and the shape within a part is
/// the sample's own. Nothing is truncated: truncating the overweights at unit
/// weight would bias exactly the tail they carry. The one bias is the ratio
/// estimator's, `σₖ·(nₖ events)/(their summed weight)`: an `O(1/nₖ)` relative
/// bias, `nₖ` the part's event count.
///
/// What the normalisation buys is the file's σ: the sample's own estimate
/// scatters with its binomial error (≈ 1 % at 10⁴ events and a few percent
/// efficiency) while the integration's is typically ten times smaller, and the
/// file now declares the integration's σ and error per part, `XSECUP` and
/// `XERRUP` per `LPRUP`. The sample's own estimate before the normalisation is
/// still measured and written to the header as
/// `sample estimate before normalisation <σ̂> +- <error>`, so a comparison of
/// the accept/reject pass against its integration keeps something to compare.
///
/// A part whose integration is nonzero but which drew no event cannot carry its
/// cross section and is missing from the file's total; the emission warns.
///
/// # Process entries
///
/// Process `p` declares `XSECUP_p = Σₖ σₖ·f_{pk}`, `f_{pk}` its share of part
/// `k`'s generator weight, which is exactly the written weights of its events
/// over `N`; where a process number is one part, as `@N` is one multiplicity, it
/// is that part's integrated `σₖ`. `XERRUP_p = √(Σₖ (f_{pk}Δσₖ)² + σₖ²f_{pk}(1 −
/// f_{pk})/nₖ)`: the integration's error, and the share's sampling error where a
/// part holds several processes. A single process entry takes the file's whole
/// total.
///
/// # What buffering costs
///
/// One `LheParticle` is 80 bytes and one `LheEvent` 88 inline plus 80 per leg on
/// the heap, so a held sample runs at roughly 424 B/event for a `2 → 2`, 504 B for
/// a `2 → 3` and 850 B for a `2 → 7` — about 42 MB for 100 000 `2 → 2` events.
/// 100 000 is the traditional ceiling for a single MadGraph sampling run, which is
/// why holding the sample is an acceptable default; a run far past it wants a
/// streaming strategy instead.
#[derive(Clone, Copy, Debug, Default)]
pub struct Buffer;

/// A part's events and how they normalise.
#[derive(Clone, Copy, Debug, Default)]
struct PartTally {
    /// Summed generator weight, and summed squared weight.
    weight: f64,
    weight_sq: f64,
    events: usize,
    /// The factor each of the part's generator weights is multiplied by to give
    /// its `XWGTUP`.
    scale: f64,
}

impl UnweightStrategy for Buffer {
    fn weight_strategy(&self, norm: EventNorm) -> WeightStrategy {
        norm_rule(norm).idwtup
    }

    fn describe(&self) -> String {
        "buffered (event_norm = average: IDWTUP = -4, XWGTUP in pb, sigma = mean weight; sum: \
         IDWTUP = -3, sigma = summed weight; unity: IDWTUP = -3, every XWGTUP = 1; each part's \
         weights normalised to its integration)"
            .to_string()
    }

    fn emit(
        &self,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
        sink: &mut dyn Write,
    ) -> Result<EmitSummary, EmitError> {
        let events = draw_all(source, plan.nevents)?;
        let n = events.len();
        let parts = plan.part_sigmas();
        let (tally, by_process) = tally_parts(plan, &events, &parts)?;
        let norm = plan.event_norm;

        let sample_sigma_pb = source.sigma_pb();
        let total_weight: f64 = tally.iter().map(|t| t.weight).sum();
        let sample_sigma_err_pb =
            sample_estimate_error(sample_sigma_pb, events.iter().map(|e| e.weight));
        let header = buffered_header(plan, &tally, &parts, sample_sigma_pb, sample_sigma_err_pb);

        let processes = buffered_processes(plan, &events, &tally, &parts, &by_process)?;
        let xsec_pb: f64 = processes.iter().map(|p| p.xsec_pb).sum();
        let xmax = processes.iter().map(|p| p.xmax).fold(0.0f64, f64::max);

        let init = init_block(plan, self.weight_strategy(norm), processes);
        let mut writer = begin(sink, &init, Some(&header), plan)?;
        for (c, chunk) in events.chunks(FINISH_CHUNK).enumerate() {
            let finished: Vec<LheEvent> = chunk
                .par_iter()
                .enumerate()
                .map(|(i, event)| {
                    let xwgtup = written_weight(norm, tally[event.part].scale, event.weight);
                    finished_record(event, xwgtup, plan, c * FINISH_CHUNK + i)
                })
                .collect::<Result<_, _>>()?;
            for record in &finished {
                writer.write_event(record)?;
            }
        }
        let written = writer.events_written();
        writer.finish()?;

        Ok(EmitSummary {
            drawn: n,
            written,
            xsec_pb,
            sample_sigma_pb,
            sample_sigma_err_pb,
            xmax,
            mean_source_weight: if n > 0 { total_weight / n as f64 } else { 0.0 },
        })
    }
}

/// What an `event_norm` makes [`Buffer`] write: the one place the setting is
/// decided.
///
/// MadGraph's rule is `MultiEventFile.unweight` (`lhe_parser.py:1225-1239`) and
/// `EventFile.unweight` (`lhe_parser.py:519-527`, `lha_stra = -1*abs(strategy)`
/// with the sign of the strategy kept): `average` weights sum to `N` times the
/// cross section and are declared `IDWTUP = -4`; `sum` weights sum to the cross
/// section (`-3`); `unity` writes weight one (`-3`).
#[derive(Clone, Copy)]
struct NormRule {
    /// The `IDWTUP` the file declares.
    idwtup: WeightStrategy,
    /// Whether a part's weights sum to its `σₖ` (`sum`), and not to `N σₖ` over
    /// the file's `N` events (the mean is then `σₖ`).
    sums_to_sigma: bool,
    /// Whether every event is written at weight one, an overweight event keeping
    /// no weight above the rest.
    flatten: bool,
}

fn norm_rule(norm: EventNorm) -> NormRule {
    match norm {
        EventNorm::Average => NormRule {
            idwtup: WeightStrategy::MeanCrossSectionPb,
            sums_to_sigma: false,
            flatten: false,
        },
        EventNorm::Sum => NormRule {
            idwtup: WeightStrategy::SumCrossSectionPb,
            sums_to_sigma: true,
            flatten: false,
        },
        EventNorm::Unity => NormRule {
            idwtup: WeightStrategy::SumCrossSectionPb,
            sums_to_sigma: false,
            flatten: true,
        },
    }
}

/// The `XWGTUP` of an event of generator weight `w` in a part whose weights are
/// scaled by `scale`, under `norm`.
fn written_weight(norm: EventNorm, scale: f64, w: f64) -> f64 {
    if norm_rule(norm).flatten {
        1.0
    } else {
        scale * w
    }
}

/// Each part's tally of `events`, its normalising scale set, and the generator
/// weight per `(process slot, part)`. An event naming a part or a process the
/// plan does not declare is refused.
fn tally_parts(
    plan: &EmitPlan,
    events: &[WeightedEvent],
    parts: &[PartSigma],
) -> Result<(Vec<PartTally>, Vec<Vec<f64>>), EmitError> {
    let slots = plan.process_ids.len().max(1);
    let mut tally = vec![PartTally::default(); parts.len()];
    let mut by_process = vec![vec![0.0f64; parts.len()]; slots];
    for event in events {
        let t = tally
            .get_mut(event.part)
            .ok_or(EmitError::UndeclaredPart(event.part))?;
        t.weight += event.weight;
        t.weight_sq += event.weight * event.weight;
        t.events += 1;
        by_process[process_slot(plan, event.record.process_id)?][event.part] += event.weight;
    }
    let n = events.len();
    for (t, part) in tally.iter_mut().zip(parts) {
        // Averaged weights have the part contribute `σₖ` as their mean over the
        // file's `n` events, and summed weights as their sum, `n` times smaller
        // each.
        let events_per_weight = if norm_rule(plan.event_norm).sums_to_sigma {
            1.0
        } else {
            n as f64
        };
        t.scale = if t.weight > 0.0 {
            part.sigma_pb * events_per_weight / t.weight
        } else {
            0.0
        };
    }
    Ok((tally, by_process))
}

/// [`Buffer`]'s `<header>` text: the plan's own header, the sample's estimate
/// before normalisation, and with several parts each part's estimate against
/// the integration it is normalised to. Each part's comparison is also logged,
/// with a warning for a part that drew no event and so is missing from the
/// file's total.
fn buffered_header(
    plan: &EmitPlan,
    tally: &[PartTally],
    parts: &[PartSigma],
    sample_sigma_pb: f64,
    sample_sigma_err_pb: f64,
) -> String {
    let total_weight: f64 = tally.iter().map(|t| t.weight).sum();
    let mut header = plan.header.clone().map(|h| h + "\n").unwrap_or_default();
    header += &format!("{SAMPLE_ESTIMATE_LINE} {sample_sigma_pb:.6e} +- {sample_sigma_err_pb:.6e}");
    for (k, (t, part)) in tally.iter().zip(parts).enumerate() {
        let own = if total_weight > 0.0 {
            sample_sigma_pb * t.weight / total_weight
        } else {
            0.0
        };
        let own_err = estimate_error_from_sums(own, t.weight, t.weight_sq);
        if parts.len() > 1 {
            header += &format!(
                "\npart {k}: {} events, sample estimate before normalisation {own:.6e} +- \
                 {own_err:.6e}, normalised to {:.6e} +- {:.6e}",
                t.events, part.sigma_pb, part.sigma_err_pb
            );
        }
        tracing::info!(
            "part {k}: {} events, sample estimate {own:.6e} ± {own_err:.6e} against the \
             integration's {:.6e} ± {:.6e} ({:+.3}%)",
            t.events,
            part.sigma_pb,
            part.sigma_err_pb,
            if part.sigma_pb != 0.0 {
                100.0 * (own / part.sigma_pb - 1.0)
            } else {
                0.0
            }
        );
        if t.events == 0 && part.sigma_pb != 0.0 {
            tracing::warn!(
                "part {k} drew no event, so its integrated {:.6e} is missing from the file",
                part.sigma_pb
            );
        }
    }
    header
}

/// [`Buffer`]'s process entries (see its "Process entries" section): each
/// process's share of every part's integrated cross section and error, and as
/// its `XMAXUP` the largest weight it writes.
fn buffered_processes(
    plan: &EmitPlan,
    events: &[WeightedEvent],
    tally: &[PartTally],
    parts: &[PartSigma],
    by_process: &[Vec<f64>],
) -> Result<Vec<LheProcess>, EmitError> {
    let mut max_written = vec![0.0f64; by_process.len()];
    for event in events {
        let slot = process_slot(plan, event.record.process_id)?;
        let w = written_weight(plan.event_norm, tally[event.part].scale, event.weight);
        max_written[slot] = max_written[slot].max(w);
    }
    Ok(by_process
        .iter()
        .zip(max_written)
        .enumerate()
        .map(|(p, (weights, xmax))| {
            let (mut xsec, mut var) = (0.0f64, 0.0f64);
            for ((t, part), &w) in tally.iter().zip(parts).zip(weights) {
                if t.weight <= 0.0 {
                    continue;
                }
                let f = w / t.weight;
                if f > 0.0 {
                    xsec += part.sigma_pb * f;
                    var += share_variance(part.sigma_pb, part.sigma_err_pb, f, t.events);
                }
            }
            LheProcess {
                xsec_pb: xsec,
                xerr_pb: var.sqrt(),
                xmax,
                id: plan.process_ids.get(p).copied().unwrap_or(1),
            }
        })
        .collect())
}

/// How many copies of an event of weight `w` a stochastic-rounding pass writes:
/// `floor(w)` certainly, plus one more with probability `frac(w)`.
///
/// The mean is exactly `w`, so the multiplicity carries the weight without
/// distorting the cross section, and the variance is `f(1−f) ≤ ¼` with
/// `f = frac(w)` — strictly below the `Var = w` of a Poisson draw with the same
/// mean, and exactly zero on the integer weights that make up almost the whole
/// sample. For `w ≤ 1` it degenerates to plain accept/reject, so it changes only
/// the overweight tail.
pub(crate) fn stochastic_multiplicity(weight: f64, rng: &mut impl Rng) -> u64 {
    if !(weight > 0.0) {
        return 0;
    }
    let whole = weight.floor();
    let frac = weight - whole;
    let mut copies = whole as u64;
    if frac > 0.0 && rng.random::<f64>() < frac {
        copies += 1;
    }
    copies
}

/// Write each event `floor(w) + Bernoulli(frac(w))` times at unit weight
/// (`IDWTUP = +3`).
///
/// Every `<init>` quantity is known before the first draw — `XSECUP` is the
/// integration's, `XMAXUP` is `1`, every `XWGTUP` is `1` — so this is a single
/// streaming pass over the source with no buffer, no second pass and no need for a
/// seekable sink.
///
/// The multiplicity is drawn from a stream of its own off `seed`, so the same
/// seed reproduces the same file and the coin flips never disturb the generator's
/// own stream.
#[derive(Clone, Copy, Debug)]
pub struct StochasticRounding {
    pub(crate) seed: u64,
}

impl StochasticRounding {
    pub fn new(seed: u64) -> Self {
        StochasticRounding { seed }
    }

    /// Whether the strategy can write a file under `norm`: its unit weights are
    /// declared `IDWTUP = +3`, which is no `event_norm` but its own, so only the
    /// card default `average` is honoured. The emitter and the command line both
    /// refuse on this.
    pub fn honours(norm: EventNorm) -> bool {
        norm == EventNorm::Average
    }

    /// Run the pass without writing, returning each process's written-event count
    /// as its share and the total written.
    fn count_shares(
        &self,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
    ) -> Result<(Vec<ProcessShare>, usize), EmitError> {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        rng.set_stream(ROUNDING_STREAM);
        let mut shares = vec![ProcessShare::default(); plan.process_ids.len()];
        let mut written = 0usize;
        let mut drawn = 0usize;
        while written < plan.nevents {
            let Some(event) = source.next_event() else {
                return Err(EmitError::Exhausted {
                    wanted: plan.nevents,
                    drawn,
                });
            };
            drawn += 1;
            let copies = stochastic_multiplicity(event.weight, &mut rng);
            written += copies as usize;
            if copies > 0 {
                // Every written copy carries unit weight, so a process's largest
                // written weight is one.
                let slot = process_slot(plan, event.record.process_id)?;
                shares[slot].weight += copies as f64;
                shares[slot].max_weight = 1.0;
            }
        }
        Ok((shares, written))
    }
}

impl UnweightStrategy for StochasticRounding {
    fn weight_strategy(&self, _norm: EventNorm) -> WeightStrategy {
        WeightStrategy::UnitWeight
    }

    fn describe(&self) -> String {
        format!(
            "streaming stochastic rounding, IDWTUP = +3 (unit weights, seed {})",
            self.seed
        )
    }

    fn emit(
        &self,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
        sink: &mut dyn Write,
    ) -> Result<EmitSummary, EmitError> {
        if !Self::honours(plan.event_norm) {
            return Err(EmitError::EventNormUnsupported {
                strategy: "stochastic rounding",
                norm: plan.event_norm,
            });
        }
        // Unit weights leave each part's share of the file to the realised sample,
        // so a sum over phase spaces would not carry each part's integration.
        if plan.parts.len() > 1 {
            return Err(EmitError::PartsUnsupported {
                strategy: "stochastic rounding",
                parts: plan.parts.len(),
            });
        }
        // Several processes need their shares in `<init>` before the first event
        // is written, so the sample is drawn once to count them and then again, the
        // source restarted and the rounding stream reseeded, to write it: the two
        // passes are the same sequence, so the shares are the file's own.
        let (shares, counted) = if plan.process_ids.len() > 1 {
            let counted = self.count_shares(source, plan)?;
            source.restart();
            counted
        } else {
            (vec![ProcessShare::default()], 0)
        };
        let init = init_block(
            plan,
            self.weight_strategy(plan.event_norm),
            shared_processes(plan, plan.sigma_pb, &shares, counted, |_| 1.0),
        );
        let mut writer = begin(sink, &init, plan.header.as_deref(), plan)?;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        rng.set_stream(ROUNDING_STREAM);

        let _span = tracing::info_span!("unweight").entered();
        let mut drawn = 0usize;
        let mut weight_total = 0.0f64;
        let mut weight_sq = 0.0f64;
        progress::unweighting(0, plan.nevents as u64);
        while writer.events_written() < plan.nevents as u64 {
            let Some(event) = source.next_event() else {
                return Err(EmitError::Exhausted {
                    wanted: plan.nevents,
                    drawn,
                });
            };
            drawn += 1;
            weight_total += event.weight;
            weight_sq += event.weight * event.weight;
            // Every copy of an event is written before the loop re-checks the
            // budget: truncating an event's copies mid-way would bias exactly the
            // overweight tail this strategy exists to represent.
            let copies = stochastic_multiplicity(event.weight, &mut rng);
            let record = finished_record(&event, 1.0, plan, drawn - 1)?;
            for _ in 0..copies {
                writer.write_event(&record)?;
            }
            progress::unweighting(writer.events_written(), plan.nevents as u64);
        }
        let written = writer.events_written();
        writer.finish()?;

        let sample_sigma_pb = source.sigma_pb();
        Ok(EmitSummary {
            drawn,
            written,
            xsec_pb: plan.sigma_pb,
            sample_sigma_pb,
            sample_sigma_err_pb: estimate_error_from_sums(sample_sigma_pb, weight_total, weight_sq),
            xmax: 1.0,
            mean_source_weight: if drawn > 0 {
                weight_total / drawn as f64
            } else {
                0.0
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhef::parse::LheFile;
    use crate::lhef::record::{LheParticle, STATUS_INCOMING, STATUS_OUTGOING};

    /// A source with a fixed weight sequence and a fixed pretend cross section, so
    /// a strategy can be checked against a sample whose every property is known in
    /// closed form.
    struct FixedWeights {
        weights: Vec<f64>,
        next: usize,
        sigma_pb: f64,
    }

    impl FixedWeights {
        fn new(weights: Vec<f64>, sigma_pb: f64) -> Self {
            FixedWeights {
                weights,
                next: 0,
                sigma_pb,
            }
        }
    }

    fn record(index: usize) -> LheEvent {
        let leg = |status, pz: f64| LheParticle {
            pdg: 13,
            status,
            mothers: if status == STATUS_INCOMING {
                [0, 0]
            } else {
                [1, 2]
            },
            color: [0, 0],
            momentum: [50.0, 0.0, index as f64, pz],
            mass: 0.0,
            lifetime: 0.0,
            spin: 1.0,
        };
        LheEvent {
            process_id: 1,
            weight: f64::NAN,
            scale: 91.188,
            alpha_qed: 0.0075,
            alpha_qcd: 0.0,
            particles: vec![
                leg(STATUS_INCOMING, 50.0),
                leg(STATUS_INCOMING, -50.0),
                leg(STATUS_OUTGOING, 30.0),
                leg(STATUS_OUTGOING, -30.0),
            ],
            trailer: Vec::new(),
            source: None,
        }
    }

    impl EventSource for FixedWeights {
        fn next_event(&mut self) -> Option<WeightedEvent> {
            let weight = *self.weights.get(self.next % self.weights.len())?;
            let event = WeightedEvent {
                record: record(self.next),
                weight,
                part: 0,
                reweights: Vec::new(),
                lo_record: None,
            };
            self.next += 1;
            Some(event)
        }
        fn restart(&mut self) {
            self.next = 0;
        }
        fn sigma_pb(&self) -> f64 {
            self.sigma_pb
        }
    }

    fn plan(nevents: usize, sigma_pb: f64) -> EmitPlan {
        EmitPlan {
            nevents,
            sigma_pb,
            sigma_err_pb: 0.01 * sigma_pb,
            parts: Vec::new(),
            beam_pdg: [-11, 11],
            beam_energy: [45.6, 45.6],
            pdf_group: [0, 0],
            pdf_set: [0, 0],
            process_ids: vec![1],
            trailer: Vec::new(),
            header: None,
            header_blocks: Vec::new(),
            reweights: Vec::new(),
            event_norm: EventNorm::Average,
            systematics: None,
        }
    }

    fn emit_to_string(
        strategy: &dyn UnweightStrategy,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
    ) -> (String, EmitSummary) {
        let mut out = Vec::new();
        let summary = strategy.emit(source, plan, &mut out).expect("emit");
        (String::from_utf8(out).expect("ASCII"), summary)
    }

    /// `IDWTUP = -4` promises a consumer that the cross section is the mean of the
    /// event weights. That has to hold of the bytes, not of the intent, so it is
    /// read back out of the parsed file.
    #[test]
    fn buffered_weights_have_the_cross_section_as_their_mean() {
        let weights = vec![1.0, 1.0, 1.0, 2.5, 1.0, 1.0, 8.4, 1.0];
        let sigma = 17.25;
        let mut source = FixedWeights::new(weights.clone(), sigma);
        let plan = plan(weights.len(), sigma);
        let (text, summary) = emit_to_string(&Buffer, &mut source, &plan);

        let file = LheFile::parse(&text).expect("our own file parses");
        assert_eq!(
            file.init.weight_strategy,
            WeightStrategy::MeanCrossSectionPb
        );
        assert_eq!(file.events.len(), weights.len());
        let mean: f64 =
            file.events.iter().map(|e| e.weight).sum::<f64>() / file.events.len() as f64;
        assert!(
            (mean / sigma - 1.0).abs() < 1e-7,
            "mean XWGTUP {mean} vs sigma {sigma}"
        );
        assert!((file.init.processes[0].xsec_pb / sigma - 1.0).abs() < 1e-6);

        // XMAXUP has to bound what was written, and the overweight event has to
        // still be recognisable as one: a strategy that clipped the tail would
        // leave XMAXUP at the ordinary weight.
        let largest = file.events.iter().map(|e| e.weight).fold(0.0f64, f64::max);
        assert!((summary.xmax / largest - 1.0).abs() < 1e-6);
        assert!(
            largest > 8.0 * mean / summary.mean_source_weight,
            "the 8.4x event did not survive into the file"
        );
    }

    /// The three `event_norm` values on one overweight-carrying sample: the
    /// `IDWTUP`, the weights and their relation to the integration's σ.
    ///
    /// `average` is the baseline (mean `XWGTUP` = σ); `sum` is the same weights
    /// divided by the event count (Σ `XWGTUP` = σ); `unity` writes one for every
    /// event. Under all three `XSECUP` is σ. A `sum` file that kept the `average`
    /// weights fails the sum and the ratio; one that flattened them fails the
    /// ratio; an `average` file that divided by the count fails the mean.
    #[test]
    fn event_norm_sets_the_idwtup_and_the_weights() {
        let weights = vec![1.0, 1.0, 1.0, 2.5, 1.0, 1.0, 8.4, 1.0];
        let sigma = 17.25;
        let written = |norm: EventNorm| {
            let mut source = FixedWeights::new(weights.clone(), sigma);
            let mut plan = plan(weights.len(), sigma);
            plan.event_norm = norm;
            let (text, summary) = emit_to_string(&Buffer, &mut source, &plan);
            (LheFile::parse(&text).expect("our own file parses"), summary)
        };
        let n = weights.len() as f64;
        let (average, _) = written(EventNorm::Average);
        let (sum, sum_summary) = written(EventNorm::Sum);
        let (unity, unity_summary) = written(EventNorm::Unity);

        assert_eq!(
            average.init.weight_strategy,
            WeightStrategy::MeanCrossSectionPb
        );
        assert_eq!(sum.init.weight_strategy, WeightStrategy::SumCrossSectionPb);
        assert_eq!(
            unity.init.weight_strategy,
            WeightStrategy::SumCrossSectionPb
        );

        let mean: f64 = average.events.iter().map(|e| e.weight).sum::<f64>() / n;
        assert!((mean / sigma - 1.0).abs() < 1e-6, "average mean {mean}");
        let total: f64 = sum.events.iter().map(|e| e.weight).sum();
        assert!((total / sigma - 1.0).abs() < 1e-6, "sum total {total}");
        for (a, s) in average.events.iter().zip(&sum.events) {
            assert!(
                (s.weight * n / a.weight - 1.0).abs() < 1e-6,
                "sum weight {} is not the average weight {} over {n}",
                s.weight,
                a.weight
            );
        }
        assert!(unity.events.iter().all(|e| e.weight == 1.0));

        for file in [&average, &sum, &unity] {
            assert!((file.init.processes[0].xsec_pb / sigma - 1.0).abs() < 1e-6);
        }
        let largest = |file: &LheFile| file.events.iter().map(|e| e.weight).fold(0.0f64, f64::max);
        assert!((sum.init.processes[0].xmax / largest(&sum) - 1.0).abs() < 1e-6);
        assert!((sum_summary.xmax / largest(&sum) - 1.0).abs() < 1e-6);
        assert_eq!(unity.init.processes[0].xmax, 1.0);
        assert_eq!(unity_summary.xmax, 1.0);
    }

    /// Stochastic rounding's `IDWTUP = +3` is no `event_norm`, so it refuses `sum`
    /// and `unity` instead of writing the unit-weight file under them.
    #[test]
    fn stochastic_rounding_refuses_an_event_norm_it_cannot_write() {
        for norm in [EventNorm::Sum, EventNorm::Unity] {
            let mut source = FixedWeights::new(vec![1.0], 1.0);
            let mut plan = plan(10, 1.0);
            plan.event_norm = norm;
            let err = StochasticRounding::new(1)
                .emit(&mut source, &plan, &mut Vec::new())
                .expect_err("unit weights cannot be normalised to the sum");
            assert!(
                matches!(err, EmitError::EventNormUnsupported { .. }),
                "{err}"
            );
        }
    }

    /// Stochastic rounding writes unit weights and pays for the tail in copies.
    /// The mean multiplicity is the mean weight, which is the property that keeps
    /// the sample unbiased.
    #[test]
    fn stochastic_rounding_writes_unit_weights_with_an_unbiased_multiplicity() {
        let weights = vec![1.0, 1.0, 1.0, 2.5, 1.0, 1.0, 8.4, 1.0];
        let mean_weight = weights.iter().sum::<f64>() / weights.len() as f64;
        let sigma = 17.25;
        let mut source = FixedWeights::new(weights, sigma);
        let plan = plan(40_000, sigma);
        let (text, summary) = emit_to_string(&StochasticRounding::new(4242), &mut source, &plan);

        let file = LheFile::parse(&text).expect("our own file parses");
        assert_eq!(file.init.weight_strategy, WeightStrategy::UnitWeight);
        assert_eq!(file.init.processes[0].xmax, 1.0);
        assert!(file.events.iter().all(|e| e.weight == 1.0));
        assert!((file.init.processes[0].xsec_pb / sigma - 1.0).abs() < 1e-6);
        assert_eq!(file.events.len() as u64, summary.written);

        // The only randomness is on the two fractional weights, so the observed
        // multiplicity converges on the mean weight; 40k events is far more than
        // enough for a 1% band.
        let observed = summary.written as f64 / summary.drawn as f64;
        assert!(
            (observed / mean_weight - 1.0).abs() < 0.01,
            "mean multiplicity {observed:.5} vs mean weight {mean_weight:.5}"
        );
        assert!(summary.written >= plan.nevents as u64);
    }

    /// The rounding rule is a claim about a distribution, not a formatting
    /// choice: each of the plausible alternatives has to move the mean, and the
    /// variance has to be the one that makes rounding preferable to a Poisson
    /// draw.
    #[test]
    fn the_rounding_rule_is_not_free() {
        let weight = 1.3_f64;
        let mut rng = ChaCha8Rng::seed_from_u64(99);
        let n = 200_000;
        let draws: Vec<u64> = (0..n)
            .map(|_| stochastic_multiplicity(weight, &mut rng))
            .collect();
        let mean = draws.iter().sum::<u64>() as f64 / n as f64;
        let variance = draws
            .iter()
            .map(|&k| (k as f64 - mean).powi(2))
            .sum::<f64>()
            / (n - 1) as f64;

        assert!((mean / weight - 1.0).abs() < 0.005, "mean {mean}");
        // Every alternative rule a reader might expect gives a different mean.
        assert!((mean - weight.floor()).abs() > 0.2, "floor would give 1");
        assert!((mean - weight.ceil()).abs() > 0.6, "ceil would give 2");
        assert!((mean - weight.round()).abs() > 0.2, "round would give 1");

        // Var = f(1-f) with f = 0.3, and strictly below the Poisson variance `w`
        // of the resampling this rule replaces.
        let f = weight - weight.floor();
        assert!(
            (variance - f * (1.0 - f)).abs() < 0.01,
            "variance {variance} vs f(1-f) = {}",
            f * (1.0 - f)
        );
        assert!(
            variance < 0.5 * weight,
            "stochastic rounding must beat Poisson's Var = w = {weight}"
        );
        // An integer weight costs no variance at all, which is why the rule is
        // free on the part of the sample that is not overweight.
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        assert!((0..1000).all(|_| stochastic_multiplicity(3.0, &mut rng) == 3));
        assert!((0..1000).all(|_| stochastic_multiplicity(1.0, &mut rng) == 1));
    }

    /// Same seed, same file — for both strategies, and through a restart of the
    /// source rather than a fresh one, since that is the path a two-pass strategy
    /// would take.
    #[test]
    fn a_seed_determines_the_file() {
        let weights = vec![1.0, 1.7, 1.0, 3.2, 1.0];
        let plan = plan(500, 4.0);
        for strategy in [
            Box::new(Buffer) as Box<dyn UnweightStrategy>,
            Box::new(StochasticRounding::new(7)),
        ] {
            let mut source = FixedWeights::new(weights.clone(), 4.0);
            let (first, _) = emit_to_string(strategy.as_ref(), &mut source, &plan);
            source.restart();
            let (second, _) = emit_to_string(strategy.as_ref(), &mut source, &plan);
            assert_eq!(first, second, "{} is not reproducible", strategy.describe());
        }
    }

    /// Each event's `<rwgt>` block carries its ratios times the `XWGTUP` the
    /// strategy wrote, so a reweighted cross section is read off a file exactly as
    /// the nominal one is. Checked on the parsed bytes, under both conventions.
    #[test]
    fn reweights_are_written_in_the_files_weight_units() {
        struct Reweighted(FixedWeights);
        impl EventSource for Reweighted {
            fn next_event(&mut self) -> Option<WeightedEvent> {
                let k = self.0.next;
                let mut event = self.0.next_event()?;
                event.reweights = vec![0.5, 2.0 + k as f64];
                Some(event)
            }
            fn restart(&mut self) {
                self.0.restart();
            }
            fn sigma_pb(&self) -> f64 {
                self.0.sigma_pb()
            }
        }
        let weights = vec![1.0, 2.5, 1.0];
        let mut plan = plan(weights.len(), 6.0);
        plan.reweights = vec![
            ("half".to_string(), "set a 1".to_string()),
            ("rising".to_string(), "set a 2".to_string()),
        ];
        for strategy in [
            Box::new(Buffer) as Box<dyn UnweightStrategy>,
            Box::new(StochasticRounding::new(3)),
        ] {
            let mut source = Reweighted(FixedWeights::new(weights.clone(), 6.0));
            let (text, _) = emit_to_string(strategy.as_ref(), &mut source, &plan);
            assert!(
                text.contains("<weight id='rising'> set a 2 </weight>"),
                "{text}"
            );
            let file = LheFile::parse(&text).unwrap();
            assert!(!file.events.is_empty());
            for event in &file.events {
                let wgt: Vec<f64> = event
                    .trailer
                    .iter()
                    .filter_map(|l| l.strip_prefix("<wgt id='"))
                    .map(|l| {
                        let value = l.split("> ").nth(1).unwrap();
                        value.trim_end_matches(" </wgt>").parse().unwrap()
                    })
                    .collect();
                assert_eq!(wgt.len(), 2, "{:?}", event.trailer);
                let rel = |a: f64, b: f64| ((a - b) / b).abs() < 1e-7;
                assert!(
                    rel(wgt[0], 0.5 * event.weight),
                    "{wgt:?} vs {}",
                    event.weight
                );
                assert!(wgt[1] >= 2.0 * event.weight * (1.0 - 1e-7));
            }
        }
    }

    /// A source that cannot fill the file says so rather than writing a short one.
    #[test]
    fn a_short_source_is_an_error_not_a_short_file() {
        struct Finite(usize);
        impl EventSource for Finite {
            fn next_event(&mut self) -> Option<WeightedEvent> {
                if self.0 == 0 {
                    return None;
                }
                self.0 -= 1;
                Some(WeightedEvent {
                    record: record(0),
                    weight: 1.0,
                    part: 0,
                    reweights: Vec::new(),
                    lo_record: None,
                })
            }
            fn restart(&mut self) {}
            fn sigma_pb(&self) -> f64 {
                1.0
            }
        }
        let plan = plan(10, 1.0);
        for strategy in [
            Box::new(Buffer) as Box<dyn UnweightStrategy>,
            Box::new(StochasticRounding::new(1)),
        ] {
            let mut out = Vec::new();
            let err = strategy
                .emit(&mut Finite(3), &plan, &mut out)
                .expect_err("a short source must be refused");
            assert!(matches!(err, EmitError::Exhausted { wanted: 10, .. }));
        }
    }

    /// A source whose `k`-th event belongs to process `ids[k % ids.len()]`.
    struct TwoProcesses {
        inner: FixedWeights,
        ids: Vec<i32>,
    }

    impl EventSource for TwoProcesses {
        fn next_event(&mut self) -> Option<WeightedEvent> {
            let k = self.inner.next;
            let mut event = self.inner.next_event()?;
            event.record.process_id = self.ids[k % self.ids.len()];
            Some(event)
        }
        fn restart(&mut self) {
            self.inner.restart();
        }
        fn sigma_pb(&self) -> f64 {
            self.inner.sigma_pb()
        }
    }

    /// MadEvent's own `<init>` under `event_norm = sum`, banked in the
    /// decay-chain event references (their run cards never set `event_norm`, so
    /// MadGraph resolves it to its system default `sum`): `IDWTUP = -3`, every
    /// process line's `XMAXUP` the weight each event carries, and that weight the
    /// summed `XSECUP` over the sample's event count (`MultiEventFile.unweight`,
    /// `lhe_parser.py:1225-1228`, `new_wgt = sum(self.across)/event_target`).
    ///
    /// The same cross sections and event count through [`Buffer`] under
    /// [`EventNorm::Sum`] give the same `IDWTUP`, the same total `XSECUP`, and a
    /// mean `XWGTUP` that is MadEvent's `XMAXUP`. The mean and not each weight:
    /// MadGraph writes one weight to every event, flattening the overweight tail
    /// that this writer keeps, so a single event's weight is not comparable but
    /// the sum is. What this cannot see is a per-event `XWGTUP` of a MadEvent
    /// file: the reference keeps the `<init>` lines and the event summaries, not
    /// the event weights, so `XMAXUP` is the weight MadGraph's rule assigns and
    /// not a weight read off an `<event>`.
    #[test]
    fn a_sum_file_carries_madevents_idwtup_and_event_weight() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/decay_chain_events_reference.json"
        );
        let text = std::fs::read_to_string(path).expect("the committed decay-chain reference");
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap();
        let rows = dump["rows"].as_object().expect("rows");
        assert!(rows.len() >= 5, "the reference lost rows");
        for (name, row) in rows {
            let lines: Vec<Vec<&str>> = row["init"]
                .as_array()
                .expect("init")
                .iter()
                .map(|l| l.as_str().unwrap().split_whitespace().collect())
                .collect();
            let banked_idwtup: i32 = lines[0][8].parse().unwrap();
            let nevents = row["runs"][0]["events"].as_u64().unwrap() as usize;
            let processes: Vec<(f64, f64, i32)> = lines[1..]
                .iter()
                .map(|l| {
                    (
                        l[0].parse().unwrap(),
                        l[2].parse().unwrap(),
                        l[3].parse().unwrap(),
                    )
                })
                .collect();
            let total: f64 = processes.iter().map(|p| p.0).sum();
            for (_, xmax, id) in &processes {
                assert!(
                    (xmax * nevents as f64 / total - 1.0).abs() < 1e-5,
                    "{name} process {id}: XMAXUP {xmax:e} is not the summed XSECUP {total:e} over \
                     {nevents} events"
                );
            }

            let ids: Vec<i32> = processes.iter().map(|p| p.2).collect();
            let mut plan = plan(nevents, total);
            plan.process_ids = ids.clone();
            plan.event_norm = EventNorm::Sum;
            let mut source = TwoProcesses {
                inner: FixedWeights::new(vec![1.0, 1.0, 2.5, 1.0, 1.0, 1.0, 8.4], total),
                ids,
            };
            let (text, _) = emit_to_string(&Buffer, &mut source, &plan);
            let file = LheFile::parse(&text).expect("our own file parses");
            assert_eq!(
                file.init.weight_strategy.as_i32(),
                banked_idwtup,
                "{name}: IDWTUP"
            );
            let xsec: f64 = file.init.processes.iter().map(|p| p.xsec_pb).sum();
            assert!((xsec / total - 1.0).abs() < 1e-6, "{name}: XSECUP {xsec}");
            let mean = file.events.iter().map(|e| e.weight).sum::<f64>() / nevents as f64;
            assert!(
                (mean / processes[0].1 - 1.0).abs() < 1e-5,
                "{name}: mean XWGTUP {mean:e} vs MadEvent's XMAXUP {:e}",
                processes[0].1
            );
        }
    }

    /// MadEvent's own `e+ e- > mu+ mu-` samples under `event_norm = sum` and
    /// `average` (`validation/madgraph/event_norm_reference.json`: one card, one
    /// seed, 1000 events, differing only in `event_norm`), read per event.
    ///
    /// MadGraph writes `IDWTUP -3` with `XMAXUP = XWGTUP = σ/N` on every event
    /// under `sum`, and `IDWTUP -4` with `XWGTUP = σ` on every event under
    /// `average`. The banked `σ` and `N` go through [`Buffer`] under the same
    /// norm, so the file's `IDWTUP`, its `XSECUP`/`XMAXUP` and each event's
    /// `XWGTUP` are this crate's weight rule against MadEvent's, not a
    /// re-integration.
    ///
    /// A source of equal generator weights has no overweight tail, and then
    /// every written `XWGTUP` equals MadEvent's. MadGraph flattens the tail to
    /// the constant weight; this writer keeps it proportional, so with a tail
    /// the bulk sits below MadEvent's constant by the tail's excess of the mean
    /// and a tail event above it. There the per-event comparison can assert
    /// only what the rule fixes: the bulk is constant and below MadEvent's
    /// weight, the tail is above it, and the sum (`sum`) or mean (`average`)
    /// of the weights is MadEvent's `σ`.
    ///
    /// What this cannot see: both samples come from one process, one card, one
    /// part and one process entry, so a rule that is wrong only when several
    /// parts or processes share the file is not exercised here (see
    /// `each_part_is_normalised_to_its_integration_under_every_event_norm`),
    /// and a rescaling that preserves the sum or mean but moves weight between
    /// events is visible only through the tail's ordering and ratio.
    #[test]
    fn event_norm_weights_match_madevents_per_event() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/event_norm_reference.json"
        );
        let text = std::fs::read_to_string(path).expect("the committed event_norm reference");
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap();
        let nevents = dump["nevents"].as_u64().unwrap() as usize;
        let samples = dump["samples"].as_object().expect("samples");
        assert_eq!(samples.len(), 2, "the reference holds sum and average");

        for (norm_name, norm, banked_idwtup, per_event_of_sigma) in [
            ("sum", EventNorm::Sum, -3, 1.0 / nevents as f64),
            ("average", EventNorm::Average, -4, 1.0),
        ] {
            let sample = &samples[norm_name];
            let lines: Vec<Vec<&str>> = sample["init"]
                .as_array()
                .expect("init")
                .iter()
                .map(|l| l.as_str().unwrap().split_whitespace().collect())
                .collect();
            assert_eq!(lines.len(), 2, "{norm_name}: one process line");
            assert_eq!(lines[0][8].parse::<i32>().unwrap(), banked_idwtup);
            let sigma: f64 = lines[1][0].parse().unwrap();
            let banked_xmax: f64 = lines[1][2].parse().unwrap();
            let process_id: i32 = lines[1][3].parse().unwrap();
            assert!(
                (banked_xmax / (sigma * per_event_of_sigma) - 1.0).abs() < 1e-5,
                "{norm_name}: banked XMAXUP {banked_xmax:e} is not {per_event_of_sigma:e} sigma"
            );
            let banked: Vec<f64> = sample["xwgtup"]
                .as_array()
                .expect("xwgtup")
                .iter()
                .map(|w| w.as_str().unwrap().parse().unwrap())
                .collect();
            assert_eq!(banked.len(), nevents, "{norm_name}: one XWGTUP per event");
            let constant = banked[0];
            assert!(
                banked.iter().all(|w| (w / constant - 1.0).abs() < 1e-6),
                "{norm_name}: MadEvent's weights are not constant"
            );

            let mut plan = plan(nevents, sigma);
            plan.process_ids = vec![process_id];
            plan.event_norm = norm;

            // No tail: every event is MadEvent's.
            let mut source = FixedWeights::new(vec![1.0], sigma);
            let (text, _) = emit_to_string(&Buffer, &mut source, &plan);
            let file = LheFile::parse(&text).expect("our own file parses");
            assert_eq!(file.init.weight_strategy.as_i32(), banked_idwtup);
            assert!((file.init.processes[0].xsec_pb / sigma - 1.0).abs() < 1e-6);
            assert!((file.init.processes[0].xmax / banked_xmax - 1.0).abs() < 1e-5);
            assert_eq!(file.events.len(), nevents);
            for (k, (ours, theirs)) in file.events.iter().zip(&banked).enumerate() {
                assert!(
                    (ours.weight / theirs - 1.0).abs() < 1e-5,
                    "{norm_name} event {k}: XWGTUP {:e} vs MadEvent's {theirs:e}",
                    ours.weight
                );
            }

            // A tail: one event in ten carries four times the generator weight.
            let mut weights = vec![1.0; 10];
            weights[9] = 4.0;
            let mut source = FixedWeights::new(weights, sigma);
            let (text, _) = emit_to_string(&Buffer, &mut source, &plan);
            let file = LheFile::parse(&text).expect("our own file parses");
            let total: f64 = file.events.iter().map(|e| e.weight).sum();
            assert!(
                (total / (sigma * nevents as f64 * per_event_of_sigma) - 1.0).abs() < 1e-5,
                "{norm_name}: the tailed weights' sum {total:e}"
            );
            let bulk: Vec<f64> = file.events.iter().map(|e| e.weight).step_by(10).collect();
            let tail: Vec<f64> = file
                .events
                .iter()
                .skip(9)
                .step_by(10)
                .map(|e| e.weight)
                .collect();
            assert!(bulk.iter().all(|w| (w / bulk[0] - 1.0).abs() < 1e-6));
            assert!(tail.iter().all(|w| (w / tail[0] - 1.0).abs() < 1e-6));
            assert!(
                bulk[0] < constant && tail[0] > constant,
                "{norm_name}: tail ordering"
            );
            assert!(
                (tail[0] / bulk[0] - 4.0).abs() < 1e-5,
                "{norm_name}: the tail keeps its generator weight ratio"
            );
        }
    }

    /// A file of several process numbers declares one `<init>` entry per
    /// number, each carrying its process's share of the sample: under `-4` the
    /// share of the written weight, under `+3` the share of the written events —
    /// which a streaming pass can know before its first event only by drawing the
    /// sample once and replaying it.
    #[test]
    fn several_processes_each_declare_their_own_share() {
        // Process 2 takes the second and fourth of every five events.
        let weights = vec![1.0, 1.0, 2.5, 1.0, 1.0];
        let ids = vec![1, 2, 1, 2, 1];
        let sigma = 30.0;
        let mut several = plan(400, sigma);
        several.process_ids = vec![1, 2];
        for strategy in [
            Box::new(Buffer) as Box<dyn UnweightStrategy>,
            Box::new(StochasticRounding::new(11)),
        ] {
            let mut source = TwoProcesses {
                inner: FixedWeights::new(weights.clone(), sigma),
                ids: ids.clone(),
            };
            let (text, _) = emit_to_string(strategy.as_ref(), &mut source, &several);
            let file = LheFile::parse(&text).expect("our own file parses");
            let declared: Vec<(i32, f64)> = file
                .init
                .processes
                .iter()
                .map(|p| (p.id, p.xsec_pb))
                .collect();
            assert_eq!(declared.len(), 2, "{}", strategy.describe());
            let total: f64 = file.events.iter().map(|e| e.weight).sum();
            for (id, xsec) in &declared {
                let own: f64 = file
                    .events
                    .iter()
                    .filter(|e| e.process_id == *id)
                    .map(|e| e.weight)
                    .sum();
                assert!(
                    (xsec / (sigma * own / total) - 1.0).abs() < 1e-6,
                    "{}: process {id} declares {xsec} pb for a {own}/{total} share",
                    strategy.describe()
                );
            }
            let summed: f64 = declared.iter().map(|(_, x)| x).sum();
            assert!((summed / sigma - 1.0).abs() < 1e-6);
            // Every declared XMAXUP bounds its own process's written weights.
            for p in &file.init.processes {
                let largest = file
                    .events
                    .iter()
                    .filter(|e| e.process_id == p.id)
                    .map(|e| e.weight)
                    .fold(0.0f64, f64::max);
                assert!(
                    largest <= p.xmax * (1.0 + 1e-6),
                    "{largest} above {}",
                    p.xmax
                );
            }
        }
        // One process keeps the integration's error and the single entry.
        let mut source = FixedWeights::new(weights, sigma);
        let (text, _) = emit_to_string(&Buffer, &mut source, &plan(40, sigma));
        let file = LheFile::parse(&text).expect("parses");
        assert_eq!(file.init.processes.len(), 1);
        assert_eq!(file.init.processes[0].xerr_pb, 0.01 * sigma);
    }

    /// An event naming a process the plan does not declare is refused.
    #[test]
    fn an_undeclared_process_is_refused() {
        let mut plan = plan(10, 1.0);
        plan.process_ids = vec![1, 3];
        let mut source = TwoProcesses {
            inner: FixedWeights::new(vec![1.0], 1.0),
            ids: vec![1, 2],
        };
        let err = Buffer
            .emit(&mut source, &plan, &mut Vec::new())
            .expect_err("process 2 is not declared");
        assert!(matches!(err, EmitError::UndeclaredProcess(2)));
    }

    /// A source whose `k`-th event belongs to part `parts[k % parts.len()]` and
    /// process `ids[k % ids.len()]`, declaring a sample cross section of its own
    /// that is deliberately not the integration's.
    struct PartedSource {
        inner: FixedWeights,
        parts: Vec<usize>,
        ids: Vec<i32>,
    }

    impl EventSource for PartedSource {
        fn next_event(&mut self) -> Option<WeightedEvent> {
            let k = self.inner.next;
            let mut event = self.inner.next_event()?;
            event.part = self.parts[k % self.parts.len()];
            event.record.process_id = self.ids[k % self.ids.len()];
            Some(event)
        }
        fn restart(&mut self) {
            self.inner.restart();
        }
        fn sigma_pb(&self) -> f64 {
            self.inner.sigma_pb()
        }
    }

    /// Stochastic rounding writes unit weights, so it cannot pin each part to its
    /// integration; a plan of several parts is refused before anything is
    /// written, and the same source with one part still emits.
    #[test]
    fn stochastic_rounding_refuses_several_parts() {
        let mut source = PartedSource {
            inner: FixedWeights::new(vec![1.0, 1.5, 1.0], 40.0),
            parts: vec![0, 1],
            ids: vec![10, 11],
        };
        let mut two = plan(20, 40.0);
        two.parts = vec![
            PartSigma {
                sigma_pb: 31.0,
                sigma_err_pb: 0.2,
            },
            PartSigma {
                sigma_pb: 9.0,
                sigma_err_pb: 0.15,
            },
        ];
        two.process_ids = vec![10, 11];
        let rounding = StochasticRounding::new(7);
        let mut sink = Vec::new();
        let err = rounding
            .emit(&mut source, &two, &mut sink)
            .expect_err("two parts are refused");
        assert!(matches!(err, EmitError::PartsUnsupported { parts: 2, .. }));
        assert!(sink.is_empty(), "nothing is written before the refusal");

        two.parts.clear();
        source.restart();
        rounding
            .emit(&mut source, &two, &mut Vec::new())
            .expect("one part emits");
    }

    fn written_weights(file: &LheFile, filter: impl Fn(&LheEvent) -> bool) -> f64 {
        file.events
            .iter()
            .filter(|e| filter(e))
            .map(|e| e.weight)
            .sum::<f64>()
    }

    /// With several parts each part's written weights are pinned to its own
    /// integration under every `event_norm`: they sum to `N σₖ` (mean `σₖ`) under
    /// `average` and to `σₖ` under `sum`, whatever share of the `N` events the
    /// part drew. A single part cannot tell the part's event count from `N`, so
    /// the parts here draw unequal shares (two thirds and one third).
    #[test]
    fn each_part_is_normalised_to_its_integration_under_every_event_norm() {
        let weights = vec![1.0, 1.0, 3.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0];
        let sigmas = [31.0, 9.0];
        for (norm, per_event) in [(EventNorm::Average, 600.0), (EventNorm::Sum, 1.0)] {
            let mut source = PartedSource {
                inner: FixedWeights::new(weights.clone(), 47.0),
                parts: vec![0, 0, 1],
                ids: vec![10, 10, 11],
            };
            let mut two = plan(600, 40.0);
            two.parts = sigmas
                .iter()
                .map(|&sigma_pb| PartSigma {
                    sigma_pb,
                    sigma_err_pb: 0.1,
                })
                .collect();
            two.process_ids = vec![10, 11];
            two.event_norm = norm;
            let (text, _) = emit_to_string(&Buffer, &mut source, &two);
            let file = LheFile::parse(&text).expect("our own file parses");
            for (id, sigma) in [(10, sigmas[0]), (11, sigmas[1])] {
                let summed = written_weights(&file, |e| e.process_id == id);
                assert!(
                    (summed / (per_event * sigma) - 1.0).abs() < 1e-6,
                    "{norm:?}: process {id}'s weights sum to {summed}, want {}",
                    per_event * sigma
                );
            }
        }
    }

    /// The buffered file pins every part to its integration: the written weights
    /// of part `k` sum, over the file's event count, to its `σₖ`, whatever the
    /// sample's own estimate was. One process number per part declares exactly
    /// that part's integrated σ and error, the overweight events keep their
    /// weight relative to the rest of their part, and the sample's own estimate
    /// before normalisation is in the header.
    #[test]
    fn each_part_is_normalised_to_its_integration() {
        // Part 1 takes every third event; two overweights land in each part.
        let weights = vec![1.0, 1.0, 3.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0];
        let parts = vec![0, 0, 1];
        let ids = vec![10, 10, 11];
        let sample_sigma = 47.0;
        let mut source = PartedSource {
            inner: FixedWeights::new(weights.clone(), sample_sigma),
            parts: parts.clone(),
            ids,
        };
        let mut two = plan(600, 40.0);
        two.parts = vec![
            PartSigma {
                sigma_pb: 31.0,
                sigma_err_pb: 0.2,
            },
            PartSigma {
                sigma_pb: 9.0,
                sigma_err_pb: 0.15,
            },
        ];
        two.process_ids = vec![10, 11];
        two.header = Some("process test".to_string());
        let (text, summary) = emit_to_string(&Buffer, &mut source, &two);
        let file = LheFile::parse(&text).expect("our own file parses");
        let n = file.events.len() as f64;

        for (id, part) in [(10, two.parts[0]), (11, two.parts[1])] {
            let summed = written_weights(&file, |e| e.process_id == id) / n;
            assert!(
                (summed / part.sigma_pb - 1.0).abs() < 1e-6,
                "process {id}'s weights sum to {summed} over N, its part's σ is {}",
                part.sigma_pb
            );
            let entry = file.init.processes.iter().find(|p| p.id == id).unwrap();
            assert_eq!(entry.xsec_pb, part.sigma_pb, "XSECUP of {id}");
            assert_eq!(entry.xerr_pb, part.sigma_err_pb, "XERRUP of {id}");
        }
        let mean = written_weights(&file, |_| true) / n;
        assert!((mean / 40.0 - 1.0).abs() < 1e-6, "mean XWGTUP {mean}");
        assert!((summary.xsec_pb / 40.0 - 1.0).abs() < 1e-12);

        // Within a part every weight keeps its ratio to the others: the
        // overweights are rescaled, not capped.
        for (k, event) in file.events.iter().enumerate() {
            let part = parts[k % parts.len()];
            let base = file
                .events
                .iter()
                .enumerate()
                .find(|&(j, _)| parts[j % parts.len()] == part && weights[j % weights.len()] == 1.0)
                .map(|(_, e)| e.weight)
                .unwrap();
            let want = weights[k % weights.len()];
            assert!(
                (event.weight / base / want - 1.0).abs() < 1e-6,
                "event {k}: written ratio {} for a generator weight {want}",
                event.weight / base
            );
        }

        assert_eq!(summary.sample_sigma_pb, sample_sigma);
        let line = format!(
            "sample estimate before normalisation {sample_sigma:.6e} +- {:.6e}",
            summary.sample_sigma_err_pb
        );
        assert!(text.contains(&line), "the header lacks `{line}`");
        let (read, read_err) = sample_estimate_in(&text).expect("the estimate reads back");
        assert!((read / sample_sigma - 1.0).abs() < 1e-6);
        assert!((read_err / summary.sample_sigma_err_pb - 1.0).abs() < 1e-6);
        assert!(text.contains("process test\n"), "the plan's header is kept");
    }

    /// A process number spread over parts, and a part holding two processes,
    /// still sum to the integration: each process declares its share of each
    /// part's σ, and the share's sampling error joins the integration's where a
    /// part is split.
    #[test]
    fn processes_that_split_parts_declare_their_shares() {
        let weights = vec![1.0, 1.0, 1.0, 2.0];
        // Events cycle parts (0, 1) and processes (1, 1, 2): part 0 holds both.
        let mut source = PartedSource {
            inner: FixedWeights::new(weights, 5.0),
            parts: vec![0, 1],
            ids: vec![1, 1, 2],
        };
        let mut split = plan(1200, 20.0);
        split.parts = vec![
            PartSigma {
                sigma_pb: 12.0,
                sigma_err_pb: 0.1,
            },
            PartSigma {
                sigma_pb: 8.0,
                sigma_err_pb: 0.1,
            },
        ];
        split.process_ids = vec![1, 2];
        let (text, _) = emit_to_string(&Buffer, &mut source, &split);
        let file = LheFile::parse(&text).expect("parses");
        let n = file.events.len() as f64;
        let declared: f64 = file.init.processes.iter().map(|p| p.xsec_pb).sum();
        assert!((declared / 20.0 - 1.0).abs() < 1e-6, "{declared}");
        for p in &file.init.processes {
            let own = written_weights(&file, |e| e.process_id == p.id) / n;
            assert!(
                (p.xsec_pb / own - 1.0).abs() < 1e-6,
                "{}: {} vs {own}",
                p.id,
                p.xsec_pb
            );
            assert!(
                p.xerr_pb > 0.1 * p.xsec_pb / 20.0,
                "{}: XERRUP {}",
                p.id,
                p.xerr_pb
            );
        }
    }

    /// An event naming a part the plan does not list is refused rather than
    /// normalised against nothing.
    #[test]
    fn an_undeclared_part_is_refused() {
        let mut source = PartedSource {
            inner: FixedWeights::new(vec![1.0], 1.0),
            parts: vec![0, 2],
            ids: vec![1],
        };
        let err = Buffer
            .emit(&mut source, &plan(10, 1.0), &mut Vec::new())
            .expect_err("part 2 is not declared");
        assert!(matches!(err, EmitError::UndeclaredPart(2)));
    }

    /// The sample estimate's error, `σ̂·√(Σw²)/Σw`, against the spread of the
    /// estimate itself over independent accept/reject samples of a fixed event
    /// count. The toy is a flat bulk and a narrow peak forty times higher, with
    /// the maximum set at a quarter of the peak: a sixth of the accepted events
    /// carry weight 4. A rule that ignored the weights (`σ̂/√N`) reads ~15 % low
    /// here, and the dropped `1/T` term makes the quoted error high by
    /// `1/√(1 − ε)`, 6 % at this toy's efficiency of 12 %.
    #[test]
    fn the_sample_estimate_error_matches_its_own_spread() {
        let w_max = 10.0;
        let events = 500;
        let replicas = 400;
        let mut rng = ChaCha8Rng::seed_from_u64(0x5A3B1E);
        let (mut estimates, mut quoted, mut naive) = (Vec::new(), Vec::new(), Vec::new());
        let (mut accepted, mut trials_all) = (0u64, 0u64);
        for _ in 0..replicas {
            let mut weights = Vec::with_capacity(events);
            let mut trials = 0u64;
            while weights.len() < events {
                trials += 1;
                let x: f64 = rng.random();
                let r: f64 = if x < 0.98 { 1.0 } else { 40.0 } / w_max;
                if rng.random::<f64>() < r.min(1.0) {
                    weights.push(r.max(1.0));
                }
            }
            accepted += events as u64;
            trials_all += trials;
            let sigma = w_max * weights.iter().sum::<f64>() / trials as f64;
            estimates.push(sigma);
            quoted.push(sample_estimate_error(sigma, weights.iter().copied()));
            naive.push(sigma / (events as f64).sqrt());
        }
        let truth = 0.98 + 0.02 * 40.0;
        let m = estimates.iter().sum::<f64>() / replicas as f64;
        let sd =
            (estimates.iter().map(|e| (e - m).powi(2)).sum::<f64>() / (replicas - 1) as f64).sqrt();
        let q = quoted.iter().sum::<f64>() / replicas as f64;
        let nv = naive.iter().sum::<f64>() / replicas as f64;
        let efficiency = accepted as f64 / trials_all as f64;
        assert!(
            (m - truth).abs() < 4.0 * sd / (replicas as f64).sqrt(),
            "mean {m} against {truth}"
        );
        // 400 replicas measure a spread to ~3.5 %.
        let ratio = sd / (q * (1.0 - efficiency).sqrt());
        assert!(
            (ratio - 1.0).abs() < 0.08,
            "observed spread {sd:.4e} against the quoted {q:.4e} at efficiency {efficiency:.3} \
             ({ratio:.3})"
        );
        assert!(
            sd / nv > 1.1,
            "the toy cannot tell the weights apart: {}",
            sd / nv
        );
    }
}
