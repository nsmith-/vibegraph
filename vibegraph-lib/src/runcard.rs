//! Parser for MadGraph's `run_card.dat` into a typed [`RunCard`].
//!
//! The file syntax is one parameter per line, `<value> = <name> ! comment`,
//! with `#`-prefixed comment lines and blank lines ignored. Every recognized
//! parameter carries the MadGraph LO default (transcribed from
//! `RunCardLO.default_setup` in `madgraph/various/banner.py`), so an *empty*
//! card reproduces MadGraph's out-of-the-box behavior and a reference MadGraph
//! run can share the literal same file.
//!
//! Parameter names are matched case-insensitively (MadGraph writes some
//! parameters in a different case than its internal `default_setup` name — e.g.
//! `sde_strategy` in the card vs `SDE_strategy` in `banner.py`), resolving to the
//! canonical name of the defaults table. Genuinely unknown names are still
//! rejected (typo protection). Two beam
//! configurations are accepted: proton–proton (`lpp1 == lpp2 == 1`, PDF
//! convolution) and fixed-energy partonic beams (`lpp1 == lpp2 == 0`, no PDF,
//! the incoming particles *are* the beam particles at `ebeam1`/`ebeam2`). Any
//! other combination is rejected because no other beam handling is supported.
//! Recognized parameters that are not consumed as typed fields are still parsed
//! and retained by name so the compiled cut filter ([`crate::cuts`]) can read
//! cut thresholds and so nothing is silently dropped.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A parsed parameter value. The variant also records the parameter's kind,
/// which drives how a card line's text is interpreted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Str(String),
    /// List/dict-valued parameters we do not model: retained verbatim so the
    /// name is recognized but the payload is opaque.
    Opaque(String),
}

impl ParamValue {
    fn kind(&self) -> Kind {
        match self {
            ParamValue::Float(_) => Kind::Float,
            ParamValue::Int(_) => Kind::Int,
            ParamValue::Bool(_) => Kind::Bool,
            ParamValue::Str(_) => Kind::Str,
            ParamValue::Opaque(_) => Kind::Opaque,
        }
    }

    /// Numeric value as `f64`, accepting either float or integer parameters.
    /// Panics on a non-numeric parameter — callers pass statically-known names.
    pub fn as_f64(&self) -> f64 {
        match self {
            ParamValue::Float(x) => *x,
            ParamValue::Int(i) => *i as f64,
            other => panic!("parameter is not numeric: {other:?}"),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            ParamValue::Int(i) => *i,
            ParamValue::Float(x) => *x as i64,
            other => panic!("parameter is not integer: {other:?}"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ParamValue::Bool(b) => *b,
            other => panic!("parameter is not boolean: {other:?}"),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ParamValue::Str(s) | ParamValue::Opaque(s) => s,
            other => panic!("parameter is not a string: {other:?}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Float,
    Int,
    Bool,
    Str,
    Opaque,
}

/// Const-constructible default descriptor for the static defaults table.
/// (`String` cannot be built in a `static`, so string defaults use `&'static str`.)
#[derive(Clone, Copy)]
enum Def {
    F(f64),
    I(i64),
    B(bool),
    S(&'static str),
    /// Opaque list/dict-valued parameter; default is an empty payload.
    O,
}

impl Def {
    fn to_value(self) -> ParamValue {
        match self {
            Def::F(x) => ParamValue::Float(x),
            Def::I(i) => ParamValue::Int(i),
            Def::B(b) => ParamValue::Bool(b),
            Def::S(s) => ParamValue::Str(s.to_string()),
            Def::O => ParamValue::Opaque(String::new()),
        }
    }
}

#[derive(Debug, Error)]
pub enum RunCardError {
    #[error("run card i/o error on '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("unknown run-card parameter '{name}' (line {line}) — check for a typo")]
    UnknownParam { name: String, line: usize },
    #[error("malformed value '{value}' for parameter '{name}' (line {line})")]
    BadValue {
        name: String,
        value: String,
        line: usize,
    },
    #[error(
        "unsupported beam configuration lpp1={lpp1}, lpp2={lpp2}: only proton-proton (1,1) \
         and fixed-energy partonic beams (0,0) are supported"
    )]
    UnsupportedLpp { lpp1: i64, lpp2: i64 },
    #[error(
        "beam polarization is not supported: polbeam1={polbeam1}, polbeam2={polbeam2} \
         (both must be 0)"
    )]
    UnsupportedPolarization { polbeam1: f64, polbeam2: f64 },
}

/// How the incoming state of a run is prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeamMode {
    /// Proton beams (`lpp = 1`): parton momenta drawn from PDFs, √ŝ = x₁x₂ s.
    Proton,
    /// Fixed-energy partonic beams (`lpp = 0`): the incoming particles are the
    /// beam particles themselves, √ŝ = ebeam1 + ebeam2, no PDF convolution.
    FixedEnergy,
}

