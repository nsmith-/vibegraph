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
//! ([`observables::kinematics`](crate::lhef::observables::kinematics)), one χ²
//! homogeneity result per categorical column (`SPINUP`, `ICOLUP`, flavour), one
//! [`FieldColumn`] per incoming leg and field, and one more per scalar the
//! `<event>` line reports. The statistics themselves are [`crate::stats`]; what
//! this module adds is turning records into columns, and deciding what is not
//! comparable.
//!
//! # A field that may or may not have a distribution
//!
//! The incoming legs and the record's scales are both *fields* rather than
//! shapes: whether either has a distribution to test is a property of the run,
//! not of the field. [`FieldColumn`] therefore carries both readings and
//! [`field_column`] picks between them off the samples themselves — a field
//! degenerate on both sides is compared as an equality against the banked
//! record's own value, at the precision that record printed it to
//! ([`printed_place`]) plus a few double-precision ulps; anything else takes the
//! same weighted two-sample Kolmogorov–Smirnov the outgoing observables do.
//!
//! # The incoming legs
//!
//! Every kinematic observable is built from the outgoing legs, so on their own
//! they say nothing about the beams. [`beam_columns`] compares the incoming legs
//! directly, field by field: `E`, `pz` and the mass the record carries.
//!
//! At fixed beams those are constants of the process; at hadron beams they carry
//! the momentum fractions and vary per event. Either way the comparison is an
//! equality against the reference's own record rather than a normalised shape.
//!
//! What this cannot see: two runs whose beams agree. A wrong beam construction
//! that happens to reproduce the reference's momenta is invisible here, as is
//! everything about the outgoing state — that is the KS and χ² columns'.
//!
//! # The scales the record reports
//!
//! `SCALUP` and `AQCDUP` are the only fields of a Les Houches event that say
//! what the matrix element was *evaluated at*, and no shape reaches them: a
//! wrong scale on every event moves the weights and so the cross section, but a
//! wrong scale on a subset that σ averages over, or a right cross section
//! reported under a scale nothing computed, leaves every other column of this
//! module untouched. [`scale_columns`] compares them field by field the way
//! [`beam_columns`] compares the legs.
//!
//! What this cannot see: a scale prescription both sides get wrong the same way,
//! and `μR` where it differs from `μF` — `SCALUP` is `max(μF)` by the accord's
//! own definition ([`scalup`](crate::lhef::build::scalup)) and `AQCDUP` reaches
//! `μR` only through `αs`, which is flat enough near `M_Z` that the field
//! constrains the scale far more weakly than it constrains the coupling.
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
use crate::lhef::record::{LheEvent, LheParticle, WeightStrategy, STATUS_INCOMING};
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

/// One scalar field of the record, compared between the two samples: an incoming
/// leg's energy, momentum or mass, or a scale the `<event>` line reports.
#[derive(Clone, Debug)]
pub struct FieldColumn {
    /// `beam1 E`, `beam1 pz`, `beam1 m` and the same for the second beam, or
    /// `SCALUP` and `AQCDUP`.
    pub field: String,
    pub kind: FieldKind,
}

/// How a field was compared, which depends on whether it varies.
#[derive(Clone, Copy, Debug)]
pub enum FieldKind {
    /// Both samples hold the field at one value, so there is no distribution:
    /// the reference is the banked record's own value and the statistic is the
    /// largest absolute departure from it on either side.
    Constant {
        /// The banked record's value.
        theirs: f64,
        /// Our value furthest from it.
        ours: f64,
        /// The largest `|v − theirs|` over every event of both samples.
        max_dev: f64,
        /// What the banked record's printed precision allows.
        tol: f64,
    },
    /// The field varies per event — a hadron beam's momentum fraction, a
    /// clustered scale — so it is compared as a weighted distribution like every
    /// outgoing observable.
    Distribution { d: f64, p: f64 },
}

impl FieldColumn {
    /// Whether the column agrees: inside the reference's printed precision for a
    /// constant field, above the floor for a varying one.
    pub fn agrees(&self, p_floor: f64) -> bool {
        match self.kind {
            FieldKind::Constant { max_dev, tol, .. } => max_dev <= tol,
            FieldKind::Distribution { p, .. } => p >= p_floor,
        }
    }

