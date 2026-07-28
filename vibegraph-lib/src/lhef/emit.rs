//! Turning an accept/reject sample into the events a Les Houches file carries.
//!
//! The unweighting pass produces events whose weight is `1` almost always and
//! `w/w_max > 1` for a point that exceeded its channel's estimated maximum. A file
//! has to represent that tail somehow, and there is more than one honest way to do
//! it. [`UnweightStrategy`] is the choice, and the two implemented here differ in
//! what they put in `IDWTUP`:
//!
//! * [`Buffer`] keeps the weights, declaring `IDWTUP = -4` — `XWGTUP` is a cross
//!   section in picobarns and the total is the *mean* of the event weights. The
//!   overweight tail stays visible in the file, event by event.
//! * [`StochasticRounding`] keeps unit weights, declaring `IDWTUP = +3`, and
//!   writes an event `floor(w) + Bernoulli(frac(w))` times. The tail is
//!   represented as multiplicity instead of weight.
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

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::build::WeightNormalisation;
use super::record::{LheEvent, LheInit, LheProcess, WeightStrategy};
use super::write::LheWriter;

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
#[derive(Clone, Debug)]
pub struct WeightedEvent {
    pub record: LheEvent,
    pub weight: f64,
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
    /// `IDBMUP`.
    pub beam_pdg: [i32; 2],
    /// `EBMUP`, in GeV.
    pub beam_energy: [f64; 2],
    /// `PDFGUP`, `0` for a beam with no parton densities.
    pub pdf_group: [i32; 2],
    /// `PDFSUP`.
    pub pdf_set: [i32; 2],
    /// `LPRUP` of the single process entry every event refers back to.
    pub process_id: i32,
    /// Lines after the process entry, `<generator>` among them.
    pub trailer: Vec<String>,
    /// Free-form provenance for the `<header>` block.
    pub header: Option<String>,
}

/// What an emission actually produced.
#[derive(Clone, Copy, Debug, Default)]
pub struct EmitSummary {
    /// Accepted events taken from the source.
    pub drawn: usize,
    /// Event records written. A strategy that duplicates overweight events writes
    /// more than it drew.
    pub written: u64,
    /// The `XSECUP` the file declares, in picobarns.
    pub xsec_pb: f64,
    /// The `XMAXUP` the file declares.
    pub xmax: f64,
    /// The sum of the emitted `XWGTUP` values. Under `IDWTUP = -4` this over
    /// [`written`](Self::written) is the cross section the file declares.
    pub weight_sum: f64,
    /// The mean generator weight over the events drawn — `1` when nothing went
    /// overweight, and the mean multiplicity a stochastic-rounding pass has to
    /// reproduce.
    pub mean_source_weight: f64,
    /// The largest generator weight drawn.
    pub max_source_weight: f64,
}

