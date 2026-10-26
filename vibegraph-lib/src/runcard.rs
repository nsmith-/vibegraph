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

use crate::generate::{GenerateSettings, Systematics};

pub(crate) mod classes;
mod matching;

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
    pub(crate) fn as_f64(&self) -> f64 {
        match self {
            ParamValue::Float(x) => *x,
            ParamValue::Int(i) => *i as f64,
            other => panic!("parameter is not numeric: {other:?}"),
        }
    }

    pub(crate) fn as_i64(&self) -> i64 {
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

    pub(crate) fn as_str(&self) -> &str {
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
    /// Opaque list/dict-valued parameter, its default stored as the payload
    /// MadGraph writes into a card (`banner.py`'s `RunCard.write`), as
    /// [`parse_value`] reads it back: an empty list or dict is the empty
    /// payload.
    O(&'static str),
}

impl Def {
    fn to_value(self) -> ParamValue {
        match self {
            Def::F(x) => ParamValue::Float(x),
            Def::I(i) => ParamValue::Int(i),
            Def::B(b) => ParamValue::Bool(b),
            Def::S(s) => ParamValue::Str(s.to_string()),
            Def::O(s) => ParamValue::Opaque(s.to_string()),
        }
    }
}

/// The parton densities a run card reads ([`RunCard::pdf_choice`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfChoice {
    /// Fixed-energy beams: no density is read.
    None,
    /// Proton beams under `pdlabel = lhapdf`: the LHAPDF id the card names.
    Lhapdf { lhaid: i64 },
}

/// The `pdlabel`s that name one of MadGraph's built-in parameterisations, or a
/// lepton or photon flux, rather than an LHAPDF grid (`pdfwrap.f`,
/// `banner.py`'s `get_pdf_id`).
const BUILTIN_PDLABELS: &[&str] = &[
    "nn23lo1", "nn23lo", "nn23nlo", "cteq6l1", "cteq6_m", "cteq6_l", "cteq6_d", "none", "iww",
    "eva", "edff", "chff",
];

/// A proton-beam run card whose `pdlabel` is not `lhapdf`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", pdlabel_refusal(.label))]
pub struct PdLabelRefused {
    pub label: String,
}

fn pdlabel_refusal(label: &str) -> String {
    let what = if BUILTIN_PDLABELS.contains(&label) {
        ", MadGraph's built-in parameterisation"
    } else {
        ""
    };
    let hint = if label == "nn23lo1" {
        " (247000 is NNPDF23_lo_as_0130_qed, the set nn23lo1 is fitted to)"
    } else {
        ""
    };
    format!(
        "run card sets pdlabel = {label}{what}; vibegraph reads parton densities only from \
         LHAPDF6 grids. Set pdlabel = lhapdf and lhaid to the set's LHAPDF id{hint}."
    )
}

/// The proton mass MadGraph gives a proton beam (`lpp = 1`): `banner.py` refuses
/// a lower beam energy and `genps.f` builds `stot` from beams of this mass.
const PROTON_BEAM_MASS: f64 = 0.938;

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
        "run card sets {name} = {value} on a proton beam: a proton beam carries at least the \
         proton mass, {PROTON_BEAM_MASS} GeV. MadGraph refuses an energy below it, and runs 0 \
         as a proton at rest (banner.py, RunCardLO.check_validity), a massive beam that this \
         build's massless proton kinematics cannot represent"
    )]
    ProtonBeamBelowMass { name: &'static str, value: f64 },
    #[error(
        "run card sets me_frame = '{value}': a frame is a list of leg numbers, such as \
         '1, 2' (the partonic centre of mass)"
    )]
    BadFrame { value: String },
    #[error(
        "run card sets ickkw = {ickkw}: MadGraph admits only 0 and 1 (banner.py declares ickkw \
         with allowed = [0, 1])"
    )]
    UnsupportedIckkw { ickkw: i64 },
    #[error(
        "run card sets maxjetflavor = 6 with ickkw = 1: MadGraph refuses it ('maxjetflavor at 6 \
         is NOT supported for matching', banner.py:4556)"
    )]
    MatchedTopJets,
    #[error(
        "run card sets xqcut = {xqcut} with a jet pT threshold of {ptj} after MadGraph's \
         rewrites (ptj follows xqcut only when auto_ptj_mjj = T and ptj >= 0): MadEvent's lower \
         limit on tau then carries energy floors of sqrt(xqcut^2 - m^2) per jet that no cut \
         implies, a cut that differs between integration channels (myamp.f setxqcuts) and is \
         not implemented"
    )]
    XqcutAboveJetThreshold { xqcut: f64, ptj: f64 },
    #[error(
        "run card sets pdlabel1 = '{beam1}' and pdlabel2 = '{beam2}' on proton beams: MadGraph \
         refuses an asymmetric proton-proton PDF (banner.py's PDLabelBlock)"
    )]
    AsymmetricBeamPdf { beam1: String, beam2: String },
    #[error(
        "run card sets event_norm = '{value}': MadGraph admits only 'average', 'sum' and 'unity' \
         (banner.py declares event_norm with allowed = ['sum', 'average', 'unity'])"
    )]
    UnsupportedEventNorm { value: String },
    #[error("run card sets iseed = {iseed}: a seed is 0, for none, or a positive integer")]
    NegativeSeed { iseed: i64 },
    #[error("run card sets nevents = {nevents}: an event count is 0 or a positive integer")]
    NegativeEventCount { nevents: i64 },
    #[error("run card sets '{name}' to {value} (MadGraph default {default}): {why}")]
    UnsupportedField {
        name: String,
        value: String,
        default: String,
        why: &'static str,
    },
}

/// The run card's `event_norm`: how the written event weights `XWGTUP` are
/// normalised. It changes the file and not the integration: the cross section
/// is the same under all three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventNorm {
    /// The weights' mean is the cross section (`IDWTUP = -4`); the card default.
    Average,
    /// The weights' sum is the cross section (`IDWTUP = -3`).
    Sum,
    /// Every weight is one (`IDWTUP = -3`).
    ///
    /// MadGraph admits `unity` on the card, but its unweighting compares the
    /// normalization against `'unit'` (`lhe_parser.py:1232` at b7687064). A
    /// MadEvent run of `e+ e- > mu+ mu-` under `event_norm = unity` (3.7.1 at
    /// b7687064, 1000 events, seed 20261021) aborted in
    /// `MultiEventFile.unweight` with `UnboundLocalError` on `new_wgt`
    /// (`lhe_parser.py:1238`) before any event file was written, so no MadGraph
    /// sample exists to compare against. This follows the `'unit'` branch
    /// MadGraph evidently intends.
    Unity,
}

