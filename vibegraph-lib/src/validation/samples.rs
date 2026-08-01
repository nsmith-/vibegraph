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
use crate::lhef::record::LheEvent;
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
    /// The weight is the record's own `XWGTUP`. Under `IDWTUP = -4` — what
    /// MadGraph writes and what this crate writes by default — that field is a
    /// cross section per event whose *mean* is the total, so the mean is the
    /// sample's σ and a run whose events do not all carry the same weight needs no
    /// special case.
    pub fn from_lhe(file: LheFile) -> Self {
        let weights: Vec<f64> = file.events.iter().map(|e| e.weight).collect();
        let sigma_pb = if weights.is_empty() {
            0.0
        } else {
            weights.iter().sum::<f64>() / weights.len() as f64
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
