//! What the param, restrict and reweight card readers refuse, and the UFO
//! `propagators.py` fields the loader requires.
//!
//! Each card case is a construct MadGraph's reader accepts and this crate
//! cannot honour as MadGraph does; the assertions read the refusal's message
//! for the construct it names. The model cases edit one line of a small valid
//! model written to a scratch directory, so a refusal is the edit's and not the
//! fixture's (`the_fixture_loads`).

use std::path::{Path, PathBuf};

use vibegraph::reweight::card::ReweightCard;
use vibegraph::reweight::resolve;
use vibegraph::ufo::bind::bind_card;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::{ParsedModel, UFOModel};

const PARTICLES: &str = "\
from object_library import all_particles, Particle
import parameters as Param

S = Particle(pdg_code = 9000001, name = 'S', antiname = 'S', spin = 1, color = 1,
             mass = Param.MS, width = Param.WS, texname = 'S', antitexname = 'S',
             charge = 0, GhostNumber = 0)
";

const PARAMETERS: &str = "\
from object_library import all_parameters, Parameter

ZERO = Parameter(name = 'ZERO', nature = 'internal', type = 'real', value = '0.0', texname = '0')
MS = Parameter(name = 'MS', nature = 'external', type = 'real', value = 10.0, texname = 'M_S',
               lhablock = 'MASS', lhacode = [ 9000001 ])
WS = Parameter(name = 'WS', nature = 'external', type = 'real', value = 2.0, texname = 'W_S',
               lhablock = 'DECAY', lhacode = [ 9000001 ])
c1 = Parameter(name = 'c1', nature = 'external', type = 'real', value = 0.5, texname = 'c_1',
               lhablock = 'NEWCOUP', lhacode = [ 1 ])
c2 = Parameter(name = 'c2', nature = 'external', type = 'real', value = 0.5, texname = 'c_2',
               lhablock = 'NEWCOUP', lhacode = [ 2 ])
c3 = Parameter(name = 'c3', nature = 'external', type = 'real', value = 1.0, texname = 'c_3',
               lhablock = 'NEWCOUP', lhacode = [ 3 ])
";

const COUPLINGS: &str = "\
from object_library import all_couplings, Coupling

GC_1 = Coupling(name = 'GC_1', value = 'c1', order = {'QED':1})
GC_2 = Coupling(name = 'GC_2', value = 'c2 * c3', order = {'QED':2})
";

const LORENTZ: &str = "\
from object_library import all_lorentz, Lorentz

SSS1 = Lorentz(name = 'SSS1', spins = [ 1, 1, 1 ], structure = '1')
SSSS1 = Lorentz(name = 'SSSS1', spins = [ 1, 1, 1, 1 ], structure = '1')
";

const VERTICES: &str = "\
from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_1 = Vertex(name = 'V_1', particles = [ P.S, P.S, P.S ], color = [ '1' ],
             lorentz = [ L.SSS1 ], couplings = {(0,0):C.GC_1})
V_2 = Vertex(name = 'V_2', particles = [ P.S, P.S, P.S, P.S ], color = [ '1' ],
             lorentz = [ L.SSSS1 ], couplings = {(0,0):C.GC_2})
";

const PROPAGATORS: &str = "\
from object_library import all_propagators, Propagator

denominator = \"P('mu', id) * P('mu', id) - Mass(id) * Mass(id)\"

S1 = Propagator(name = 'S1', numerator = '1', denominator = denominator)
S2 = Propagator(name = 'S2', numerator = 'complex(0,1)', denominator = denominator)
";

const FILES: [(&str, &str); 6] = [
    ("particles.py", PARTICLES),
    ("parameters.py", PARAMETERS),
    ("couplings.py", COUPLINGS),
    ("lorentz.py", LORENTZ),
    ("vertices.py", VERTICES),
    ("propagators.py", PROPAGATORS),
];

/// The fixture's complete restrict card: `c1 = c2` (one block, equal values)
/// merges them, and `c3 = 1` fixes `c3` to one.
const RESTRICT: &str = "\
Block MASS
 9000001 1.000000e+01 # MS