impl EventNorm {
    /// The value MadGraph's `allowed` list matches `text` to, case-insensitively
    /// (`ConfigFile.__setitem__`, `banner.py:1285-1298`), or `None` for a value
    /// outside the list.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        match text.to_ascii_lowercase().as_str() {
            "average" => Some(EventNorm::Average),
            "sum" => Some(EventNorm::Sum),
            "unity" => Some(EventNorm::Unity),
            _ => None,
        }
    }

    /// The spelling the card carries.
    pub fn as_str(self) -> &'static str {
        match self {
            EventNorm::Average => "average",
            EventNorm::Sum => "sum",
            EventNorm::Unity => "unity",
        }
    }
}

/// The seed an integration runs at when neither the command line nor the run
/// card's `iseed` names one.
pub const DEFAULT_INTEGRATION_SEED: u64 = 20_260_719;

/// A parameter value as a card line spells it: the text MadGraph's own record
/// of the card writes for it: a real in its shortest round-tripping form with a
/// decimal point (`20.0`), a flag as `True` or `False`, and a list or string as
/// its text.
pub(crate) fn card_text(value: &ParamValue) -> String {
    match value {
        ParamValue::Float(x) => format!("{x:?}"),
        ParamValue::Int(i) => i.to_string(),
        ParamValue::Bool(true) => "True".to_string(),
        ParamValue::Bool(false) => "False".to_string(),
        ParamValue::Str(s) | ParamValue::Opaque(s) => s.clone(),
    }
}

/// How the incoming state of a run is prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeamMode {
    /// Proton beams (`lpp = 1`): parton momenta drawn from PDFs, ŝ = x₁x₂ s with
    /// s = 4·ebeam1·ebeam2.
    Proton,
    /// Fixed-energy partonic beams (`lpp = 0`): the incoming particles are the
    /// beam particles themselves, ŝ = (p₁ + p₂)² of the two beams on their mass
    /// shells (4·ebeam1·ebeam2 when massless), no PDF convolution.
    FixedEnergy,
}

/// A run card resolved against the MadGraph LO defaults.
///
/// Typed fields expose the non-cut parameters consumed by the generator; every
/// parameter (including all cut thresholds and recognized-but-unconsumed
/// bookkeeping params) is retained in [`RunCard::values`] keyed by name.
///
/// A card as parsed holds every recognized parameter. [`RunCard::split`] narrows
/// it to the parameters an integration reads, which is the card an integration
/// artifact records and every later step reads back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCard {
    pub(crate) iseed: i64,
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
    /// The parameters whose value in MadGraph's own record of the card differs
    /// from the resolved one ([`banner_values`](Self::banner_values)). Recorded
    /// with the card, since the resolved values alone do not determine them.
    banner_overrides: BTreeMap<String, ParamValue>,
    /// Whether the card spelled `pdlabel` on a line of its own. MadGraph's
    /// `PDFSUP` reads that line and nothing else, so on fixed-energy beams a
    /// card without it reports MadGraph's template label ([`RunCard::pdfsup`]).
    pdlabel_written: bool,
}