#[derive(Debug)]
pub enum EmitError {
    Io(io::Error),
    /// The source ran out before the file was full.
    Exhausted {
        wanted: usize,
        drawn: usize,
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
    /// The `IDWTUP` a file written by this strategy declares.
    fn weight_strategy(&self) -> WeightStrategy;

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

/// Assemble the `<init>` block from the plan and the sample-dependent fields a
/// strategy has resolved.
fn init_block(plan: &EmitPlan, strategy: WeightStrategy, xsec_pb: f64, xmax: f64) -> LheInit {
    LheInit {
        beam_pdg: plan.beam_pdg,
        beam_energy: plan.beam_energy,
        pdf_group: plan.pdf_group,
        pdf_set: plan.pdf_set,
        weight_strategy: strategy,
        processes: vec![LheProcess {
            xsec_pb,
            xerr_pb: plan.sigma_err_pb,
            xmax,
            id: plan.process_id,
        }],
        trailer: plan.trailer.clone(),
    }
}

/// Take `n` accepted events from the source, or report how far it got.
fn draw_all(source: &mut dyn EventSource, n: usize) -> Result<Vec<WeightedEvent>, EmitError> {
    let mut events = Vec::with_capacity(n);
    while events.len() < n {
        let Some(event) = source.next_event() else {
            return Err(EmitError::Exhausted {
                wanted: n,
                drawn: events.len(),
            });
        };
        events.push(event);
    }
    Ok(events)
}

/// Generate the whole sample in memory, then write it with the weights it
/// carries (`IDWTUP = -4`).
///
/// The overweight events an accept/reject pass keeps above weight `1` stay in the
/// file as weights, so a consumer sees the tail rather than a smoothed version of
/// it, and `XMAXUP` is the largest weight actually written rather than a
/// prediction of it.
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

impl UnweightStrategy for Buffer {
    fn weight_strategy(&self) -> WeightStrategy {
        WeightStrategy::MeanCrossSectionPb
    }

    fn describe(&self) -> String {
        "buffered, IDWTUP = -4 (XWGTUP in pb, sigma = mean weight)".to_string()
    }

    fn emit(
        &self,
        source: &mut dyn EventSource,
        plan: &EmitPlan,
        sink: &mut dyn Write,
    ) -> Result<EmitSummary, EmitError> {
        let events = draw_all(source, plan.nevents)?;
        // The cross section the file declares is the sample's own, not the
        // integration's: comparing the two is then a real check on the
        // accept/reject pass rather than a restatement of its input.
        let xsec_pb = source.sigma_pb();

        let total: f64 = events.iter().map(|e| e.weight).sum();
        let mean = if events.is_empty() {
            0.0
        } else {
            total / events.len() as f64
        };
        let max_source_weight = events.iter().map(|e| e.weight).fold(0.0f64, f64::max);
        let normalisation = WeightNormalisation::new(xsec_pb, mean);
        let xmax = normalisation.xwgtup(max_source_weight);

        let init = init_block(plan, self.weight_strategy(), xsec_pb, xmax);
        let mut writer = LheWriter::begin(&mut *sink, &init, plan.header.as_deref())?;
        let mut weight_sum = 0.0;
        for event in &events {
            let mut record = event.record.clone();
            record.weight = normalisation.xwgtup(event.weight);
            weight_sum += record.weight;
            writer.write_event(&record)?;
        }
        let written = writer.events_written();
        writer.finish()?;

        Ok(EmitSummary {
            drawn: events.len(),
            written,
            xsec_pb,
            xmax,
            weight_sum,
            mean_source_weight: mean,
            max_source_weight,
        })
    }
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
pub fn stochastic_multiplicity(weight: f64, rng: &mut impl Rng) -> u64 {
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
    pub seed: u64,
}

impl StochasticRounding {
    pub fn new(seed: u64) -> Self {
        StochasticRounding { seed }
    }
}

impl UnweightStrategy for StochasticRounding {
    fn weight_strategy(&self) -> WeightStrategy {
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
        let init = init_block(plan, self.weight_strategy(), plan.sigma_pb, 1.0);
        let mut writer = LheWriter::begin(&mut *sink, &init, plan.header.as_deref())?;

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        rng.set_stream(ROUNDING_STREAM);

        let mut drawn = 0usize;
        let mut weight_total = 0.0f64;
        let mut max_source_weight = 0.0f64;
        while writer.events_written() < plan.nevents as u64 {
            let Some(event) = source.next_event() else {
                return Err(EmitError::Exhausted {
                    wanted: plan.nevents,
                    drawn,
                });
            };
            drawn += 1;
            weight_total += event.weight;
            max_source_weight = max_source_weight.max(event.weight);
            // Every copy of an event is written before the loop re-checks the
            // budget: truncating an event's copies mid-way would bias exactly the
            // overweight tail this strategy exists to represent.
            let copies = stochastic_multiplicity(event.weight, &mut rng);
            let mut record = event.record.clone();
            record.weight = 1.0;
            for _ in 0..copies {
                writer.write_event(&record)?;
            }
        }
        let written = writer.events_written();
        writer.finish()?;

        Ok(EmitSummary {
            drawn,
            written,
            xsec_pb: plan.sigma_pb,
            xmax: 1.0,
            weight_sum: written as f64,
            mean_source_weight: if drawn > 0 {
                weight_total / drawn as f64
            } else {
                0.0
            },
            max_source_weight,
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
        }
    }

    impl EventSource for FixedWeights {
        fn next_event(&mut self) -> Option<WeightedEvent> {
            let weight = *self.weights.get(self.next % self.weights.len())?;
            let event = WeightedEvent {
                record: record(self.next),
                weight,
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
            beam_pdg: [-11, 11],
            beam_energy: [45.6, 45.6],
            pdf_group: [0, 0],
            pdf_set: [0, 0],
            process_id: 1,
            trailer: Vec::new(),
            header: None,
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
}