    /// The column as one log line: the value furthest from the reference against
    /// the reference itself for a constant field, the KS reading for one that
    /// varies.
    pub fn describe(&self) -> String {
        match self.kind {
            FieldKind::Constant {
                theirs, ours, tol, ..
            } => format!(
                "{:<9} {ours:.11e} against {theirs:.11e} (tol {tol:.2e})",
                self.field
            ),
            FieldKind::Distribution { d, p } => {
                format!("{:<9} KS p {p:.3e} (D {d:.4})", self.field)
            }
        }
    }

    /// How the column disagrees, in the record's own terms, or `None` where it
    /// does not. `what` names the family the field belongs to.
    pub fn disagreement(&self, what: &str, p_floor: f64) -> Option<String> {
        if self.agrees(p_floor) {
            return None;
        }
        Some(match self.kind {
            FieldKind::Constant {
                theirs,
                ours,
                max_dev,
                tol,
            } => format!(
                "{what} {} is {ours:.11e} against the record's {theirs:.11e}: {max_dev:.4e} \
                 outside the {tol:.2e} its printing allows",
                self.field
            ),
            FieldKind::Distribution { d, p } => format!(
                "{what} {} KS p {p:.3e} (D {d:.4}) below the {p_floor:.0e} floor",
                self.field
            ),
        })
    }

    /// A constant field's deviation as a multiple of its tolerance, which is how
    /// two constant columns are ranked against each other. `0` where the two
    /// records agree exactly, whatever the tolerance is.
    pub fn deviation_ratio(&self) -> f64 {
        match self.kind {
            FieldKind::Constant { max_dev, tol, .. } if max_dev > 0.0 => {
                max_dev / tol.max(f64::MIN_POSITIVE)
            }
            _ => 0.0,
        }
    }
}

/// What comparing two samples found.
#[derive(Clone, Debug)]
pub struct Comparison {
    pub ks: Vec<KsColumn>,
    pub chi2: Vec<Chi2Column>,
    /// The incoming legs, field by field.
    pub beams: Vec<FieldColumn>,
    /// The scales the `<event>` line reports, field by field.
    pub scales: Vec<FieldColumn>,
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

    /// The constant incoming-leg field sitting furthest outside the reference's
    /// printed precision.
    pub fn worst_beam_constant(&self) -> Option<&FieldColumn> {
        worst_constant(&self.beams)
    }

    /// The smallest KS p-value over the incoming-leg fields that vary.
    pub fn worst_beam_distribution(&self) -> Option<&FieldColumn> {
        worst_distribution(&self.beams)
    }

    /// The constant scale field sitting furthest outside the reference's printed
    /// precision.
    pub fn worst_scale_constant(&self) -> Option<&FieldColumn> {
        worst_constant(&self.scales)
    }

    /// The smallest KS p-value over the scale fields that vary.
    pub fn worst_scale_distribution(&self) -> Option<&FieldColumn> {
        worst_distribution(&self.scales)
    }
}

/// The column whose constant departs furthest from the reference as a multiple
/// of its own tolerance.
fn worst_constant(columns: &[FieldColumn]) -> Option<&FieldColumn> {
    columns
        .iter()
        .filter(|c| matches!(c.kind, FieldKind::Constant { .. }))
        .max_by(|a, b| a.deviation_ratio().total_cmp(&b.deviation_ratio()))
}