/// The `pdlabel` of MadGraph's LO template (`banner.py:4253`), which its
/// event-file `<init>` reports for a card with no `pdlabel` line.
const MADGRAPH_TEMPLATE_PDLABEL: &str = "nn23lo1";

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

    /// The seed an integration over this card runs at: `flag` when the command
    /// line gives one, else the card's `iseed` when it is not 0, else
    /// [`DEFAULT_INTEGRATION_SEED`]. MadGraph reads `iseed = 0` as "assign one",
    /// so a card at its default asks for no particular seed.
    pub fn integration_seed(&self, flag: Option<u64>) -> u64 {
        match flag {
            Some(seed) => seed,
            None if self.iseed != 0 => {
                u64::try_from(self.iseed).expect("from_values refuses a negative iseed")
            }
            None => DEFAULT_INTEGRATION_SEED,
        }
    }

    /// Split a card as parsed into the card an integration reads and the
    /// settings a generation reads ([`classes::Stage`]).
    ///
    /// The card returned holds every [`classes::Stage::Integrate`] parameter
    /// and none of the others. `use_syst` goes to the settings: what it does at
    /// integration, forcing `alpsfact` to 1, is already in the resolved card,
    /// so a generation that changes it does not move `alpsfact`. The three
    /// generate-stage fields that are refused off their default
    /// (`time_of_flight`, `boost_event`, `lhe_version`) are not carried.
    ///
    /// Panics on a card that has already been split.
    pub fn split(mut self) -> (RunCard, GenerateSettings) {
        let mut taken = BTreeMap::new();
        for name in classes::generate_fields() {
            let value = self
                .values
                .remove(name)
                .unwrap_or_else(|| panic!("'{name}' is absent: the card was split already"));
            self.banner_overrides.remove(name);
            taken.insert(name, value);
        }
        let event_norm = EventNorm::parse(taken["event_norm"].as_str())
            .expect("from_values admits only MadGraph's event_norm values");
        let systematics = Systematics {
            enabled: taken["use_syst"].as_bool(),
            fields: classes::systematics_request_fields()
                .map(|name| (name.to_string(), card_text(&taken[name])))
                .collect(),
        };
        let settings = GenerateSettings {
            nevents: u64::try_from(taken["nevents"].as_i64())
                .expect("from_values refuses a negative nevents"),
            event_norm,
            systematics,
            clustering_info: taken["clusinfo"].as_bool(),
        };
        (self, settings)
    }

    /// Every parameter of a run as MadGraph records it in an event file's
    /// `<MGRunCard>`: this integration card's [`banner_values`](Self::banner_values),
    /// the generation's `settings`, the generate-stage fields that are refused
    /// off their default at that default, and `iseed` as the generation's
    /// `seed` where it fits MadGraph's 32-bit integer (the card's own otherwise).
    pub(crate) fn banner_with(
        &self,
        settings: &GenerateSettings,
        seed: u64,
    ) -> BTreeMap<String, ParamValue> {
        let mut out: BTreeMap<String, ParamValue> = self
            .banner_values()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect();
        for name in classes::generate_fields() {
            out.insert(
                name.to_string(),
                param_default(name).expect("a recognized parameter"),
            );
        }
        let mut set = |name: &str, value: ParamValue| {
            out.insert(name.to_string(), value);
        };
        set(
            "nevents",
            ParamValue::Int(i64::try_from(settings.nevents).unwrap_or(i64::MAX)),
        );
        set(
            "event_norm",
            ParamValue::Str(settings.event_norm.as_str().to_string()),
        );
        set("use_syst", ParamValue::Bool(settings.systematics.enabled));
        set("clusinfo", ParamValue::Bool(settings.clustering_info));
        for (name, text) in &settings.systematics.fields {
            set(name, ParamValue::Str(text.clone()));
        }
        if let Ok(seed) = i32::try_from(seed) {
            set("iseed", ParamValue::Int(i64::from(seed)));
        }
        out
    }

    /// MadGraph's `frame_id` for the card's `me_frame`: the legs whose summed
    /// momentum defines the rest frame a matrix element that is not Lorentz
    /// invariant is evaluated in, as `Σ 2^n` over the listed leg numbers
    /// (`update_system_parameter_for_include`, `banner.py:4705`). The default
    /// `[1, 2]` is 6, which MadEvent treats as "no boost": the partonic
    /// centre-of-mass momenta it generates reach the matrix element unchanged
    /// (`auto_dsig_v4.inc:134`). The card's own `frame_id` is not read, since
    /// MadGraph overwrites it from `me_frame`.
    pub(crate) fn frame_id(&self) -> Result<i64, RunCardError> {
        let raw = self
            .values
            .get("me_frame")
            .expect("me_frame is a recognized parameter")
            .as_str();
        frame_id_of(raw)
    }

    /// The `PDFSUP` this card's event file reports.
    ///
    /// On proton beams it is the LHAPDF id the densities are read from
    /// ([`pdf_choice`](Self::pdf_choice)). On fixed-energy beams, which read no
    /// density, it is what MadEvent writes: the `<init>` line comes from
    /// `get_banner_init_information` (`banner.py:3824-3837`) over the banner's
    /// run card, which `charge_card` reads with `consistency=False`
    /// (`banner.py:538`), so `PDLabelBlock` never folds the per-beam labels into
    /// `pdlabel`. `PDFSUP` is then `get_pdf_id(pdlabel)` (`banner.py:3839-3853`)
    /// of the card's own `pdlabel` line, or of the template's `nn23lo1` when
    /// the card has none: `247000` on every banked MadEvent 3.7.1
    /// lepton-collider card, which spells `none` per beam and `lhaid = 230000`.
    pub fn pdfsup(&self) -> i32 {
        let label = match self.pdf_choice() {
            Ok(PdfChoice::Lhapdf { lhaid }) => return i32::try_from(lhaid).unwrap_or(0),
            Ok(PdfChoice::None) if !self.pdlabel_written => MADGRAPH_TEMPLATE_PDLABEL,
            _ => self.pdlabel.as_str(),
        };
        match label {
            "lhapdf" => i32::try_from(self.lhaid).unwrap_or(0),
            "cteq6_m" => 10000,
            "cteq6_l" => 10041,
            "cteq6l1" => 10042,
            "nn23lo" => 246800,
            "nn23lo1" => 247000,
            "nn23nlo" => 244800,
            _ => 0,
        }
    }

    /// The parton densities this card reads: none on fixed-energy beams, where
    /// the label is inert (`setrun.f` overwrites it with `none`), and on proton
    /// beams the LHAPDF id `lhaid` under `pdlabel = lhapdf`. Every other label
    /// is refused on proton beams: each names a parameterisation of MadGraph's
    /// own, not an LHAPDF6 grid this crate can read.
    pub fn pdf_choice(&self) -> Result<PdfChoice, PdLabelRefused> {
        if self.beam_mode() == BeamMode::FixedEnergy {
            return Ok(PdfChoice::None);
        }
        match self.pdlabel.as_str() {
            "lhapdf" => Ok(PdfChoice::Lhapdf { lhaid: self.lhaid }),
            label => Err(PdLabelRefused {
                label: label.to_string(),
            }),
        }
    }

    /// Iterate all resolved (name, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ParamValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Parse a run card from text.
    pub fn parse(text: &str) -> Result<Self, RunCardError> {
        let mut pdlabel_written = false;
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
            pdlabel_written |= canonical == "pdlabel";
            values.insert(canonical.to_string(), parsed);
        }

        let mut card = Self::from_values(values)?;
        card.pdlabel_written = pdlabel_written;
        Ok(card)
    }

    /// Parse a run card from a file path.
    pub fn parse_file(path: &Path) -> Result<Self, RunCardError> {
        let text = std::fs::read_to_string(path).map_err(|source| RunCardError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::parse(&text)
    }

    /// MadGraph's out-of-the-box run card for a `1 → n` decay process: the LO
    /// defaults with every cut removed (`banner.py`'s `create_default_for_process`,
    /// which calls `remove_all_cut` when `ninitial == 1`), `SDE_strategy = 1`
    /// (forced there for one initial particle) and systematics off.
    ///
    /// `remove_all_cut` resets each of the card's cut parameters by name: a
    /// boolean to false, a `min` to `0`, a `max` to `-1`, a rapidity bound to
    /// `-1` and anything else to `0`. [`DECAY_CUT_RESETS`] is that rule applied to
    /// every cut parameter whose LO default it moves.
    pub fn decay_default() -> Self {
        let mut values = Self::default().values;
        for (name, value) in DECAY_CUT_RESETS {
            values.insert((*name).to_string(), value.to_value());
        }
        values.insert("SDE_strategy".to_string(), ParamValue::Int(1));
        values.insert("use_syst".to_string(), ParamValue::Bool(false));
        Self::from_values(values).expect("the decay defaults are self-consistent")
    }

    /// This card as MadEvent runs it for the decay of a particle of pole mass
    /// `mass` (`setcuts.f`, the `nincoming = 1` branch).
    ///
    /// A decay has no parton densities, so both beams become fixed-energy ones at
    /// `M/2`. The renormalisation scale is fixed at `M` unless the card already
    /// fixes it, and both factorisation scales are fixed at the card's own
    /// `dsqrt_q2fact1`/`dsqrt_q2fact2` — so no dynamical prescription, clustering
    /// or otherwise, is ever evaluated for a decay, whatever
    /// `dynamical_scale_choice` says. Systematics are switched off.
    ///
    /// The card returned holds the same parameters as this one, so the decay
    /// card of an integration card is an integration card.
    pub(crate) fn for_decay(&self, mass: f64) -> Self {
        let mut values: BTreeMap<String, ParamValue> = PARAM_DEFAULTS
            .iter()
            .map(|(name, default)| (name.to_string(), default.to_value()))
            .collect();
        values.extend(self.values.clone());
        let mut set = |name: &str, value: ParamValue| {
            values.insert(name.to_string(), value);
        };
        set("lpp1", ParamValue::Int(0));
        set("lpp2", ParamValue::Int(0));
        set("ebeam1", ParamValue::Float(mass / 2.0));
        set("ebeam2", ParamValue::Float(mass / 2.0));
        if !self.fixed_ren_scale {
            set("scale", ParamValue::Float(mass));
            set("fixed_ren_scale", ParamValue::Bool(true));
        }
        set("fixed_fac_scale", ParamValue::Bool(true));
        set("fixed_fac_scale1", ParamValue::Bool(true));
        set("fixed_fac_scale2", ParamValue::Bool(true));
        set("use_syst", ParamValue::Bool(false));
        let mut card = Self::from_values(values)
            .expect("a decay card relaxes the beam checks, never tightens them");
        card.values.retain(|name, _| self.values.contains_key(name));
        card.banner_overrides
            .retain(|name, _| self.values.contains_key(name));
        card.pdlabel_written = self.pdlabel_written;
        card
    }

    /// Every parameter as MadGraph records the card in an event file's
    /// `<MGRunCard>`: after `banner.py`'s own edits and before the Fortran's
    /// (`setrun.f`, `setcuts.f`), which rerun on any card read back. The two
    /// differ only on a matched card: `setcuts.f`'s jet-cut rewrite under
    /// `xqcut` and `setrun.f`'s `alpsfact` under `use_syst` are the Fortran's,
    /// so the record keeps the card's `ptj`, `mmjj` and (without matching)
    /// `alpsfact`.
    pub(crate) fn banner_values(&self) -> impl Iterator<Item = (&str, &ParamValue)> {
        self.values.iter().map(|(k, v)| {
            (
                k.as_str(),
                self.banner_overrides.get(k.as_str()).unwrap_or(v),
            )
        })
    }

    fn from_values(mut values: BTreeMap<String, ParamValue>) -> Result<Self, RunCardError> {
        let raw = values
            .get("event_norm")
            .expect("event_norm is a recognized parameter")
            .as_str()
            .to_string();
        let norm =
            EventNorm::parse(&raw).ok_or(RunCardError::UnsupportedEventNorm { value: raw })?;
        values.insert(
            "event_norm".to_string(),
            ParamValue::Str(norm.as_str().to_string()),
        );
        let as_parsed = values.clone();
        matching::resolve(&mut values)?;
        let beam = |name: &str| values.get(name).expect("known param").as_i64();
        if (beam("lpp1"), beam("lpp2")) == (1, 1) {
            resolve_beam_pdf_labels(&mut values)?;
        }
        let f = |name: &str| values.get(name).expect("known param").as_f64();
        let i = |name: &str| values.get(name).expect("known param").as_i64();
        let b = |name: &str| values.get(name).expect("known param").as_bool();
        let s = |name: &str| values.get(name).expect("known param").as_str().to_string();

        let lpp1 = i("lpp1");
        let lpp2 = i("lpp2");
        if (lpp1, lpp2) != (1, 1) && (lpp1, lpp2) != (0, 0) {
            return Err(RunCardError::UnsupportedLpp { lpp1, lpp2 });
        }
        if (lpp1, lpp2) == (1, 1) {
            for name in ["ebeam1", "ebeam2"] {
                let value = f(name);
                if !(value.is_finite() && value >= PROTON_BEAM_MASS) {
                    return Err(RunCardError::ProtonBeamBelowMass { name, value });
                }
            }
        }
        classes::refuse_ignored_physics(&values)?;
        classes::warn_unwritten_systematics(&values);
        frame_id_of(values.get("me_frame").expect("known param").as_str())?;

        let iseed = i("iseed");
        if iseed < 0 {
            return Err(RunCardError::NegativeSeed { iseed });
        }
        let nevents = i("nevents");
        if nevents < 0 {
            return Err(RunCardError::NegativeEventCount { nevents });
        }
        Ok(RunCard {
            iseed,
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
            banner_overrides: matching::banner_overrides(&as_parsed, &values),
            pdlabel_written: false,
            values,
        })
    }
}

