//! Comparing two event samples of the same process, column by column.
//!
//! The `samples` category's machinery, shared by the gates of both crates: the
//! fixed-beam rows generate their events in process through the library, the
//! hadron-collider rows through the shipped binary, and both end up with a list of
//! Les Houches records to compare against MadGraph's banked ones. What that
//! comparison is lives here so it is one implementation rather than two.
//!
//! [`EventSample`] is a sample as the tests hold it — records plus the weight each
//! carries. [`compare`] turns two of them into a [`Comparison`]: one
//! Kolmogorov–Smirnov result per named continuous observable
//! ([`observables::kinematics`](crate::lhef::observables::kinematics)) and one χ²
//! homogeneity result per categorical column (`SPINUP`, `ICOLUP`, flavour). The
//! statistics themselves are [`crate::stats`]; what this module adds is turning
//! records into columns, and deciding what is not comparable.
//!
//! # What is deliberately not compared
//!
//! * An observable that is a **constant of the process** — `m(l+,l-)` is `√s` on
//!   every event of a fixed-beam `2 → 2` row — has no distribution. It is named in
//!   [`Comparison::constant`] rather than compared at `D = 0`, which would report
//!   `p = 1` and read as agreement.
//! * A categorical column with a **single category** — a colourless process has
//!   one colour flow — has no degrees of freedom. It is named in
//!   [`Comparison::single_category`] for the same reason.
//!
//! Both lists exist because the alternative is a gate that passes on columns it
//! never looked at.

use std::collections::{BTreeMap, BTreeSet};

use crate::lhef::observables::{
    canonical, colour_key, flavour_key, helicity_key, kinematics, Labelling,
};
use crate::lhef::parse::LheFile;
use crate::lhef::record::{LheEvent, WeightStrategy};
use crate::stats::{chi2_homogeneity, effective_counts, effective_size, ks_two_sample};

/// An observable whose values span less than this fraction of their own scale is
/// a constant of the process.
const DEGENERATE_SPAN: f64 = 1e-9;

/// Categorical columns with at most this many distinct keys carry their per-key
/// counts into the comparison, so a χ² that fails says which category moved.
pub const MAX_CATEGORY_DETAIL: usize = 32;

/// A sample of events with the weight each carries and the cross section the
/// whole sample represents.
#[derive(Clone, Debug)]
pub struct EventSample {
    pub events: Vec<LheEvent>,
    pub weights: Vec<f64>,
    /// Picobarns.
    pub sigma_pb: f64,
}

