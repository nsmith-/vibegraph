//! Les Houches Event File output.
//!
//! A [`record`] layer holding one `<init>` block and one `<event>` block as data,
//! a [`write`] layer serialising them in the fixed column layout a downstream
//! shower expects, a [`parse`] layer reading the same layout back, a
//! [`build`] layer that assembles an event record out of what the generator
//! produces — the accepted point's momenta, the helicity combination and colour
//! flow selected for it, and the scales it was evaluated at — and an [`emit`]
//! layer choosing how the accept/reject pass's weights become a file's events.
//! An [`observables`] layer reads a parsed event back as the named kinematic and
//! categorical quantities two samples of the same process are compared in.
//!
//! # The column layout is MadGraph's, not an invention
//!
//! The accord fixes the *fields* and their order but not their formatting, and
//! shower authors have historically parsed the columns rather than the
//! whitespace. The layout written here is byte-for-byte the one MadGraph's
//! delivered `unweighted_events.lhe` carries:
//!
//! ```text
//! init beam line:    "%d %d %e %e %d %d %d %d %d %d"
//! init process line: "%e %e %e %d"
//! event line:        "%2d %6d %+13.7e %14.8e %14.8e %14.8e"
//! particle line:     " %8d %2d %4d %4d %4d %4d %+13.10e %+13.10e %+13.10e %14.10e %14.10e %10.4e %10.4e"
//! ```
//!
//! Those come from `madgraph/various/lhe_parser.py`, **not** from the Fortran
//! `Source/rw_events.f`, whose `(i2,i5,e16.7e3,3e15.7)` writes the intermediate
//! per-channel event files. MadGraph reads those back in Python and rewrites the
//! delivered file, so the exponents carry two digits rather than three and the
//! scale fields nine significant digits rather than seven. A writer built against
//! the Fortran specifier disagrees with the file a shower actually receives on
//! every line.
//!
//! Two consequences worth stating, because neither is visible from the format
//! strings alone. The fields are wider than the numbers that reach them:
//! MadGraph's Fortran produced the scale and coupling fields at seven significant
//! digits and the momenta at eleven, and reprinting them at nine and eleven leaves
//! a banked file's scale fields with two digits of padding zeros. And the widths
//! never actually pad — every value is at least as wide as its field — so the
//! layout is really "one space between columns", with the widths mattering only
//! for the small integers.
//!
//! # A file is a lossy record of the run that wrote it
//!
//! Cross sections in `<init>` get seven significant digits, `XWGTUP` eight, and
//! momenta eleven. Reading a file back therefore recovers the *file* exactly —
//! re-serialising reproduces it byte for byte — but recovers the run that wrote
//! it only to those precisions.
//!
//! # `SCALUP` is the factorisation scale
//!
//! See [`build::scalup`]. The accord defines `SCALUP` as the scale the parton
//! densities were evaluated at, and that is what MadGraph writes; it is *not* the
//! renormalisation scale, and reading it as one is wrong on any event whose
//! clustering assigns the two off different vertices.
//!
//! # `AQCDUP` is `αs(μR)`, without MadGraph's truncation
//!
//! MadGraph's `unwgt.f` forms the field as `g²/4/3.1415926`, having built `g` from
//! full-precision π, so every LHE file it writes carries `αs·(1 + 1.7e-8)`. This
//! writer emits `αs(μR)`. A comparison against a banked field has to add the
//! truncation back rather than widen a tolerance.

use thiserror::Error;

pub mod build;
pub mod emit;
pub mod observables;
pub mod parse;
pub mod record;
pub mod write;

/// The LHE version this writer emits and the parser accepts.
pub const LHE_VERSION: &str = "3.0";

#[derive(Debug, Error, PartialEq)]
pub enum LhefError {
    #[error("event record needs {want} momenta (one per external leg), got {got}")]
    MomentumCount { want: usize, got: usize },
    #[error("event record needs {want} helicities (one per external leg), got {got}")]
    HelicityCount { want: usize, got: usize },
    #[error("colour flow {flow} is out of range for a subprocess with {n_flows} flows")]
    FlowOutOfRange { flow: usize, n_flows: usize },
    #[error("external leg {leg} carries PDG code {pdg}, which does not fit a Les Houches IDUP")]
    PdgOutOfRange { leg: usize, pdg: i64 },
    #[error(
        "leg relabelling {order:?} is not a permutation of {n_ext} external legs that keeps the \
         incoming ones incoming, or does not come with one PDG code per leg"
    )]
    LegOrder { order: Vec<usize>, n_ext: usize },
    #[error("line {line}: {reason}")]
    Malformed { line: usize, reason: String },
    #[error("no <init> block")]
    MissingInit,
    #[error("XML structure: {0}")]
    Xml(String),
}

impl LhefError {
    fn xml(error: quick_xml::Error) -> Self {
        LhefError::Xml(error.to_string())
    }
}