/// `banner.py`'s `PDLabelBlock` at proton beams: a card spelling the PDF per beam
/// (`pdlabel1`, `pdlabel2`) sets `pdlabel` from them, and MadGraph refuses two
/// different sets on two proton beams. A card that leaves both at their default
/// spells the PDF through `pdlabel` and is left alone, so a card that never
/// names the per-beam labels resolves to exactly the values it parsed to.
fn resolve_beam_pdf_labels(values: &mut BTreeMap<String, ParamValue>) -> Result<(), RunCardError> {
    let label = |values: &BTreeMap<String, ParamValue>, name: &str| {
        values.get(name).expect("known param").as_str().to_string()
    };
    let (beam1, beam2) = (label(values, "pdlabel1"), label(values, "pdlabel2"));
    let default = |name: &str| {
        param_default(name)
            .expect("known param")
            .as_str()
            .to_string()
    };
    if beam1 == default("pdlabel1") && beam2 == default("pdlabel2") {
        return Ok(());
    }
    if beam1 != beam2 {
        return Err(RunCardError::AsymmetricBeamPdf { beam1, beam2 });
    }
    values.insert("pdlabel".to_string(), ParamValue::Str(beam1));
    Ok(())
}

/// `frame_id` of an `me_frame` payload: `[1, 2]`, `1, 2` and `1 2` are all the
/// same list. An empty list is refused: MadGraph would boost into the rest
/// frame of no momentum at all.
fn frame_id_of(raw: &str) -> Result<i64, RunCardError> {
    let bad = || RunCardError::BadFrame {
        value: raw.to_string(),
    };
    let inner = raw.trim().trim_start_matches('[').trim_end_matches(']');
    let legs = inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .map(|t| t.parse::<u32>().map_err(|_| bad()))
        .collect::<Result<Vec<_>, _>>()?;
    if legs.is_empty() {
        return Err(bad());
    }
    legs.iter().try_fold(0i64, |acc, &n| {
        1i64.checked_shl(n)
            .filter(|_| n < 62)
            .map(|bit| acc + bit)
            .ok_or_else(bad)
    })
}