impl EventSample {
    /// A parsed Les Houches file as a sample.
    ///
    /// The weight is the record's own `XWGTUP` and the sample's σ is what the
    /// file's `IDWTUP` says those weights combine to — the mean under `-4`, the
    /// sum under `-3`. The two differ by the event count and nothing else in the
    /// file tells them apart, so reading the field is the only way to get the
    /// normalisation right; a file carrying neither is refused rather than
    /// guessed at, because the wrong guess is off by orders of magnitude and
    /// silent.
    ///
    /// Under `+3` every event carries the same weight and the cross section is
    /// the `<init>` block's `XSECUP` instead. The per-event weight is set to its
    /// share of that, so the columns below see a uniform weight either way.
    ///
    /// # Panics
    ///
    /// On an `IDWTUP` outside those three.
    pub fn from_lhe(file: LheFile) -> Self {
        let n = file.events.len();
        let raw: Vec<f64> = file.events.iter().map(|e| e.weight).collect();
        let (weights, sigma_pb) = match (n, file.init.weight_strategy) {
            (0, _) => (raw, 0.0),
            (n, WeightStrategy::MeanCrossSectionPb) => {
                let sigma = raw.iter().sum::<f64>() / n as f64;
                (raw, sigma)
            }
            (_, WeightStrategy::SumCrossSectionPb) => {
                let sigma = raw.iter().sum::<f64>();
                (raw, sigma)
            }
            (n, WeightStrategy::UnitWeight) => {
                let sigma: f64 = file.init.processes.iter().map(|p| p.xsec_pb).sum();
                (vec![sigma / n as f64; n], sigma)
            }
            (_, WeightStrategy::Other(v)) => {
                panic!("IDWTUP = {v} says nothing this reader knows about how to combine XWGTUP")
            }
        };
        EventSample {
            events: file.events,
            weights,
            sigma_pb,
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The effective number of independent events the weights leave.
    pub fn effective_size(&self) -> f64 {
        effective_size(self.weights.iter().copied())
    }
}

/// One continuous observable's Kolmogorov–Smirnov comparison.
#[derive(Clone, Debug)]
pub struct KsColumn {
    pub observable: String,
    pub d: f64,
    pub p: f64,
}

/// One categorical column's χ² homogeneity comparison.
#[derive(Clone, Debug)]
pub struct Chi2Column {
    pub column: &'static str,
    pub chi2: f64,
    pub dof: usize,
    pub p: f64,
    pub categories: usize,
    pub distinct_keys: usize,
    pub pooled_share: f64,
    /// `(key, ours, theirs)` in effective counts, for a column with few enough
    /// categories to read.
    pub detail: Vec<(String, f64, f64)>,
}

/// What comparing two samples found.
#[derive(Clone, Debug)]
pub struct Comparison {
    pub ks: Vec<KsColumn>,
    pub chi2: Vec<Chi2Column>,
    /// Observables that are constants of the process.
    pub constant: Vec<String>,
    /// Categorical columns with a single category.
    pub single_category: Vec<&'static str>,
}

impl Comparison {
    /// The smallest KS p-value and the observable it came from.
    pub fn worst_ks(&self) -> Option<&KsColumn> {
        self.ks.iter().min_by(|a, b| a.p.total_cmp(&b.p))
    }

    /// The smallest χ² p-value and the column it came from.
    pub fn worst_chi2(&self) -> Option<&Chi2Column> {
        self.chi2.iter().min_by(|a, b| a.p.total_cmp(&b.p))
    }
}

/// One sample's differential cross section in a named observable, binned.
///
/// The p-value comparisons above are *shape* statements: a Kolmogorov–Smirnov
/// test on `m(l+,l-)` sees only the two samples' cumulative distributions and is
/// blind to both normalisations, and a χ² homogeneity test on a categorical
/// column is blind the same way. A spectrum that agrees in shape and sits
/// uniformly low therefore passes every column of [`compare`].
///
/// A spectrum scaled to its own sample's cross section is what closes that: each
/// bin carries picobarns, so the two sides are compared in absolute terms bin by
/// bin, and a normalisation error moves every bin together rather than none.
///
/// # What it cannot detect
///
/// Structure narrower than a bin: a resonance inside one bin trades against the
/// continuum around it, and the bin total can be right for the wrong reason. The
/// edges are a per-gate choice for that reason, and a suspected feature is
/// resolved by giving it its own bin.
#[derive(Clone, Debug)]
pub struct Spectrum {
    edges: Vec<f64>,
    sum: Vec<f64>,
    sum_sq: Vec<f64>,
    below: f64,
    above: f64,
}

/// One bin of a [`Spectrum`], in picobarns.
#[derive(Clone, Copy, Debug)]
pub struct Bin {
    pub low: f64,
    pub high: f64,
    /// The bin's cross section.
    pub sigma_pb: f64,
    /// The Monte-Carlo error the bin's own weights imply.
    pub err_pb: f64,
}

impl Spectrum {
    /// An empty spectrum over the given ascending bin edges.
    ///
    /// # Panics
    ///
    /// If fewer than two edges are given, or they do not ascend.
    pub fn new(edges: &[f64]) -> Self {
        assert!(edges.len() >= 2, "a spectrum needs at least one bin");
        assert!(
            edges.windows(2).all(|e| e[0] < e[1]),
            "the bin edges must ascend"
        );
        Spectrum {
            edges: edges.to_vec(),
            sum: vec![0.0; edges.len() - 1],
            sum_sq: vec![0.0; edges.len() - 1],
            below: 0.0,
            above: 0.0,
        }
    }

    /// Add one weighted entry. Values outside the edges are tallied as underflow
    /// and overflow rather than dropped, so the scale [`Spectrum::as_sigma`]
    /// applies is the whole sample's.
    pub fn fill(&mut self, x: f64, w: f64) {
        if x < self.edges[0] {
            self.below += w;
            return;
        }
        match self.edges.windows(2).position(|e| x >= e[0] && x < e[1]) {
            Some(k) => {
                self.sum[k] += w;
                self.sum_sq[k] += w * w;
            }
            None => self.above += w,
        }
    }

    /// Add every event of a sample, binned on one of the observables
    /// [`kinematics`] names.
    ///
    /// # Panics
    ///
    /// If a non-empty sample carries no observable of that name — filling
    /// nothing would leave an empty spectrum that reads as a sample with no
    /// events in range.
    pub fn fill_from(&mut self, sample: &EventSample, observable: &str, labelling: Labelling) {
        let mut seen = false;
        for (event, &w) in sample.events.iter().zip(&sample.weights) {
            let event = canonical(event, labelling);
            for (name, value) in kinematics(&event, labelling) {
                if name == observable {
                    seen = true;
                    self.fill(value, w);
                }
            }
        }
        assert!(
            seen || sample.events.is_empty(),
            "no event carries an observable named {observable}"
        );
    }

    /// Total weight, including what fell outside the edges.
    pub fn total(&self) -> f64 {
        self.sum.iter().sum::<f64>() + self.below + self.above
    }

    /// The weight below the first edge and above the last.
    pub fn outside(&self) -> (f64, f64) {
        (self.below, self.above)
    }

    /// The bins, scaled so the whole histogram — underflow and overflow
    /// included — carries `sigma_pb`.
    pub fn as_sigma(&self, sigma_pb: f64) -> Vec<Bin> {
        let total = self.total();
        let scale = if total > 0.0 { sigma_pb / total } else { 0.0 };
        self.edges
            .windows(2)
            .enumerate()
            .map(|(k, e)| Bin {
                low: e[0],
                high: e[1],
                sigma_pb: self.sum[k] * scale,
                err_pb: self.sum_sq[k].sqrt() * scale,
            })
            .collect()
    }
}

/// The columns of one sample.
struct Columns {
    kinematic: Vec<(String, Vec<(f64, f64)>)>,
    categorical: [BTreeMap<String, (f64, f64)>; 3],
}

/// The categorical columns, in the order [`Columns::categorical`] holds them.
const CATEGORICAL: [&str; 3] = ["SPINUP", "ICOLUP", "flavour"];

fn columns(sample: &EventSample, labelling: Labelling) -> Columns {
    let mut kinematic: Vec<(String, Vec<(f64, f64)>)> = Vec::new();
    let mut categorical: [BTreeMap<String, (f64, f64)>; 3] = Default::default();
    for (event, &w) in sample.events.iter().zip(&sample.weights) {
        let event = canonical(event, labelling);
        for (k, (name, value)) in kinematics(&event, labelling).into_iter().enumerate() {
            if k == kinematic.len() {
                kinematic.push((name.clone(), Vec::with_capacity(sample.len())));
            }
            assert_eq!(
                kinematic[k].0, name,
                "an observable's name must not depend on the event"
            );
            kinematic[k].1.push((value, w));
        }
        for (map, key) in categorical.iter_mut().zip([
            helicity_key(&event),
            colour_key(&event),
            flavour_key(&event),
        ]) {
            let entry = map.entry(key).or_insert((0.0, 0.0));
            entry.0 += w;
            entry.1 += w * w;
        }
    }
    Columns {
        kinematic,
        categorical,
    }
}

/// Whether a column's values span enough of their own scale to have a
/// distribution.
fn degenerate(values: &[(f64, f64)]) -> bool {
    let (lo, hi) = values
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &(v, _)| {
            (lo.min(v), hi.max(v))
        });
    (hi - lo).abs() <= DEGENERATE_SPAN * hi.abs().max(lo.abs()).max(1.0)
}

/// Fine labels when both samples carry one final-state species multiset, coarse
/// ones otherwise.
///
/// A property of the samples, not a declaration about the process: a flavour
/// group's events name the same slot differently from one event to the next, and
/// only coarse labels give it one set of observable names.
pub fn labelling_for(a: &EventSample, b: &EventSample) -> Labelling {
    let key = |s: &EventSample| {
        let mut keys: BTreeSet<String> = BTreeSet::new();
        for e in &s.events {
            keys.insert(flavour_key(&canonical(e, Labelling::Fine)));
        }
        keys
    };
    let (ka, kb) = (key(a), key(b));
    if ka.len() == 1 && ka == kb {
        Labelling::Fine
    } else {
        Labelling::Coarse
    }
}

/// Compare two samples of the same process.
///
/// # Panics
///
/// If the two samples produce different observable names, which means they are
/// not samples of the same process as far as any of this can tell.
pub fn compare(ours: &EventSample, theirs: &EventSample, labelling: Labelling) -> Comparison {
    let mine = columns(ours, labelling);
    let mg = columns(theirs, labelling);
    assert_eq!(
        mine.kinematic.iter().map(|c| &c.0).collect::<Vec<_>>(),
        mg.kinematic.iter().map(|c| &c.0).collect::<Vec<_>>(),
        "the two samples produced different observable names"
    );

    let mut ks = Vec::new();
    let mut constant = Vec::new();
    for ((name, a), (_, b)) in mine.kinematic.iter().zip(&mg.kinematic) {
        if degenerate(a) && degenerate(b) {
            constant.push(name.clone());
            continue;
        }
        let test = ks_two_sample(a, b).expect("both columns are finite and non-empty");
        ks.push(KsColumn {
            observable: name.clone(),
            d: test.d,
            p: test.p,
        });
    }

    let mut chi2 = Vec::new();
    let mut single_category = Vec::new();
    for (k, column) in CATEGORICAL.iter().enumerate() {
        match categorical(&mine.categorical[k], &mg.categorical[k], column) {
            Some(cell) => chi2.push(cell),
            None => single_category.push(*column),
        }
    }

    Comparison {
        ks,
        chi2,
        constant,
        single_category,
    }
}

/// χ² homogeneity between two categorical columns, over the union of their
/// categories, or `None` when there is nothing to compare.
fn categorical(
    ours: &BTreeMap<String, (f64, f64)>,
    theirs: &BTreeMap<String, (f64, f64)>,
    column: &'static str,
) -> Option<Chi2Column> {
    let keys: Vec<&String> = ours
        .keys()
        .chain(theirs.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if keys.len() < 2 {
        return None;
    }
    let pick =
        |m: &BTreeMap<String, (f64, f64)>, k: &String| m.get(k).copied().unwrap_or((0.0, 0.0));
    let (a_w, a_w2): (Vec<f64>, Vec<f64>) = keys.iter().map(|k| pick(ours, k)).unzip();
    let (b_w, b_w2): (Vec<f64>, Vec<f64>) = keys.iter().map(|k| pick(theirs, k)).unzip();
    let a = effective_counts(&a_w, &a_w2);
    let b = effective_counts(&b_w, &b_w2);
    let test = chi2_homogeneity(&a, &b).ok()?;
    let detail = if keys.len() <= MAX_CATEGORY_DETAIL {
        keys.iter()
            .zip(a.iter().zip(&b))
            .map(|(k, (&ours, &theirs))| ((*k).clone(), ours, theirs))
            .collect()
    } else {
        Vec::new()
    };
    Some(Chi2Column {
        column,
        chi2: test.chi2,
        dof: test.dof,
        p: test.p,
        categories: test.categories,
        distinct_keys: keys.len(),
        pooled_share: test.pooled_share,
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhef::record::{LheInit, LheParticle, LheProcess, STATUS_OUTGOING};

    fn one_event(weight: f64) -> LheEvent {
        LheEvent {
            process_id: 1,
            weight,
            scale: 91.188,
            alpha_qed: 0.0,
            alpha_qcd: 0.0,
            particles: vec![LheParticle {
                pdg: 11,
                status: STATUS_OUTGOING,
                mothers: [1, 2],
                color: [0, 0],
                momentum: [1.0, 0.0, 0.0, 1.0],
                mass: 0.0,
                lifetime: 0.0,
                spin: 1.0,
            }],
            trailer: Vec::new(),
            source: None,
        }
    }

    fn file(strategy: WeightStrategy, xsec_pb: f64, weights: &[f64]) -> LheFile {
        LheFile {
            init: LheInit {
                beam_pdg: [2212, 2212],
                beam_energy: [6500.0, 6500.0],
                pdf_group: [0, 0],
                pdf_set: [247000, 247000],
                weight_strategy: strategy,
                processes: vec![LheProcess {
                    xsec_pb,
                    xerr_pb: 0.0,
                    xmax: 0.0,
                    id: 1,
                }],
                trailer: Vec::new(),
                source: None,
            },
            events: weights.iter().map(|&w| one_event(w)).collect(),
        }
    }

    /// The same four events under the two cross-section strategies are the same
    /// sample scaled by the event count — which is why guessing is not an option:
    /// the four weights alone cannot say which was meant.
    #[test]
    fn a_samples_cross_section_follows_the_files_idwtup() {
        let weights = [1.0, 2.0, 3.0, 4.0];
        let mean = EventSample::from_lhe(file(
            WeightStrategy::MeanCrossSectionPb,
            0.0,
            &weights,
        ));
        let sum = EventSample::from_lhe(file(WeightStrategy::SumCrossSectionPb, 0.0, &weights));
        assert_eq!(mean.sigma_pb, 2.5);
        assert_eq!(sum.sigma_pb, 10.0);
        assert_eq!(sum.sigma_pb, mean.sigma_pb * weights.len() as f64);
        assert_eq!(mean.weights, weights);
        assert_eq!(sum.weights, weights);
    }

    /// Under unit weights the file's own weights carry no cross section at all, so
    /// it comes from `XSECUP` and the per-event weight is its share.
    #[test]
    fn unit_weights_take_the_cross_section_from_the_init_block() {
        let sample =
            EventSample::from_lhe(file(WeightStrategy::UnitWeight, 40.0, &[1.0, 1.0, 1.0, 1.0]));
        assert_eq!(sample.sigma_pb, 40.0);
        assert_eq!(sample.weights, vec![10.0; 4]);
    }

    #[test]
    #[should_panic(expected = "IDWTUP = 1")]
    fn an_unknown_strategy_is_refused_rather_than_guessed() {
        EventSample::from_lhe(file(WeightStrategy::Other(1), 1.0, &[1.0]));
    }
}