Block NEWCOUP
 1 5.000000e-01 # c1
 2 5.000000e-01 # c2
 3 1.000000e+00 # c3
DECAY 9000001 2.000000e+00 # WS
";

/// The fixture written to a fresh directory with `from` replaced by `to` in
/// `file`, each replacement asserted to have happened exactly once.
fn model_with(edits: &[(&str, &str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in FILES {
        let mut text = text.to_owned();
        for &(file, from, to) in edits {
            if file == name {
                assert_eq!(text.matches(from).count(), 1, "{file}: `{from}`");
                text = text.replacen(from, to, 1);
            }
        }
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    dir
}

/// The error parsing the fixture with one edit gives; panics if it loads.
fn model_refusal(file: &str, from: &str, to: &str) -> String {
    let dir = model_with(&[(file, from, to)]);
    match ParsedModel::parse(dir.path()) {
        Ok(_) => panic!("{file}: `{from}` -> `{to}` loads"),
        Err(e) => e.to_string(),
    }
}

/// Asserts `message` contains every one of `needles`.
fn assert_mentions(message: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            message.contains(needle),
            "expected `{needle}` in: {message}"
        );
    }
}

#[test]
fn the_fixture_loads() {
    let dir = model_with(&[]);
    let model = UFOModel::load(dir.path(), None).unwrap();
    assert!(model.vertices.contains_key("V_1"));
    let card = restrict_file(dir.path(), "restrict_test.dat", RESTRICT);
    let model = UFOModel::load(dir.path(), Some(&card)).unwrap();
    assert!(model.vertices.contains_key("V_2"));
}

// ---- propagators.py ----

/// `Propagator.__init__(self, name, numerator, denominator=None)`: `name` and
/// `numerator` are required, and each is refused by name when absent or not a
/// string the loader can fold, with the refusal of the one case or the other.
#[test]
fn every_required_propagator_field_is_refused_by_name() {
    for (field, from, to, missing) in [
        ("name", "name = 'S1', ", "", true),
        ("numerator", "numerator = '1', ", "", true),
        ("numerator", "numerator = '1'", "numerator = 1", false),
        (
            "denominator",
            "numerator = '1', denominator = denominator",
            "numerator = '1', denominator = 2",
            false,
        ),
    ] {
        let message = model_refusal("propagators.py", from, to);
        let form = if missing {
            format!("'S1' has no `{field}`")
        } else {
            format!("'S1': `{field}` must be")
        };
        assert_mentions(&message, &[&form]);
    }
}

/// A missing denominator is UFO's default, which ALOHA reads as the standard
/// one; the model loads and records none.
#[test]
fn a_propagator_without_a_denominator_loads() {
    let dir = model_with(&[(
        "propagators.py",
        "numerator = '1', denominator = denominator",
        "numerator = '1'",
    )]);
    let parsed = ParsedModel::parse(dir.path()).unwrap();
    assert!(parsed.propagators.contains_key("S1"));
}

#[test]
fn a_propagator_named_twice_is_refused() {
    let message = model_refusal("propagators.py", "name = 'S2'", "name = 'S1'");
    assert_mentions(&message, &["'S1'", "twice"]);
}

// ---- param card ----

/// Bind a card to the default-restricted SM and return the refusal's text,
/// whether the reader or the binding refuses it; panics if it binds.
fn param_refusal(text: &str) -> String {
    let card: ParamCard = match text.parse() {
        Ok(card) => card,
        Err(e) => return e.to_string(),
    };
    match bind_card(sm_model(SMRestrict::Default), &card) {
        Ok(_) => panic!("{text:?} binds"),
        Err(e) => e.to_string(),
    }
}

/// MadGraph writes a card's branching ratios into the event file's `<slha>`
/// header, where a shower reads them for the states it decays, and checks
/// neither their sum nor their daughters (`check_param_card.py` reads a row as
/// a number and a list of codes). A table is carried as written: one that sums
/// to 0.7 and names a daughter the model lacks binds, and the width beside it
/// binds as it did without a table.
#[test]
fn a_decay_table_is_carried_as_written() {
    let model = sm_model(SMRestrict::Default);
    let card: ParamCard = "DECAY 6 1.5\n 0.6 2 5 24 # t > b w+\n 0.1 2 9999999 24\n"
        .parse()
        .unwrap();
    let bound = bind_card(model.clone(), &card).unwrap();
    let plain = bind_card(model, &"DECAY 6 1.5\n".parse().unwrap()).unwrap();
    assert_eq!(bound.moved, plain.moved);
    assert_eq!(bound.evaluated.param_values["WT"].re, 1.5);
}

/// A row whose `NDA` disagrees with its daughters: MadGraph counts the
/// daughters, a shower reading the header reads `NDA`.
#[test]
fn a_decay_row_whose_nda_disagrees_is_refused() {
    let message = param_refusal("DECAY 6 1.5\n 1.0 3 5 24\n");
    assert_mentions(&message, &["NDA = 3", "line 2"]);
}

/// A channel given two branching ratios: MadGraph keeps the first silently,
/// a shower reading the header reads both.
#[test]
fn a_decay_channel_given_two_branching_ratios_is_refused() {
    let message = param_refusal("DECAY 6 1.5\n 0.9 2 5 24\n 0.8 2 24 5\n");
    assert_mentions(&message, &["line 3", "on line 2"]);
}

/// A table below a `DECAY` line for an antiparticle's code: the header lists
/// each particle once, under its positive PDG code, so the rows would have no
/// line to sit under.
#[test]
fn a_decay_table_under_a_negative_code_is_refused() {
    let message = param_refusal("DECAY -24 2.085\n 1.0 2 11 -12\n");
    assert_mentions(&message, &["-24", "positive PDG code"]);
}

/// MadGraph reads `DECAY 6 1.5 0.3` as a width of `"1.5 0.3"`, a string.
#[test]
fn a_decay_line_with_extra_fields_is_refused() {
    let message = param_refusal("DECAY 6 1.5 0.3\n");
    assert_mentions(&message, &["DECAY", "line 1"]);
}

/// SLHA's `XSECTION` block is information for other tools: MadGraph keeps its
/// rows unparsed up to the next `BLOCK` or `DECAY` and reads the blocks after
/// it, and so does this reader.
#[test]
fn an_xsection_block_is_skipped_up_to_the_next_block() {
    let card: ParamCard =
        "Block MASS\n 23 9.2e1\nXSECTION 13000 2212 2212 2 6 -6\n 0 0 0 0 0 0 9.0e-1 0 0\n\
         Block YUKAWA\n 5 4.5\n"
            .parse()
            .unwrap();
    assert_eq!(card.get("YUKAWA", &[5]), Some(4.5));
    assert_eq!(card.get("MASS", &[23]), Some(92.0));
    bind_card(sm_model(SMRestrict::Default), &card).unwrap();
}

/// A `scan:` value is MadGraph's parameter scan, one run per value.
#[test]
fn a_scan_value_is_refused() {
    let message = param_refusal("Block MASS\n 6 scan:[170, 175]\n");
    assert_mentions(&message, &["scan", "line 2"]);
}

/// A negative mass on a particle that is not a Majorana fermion: MadGraph's
/// Fortran negates the width beside it (`write_inc_file`), and the evaluation
/// here does too, but the phase-space, on-shell, cut and event-record code
/// downstream is not validated for a negative mass, so binding refuses it.
#[test]
fn a_negative_mass_off_a_majorana_fermion_is_refused() {
    let message = param_refusal("Block MASS\n 6 -1.75e2\n");
    assert_mentions(
        &message,
        &["particle t ", "MT", "negative", "not validated"],
    );
}

/// Every problem in the card is reported at once, the reader's and the
/// binding's alike.
#[test]
fn every_problem_in_a_card_is_reported_at_once() {
    let message = param_refusal("Block MASS\n 6 x\n 23 y\n");
    assert_mentions(&message, &["line 2", "line 3"]);
    let message = param_refusal("Block MASS\n 99 1.0\nBlock NOSUCH\n 1 2.0\n");
    assert_mentions(&message, &["code 99", "block nosuch"]);
}

// ---- restrict card ----

fn restrict_file(dir: &Path, name: &str, text: &str) -> PathBuf {
    let card = dir.join(name);
    std::fs::write(&card, text).unwrap();
    card
}

/// Load the fixture under `RESTRICT` with `from` replaced by `to`, returning the
/// refusal's text; panics if it loads.
fn restrict_refusal(from: &str, to: &str) -> String {
    assert_eq!(RESTRICT.matches(from).count(), 1, "`{from}`");
    let dir = model_with(&[]);
    let card = restrict_file(
        dir.path(),
        "restrict_test.dat",
        &RESTRICT.replacen(from, to, 1),
    );
    match UFOModel::load(dir.path(), Some(&card)) {
        Ok(_) => panic!("`{from}` -> `{to}` loads"),
        Err(e) => e.to_string(),
    }
}

/// MadGraph's restriction requires a value for every external parameter
/// (`model_reader.py`: "not define", and a missing block is "Invalid
/// restriction card"); the loader would otherwise keep `parameters.py`'s.
#[test]
fn a_restrict_card_missing_a_parameter_is_refused_naming_it() {
    let message = restrict_refusal(" 2 5.000000e-01 # c2\n", "");
    assert_mentions(&message, &["c2", "NEWCOUP"]);
    let message = restrict_refusal("Block MASS\n 9000001 1.000000e+01 # MS\n", "");
    assert_mentions(&message, &["MS", "MASS"]);
}

/// `9.999999e-1` and `0.000001e-99` are MadGraph's values for "one" and
/// "zero" that the restriction must not fix (`import_ufo.py`,
/// `restrict_model`): neither is fixed or merged, and each is replaced by 1 or 0
/// afterwards.
#[test]
fn madgraphs_unrestricted_one_and_zero_are_refused() {
    for value in ["9.999999e-01", "0.000001e-99"] {
        let message = restrict_refusal(" 3 1.000000e+00 # c3", &format!(" 3 {value} # c3"));
        assert_mentions(&message, &["c3", "leaves free"]);
    }
}

#[test]
fn a_non_finite_restrict_value_is_refused() {
    let message = restrict_refusal(" 3 1.000000e+00 # c3", " 3 nan # c3");
    assert_mentions(&message, &["c3", "finite"]);
}

/// `restrict_model` reads the defaults from `param_<x>.dat` beside a
/// `restrict_<x>.dat` when the file exists (`import_ufo.py`).
#[test]
fn a_param_card_beside_the_restrict_card_is_refused() {
    let dir = model_with(&[]);
    let card = restrict_file(dir.path(), "restrict_test.dat", RESTRICT);
    restrict_file(dir.path(), "param_test.dat", RESTRICT);
    let message = match UFOModel::load(dir.path(), Some(&card)) {
        Ok(_) => panic!("loads beside param_test.dat"),
        Err(e) => e.to_string(),
    };
    assert_mentions(&message, &["param_test.dat"]);
}

/// The restriction reads no branching ratio, and MadGraph's process card is
/// written from the model, so a restrict card's decay table is inert.
#[test]
fn a_restrict_card_decay_table_is_accepted() {
    let dir = model_with(&[]);
    let card = restrict_file(
        dir.path(),
        "restrict_test.dat",
        &format!("{RESTRICT} 1.0 2 9000001 9000001\n"),
    );
    UFOModel::load(dir.path(), Some(&card)).unwrap();
}

/// A negative mass from a restriction card, and from the model's own
/// default, binds to the same refusal as a param card's: binding checks every
/// evaluated mass, whatever gave it.
#[test]
fn a_negative_default_or_restricted_mass_is_refused_at_binding() {
    let dir = model_with(&[]);
    let card = restrict_file(
        dir.path(),
        "restrict_test.dat",
        &RESTRICT.replacen(
            " 9000001 1.000000e+01 # MS",
            " 9000001 -1.000000e+01 # MS",
            1,
        ),
    );
    let model = UFOModel::load(dir.path(), Some(&card)).unwrap();
    let message = bind_card(model, &ParamCard::default())
        .err()
        .expect("a negative restricted mass binds")
        .to_string();
    assert_mentions(&message, &["particle S ", "MS", "not validated"]);

    let dir = model_with(&[("parameters.py", "value = 10.0", "value = -10.0")]);
    let model = UFOModel::load(dir.path(), None).unwrap();
    let message = bind_card(model, &ParamCard::default())
        .err()
        .expect("a negative default mass binds")
        .to_string();
    assert_mentions(&message, &["particle S ", "MS", "not validated"]);
}

// ---- reweight card ----

/// Resolve `text` on the fixture under `RESTRICT`.
fn reweight_on_fixture(text: &str) -> Result<usize, String> {
    let dir = model_with(&[]);
    let card = restrict_file(dir.path(), "restrict_test.dat", RESTRICT);
    let model = UFOModel::load(dir.path(), Some(&card)).unwrap();
    let card: ReweightCard = text.parse().map_err(|e| format!("{e}"))?;
    resolve(&card, &model)
        .map(|launches| launches.len())
        .map_err(|e| e.to_string())
}

/// Resolve `text` on the default-restricted SM.
fn reweight_on_sm(text: &str) -> Result<usize, String> {
    let card: ReweightCard = text.parse().map_err(|e| format!("{e}"))?;
    resolve(&card, &sm_model(SMRestrict::Default))
        .map(|launches| launches.len())
        .map_err(|e| e.to_string())
}

/// MadGraph's restriction removes a parameter it fixes to one, so its
/// reweighting cannot move it.
#[test]
fn reweighting_a_parameter_fixed_to_one_is_refused() {
    let message = reweight_on_fixture("launch\n set c3 2.0\n").unwrap_err();
    assert_mentions(&message, &["c3", "one"]);
    assert_eq!(reweight_on_fixture("launch\n set c3 1.0\n"), Ok(1));
}

/// MadGraph's restriction rewrites `c2` as `c1`, so a hypothesis moving one
/// without the other is refused; moving both together is not.
#[test]
fn reweighting_one_parameter_of_a_merged_pair_is_refused() {
    let message = reweight_on_fixture("launch\n set c2 0.7\n").unwrap_err();
    assert_mentions(&message, &["c1", "c2"]);
    assert_eq!(
        reweight_on_fixture("launch\n set c1 0.7\n set newcoup 2 0.7\n"),
        Ok(1)
    );
}

/// MadGraph's `done` ends a parameter edit, not the card: the lines after it
/// still run, so they would be hypotheses this reader dropped.
#[test]
fn a_command_after_done_is_refused() {
    for end in ["done", "exit", "quit"] {
        let message = reweight_on_sm(&format!(
            "launch\n set ymt 150\n{end}\nlaunch\n set ymt 160\n"
        ))
        .unwrap_err();
        assert_mentions(&message, &["line 4", end]);
    }
    assert_eq!(reweight_on_sm("launch\n set ymt 150\ndone\n# end\n"), Ok(1));
}

/// The `change` options that select what this reweighting already does, or
/// that touch only MadGraph's own working files, are accepted.
#[test]
fn inert_change_options_are_accepted() {
    for line in [
        "change helicity False",
        "change helicity false",
        "change output default",
        "change rwgt_dir ./rw",
        "change multicore 4",
    ] {
        assert_eq!(
            reweight_on_sm(&format!("{line}\nlaunch\n set ymt 150\n")),
            Ok(1),
            "{line}"
        );
    }
}

/// Every problem in a reweight card is reported at once.
#[test]
fn every_problem_in_a_reweight_card_is_reported_at_once() {
    let message = reweight_on_sm("launch --bogus\n set ymt abc\nfrobnicate\n").unwrap_err();
    assert_mentions(&message, &["line 1", "line 2", "line 3"]);
}