/// Split a raw card line into `(value, name)`, or `None` for comment / blank /
/// structural lines. The comment is stripped first, at the first `#` or `!`, so
/// a hidden field written `<value> = <name> # hidden_parameter` reads as
/// `<value> = <name>`; a value cannot itself contain `#` or `!`. The line is
/// then split on its first `=`.
fn split_line(raw: &str) -> Option<(&str, &str)> {
    let no_comment = match raw.find(['#', '!']) {
        Some(pos) => &raw[..pos],
        None => raw,
    };
    let trimmed = no_comment.trim();
    if trimmed.is_empty() || trimmed.starts_with('$') {
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
/// name is recognized while the payload is not modeled; each default payload is
/// MadGraph's own default as its card writer spells it.
#[rustfmt::skip]
static PARAM_DEFAULTS: &[(&str, Def)] = &[
    // ── run / seeding ────────────────────────────────────────────────────
    ("run_tag", Def::S("tag_1")),
    ("gridpack", Def::B(false)),
    ("time_of_flight", Def::F(-1.0)),
    ("nevents", Def::I(10000)),
    ("iseed", Def::I(0)),
    ("bypass_check", Def::O("")),
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
    // A departure from MadGraph's template (`nn23lo1`, `230000`): vibegraph
    // reads parton densities only from LHAPDF6 grids and refuses every built-in
    // `pdlabel` on proton beams, so a card that names no PDF reads LHAPDF set
    // 247000, NNPDF23_lo_as_0130_qed, the fit `nn23lo1` parameterises. Its
    // PDFSUP is 247000 under either spelling.
    ("pdlabel", Def::S("lhapdf")),
    ("pdlabel1", Def::S("lhapdf")),
    ("pdlabel2", Def::S("lhapdf")),
    ("lhaid", Def::I(247000)),
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
    ("bias_parameters", Def::O("")),
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
    ("custom_fcts", Def::O("")),
    // ── output / frame ───────────────────────────────────────────────────
    ("lhe_version", Def::F(3.0)),
    ("boost_event", Def::S("False")),
    ("me_frame", Def::O("1, 2")),
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
    ("pdgs_for_merging_cut", Def::O("21, 1, 2, 3, 4, 5, 6")),
    ("maxjetflavor", Def::I(4)),
    ("xqcut", Def::F(0.0)),
    // ── systematics ──────────────────────────────────────────────────────
    ("use_syst", Def::B(true)),
    ("systematics_program", Def::S("systematics")),
    ("systematics_arguments", Def::O("['--mur=0.5,1,2', '--muf=0.5,1,2', '--pdf=errorset']")),
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
    ("pt_min_pdg", Def::O("")),
    ("pt_max_pdg", Def::O("")),
    ("E_min_pdg", Def::O("")),
    ("E_max_pdg", Def::O("")),
    ("eta_min_pdg", Def::O("")),
    ("eta_max_pdg", Def::O("")),
    ("mxx_min_pdg", Def::O("")),
    ("mxx_only_part_antipart", Def::O("{'default': False}")),
];

/// The cut parameters whose LO default `remove_all_cut` moves, with the value it
/// moves them to — the whole difference between [`RunCard::default`] and
/// [`RunCard::decay_default`] on the cuts. Every other cut parameter already sits
/// at its reset value.
const DECAY_CUT_RESETS: &[(&str, Def)] = &[
    ("deltaeta", Def::F(-1.0)),
    ("dparameter", Def::F(0.0)),
    ("draa", Def::F(0.0)),
    ("draj", Def::F(0.0)),
    ("dral", Def::F(0.0)),
    ("drjj", Def::F(0.0)),
    ("drjl", Def::F(0.0)),
    ("drll", Def::F(0.0)),
    ("etaa", Def::F(-1.0)),
    ("etaj", Def::F(-1.0)),
    ("etal", Def::F(-1.0)),
    ("ktdurham", Def::F(0.0)),
    ("pta", Def::F(0.0)),
    ("ptj", Def::F(0.0)),
    ("ptl", Def::F(0.0)),
    ("ptlund", Def::F(0.0)),
];

/// The value `remove_all_cut` resets a cut parameter to, where that differs from
/// its LO default ([`DECAY_CUT_RESETS`]). Both are values at which MadGraph's
/// `cuts.f` leaves the cut off.
pub(crate) fn decay_cut_reset(name: &str) -> Option<ParamValue> {
    DECAY_CUT_RESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, d)| d.to_value())
}