/// A run card resolved against the MadGraph LO defaults.
///
/// Typed fields expose the non-cut parameters consumed by the generator; every
/// parameter (including all cut thresholds and recognized-but-unconsumed
/// bookkeeping params) is retained in [`RunCard::values`] keyed by name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCard {
    pub nevents: i64,
    pub iseed: i64,
    pub lpp1: i64,
    pub lpp2: i64,
    pub ebeam1: f64,
    pub ebeam2: f64,
    pub pdlabel: String,
    pub lhaid: i64,
    pub fixed_ren_scale: bool,
    pub fixed_fac_scale: bool,
    pub scale: f64,
    pub dsqrt_q2fact1: f64,
    pub dsqrt_q2fact2: f64,
    pub maxjetflavor: i64,
    values: BTreeMap<String, ParamValue>,
}

impl Default for RunCard {
    /// The MadGraph LO out-of-the-box configuration (an empty run card).
    fn default() -> Self {
        let mut values = BTreeMap::new();
        for (name, default) in PARAM_DEFAULTS {
            values.insert((*name).to_string(), default.to_value());
        }
        Self::from_values(values).expect("MG defaults must be self-consistent")
    }
}

impl RunCard {
    /// The beam preparation implied by `lpp1`/`lpp2`. The parser only admits the
    /// two supported combinations, so this never encounters an invalid pair.
    pub fn beam_mode(&self) -> BeamMode {
        match (self.lpp1, self.lpp2) {
            (0, 0) => BeamMode::FixedEnergy,
            _ => BeamMode::Proton,
        }
    }

    /// Look up a resolved parameter by name. `None` only for names absent from
    /// the recognized inventory.
    pub fn get(&self, name: &str) -> Option<&ParamValue> {
        self.values.get(name)
    }

    /// Numeric parameter as `f64` (float or int). Panics on unknown or
    /// non-numeric names — callers pass statically-known parameter names.
    pub fn float(&self, name: &str) -> f64 {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("no such parameter: {name}"))
            .as_f64()
    }

    /// Integer parameter. Panics on unknown/non-integer names.
    pub fn int(&self, name: &str) -> i64 {
        self.values
            .get(name)
            .unwrap_or_else(|| panic!("no such parameter: {name}"))
            .as_i64()
    }

    /// Iterate all resolved (name, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ParamValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Parse a run card from text.
    pub fn parse(text: &str) -> Result<Self, RunCardError> {
        let mut values = BTreeMap::new();
        for (name, default) in PARAM_DEFAULTS {
            values.insert((*name).to_string(), default.to_value());
        }

        for (idx, raw) in text.lines().enumerate() {
            let line_no = idx + 1;
            let Some((value_tok, name)) = split_line(raw) else {
                continue;
            };
            let Some(canonical) = canonical_name(name) else {
                return Err(RunCardError::UnknownParam {
                    name: name.to_string(),
                    line: line_no,
                });
            };
            let kind = values
                .get(canonical)
                .expect("canonical name present")
                .kind();
            let parsed = parse_value(value_tok, kind).ok_or_else(|| RunCardError::BadValue {
                name: name.to_string(),
                value: value_tok.to_string(),
                line: line_no,
            })?;
            values.insert(canonical.to_string(), parsed);
        }

        Self::from_values(values)
    }

    /// Parse a run card from a file path.
    pub fn parse_file(path: &Path) -> Result<Self, RunCardError> {
        let text = std::fs::read_to_string(path).map_err(|source| RunCardError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::parse(&text)
    }

    fn from_values(values: BTreeMap<String, ParamValue>) -> Result<Self, RunCardError> {
        let f = |name: &str| values.get(name).expect("known param").as_f64();
        let i = |name: &str| values.get(name).expect("known param").as_i64();
        let b = |name: &str| values.get(name).expect("known param").as_bool();
        let s = |name: &str| values.get(name).expect("known param").as_str().to_string();

        let lpp1 = i("lpp1");
        let lpp2 = i("lpp2");
        if (lpp1, lpp2) != (1, 1) && (lpp1, lpp2) != (0, 0) {
            return Err(RunCardError::UnsupportedLpp { lpp1, lpp2 });
        }

        let polbeam1 = f("polbeam1");
        let polbeam2 = f("polbeam2");
        if polbeam1 != 0.0 || polbeam2 != 0.0 {
            return Err(RunCardError::UnsupportedPolarization { polbeam1, polbeam2 });
        }

        Ok(RunCard {
            nevents: i("nevents"),
            iseed: i("iseed"),
            lpp1,
            lpp2,
            ebeam1: f("ebeam1"),
            ebeam2: f("ebeam2"),
            pdlabel: s("pdlabel"),
            lhaid: i("lhaid"),
            fixed_ren_scale: b("fixed_ren_scale"),
            fixed_fac_scale: b("fixed_fac_scale"),
            scale: f("scale"),
            dsqrt_q2fact1: f("dsqrt_q2fact1"),
            dsqrt_q2fact2: f("dsqrt_q2fact2"),
            maxjetflavor: i("maxjetflavor"),
            values,
        })
    }
}