/// The column with the smallest KS p-value among those that vary.
fn worst_distribution(columns: &[FieldColumn]) -> Option<&FieldColumn> {
    columns
        .iter()
        .filter_map(|c| match c.kind {
            FieldKind::Distribution { p, .. } => Some((p, c)),
            FieldKind::Constant { .. } => None,
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, c)| c)
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

/// One incoming-leg field a comparison reads.
///
/// `px` and `py` get no column: the accord fixes them at zero on a beam, so no
/// generator has a choice about them. `E`, `pz` and the mass are the three a
/// beam construction can get wrong.
struct BeamField {
    /// The name the column is reported under.
    name: &'static str,
    /// The value on a parsed leg.
    value: fn(&LheParticle) -> f64,
    /// Where the field sits on a Les Houches particle line, so the tolerance can
    /// be read off the spelling the file used.
    line_field: usize,
}

const BEAM_FIELDS: [BeamField; 3] = [
    BeamField {
        name: "E",
        value: |p| p.momentum[0],
        line_field: 9,
    },
    BeamField {
        name: "pz",
        value: |p| p.momentum[3],
        line_field: 8,
    },
    BeamField {
        name: "m",
        value: |p| p.mass,
        line_field: 10,
    },
];

/// Eleven significant digits, which is what both of MadGraph's serialisation
/// dialects print momenta and masses at — the Fortran writer's
/// `0.24135011226E+03` and the Python post-processing's `+1.1855987927e+01`.
/// Used only for a record that carries no text of its own to read.
const PRINTED_SIGNIFICANT_DIGITS: i32 = 11;

/// The place value of the last digit a field was printed with: `1e-8` for
/// `0.24135011226E+03`, `1e-9` for `+1.1855987927e+01`.
///
/// This is what makes the tolerance a property of the *file* rather than of a
/// dialect this code knows about: a third spelling with a different width is
/// read correctly without being enumerated anywhere.
pub fn printed_place(text: &str) -> Option<f64> {
    let (mantissa, exponent) = text.split_once(['e', 'E'])?;
    let exponent: i32 = exponent.parse().ok()?;
    let fraction = mantissa.split_once('.')?.1;
    if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(10f64.powi(exponent - fraction.len() as i32))
}

/// How many significant digits a field was printed with: `11` for
/// `0.24135011226E+03` and for `+1.1855987927e+01`.
///
/// The mantissa's leading zeros are not significant, which is what makes the two
/// dialects — a leading `0.` and a leading digit — give the same answer for the
/// same number.
pub fn printed_digits(text: &str) -> Option<u32> {
    let mantissa = text.split_once(['e', 'E'])?.0;
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    if !significant.is_empty() {
        return Some(significant.len() as u32);
    }
    // An all-zero mantissa carries no significant digit at all; what its
    // spelling resolves is the fractional width, the same quantity
    // `printed_place` reads.
    let fraction = mantissa.split_once('.')?.1;
    (!fraction.is_empty()).then_some(fraction.len() as u32)
}

/// `value` on the grid the reference's own printing puts its values on:
/// rounded to `digits` significant decimal digits.
///
/// A Kolmogorov–Smirnov statistic reads the two samples' values as exact reals,
/// and the reference's are not — they were read back from a file that printed
/// them to a fixed width. Two near-constant columns whose values differ in the
/// digit after that width are then *disjoint in order*, and the statistic reads
/// `D = 1` on a disagreement of one part in `1e8`. Rounding both sides onto the
/// reference's own grid is what makes the KS branch a comparison at the
/// precision the file states, which is what the constant branch's tolerance
/// already is.
fn round_significant(value: f64, digits: u32) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let decade = value.abs().log10().floor() as i32;
    let scale = 10f64.powi(digits as i32 - 1 - decade);
    (value * scale).round() / scale
}

/// The `<event>` info line's scalar fields carry seven significant digits
/// whatever width a file spells them at.
///
/// `rw_events.f` writes the line as `(i2,i5,e16.7e3,3e15.7)`, and MadGraph's
/// Python post-processing re-serialises the same numbers two digits wider — so a
/// banked file prints `1.30002700e-01` for a value that was only ever written as
/// `1.300027e-01`: seven digits of information and two of padding. Reading the
/// spelling alone would state a precision the reference does not have and
/// compare against digits it never wrote. `validate_alphas` reads the same fact
/// off the bank directly, over every event of every banked run.
///
/// The particle lines are not capped: there the spelled width *is* the written
/// width in both dialects.
const EVENT_LINE_DIGITS: u32 = 7;

/// What a reference field's own printing states about its value: the place value
/// of its last digit, and how many significant digits it carries.
#[derive(Clone, Copy, Debug)]
struct Printed {
    place: f64,
    digits: u32,
}