/// The MadGraph LO default value for a recognized parameter, or `None` for an
/// unknown name.
pub(crate) fn param_default(name: &str) -> Option<ParamValue> {
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

/// The card text `banner.py`'s `RunCard.write` writes for a list or dict
/// value, from its JSON dump: a list of strings as `['a', 'b']`, any other
/// list as `1, 2`, and a dict as Python's repr, `{'key': False}`. An empty
/// list is written `[]`, which the parser reads as the empty payload whatever
/// the element type.
#[cfg(test)]
pub(crate) fn card_spelling(value: &serde_json::Value) -> String {
    use serde_json::Value;
    fn scalar(v: &Value) -> String {
        match v {
            Value::Bool(true) => "True".to_string(),
            Value::Bool(false) => "False".to_string(),
            Value::String(s) => format!("'{s}'"),
            other => other.to_string(),
        }
    }
    match value {
        Value::Array(items) if items.is_empty() => "[]".to_string(),
        Value::Array(items) if items.iter().all(Value::is_string) => {
            let quoted: Vec<String> = items.iter().map(scalar).collect();
            format!("[{}]", quoted.join(", "))
        }
        Value::Array(items) => items
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                other => scalar(other),
            })
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(entries) => {
            let pairs: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let key = if k.parse::<i64>().is_ok() {
                        k.clone()
                    } else {
                        format!("'{k}'")
                    };
                    format!("{key}: {}", scalar(v))
                })
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        other => panic!("{other} is not a list or a dict"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `event_norm` takes MadGraph's three values, spelled in any case, and an
    /// unset card resolves to `average`; a value outside the list is refused at
    /// the parse rather than carried to the writer.
    #[test]
    fn event_norm_is_parsed_against_madgraphs_allowed_list() {
        assert_eq!(RunCard::default().split().1.event_norm, EventNorm::Average);
        for (text, norm) in [
            ("average", EventNorm::Average),
            ("sum", EventNorm::Sum),
            ("Sum", EventNorm::Sum),
            ("UNITY", EventNorm::Unity),
        ] {
            let card = RunCard::parse(&format!("  {text} = event_norm\n")).expect(text);
            assert_eq!(card.get("event_norm").unwrap().as_str(), norm.as_str());
            assert_eq!(card.split().1.event_norm, norm, "{text}");
        }
        for text in ["unit", "bias", "mean", "average sum"] {
            let err = RunCard::parse(&format!("  {text} = event_norm\n"))
                .expect_err("a value outside the allowed list");
            assert!(
                matches!(err, RunCardError::UnsupportedEventNorm { .. }),
                "{text}: {err}"
            );
        }
    }

    /// The split hands every generate-stage field to the settings and keeps
    /// every other one: the card returned holds no generate field, the settings
    /// carry the card's values, and the integration card is resolved as before,
    /// `alpsfact` forced to 1 under `use_syst` included.
    #[test]
    fn split_hands_the_generate_fields_to_the_settings() {
        let parsed = RunCard::parse(
            "250 = nevents\nsum = event_norm\nT = use_syst\n2 = alpsfact\n\
             F = clusinfo\n'0.5 1' = sys_scalefact\n3 = sys_scalecorrelation\n\
             15 = ptl\n",
        )
        .expect("card");
        let (card, settings) = parsed.clone().split();
        let generate: Vec<&str> = classes::generate_fields().collect();
        for (name, value) in parsed.iter() {
            if generate.contains(&name) {
                assert!(card.get(name).is_none(), "'{name}' stayed on the card");
            } else {
                assert_eq!(card.get(name), Some(value), "'{name}' moved");
            }
        }
        assert_eq!(card.iter().count() + generate.len(), PARAM_DEFAULTS.len());
        assert_eq!(card.float("alpsfact"), 1.0, "resolved under use_syst");
        assert_eq!(settings.nevents, 250);
        assert_eq!(settings.event_norm, EventNorm::Sum);
        assert!(settings.systematics.enabled);
        assert!(!settings.clustering_info);
        let field = |name: &str| {
            settings
                .systematics
                .fields
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(field("sys_scalefact"), Some("0.5 1"));
        assert_eq!(field("sys_scalecorrelation"), Some("3"));
        assert_eq!(field("sys_pdf"), Some("errorset"));
        assert_eq!(settings.systematics.fields.len(), 7);
    }

    /// The integration card survives the artifact's encoding with MadGraph's
    /// own record of it: the `ptj` the card wrote under `xqcut`, which the
    /// resolved card no longer holds, reads back.
    #[test]
    fn a_split_card_keeps_madgraphs_record_through_an_encoding() {
        let (card, _) = RunCard::parse("1 = ickkw\n30 = xqcut\n45 = ptj\n")
            .expect("card")
            .split();
        let back: RunCard =
            bincode::deserialize(&bincode::serialize(&card).expect("encode")).expect("decode");
        let banner = |rc: &RunCard| -> Vec<(String, ParamValue)> {
            rc.banner_values()
                .map(|(n, v)| (n.to_string(), v.clone()))
                .collect()
        };
        assert_eq!(banner(&back), banner(&card));
        assert_eq!(
            back.banner_values()
                .find(|(n, _)| *n == "ptj")
                .map(|(_, v)| v.clone()),
            Some(ParamValue::Float(45.0))
        );
    }

    /// The integration seed: the command line's when given, else the card's
    /// `iseed` when it is not 0, else the fixed default.
    #[test]
    fn the_integration_seed_is_the_flag_then_iseed_then_the_default() {
        let unset = RunCard::parse("").expect("card");
        let set = RunCard::parse("  31 = iseed\n").expect("card");
        assert_eq!(unset.integration_seed(None), DEFAULT_INTEGRATION_SEED);
        assert_eq!(unset.integration_seed(Some(5)), 5);
        assert_eq!(set.integration_seed(None), 31);
        assert_eq!(set.integration_seed(Some(5)), 5);
        assert_eq!(set.clone().split().0.integration_seed(None), 31);
        let explicit_zero = RunCard::parse("  0 = iseed\n").expect("card");
        assert_eq!(
            explicit_zero.integration_seed(None),
            DEFAULT_INTEGRATION_SEED
        );
    }

    #[test]
    fn a_negative_iseed_is_refused() {
        let err = RunCard::parse("  -3 = iseed\n").expect_err("a negative seed");
        assert!(
            matches!(err, RunCardError::NegativeSeed { iseed: -3 }),
            "{err}"
        );
    }

    #[test]
    fn a_negative_nevents_is_refused() {
        let err = RunCard::parse("  -5 = nevents\n").expect_err("a negative event count");
        assert!(
            matches!(err, RunCardError::NegativeEventCount { nevents: -5 }),
            "{err}"
        );
        assert!(err.to_string().contains("nevents = -5"), "{err}");
    }

    /// A decay's card derived from an integration card is an integration card:
    /// the same parameters, no generate field brought back.
    #[test]
    fn a_decay_card_of_a_split_card_holds_the_same_parameters() {
        let (card, _) = RunCard::decay_default().split();
        let decay = card.for_decay(125.0);
        let names = |rc: &RunCard| rc.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>();
        assert_eq!(names(&decay), names(&card));
        assert_eq!(decay.ebeam1, 62.5);
        assert_eq!(decay.scale, 125.0);
    }

    #[test]
    fn empty_card_reproduces_mg_defaults() {
        let rc = RunCard::parse("").unwrap();
        assert_eq!(rc.ebeam1, 6500.0);
        assert_eq!(rc.ebeam2, 6500.0);
        assert_eq!(rc.lpp1, 1);
        assert_eq!(rc.lpp2, 1);
        assert_eq!(rc.maxjetflavor, 4);
        assert_eq!(rc.int("nevents"), 10000);
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

    /// `# hidden_parameter` comments are cut as `banner.py` cuts them, so the
    /// field takes effect: a Float, a Bool and a consumed field all read as
    /// they do without the comment.
    #[test]
    fn hidden_parameter_comment_is_stripped() {
        let plain = RunCard::parse("50.0 = mmll\nFalse = pdfwgt\n7000 = ebeam1\n").unwrap();
        let hidden = RunCard::parse(
            "50.0 = mmll # hidden_parameter\n\
             False = pdfwgt # hidden_parameter\n\
             7000 = ebeam1 # hidden_parameter\n",
        )
        .unwrap();
        assert_eq!(hidden.float("mmll"), 50.0);
        assert!(!hidden.get("pdfwgt").unwrap().as_bool());
        assert_eq!(hidden.ebeam1, 7000.0);
        for (name, value) in plain.iter() {
            assert_eq!(hidden.get(name), Some(value), "{name}");
        }
        assert!(RunCard::parse("True = pdfwgt # c")
            .unwrap()
            .get("pdfwgt")
            .unwrap()
            .as_bool());
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

    /// `frame_id` is MadGraph's `Σ 2^n` over `me_frame`, whatever the list's
    /// spelling, and 6 for the default: the partonic centre of mass.
    #[test]
    fn me_frame_reads_as_madgraphs_frame_id() {
        assert_eq!(RunCard::default().frame_id().unwrap(), 6);
        assert_eq!(RunCard::parse("").unwrap().frame_id().unwrap(), 6);
        for spelling in ["1, 2", "[1, 2]", "[1,2]", "1 2", "2, 1"] {
            let rc = RunCard::parse(&format!("  {spelling} = me_frame\n")).unwrap();
            assert_eq!(rc.frame_id().unwrap(), 6, "{spelling}");
        }
        let rc = RunCard::parse("  3, 4 = me_frame\n").unwrap();
        assert_eq!(rc.frame_id().unwrap(), 24);
        for bad in ["[]", "a, b", "1, 99"] {
            assert!(
                matches!(
                    RunCard::parse(&format!("  {bad} = me_frame\n")),
                    Err(RunCardError::BadFrame { .. })
                ),
                "{bad}"
            );
        }
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
        assert!(matches!(err, RunCardError::UnsupportedField { .. }));
        assert!(err.to_string().contains("polbeam1"));
        assert!(err.to_string().contains("beam polarisation"));
    }

    #[test]
    fn polbeam2_nonzero_is_rejected() {
        let err = RunCard::parse("  -1.0 = polbeam2\n").unwrap_err();
        assert!(matches!(err, RunCardError::UnsupportedField { .. }));
        assert!(err.to_string().contains("polbeam2"));
        assert!(err.to_string().contains("beam polarisation"));
    }

    #[test]
    fn polbeam1_fractional_is_rejected() {
        let err = RunCard::parse("  0.5 = polbeam1\n").unwrap_err();
        assert!(matches!(err, RunCardError::UnsupportedField { .. }));
        assert!(err.to_string().contains("polbeam1"));
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

    /// MadGraph's own line (`banner.py`, `RunCardLO.check_validity`): a proton
    /// beam below 0.938 GeV is refused, and 0, which MadGraph runs as a proton at
    /// rest, is refused here too. A beam exactly at the mass, and fixed-energy beams
    /// of any energy, still parse.
    #[test]
    fn a_proton_beam_below_the_proton_mass_is_refused_naming_it() {
        for (line, name, shown) in [
            ("  0 = ebeam1\n", "ebeam1", "ebeam1 = 0 "),
            ("  -6500 = ebeam2\n", "ebeam2", "ebeam2 = -6500 "),
            ("  0.9 = ebeam1\n", "ebeam1", "ebeam1 = 0.9 "),
            ("  nan = ebeam2\n", "ebeam2", "ebeam2 = NaN "),
            ("  inf = ebeam1\n", "ebeam1", "ebeam1 = inf "),
        ] {
            let err = RunCard::parse(line).unwrap_err();
            assert!(
                matches!(err, RunCardError::ProtonBeamBelowMass { name: n, .. } if n == name),
                "{line:?}: {err:?}"
            );
            let message = err.to_string();
            assert!(
                message.contains(shown) && message.contains("0.938 GeV"),
                "{message}"
            );
        }
        let at_mass = RunCard::parse("  0.938 = ebeam1\n  6500 = ebeam2\n").unwrap();
        assert_eq!((at_mass.ebeam1, at_mass.ebeam2), (0.938, 6500.0));
        let unequal = RunCard::parse("  6500 = ebeam1\n  4000 = ebeam2\n").unwrap();
        assert_eq!((unequal.ebeam1, unequal.ebeam2), (6500.0, 4000.0));
        let fixed = RunCard::parse("  0 = lpp1\n  0 = lpp2\n  0.5 = ebeam1\n  0.5 = ebeam2\n");
        assert!(fixed.is_ok(), "{fixed:?}");
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
        assert_eq!(rc.int("nevents"), 10000);
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

    /// The defaults that depart from MadGraph's on purpose, with MadGraph's own
    /// value: the PDF rows (see the comment above them in [`PARAM_DEFAULTS`]).
    fn departures() -> Vec<(&'static str, serde_json::Value)> {
        let s = |v: &str| serde_json::Value::from(v);
        vec![
            ("pdlabel", s("nn23lo1")),
            ("pdlabel1", s("nn23lo1")),
            ("pdlabel2", s("nn23lo1")),
            ("lhaid", serde_json::Value::from(230000)),
        ]
    }

    /// Transcription oracle: every default in [`PARAM_DEFAULTS`] must
    /// match `RunCardLO.default_setup` as dumped by
    /// `validation/madgraph/dump_runcard_defaults.py`, except the deliberate
    /// departures, where the dump must still say what the departure is from.
    /// Regenerate the JSON with `pixi run -e madgraph dump-runcard-defaults`
    /// after a MadGraph bump.
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

        let departures = departures();
        for (name, def) in PARAM_DEFAULTS {
            let Some(actual) = obj.get(*name) else {
                panic!("parameter '{name}' absent from banner.py dump");
            };
            if let Some((_, madgraph)) = departures.iter().find(|(n, _)| n == name) {
                assert_eq!(actual, madgraph, "'{name}': MadGraph's default moved");
                let ours = match def {
                    Def::S(v) => serde_json::Value::from(*v),
                    Def::I(i) => serde_json::Value::from(*i),
                    _ => panic!("'{name}': a departure is a string or an integer"),
                };
                assert_ne!(&ours, madgraph, "'{name}' no longer departs from MadGraph");
                continue;
            }
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
                // A list or dict: the table's payload is what the parser reads
                // from the card line MadGraph writes for its own default.
                Def::O(s) => {
                    assert_eq!(
                        parse_value(s, Kind::Opaque),
                        Some(def.to_value()),
                        "'{name}': table payload {s:?} is not the parser's own spelling"
                    );
                    let spelled = card_spelling(actual);
                    assert_eq!(
                        parse_value(&spelled, Kind::Opaque),
                        Some(def.to_value()),
                        "'{name}': table {s:?} vs dump {actual}, written as {spelled:?}"
                    );
                }
            }
        }
    }

    /// Transcription oracle for [`RunCard::decay_default`]: every cut parameter
    /// as `banner.py`'s `remove_all_cut` leaves it, dumped from MadGraph's own
    /// `RunCardLO` into `validation/madgraph/runcard_decay_defaults.json`
    /// (`dump_runcard_defaults.py`). A reset missing from [`DECAY_CUT_RESETS`]
    /// leaves that cut at its scattering default, and a decay with no run card
    /// then gets a cut MadGraph's does not have.
    #[test]
    fn decay_defaults_match_banner_py_remove_all_cut() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/runcard_decay_defaults.json"
        );
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "missing decay-defaults oracle {path}: {e}\n\
                 run `pixi run -e madgraph dump-runcard-defaults` to (re)generate it"
            )
        });
        let dump: serde_json::Value = serde_json::from_str(&text).unwrap();
        let card = RunCard::decay_default();
        let mut compared = 0;
        for (name, expected) in dump.as_object().expect("oracle is a JSON object") {
            let canonical = canonical_name(name)
                .unwrap_or_else(|| panic!("cut parameter '{name}' is not in the defaults table"));
            let actual = card.get(canonical).expect("every canonical name resolves");
            match (actual, expected) {
                (ParamValue::Opaque(_), _) => continue,
                (ParamValue::Bool(b), e) => assert_eq!(e.as_bool(), Some(*b), "'{name}'"),
                (v, e) => {
                    let e = e.as_f64().unwrap_or_else(|| panic!("'{name}': {e}"));
                    assert_eq!(v.as_f64(), e, "'{name}': decay default vs remove_all_cut");
                }
            }
            compared += 1;
        }
        assert!(compared > 90, "compared only {compared} cut parameters");
        assert_eq!(card.int("SDE_strategy"), 1);
        assert!(!card.get("use_syst").unwrap().as_bool());
    }

    /// The `setcuts.f` decay branch: both scales fixed, `μR` at the mass unless
    /// the card fixed it already, no parton densities.
    #[test]
    fn a_decay_reads_the_card_as_madevent_does() {
        let floating = RunCard::decay_default().for_decay(173.0);
        assert_eq!((floating.lpp1, floating.lpp2), (0, 0));
        assert_eq!((floating.ebeam1, floating.ebeam2), (86.5, 86.5));
        assert!(floating.fixed_ren_scale && floating.fixed_fac_scale);
        assert!(floating.get("fixed_fac_scale1").unwrap().as_bool());
        assert!(floating.get("fixed_fac_scale2").unwrap().as_bool());
        assert_eq!(floating.scale, 173.0);
        assert_eq!(floating.dsqrt_q2fact1, 91.188);

        let fixed = RunCard::parse("T = fixed_ren_scale\n70 = scale\n")
            .unwrap()
            .for_decay(173.0);
        assert_eq!(fixed.scale, 70.0, "a fixed renormalisation scale is kept");
    }

    /// `PDFSUP` is MadEvent's `get_pdf_id(pdlabel)`: the `lhaid` only when the
    /// label is `lhapdf`, the built-in set's id otherwise — `247000` on a
    /// fixed-energy card that leaves `pdlabel` at its default, whatever `lhaid`
    /// says, as every banked MadGraph 3.7.1 lepton-collider run writes.
    #[test]
    fn pdfsup_is_read_off_pdlabel() {
        let card = |text: &str| RunCard::parse(text).expect("card");
        assert_eq!(card("  0 = lpp1\n  0 = lpp2\n").pdfsup(), 247000);
        assert_eq!(
            card("  0 = lpp1\n  0 = lpp2\n  none = pdlabel1\n  none = pdlabel2\n").pdfsup(),
            247000
        );
        assert_eq!(
            card("  lhapdf = pdlabel\n  230000 = lhaid\n").pdfsup(),
            230000
        );
        assert_eq!(
            card("  lhapdf = pdlabel\n  247000 = lhaid\n").pdfsup(),
            247000
        );
        assert_eq!(card("  cteq6l1 = pdlabel\n").pdfsup(), 10042);
        assert_eq!(RunCard::default().pdfsup(), 247000);
    }

    /// The banked MadEvent 3.7.1 lepton-collider cards (`ee_to_ee` and every
    /// other `ee_*` row) spell the PDF per beam as `none`, keep `lhaid` at
    /// MadGraph's `230000` and carry no `pdlabel` line; their `<init>` reports
    /// `247000`, the template `nn23lo1`'s id. A fixed-energy card that does
    /// write `pdlabel = lhapdf` reports its `lhaid`, as `get_pdf_id` does.
    #[test]
    fn a_madgraph_fixed_energy_card_reports_the_template_pdfsup() {
        let card = |text: &str| RunCard::parse(text).expect("card");
        let banked_ee = "  0\t= lpp1 ! beam 1 type\n  0\t= lpp2 ! beam 2 type\n     \
                         none    = pdlabel1     ! PDF type for beam #1\n     \
                         none    = pdlabel2     ! PDF type for beam #2\n  \
                         230000\t= lhaid ! if pdlabel=lhapdf, this is the lhapdf number\n";
        assert_eq!(card(banked_ee).pdfsup(), 247000);
        assert_eq!(
            card("  0 = lpp1\n  0 = lpp2\n  nn23lo1 = pdlabel\n  230000 = lhaid\n").pdfsup(),
            247000
        );
        assert_eq!(
            card("  0 = lpp1\n  0 = lpp2\n  lhapdf = pdlabel\n  230000 = lhaid\n").pdfsup(),
            230000
        );
        // A decay's card keeps whether the integration card wrote the line.
        assert_eq!(
            card("  lhapdf = pdlabel\n  325300 = lhaid\n")
                .for_decay(125.0)
                .pdfsup(),
            325300
        );
    }

    /// The default card names LHAPDF set 247000 on every PDF line, a
    /// departure from MadGraph's `nn23lo1` / `230000`, so a proton run with no
    /// card reads a density rather than being refused.
    #[test]
    fn the_default_card_reads_lhapdf_247000() {
        let rc = RunCard::default();
        assert_eq!((rc.pdlabel.as_str(), rc.lhaid), ("lhapdf", 247000));
        for name in ["pdlabel1", "pdlabel2"] {
            assert_eq!(rc.get(name).unwrap().as_str(), "lhapdf", "{name}");
        }
        assert_eq!(rc.pdf_choice(), Ok(PdfChoice::Lhapdf { lhaid: 247000 }));
        assert_eq!(
            RunCard::parse("  325300 = lhaid\n").unwrap().pdf_choice(),
            Ok(PdfChoice::Lhapdf { lhaid: 325300 })
        );
    }

    /// Every label but `lhapdf` is refused on proton beams with the message
    /// that says to set `pdlabel = lhapdf`; the `247000` hint is `nn23lo1`'s
    /// alone. On fixed-energy beams, and so on a decay's card, the same labels
    /// read nothing and pass, and their `PDFSUP` is unchanged.
    #[test]
    fn every_label_but_lhapdf_is_refused_on_proton_beams() {
        let card = |text: &str| RunCard::parse(text).expect("card");
        for label in BUILTIN_PDLABELS.iter().copied().chain(["ct14lo"]) {
            let proton = card(&format!("  {label} = pdlabel\n"));
            let why = proton.pdf_choice().unwrap_err().to_string();
            assert!(
                why.starts_with(&format!("run card sets pdlabel = {label}"))
                    && why.contains("Set pdlabel = lhapdf and lhaid to the set's LHAPDF id"),
                "{why}"
            );
            assert_eq!(why.contains("247000"), label == "nn23lo1", "{why}");
            assert_eq!(
                why.contains("MadGraph's built-in parameterisation"),
                label != "ct14lo",
                "{why}"
            );

            let fixed = card(&format!("  0 = lpp1\n  0 = lpp2\n  {label} = pdlabel\n"));
            assert_eq!(fixed.pdf_choice(), Ok(PdfChoice::None), "{label}");
            assert_eq!(fixed.pdfsup(), proton.pdfsup(), "{label}");
            let decay = proton.for_decay(125.0);
            assert_eq!(decay.pdf_choice(), Ok(PdfChoice::None), "{label}");
            assert_eq!(decay.pdfsup(), proton.pdfsup(), "{label}");
        }
        assert_eq!(
            card("  nn23lo1 = pdlabel\n")
                .pdf_choice()
                .unwrap_err()
                .to_string(),
            "run card sets pdlabel = nn23lo1, MadGraph's built-in parameterisation; vibegraph \
             reads parton densities only from LHAPDF6 grids. Set pdlabel = lhapdf and lhaid to \
             the set's LHAPDF id (247000 is NNPDF23_lo_as_0130_qed, the set nn23lo1 is fitted \
             to)."
        );
    }
}