/// Split a raw card line into `(value, name)`, or `None` for comment / blank /
/// structural lines. The comment (`!...`) is stripped first, then the line is
/// split on the single `=`.
fn split_line(raw: &str) -> Option<(&str, &str)> {
    let no_comment = match raw.find('!') {
        Some(pos) => &raw[..pos],
        None => raw,
    };
    let trimmed = no_comment.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('$') {
        return None;
    }
    let (value, name) = trimmed.split_once('=')?;
    let name = name.trim();
    // A bare template placeholder (`%(x)s`) or empty name is not a parameter.
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    Some((value.trim(), name))
}

fn parse_value(tok: &str, kind: Kind) -> Option<ParamValue> {
    match kind {
        Kind::Float => parse_f64(tok).map(ParamValue::Float),
        Kind::Int => parse_i64(tok).map(ParamValue::Int),
        Kind::Bool => parse_fortran_bool(tok).map(ParamValue::Bool),
        Kind::Str => Some(ParamValue::Str(strip_quotes(tok).to_string())),
        // An empty dict/list is MadGraph's "unset" spelling for a list/dict-valued
        // parameter (real cards write `{}`); normalize it to the empty default so
        // it compares equal to the table default rather than reading as an active
        // override (which the cut detector would reject).
        Kind::Opaque => {
            let payload = if matches!(tok, "{}" | "[]") { "" } else { tok };
            Some(ParamValue::Opaque(payload.to_string()))
        }
    }
}

/// Parse a floating value, tolerating Fortran `d`/`D` exponent markers.
fn parse_f64(tok: &str) -> Option<f64> {
    let normalized = tok.replace(['d', 'D'], "e");
    normalized.parse::<f64>().ok()
}

/// Parse an integer, tolerating a value written with a trailing `.0`.
fn parse_i64(tok: &str) -> Option<i64> {
    if let Ok(v) = tok.parse::<i64>() {
        return Some(v);
    }
    let v = parse_f64(tok)?;
    if v.fract() == 0.0 {
        Some(v as i64)
    } else {
        None
    }
}

fn parse_fortran_bool(tok: &str) -> Option<bool> {
    match tok.trim_matches('.').to_ascii_lowercase().as_str() {
        "t" | "true" => Some(true),
        "f" | "false" => Some(false),
        _ => None,
    }
}