impl Printed {
    /// Read off the reference's spelling, falling back to
    /// [`PRINTED_SIGNIFICANT_DIGITS`] for a record that carries no text of its
    /// own. `max_digits` caps what the spelling is allowed to claim, for a field
    /// whose writer is narrower than the dialect that re-printed it.
    fn of(spelling: Option<&str>, value: f64, max_digits: Option<u32>) -> Self {
        let digits = spelling
            .and_then(printed_digits)
            .unwrap_or(PRINTED_SIGNIFICANT_DIGITS as u32)
            .min(max_digits.unwrap_or(u32::MAX));
        let place = match spelling.and_then(printed_place) {
            // The value's own decade rather than the spelling's exponent, so a
            // capped width states the place of the last digit it actually wrote.
            Some(spelled) => spelled.max(place_of(value, digits)),
            None => place_of(value, digits),
        };
        Printed { place, digits }
    }

    /// What this precision allows between two records of the field: half the
    /// place value of its last digit, plus a few double-precision ulps for the
    /// arithmetic that produced the value being compared against it.
    fn tolerance(&self, value: f64) -> f64 {
        0.5 * self.place + 8.0 * f64::EPSILON * value.abs()
    }
}

/// The place value of the last of `digits` significant digits of `value`.
fn place_of(value: f64, digits: u32) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return 0.0;
    }
    let decade = value.abs().log10().floor() as i32;
    10f64.powi(decade - (digits as i32 - 1))
}

/// The particle lines of an event as its file spelled them, when it was read
/// from one.
///
/// The source block is the info line followed by one line per leg, so a block
/// whose line count no longer matches the record has been edited and is not
/// used.
fn particle_lines(event: &LheEvent) -> Option<Vec<&str>> {
    let text = event.source.as_ref()?.as_str();
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    lines.next()?;
    let lines: Vec<&str> = lines.collect();
    (lines.len() == event.particles.len()).then_some(lines)
}

/// The record's own spelling of one leg's field, for the tolerance it implies.
fn particle_field(event: &LheEvent, leg: usize, field: usize) -> Option<String> {
    let line = *particle_lines(event)?.get(leg)?;
    line.split_whitespace().nth(field).map(str::to_string)
}

/// The record's own spelling of one `<event>` info-line field, for the tolerance
/// it implies. The info line is the block's first non-blank line.
fn info_field(event: &LheEvent, field: usize) -> Option<String> {
    let text = event.source.as_ref()?.as_str();
    let line = text.lines().find(|l| !l.trim().is_empty())?;
    line.split_whitespace().nth(field).map(str::to_string)
}

/// The indices of an event's incoming legs, in the record's own order.
fn incoming(event: &LheEvent) -> Vec<usize> {
    event
        .particles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.status == STATUS_INCOMING)
        .map(|(i, _)| i)
        .collect()
}

/// Compare the two samples' incoming legs, field by field.
///
/// # Panics
///
/// If the two samples list different numbers of incoming legs, or an event of
/// either lists a different number from its own sample's first — neither is a
/// disagreement a statistic can express.
pub fn beam_columns(ours: &EventSample, theirs: &EventSample) -> Vec<FieldColumn> {
    if ours.is_empty() || theirs.is_empty() {
        return Vec::new();
    }
    let legs = |sample: &EventSample, side: &str| {
        let n = incoming(&sample.events[0]).len();
        for (k, event) in sample.events.iter().enumerate() {
            assert_eq!(
                incoming(event).len(),
                n,
                "{side} event {k} lists a different number of incoming legs from its own first"
            );
        }
        n
    };
    let n = legs(ours, "our");
    assert_eq!(
        n,
        legs(theirs, "the reference's"),
        "the two samples list different numbers of incoming legs"
    );

    let mut columns = Vec::with_capacity(n * BEAM_FIELDS.len());
    for beam in 0..n {
        for BeamField {
            name,
            value,
            line_field,
        } in BEAM_FIELDS
        {
            let read = |sample: &EventSample| -> Vec<(f64, f64)> {
                sample
                    .events
                    .iter()
                    .zip(&sample.weights)
                    .map(|(event, &w)| {
                        let leg = incoming(event)[beam];
                        (value(&event.particles[leg]), w)
                    })
                    .collect()
            };
            let (a, b) = (read(ours), read(theirs));
            let leg = incoming(&theirs.events[0])[beam];
            let spelling = particle_field(&theirs.events[0], leg, line_field);
            columns.push(field_column(
                format!("beam{} {name}", beam + 1),
                &a,
                &b,
                Printed::of(spelling.as_deref(), b[0].0, None),
            ));
        }
    }
    columns
}

/// One scalar of the `<event>` line a comparison reads.
///
/// `AQEDUP` gets no column: nothing in this crate's integration path depends on
/// the electromagnetic coupling running, so the field is a parameter-card
/// constant on both sides and would compare the parameter card to itself.
struct ScaleField {
    /// The name the column is reported under, which is the accord's own.
    name: &'static str,
    /// The value on a parsed event.
    value: fn(&LheEvent) -> f64,
    /// Where the field sits on the `<event>` info line, so the tolerance can be
    /// read off the spelling the file used.
    line_field: usize,
}

const SCALE_FIELDS: [ScaleField; 2] = [
    ScaleField {
        name: "SCALUP",
        value: |e| e.scale,
        line_field: 3,
    },
    ScaleField {
        name: "AQCDUP",
        value: |e| e.alpha_qcd,
        line_field: 5,
    },
];

/// Compare the two samples' reported scales, field by field.
///
/// A run whose every scale is a run-card constant gives one value per field and
/// the comparison is an equality against the banked record; a clustered or
/// otherwise dynamical prescription gives a distribution and the comparison is
/// the same weighted KS the outgoing observables take. Which one applies is read
/// off the samples rather than declared, so a prescription that silently
/// collapsed to a constant reports as one.
pub fn scale_columns(ours: &EventSample, theirs: &EventSample) -> Vec<FieldColumn> {
    if ours.is_empty() || theirs.is_empty() {
        return Vec::new();
    }
    SCALE_FIELDS
        .iter()
        .map(
            |ScaleField {
                 name,
                 value,
                 line_field,
             }| {
                let read = |sample: &EventSample| -> Vec<(f64, f64)> {
                    sample
                        .events
                        .iter()
                        .zip(&sample.weights)
                        .map(|(event, &w)| (value(event), w))
                        .collect()
                };
                let (a, b) = (read(ours), read(theirs));
                let spelling = info_field(&theirs.events[0], *line_field);
                field_column(
                    (*name).to_string(),
                    &a,
                    &b,
                    Printed::of(spelling.as_deref(), b[0].0, Some(EVENT_LINE_DIGITS)),
                )
            },
        )
        .collect()
}