fn strip_quotes(tok: &str) -> &str {
    let t = tok.trim();
    if (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
        || (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
    {
        &t[1..t.len() - 1]
    } else {
        t
    }
}

/// The MadGraph LO defaults, transcribed from `RunCardLO.default_setup`
/// (`madgraph/various/banner.py`). This table is both the source of default
/// values and the set of recognized parameter names (typo protection).
///
/// List/dict-valued parameters are represented as [`ParamValue::Opaque`] so the
/// name is recognized while the payload is not modeled.
#[rustfmt::skip]
static PARAM_DEFAULTS: &[(&str, Def)] = &[
    // ── run / seeding ────────────────────────────────────────────────────
    ("run_tag", Def::S("tag_1")),
    ("gridpack", Def::B(false)),
    ("time_of_flight", Def::F(-1.0)),
    ("nevents", Def::I(10000)),
    ("iseed", Def::I(0)),
    ("bypass_check", Def::O),
    ("python_seed", Def::I(-2)),
    // ── beams ────────────────────────────────────────────────────────────
    ("lpp1", Def::I(1)),
    ("lpp2", Def::I(1)),
    ("ebeam1", Def::F(6500.0)),
    ("ebeam2", Def::F(6500.0)),
    ("polbeam1", Def::F(0.0)),
    ("polbeam2", Def::F(0.0)),
    ("nb_proton1", Def::I(1)),
    ("nb_proton2", Def::I(1)),
    ("nb_neutron1", Def::I(0)),
    ("nb_neutron2", Def::I(0)),
    ("mass_ion1", Def::F(-1.0)),
    ("mass_ion2", Def::F(-1.0)),
    // ── PDF ──────────────────────────────────────────────────────────────
    ("pdlabel", Def::S("nn23lo1")),
    ("pdlabel1", Def::S("nn23lo1")),
    ("pdlabel2", Def::S("nn23lo1")),
    ("lhaid", Def::I(230000)),
    // ── scales ───────────────────────────────────────────────────────────
    ("fixed_ren_scale", Def::B(false)),
    ("fixed_fac_scale", Def::B(false)),
    ("fixed_fac_scale1", Def::B(false)),
    ("fixed_fac_scale2", Def::B(false)),
    ("fixed_extra_scale", Def::B(false)),
    ("scale", Def::F(91.1880)),
    ("dsqrt_q2fact1", Def::F(91.1880)),
    ("dsqrt_q2fact2", Def::F(91.1880)),
    ("mue_ref_fixed", Def::F(91.1880)),
    ("dynamical_scale_choice", Def::I(-1)),
    ("mue_over_ref", Def::F(1.0)),
    ("ievo_eva", Def::I(0)),
    ("evaorder", Def::I(0)),
    ("eva_xcut", Def::I(1)),
    // ── bias ─────────────────────────────────────────────────────────────
    ("bias_module", Def::S("None")),
    ("bias_parameters", Def::O),
    // ── matching ─────────────────────────────────────────────────────────
    ("scalefact", Def::F(1.0)),
    ("ickkw", Def::I(0)),
    ("highestmult", Def::I(1)),
    ("ktscheme", Def::I(1)),
    ("alpsfact", Def::F(1.0)),
    ("chcluster", Def::B(false)),
    ("pdfwgt", Def::B(true)),
    ("asrwgtflavor", Def::I(5)),
    ("clusinfo", Def::B(true)),
    ("custom_fcts", Def::O),
    // ── output / frame ───────────────────────────────────────────────────
    ("lhe_version", Def::F(3.0)),
    ("boost_event", Def::S("False")),
    ("me_frame", Def::O),
    ("frame_id", Def::I(6)),
    ("event_norm", Def::S("average")),
    ("keep_log", Def::S("normal")),
    // ── ŝ / decay cuts ───────────────────────────────────────────────────
    ("auto_ptj_mjj", Def::B(true)),
    ("bwcutoff", Def::F(15.0)),
    ("cut_decays", Def::B(false)),
    ("dsqrt_shat", Def::F(0.0)),
    ("dsqrt_shatmax", Def::F(-1.0)),
    ("nhel", Def::I(0)),
    ("limhel", Def::F(1e-8)),
    // ── single-leg pT ────────────────────────────────────────────────────
    ("ptj", Def::F(20.0)),
    ("ptb", Def::F(0.0)),
    ("pta", Def::F(10.0)),
    ("ptl", Def::F(10.0)),
    ("misset", Def::F(0.0)),
    ("ptheavy", Def::F(0.0)),
    ("ptonium", Def::F(1.0)),
    ("ptjmax", Def::F(-1.0)),
    ("ptbmax", Def::F(-1.0)),
    ("ptamax", Def::F(-1.0)),
    ("ptlmax", Def::F(-1.0)),
    ("missetmax", Def::F(-1.0)),
    // ── single-leg E ─────────────────────────────────────────────────────
    ("ej", Def::F(0.0)),
    ("eb", Def::F(0.0)),
    ("ea", Def::F(0.0)),
    ("el", Def::F(0.0)),
    ("ejmax", Def::F(-1.0)),
    ("ebmax", Def::F(-1.0)),
    ("eamax", Def::F(-1.0)),
    ("elmax", Def::F(-1.0)),
    // ── single-leg η ─────────────────────────────────────────────────────
    ("etaj", Def::F(5.0)),
    ("etab", Def::F(-1.0)),
    ("etaa", Def::F(2.5)),
    ("etal", Def::F(2.5)),
    ("etaonium", Def::F(0.6)),
    ("etajmin", Def::F(0.0)),
    ("etabmin", Def::F(0.0)),
    ("etaamin", Def::F(0.0)),
    ("etalmin", Def::F(0.0)),
    // ── pairwise ΔR ──────────────────────────────────────────────────────
    ("drjj", Def::F(0.4)),
    ("drbb", Def::F(0.0)),
    ("drll", Def::F(0.4)),
    ("draa", Def::F(0.4)),
    ("drbj", Def::F(0.0)),
    ("draj", Def::F(0.4)),
    ("drjl", Def::F(0.4)),
    ("drab", Def::F(0.0)),
    ("drbl", Def::F(0.0)),
    ("dral", Def::F(0.4)),
    ("drjjmax", Def::F(-1.0)),
    ("drbbmax", Def::F(-1.0)),
    ("drllmax", Def::F(-1.0)),
    ("draamax", Def::F(-1.0)),
    ("drbjmax", Def::F(-1.0)),
    ("drajmax", Def::F(-1.0)),
    ("drjlmax", Def::F(-1.0)),
    ("drabmax", Def::F(-1.0)),
    ("drblmax", Def::F(-1.0)),
    ("dralmax", Def::F(-1.0)),
    // ── pairwise invariant mass ──────────────────────────────────────────
    ("mmjj", Def::F(0.0)),
    ("mmbb", Def::F(0.0)),
    ("mmaa", Def::F(0.0)),
    ("mmll", Def::F(0.0)),
    ("mmjjmax", Def::F(-1.0)),
    ("mmbbmax", Def::F(-1.0)),
    ("mmaamax", Def::F(-1.0)),
    ("mmllmax", Def::F(-1.0)),
    ("mmnl", Def::F(0.0)),
    ("mmnlmax", Def::F(-1.0)),
    // ── dilepton-system pT ───────────────────────────────────────────────
    ("ptllmin", Def::F(0.0)),
    ("ptllmax", Def::F(-1.0)),
    ("xptj", Def::F(0.0)),
    ("xptb", Def::F(0.0)),
    ("xpta", Def::F(0.0)),
    ("xptl", Def::F(0.0)),
    // ── ordered-object pT ────────────────────────────────────────────────
    ("ptj1min", Def::F(0.0)),
    ("ptj1max", Def::F(-1.0)),
    ("ptj2min", Def::F(0.0)),
    ("ptj2max", Def::F(-1.0)),
    ("ptj3min", Def::F(0.0)),
    ("ptj3max", Def::F(-1.0)),
    ("ptj4min", Def::F(0.0)),
    ("ptj4max", Def::F(-1.0)),
    ("cutuse", Def::I(0)),
    ("ptl1min", Def::F(0.0)),
    ("ptl1max", Def::F(-1.0)),
    ("ptl2min", Def::F(0.0)),
    ("ptl2max", Def::F(-1.0)),
    ("ptl3min", Def::F(0.0)),
    ("ptl3max", Def::F(-1.0)),
    ("ptl4min", Def::F(0.0)),
    ("ptl4max", Def::F(-1.0)),
    // ── HT sums ──────────────────────────────────────────────────────────
    ("htjmin", Def::F(0.0)),
    ("htjmax", Def::F(-1.0)),
    ("ihtmin", Def::F(0.0)),
    ("ihtmax", Def::F(-1.0)),
    ("ht2min", Def::F(0.0)),
    ("ht3min", Def::F(0.0)),
    ("ht4min", Def::F(0.0)),
    ("ht2max", Def::F(-1.0)),
    ("ht3max", Def::F(-1.0)),
    ("ht4max", Def::F(-1.0)),
    // ── photon isolation / VBF / merging ─────────────────────────────────
    ("ptgmin", Def::F(0.0)),
    ("r0gamma", Def::F(0.4)),
    ("xn", Def::F(1.0)),
    ("epsgamma", Def::F(1.0)),
    ("isoem", Def::B(true)),
    ("xetamin", Def::F(0.0)),
    ("deltaeta", Def::F(0.0)),
    ("ktdurham", Def::F(-1.0)),
    ("dparameter", Def::F(0.4)),
    ("ptlund", Def::F(-1.0)),
    ("pdgs_for_merging_cut", Def::O),
    ("maxjetflavor", Def::I(4)),
    ("xqcut", Def::F(0.0)),
    // ── systematics ──────────────────────────────────────────────────────
    ("use_syst", Def::B(true)),
    ("systematics_program", Def::S("systematics")),
    ("systematics_arguments", Def::O),
    ("sys_scalefact", Def::S("0.5 1 2")),
    ("sys_alpsfact", Def::S("None")),
    ("sys_matchscale", Def::S("auto")),
    ("sys_pdf", Def::S("errorset")),
    ("sys_scalecorrelation", Def::I(-1)),
    // ── job handling / internals ─────────────────────────────────────────
    ("gridrun", Def::B(false)),
    ("fixed_couplings", Def::B(true)),
    ("mc_grouped_subproc", Def::B(true)),
    ("xmtcentral", Def::F(0.0)),
    ("d", Def::F(1.0)),
    ("gseed", Def::I(0)),
    ("issgridfile", Def::S("")),
    ("job_strategy", Def::I(0)),
    ("hard_survey", Def::I(0)),
    ("tmin_for_channel", Def::F(-1.0)),
    ("second_refine_treshold", Def::F(0.9)),
    ("survey_splitting", Def::I(-1)),
    ("survey_nchannel_per_job", Def::I(2)),
    ("refine_evt_by_job", Def::I(-1)),
    ("small_width_treatment", Def::F(1e-6)),
    ("hel_recycling", Def::B(true)),
    ("hel_filtering", Def::B(true)),
    ("hel_splitamp", Def::B(true)),
    ("hel_zeroamp", Def::B(true)),
    ("SDE_strategy", Def::I(1)),
    ("global_flag", Def::S("-O")),
    ("aloha_flag", Def::S("")),
    ("matrix_flag", Def::S("")),
    ("vector_size", Def::I(1)),
    ("nb_warp", Def::I(1)),
    ("vecsize_memmax", Def::I(0)),
    // ── per-PDG dict cuts (opaque; parse-and-detect in cuts.rs) ───────────
    ("pt_min_pdg", Def::O),
    ("pt_max_pdg", Def::O),
    ("E_min_pdg", Def::O),
    ("E_max_pdg", Def::O),
    ("eta_min_pdg", Def::O),
    ("eta_max_pdg", Def::O),
    ("mxx_min_pdg", Def::O),
    ("mxx_only_part_antipart", Def::O),
];

/// The MadGraph LO default value for a recognized parameter, or `None` for an
/// unknown name.
pub fn param_default(name: &str) -> Option<ParamValue> {
    PARAM_DEFAULTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v.to_value())
}

/// Resolve a card-line parameter name to its canonical [`PARAM_DEFAULTS`] name,
/// matching case-insensitively. MadGraph writes some parameters in a different
/// case than their internal name (`sde_strategy` vs `SDE_strategy`,
/// `e_min_pdg` vs `E_min_pdg`), so an exact match would spuriously reject them.
fn canonical_name(name: &str) -> Option<&'static str> {
    PARAM_DEFAULTS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(n, _)| *n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_card_reproduces_mg_defaults() {
        let rc = RunCard::parse("").unwrap();
        assert_eq!(rc.ebeam1, 6500.0);
        assert_eq!(rc.ebeam2, 6500.0);
        assert_eq!(rc.lpp1, 1);
        assert_eq!(rc.lpp2, 1);
        assert_eq!(rc.maxjetflavor, 4);
        assert_eq!(rc.nevents, 10000);
        assert_eq!(rc.scale, 91.1880);
        assert!(!rc.fixed_ren_scale);
        // Active default lepton cuts.
        assert_eq!(rc.float("ptl"), 10.0);
        assert_eq!(rc.float("etal"), 2.5);
        assert_eq!(rc.float("drll"), 0.4);
        assert_eq!(rc.float("dsqrt_shat"), 0.0);
        assert_eq!(rc.float("dsqrt_shatmax"), -1.0);
        assert_eq!(rc.float("mmll"), 0.0);
    }

    #[test]
    fn default_matches_parse_empty() {
        let a = RunCard::default();
        let b = RunCard::parse("").unwrap();
        for (name, _) in PARAM_DEFAULTS {
            assert_eq!(a.get(name), b.get(name), "param {name}");
        }
    }

    #[test]
    fn parses_value_name_comment_syntax() {
        let card = "\
# a comment
  20  = ptl   ! lepton pt
 3.0 = etal  ! lepton eta
 'lhapdf' = pdlabel ! pdf set
 T = fixed_fac_scale ! fix it
";
        let rc = RunCard::parse(card).unwrap();
        assert_eq!(rc.float("ptl"), 20.0);
        assert_eq!(rc.float("etal"), 3.0);
        assert_eq!(rc.pdlabel, "lhapdf");
        assert!(rc.fixed_fac_scale);
    }

    #[test]
    fn param_names_match_case_insensitively() {
        // Real MadGraph cards write these in a different case than banner.py's
        // internal name; they must resolve to the canonical key, not be rejected.
        let rc = RunCard::parse("  2 = sde_strategy\n").unwrap();
        assert_eq!(rc.int("SDE_strategy"), 2);
        let rc = RunCard::parse("  {6: 100} = e_min_pdg\n").unwrap();
        assert_eq!(rc.get("E_min_pdg").unwrap().as_str(), "{6: 100}");
    }

    #[test]
    fn empty_dict_opaque_normalizes_to_default() {
        // `{}` is MG's unset spelling for a per-pdg dict cut; it must read as the
        // empty default, not as an active override.
        let rc = RunCard::parse("  {} = pt_min_pdg\n").unwrap();
        assert_eq!(rc.get("pt_min_pdg"), param_default("pt_min_pdg").as_ref());
        // A non-empty dict is retained verbatim (and would trip the cut detector).
        let rc = RunCard::parse("  {6: 100} = pt_min_pdg\n").unwrap();
        assert_eq!(rc.get("pt_min_pdg").unwrap().as_str(), "{6: 100}");
    }

    #[test]
    fn unknown_param_is_hard_error() {
        let err = RunCard::parse("  10 = ptlx ! typo\n").unwrap_err();
        assert!(matches!(err, RunCardError::UnknownParam { .. }));
    }

    #[test]
    fn mixed_beams_rejected() {
        // A single lpp flipped to 0 leaves an unsupported (0, 1) pair.
        let err = RunCard::parse("  0 = lpp1\n").unwrap_err();
        assert!(matches!(
            err,
            RunCardError::UnsupportedLpp { lpp1: 0, lpp2: 1 }
        ));
    }

    #[test]
    fn polbeam1_nonzero_is_rejected() {
        let err = RunCard::parse("  1.0 = polbeam1\n").unwrap_err();
        assert!(matches!(err, RunCardError::UnsupportedPolarization { .. }));
        assert!(err.to_string().contains("beam polarization is not supported"));
    }

    #[test]
    fn polbeam2_nonzero_is_rejected() {
        let err = RunCard::parse("  -1.0 = polbeam2\n").unwrap_err();
        assert!(matches!(err, RunCardError::UnsupportedPolarization { .. }));
        assert!(err.to_string().contains("beam polarization is not supported"));
    }

    #[test]
    fn unpolarized_default_still_parses() {
        RunCard::parse("").unwrap();
    }

    #[test]
    fn fixed_energy_beams_accepted() {
        let rc =
            RunCard::parse("  0 = lpp1\n  0 = lpp2\n  250 = ebeam1\n  250 = ebeam2\n").unwrap();
        assert_eq!(rc.beam_mode(), BeamMode::FixedEnergy);
        assert_eq!(rc.ebeam1, 250.0);
        assert_eq!(rc.ebeam2, 250.0);
    }

    #[test]
    fn proton_beams_are_default_mode() {
        assert_eq!(RunCard::default().beam_mode(), BeamMode::Proton);
    }

    #[test]
    fn fortran_bool_and_d_exponent() {
        assert_eq!(parse_fortran_bool(".true."), Some(true));
        assert_eq!(parse_fortran_bool(".FALSE."), Some(false));
        assert_eq!(parse_fortran_bool("T"), Some(true));
        assert_eq!(parse_fortran_bool("f"), Some(false));
        assert_eq!(parse_f64("1.5d0"), Some(1.5));
        assert_eq!(parse_f64("91.1880"), Some(91.1880));
    }

    #[test]
    fn int_accepts_trailing_zero_fraction() {
        assert_eq!(parse_i64("4"), Some(4));
        assert_eq!(parse_i64("4.0"), Some(4));
        assert_eq!(parse_i64("4.5"), None);
    }

    #[test]
    fn parses_real_format_run_card() {
        // A card in MadGraph's LO `run_card.dat` syntax: exercises the format,
        // quoting and comment stripping a reference MadGraph run feeds in. It is
        // written for this test rather than copied out of a run, so its values are
        // free to differ from every banked card — see the file's own header.
        let text = include_str!("../tests/data/run_card_parser_fixture.dat");
        let rc = RunCard::parse(text).unwrap();
        assert_eq!(rc.nevents, 10000);
        assert_eq!(rc.ebeam1, 6500.0);
        assert_eq!(rc.pdlabel, "lhapdf");
        assert_eq!(rc.lhaid, 230000);
        assert!(rc.fixed_fac_scale);
        assert!(!rc.fixed_ren_scale);
        assert_eq!(rc.scale, 91.1880);
        assert_eq!(rc.maxjetflavor, 4);
        assert_eq!(rc.float("ptl"), 10.0);
        assert_eq!(rc.float("etal"), 2.5);
        assert_eq!(rc.float("drll"), 0.4);
        // Params omitted from the card fall back to MG defaults.
        assert_eq!(rc.float("pta"), 10.0);
        assert_eq!(rc.float("dsqrt_shatmax"), -1.0);
    }

    /// Transcription oracle: every scalar default in [`PARAM_DEFAULTS`] must
    /// match `RunCardLO.default_setup` as dumped by
    /// `validation/madgraph/dump_runcard_defaults.py`. Regenerate the JSON with
    /// `pixi run -e madgraph dump-runcard-defaults` after a MadGraph bump.
    #[test]
    fn defaults_match_banner_py_dump() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/runcard_defaults.json"
        );
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "missing defaults oracle {path}: {e}\n\
                 run `pixi run -e madgraph dump-runcard-defaults` to (re)generate it"
            )
        });
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap();
        let obj = dump.as_object().expect("oracle is a JSON object");

        for (name, def) in PARAM_DEFAULTS {
            let Some(actual) = obj.get(*name) else {
                panic!("parameter '{name}' absent from banner.py dump");
            };
            match def {
                Def::F(x) => {
                    let a = actual
                        .as_f64()
                        .unwrap_or_else(|| panic!("'{name}' expected numeric, got {actual}"));
                    let tol = 1e-9 * x.abs().max(1.0);
                    assert!((a - x).abs() <= tol, "'{name}': table {x} vs dump {a}");
                }
                Def::I(i) => {
                    let a = actual
                        .as_f64()
                        .unwrap_or_else(|| panic!("'{name}' expected numeric, got {actual}"));
                    assert_eq!(a, *i as f64, "'{name}': table {i} vs dump {a}");
                }
                Def::B(b) => {
                    assert_eq!(
                        actual.as_bool(),
                        Some(*b),
                        "'{name}': table {b} vs dump {actual}"
                    );
                }
                Def::S(s) => {
                    assert_eq!(
                        actual.as_str(),
                        Some(*s),
                        "'{name}': table {s:?} vs dump {actual}"
                    );
                }
                // Opaque list/dict params: name recognized, payload not compared.
                Def::O => {}
            }
        }
    }
}