/// One field's column: an equality against the reference's own value where
/// neither sample gives it a distribution, and a weighted KS where either does.
///
/// `printed` is what the reference's own record states about the field, which is
/// what sets both the equality's tolerance and the grid the KS branch rounds
/// onto.
fn field_column(
    field: String,
    ours: &[(f64, f64)],
    theirs: &[(f64, f64)],
    printed: Printed,
) -> FieldColumn {
    let kind = if degenerate(ours) && degenerate(theirs) {
        let reference = theirs[0].0;
        let tol = printed.tolerance(reference);
        let furthest = |values: &[(f64, f64)]| {
            values.iter().map(|&(v, _)| v).fold(reference, |best, v| {
                if (v - reference).abs() > (best - reference).abs() {
                    v
                } else {
                    best
                }
            })
        };
        let (mine, other) = (furthest(ours), furthest(theirs));
        let max_dev = (mine - reference).abs().max((other - reference).abs());
        FieldKind::Constant {
            theirs: reference,
            ours: mine,
            max_dev,
            tol,
        }
    } else {
        let grid = |values: &[(f64, f64)]| -> Vec<(f64, f64)> {
            values
                .iter()
                .map(|&(v, w)| (round_significant(v, printed.digits), w))
                .collect()
        };
        let test = ks_two_sample(&grid(ours), &grid(theirs))
            .expect("both columns are finite and non-empty");
        FieldKind::Distribution {
            d: test.d,
            p: test.p,
        }
    };
    FieldColumn { field, kind }
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
/// not samples of the same process as far as any of this can tell, or if they
/// list different numbers of incoming legs.
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
        beams: beam_columns(ours, theirs),
        scales: scale_columns(ours, theirs),
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
        let mean = EventSample::from_lhe(file(WeightStrategy::MeanCrossSectionPb, 0.0, &weights));
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
        let sample = EventSample::from_lhe(file(
            WeightStrategy::UnitWeight,
            40.0,
            &[1.0, 1.0, 1.0, 1.0],
        ));
        assert_eq!(sample.sigma_pb, 40.0);
        assert_eq!(sample.weights, vec![10.0; 4]);
    }

    #[test]
    #[should_panic(expected = "IDWTUP = 1")]
    fn an_unknown_strategy_is_refused_rather_than_guessed() {
        EventSample::from_lhe(file(WeightStrategy::Other(1), 1.0, &[1.0]));
    }

    /// Two events of a fixed-beam `2 → 2` run in MadGraph's Fortran dialect: the
    /// beams on their own mass shells at 60 and 70 GeV, spelled at eleven
    /// significant digits, which is where the tolerance below comes from.
    const BANKED_LHE: &str = "\
<init>
9000051 9000052 0.25000000E+03 0.25000000E+03 0 0 0 0 3 1
0.22159200E-02 0.95337000E-06 0.22159200E-02 1
</init>
<event>
4 1 +2.2159200e-03 0.2512964E+03 0.7957747E-01 0.4333599E+00
9000051   -1    0    0  501    0  0.00000000000E+00  0.00000000000E+00  0.24135011226E+03  0.24869635439E+03  0.60000000000E+02 0.  0.
9000052   -1    0    0  502    0 -0.00000000000E+00 -0.00000000000E+00 -0.24135011226E+03  0.25129639211E+03  0.70000000000E+02 0.  0.
9000051    1    1    2  501    0 -0.66923528756E+02 -0.87907914694E+02  0.21457706429E+03  0.24869635439E+03  0.60000000000E+02 0.  0.
9000052    1    1    2  502    0  0.66923528756E+02  0.87907914694E+02 -0.21457706429E+03  0.25129639211E+03  0.70000000000E+02 0.  0.
</event>
<event>
4 1 +2.2159200e-03 0.2512964E+03 0.7957747E-01 0.4333599E+00
9000051   -1    0    0  501    0  0.00000000000E+00  0.00000000000E+00  0.24135011226E+03  0.24869635439E+03  0.60000000000E+02 0.  0.
9000052   -1    0    0  502    0 -0.00000000000E+00 -0.00000000000E+00 -0.24135011226E+03  0.25129639211E+03  0.70000000000E+02 0.  0.
9000051    1    1    2  501    0 -0.12772289271E+03  0.34171648500E+02  0.20191344137E+03  0.24869635439E+03  0.60000000000E+02 0.  0.
9000052    1    1    2  502    0  0.12772289271E+03 -0.34171648500E+02 -0.20191344137E+03  0.25129639211E+03  0.70000000000E+02 0.  0.
</event>
";

    fn banked() -> EventSample {
        EventSample::from_lhe(LheFile::parse(BANKED_LHE).expect("the fixture parses"))
    }

    /// A constant column that agrees with the banked record to the last bit.
    fn exact(cell: &FieldColumn) -> bool {
        matches!(cell.kind, FieldKind::Constant { max_dev, .. } if max_dev == 0.0)
    }

    fn column<'a>(columns: &'a [FieldColumn], field: &str) -> &'a FieldColumn {
        columns
            .iter()
            .find(|c| c.field == field)
            .unwrap_or_else(|| panic!("no {field} column"))
    }

    /// Both of MadGraph's spellings, read for the place value of their last
    /// digit — the quantity the fixed-beam tolerance is half of.
    #[test]
    fn a_printed_fields_precision_is_read_off_its_own_spelling() {
        assert_eq!(printed_place("0.24135011226E+03"), Some(1e-8));
        assert_eq!(printed_place("-0.24135011226E+03"), Some(1e-8));
        assert_eq!(printed_place("+1.1855987927e+01"), Some(1e-9));
        assert_eq!(printed_place("0.0000000000e+00"), Some(1e-10));
        assert_eq!(printed_place("501"), None);
    }

    /// The pin the fixed-beam column stands on: a perturbation of one incoming
    /// `pz` inside what the record's own printing states passes, and one outside
    /// it fails, on the same pair of samples.
    ///
    /// It is a *per-event* maximum, so perturbing a single event of a sample
    /// whose beams are otherwise identical is enough.
    #[test]
    fn an_incoming_pz_is_compared_at_the_records_own_printed_precision() {
        let mg = banked();
        let identical = beam_columns(&mg, &mg);
        for cell in &identical {
            assert!(
                exact(cell),
                "{} against itself is not exact: {:?}",
                cell.field,
                cell.kind
            );
        }
        assert_eq!(identical.len(), 6, "two beams times (E, pz, m)");

        // Half the last printed digit's place, plus a few ulps of the value.
        let tol = match column(&identical, "beam1 pz").kind {
            FieldKind::Constant { tol, .. } => tol,
            other => panic!("a fixed beam is not a constant: {other:?}"),
        };
        assert!((5e-9..5.1e-9).contains(&tol), "tolerance {tol:e}");

        for (shift, agrees) in [(0.8 * tol, true), (4.0 * tol, false)] {
            let mut ours = mg.clone();
            ours.events[0].particles[0].momentum[3] += shift;
            let found = beam_columns(&ours, &mg);
            let cell = column(&found, "beam1 pz");
            assert_eq!(
                cell.agrees(1e-4),
                agrees,
                "a {shift:e} GeV shift against a {tol:e} GeV tolerance: {:?}",
                cell.kind
            );
            for other in &found {
                if other.field != "beam1 pz" {
                    assert!(
                        other.agrees(1e-4),
                        "{} moved too: {:?}",
                        other.field,
                        other.kind
                    );
                }
            }
        }
    }

    /// The record's scales are read at the precision the `<event>` info line
    /// printed them to, which is four places coarser for `SCALUP` than for a
    /// momentum: MadGraph's `unwgt.f` writes the info line at seven significant
    /// digits and the particle lines at eleven.
    #[test]
    fn the_reported_scales_are_compared_at_the_info_lines_own_printed_precision() {
        let mg = banked();
        let identical = scale_columns(&mg, &mg);
        assert_eq!(identical.len(), 2, "SCALUP and AQCDUP");
        for cell in &identical {
            assert!(
                exact(cell),
                "{} against itself is not exact: {:?}",
                cell.field,
                cell.kind
            );
        }

        let tol = |columns: &[FieldColumn], field: &str| match column(columns, field).kind {
            FieldKind::Constant { tol, .. } => tol,
            other => panic!("{field} is not a constant here: {other:?}"),
        };
        // `0.2512964E+03` and `0.4333599E+00`: half of `1e-4` and half of `1e-7`.
        let scalup = tol(&identical, "SCALUP");
        assert!(
            (5e-5..5.1e-5).contains(&scalup),
            "SCALUP tolerance {scalup:e}"
        );
        let aqcdup = tol(&identical, "AQCDUP");
        assert!(
            (5e-8..5.1e-8).contains(&aqcdup),
            "AQCDUP tolerance {aqcdup:e}"
        );

        for (shift, agrees) in [(0.8 * scalup, true), (4.0 * scalup, false)] {
            let mut ours = mg.clone();
            for event in &mut ours.events {
                event.scale += shift;
            }
            let found = scale_columns(&ours, &mg);
            assert_eq!(
                column(&found, "SCALUP").agrees(1e-4),
                agrees,
                "a {shift:e} GeV shift against a {scalup:e} GeV tolerance: {:?}",
                column(&found, "SCALUP").kind
            );
            assert!(exact(column(&found, "AQCDUP")), "AQCDUP moved too");
        }
    }

    /// A single event's scale out of line with the rest of its own sample is a
    /// *distribution* disagreement, not a deviation: the sample stops being
    /// degenerate and the column takes the KS branch.
    ///
    /// This is where the scales differ from the incoming legs, which are pinned
    /// by a per-event maximum. `DEGENERATE_SPAN` is `1e-9` of the field's own
    /// scale and the info line prints `SCALUP` to seven significant digits, so
    /// every departure the printed precision can resolve is already wide enough
    /// to end the degeneracy — and the two branches partition the failures
    /// between them rather than one covering both.
    #[test]
    fn one_events_scale_out_of_line_ends_the_constant_comparison() {
        let mg = banked();
        let mut ours = mg.clone();
        ours.events[0].scale += 1e-3;
        let found = scale_columns(&ours, &mg);
        assert!(matches!(
            column(&found, "SCALUP").kind,
            FieldKind::Distribution { .. }
        ));
    }

    /// The reference's printed width, read the same way off both of MadGraph's
    /// dialects, and the grid it puts a value on.
    #[test]
    fn a_ks_column_is_compared_on_the_references_own_printed_grid() {
        assert_eq!(printed_digits("0.24135011226E+03"), Some(11));
        assert_eq!(printed_digits("+1.1855987927e+01"), Some(11));
        assert_eq!(printed_digits("0.2512964E+03"), Some(7));
        assert_eq!(printed_digits("0.0000000000e+00"), Some(10));
        assert_eq!(printed_digits("501"), None);

        // The event line's fields are capped at what `rw_events.f` writes, so a
        // nine-digit re-serialisation of a seven-digit field states seven.
        let padded = Printed::of(Some("1.30002700e-01"), 0.130002711, Some(EVENT_LINE_DIGITS));
        assert_eq!(padded.digits, 7);
        assert!((5e-8..5.1e-8).contains(&padded.tolerance(0.13)));
        // A particle line is not capped: there the spelled width is the written
        // width in both dialects.
        let momentum = Printed::of(Some("0.24135011226E+03"), 241.35011226, None);
        assert_eq!(momentum.digits, 11);
        assert!((5e-9..5.1e-9).contains(&momentum.tolerance(241.35)));

        assert_eq!(round_significant(0.10246487912, 7), 0.1024649);
        assert_eq!(round_significant(-251.29639211, 7), -251.2964);
        assert_eq!(round_significant(0.0, 7), 0.0);

        // Two near-constant columns a hair apart are disjoint in order, so an
        // exact-value KS reads the largest gap there is on a disagreement of one
        // part in 1e8. On the reference's own grid they coincide.
        let mg = banked();
        let mut ours = mg.clone();
        for (k, event) in ours.events.iter_mut().enumerate() {
            event.alpha_qcd *= 1.0 + 1e-8 * (1.0 + k as f64);
        }
        let found = scale_columns(&ours, &mg);
        match column(&found, "AQCDUP").kind {
            FieldKind::Distribution { d, .. } => {
                assert_eq!(d, 0.0, "the reference's grid did not absorb the printing")
            }
            other => panic!("a perturbed column is not a constant: {other:?}"),
        }
    }

    /// A record whose scale prescription reads the event gives a distribution
    /// rather than a constant, and the column follows the samples into the
    /// weighted KS the outgoing observables take.
    ///
    /// This is the branch that sees a run reporting the run card's own scale
    /// where the reference clustered one per event: the two sides then disagree
    /// on *which* comparison applies, and a column that is a constant on one side
    /// and a distribution on the other is compared as a distribution — where a
    /// single value against a spread is the largest KS gap there is.
    #[test]
    fn a_scale_that_varies_per_event_is_compared_as_a_distribution() {
        let mg = banked();
        let mut ours = mg.clone();
        ours.events[0].scale *= 0.5;
        let found = scale_columns(&ours, &mg);
        assert!(matches!(
            column(&found, "SCALUP").kind,
            FieldKind::Distribution { .. }
        ));
        assert!(exact(column(&found, "AQCDUP")));
    }

    /// A beam whose energy varies event to event — a hadron beam's momentum
    /// fraction — has a distribution, and the column becomes the same weighted
    /// KS the outgoing observables take. Which branch a field falls into is read
    /// off the samples, not declared.
    #[test]
    fn a_beam_that_varies_per_event_is_compared_as_a_distribution() {
        let mg = banked();
        let mut ours = mg.clone();
        ours.events[0].particles[0].momentum[0] *= 0.5;
        ours.events[0].particles[0].momentum[3] *= 0.5;
        let found = beam_columns(&ours, &mg);
        assert!(matches!(
            column(&found, "beam1 E").kind,
            FieldKind::Distribution { .. }
        ));
        assert!(exact(column(&found, "beam1 m")));
        assert!(exact(column(&found, "beam2 E")));
    }
}
