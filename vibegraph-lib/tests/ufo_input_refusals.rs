//! What the UFO loader refuses, and how a restriction decides a coupling is zero.
//!
//! Every case edits one line of a small, valid model written to a scratch
//! directory, so a refusal is the edit's and not the fixture's: the unedited
//! model loads (`the_fixture_loads_and_keeps_both_vertices`).
//!
//! The field refusals follow the constructors of UFO's `object_library.py`
//! (`mg5amcnlo/models/sm/object_library.py`): a field in a class's
//! `require_args` is a positional parameter without a default there, so a model
//! missing one does not import in MadGraph either. Each refusal names the
//! object and the field, which is what the assertions read.

use std::path::Path;

use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, ParsedModel, UFOModel};

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
WS = Parameter(name = 'WS', nature = 'external', type = 'real', value = 1.0, texname = 'W_S',
               lhablock = 'DECAY', lhacode = [ 9000001 ])
c1 = Parameter(name = 'c1', nature = 'external', type = 'real', value = 1.0, texname = 'c_1',
               lhablock = 'NEWCOUP', lhacode = [ 1 ])
c2 = Parameter(name = 'c2', nature = 'external', type = 'real', value = 1.0, texname = 'c_2',
               lhablock = 'NEWCOUP', lhacode = [ 2 ])
c3 = Parameter(name = 'c3', nature = 'external', type = 'real', value = 1.0, texname = 'c_3',
               lhablock = 'NEWCOUP', lhacode = [ 3 ])
";

const COUPLINGS: &str = "\
from object_library import all_couplings, Coupling

GC_1 = Coupling(name = 'GC_1', value = 'c1', order = {'QED':1})
GC_2 = Coupling(name = 'GC_2', value = 'c2', order = {'QED':2})
GC_3 = Coupling(name = 'GC_3', value = 'c3', order = {'QED':3})
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

const FILES: [(&str, &str); 5] = [
    ("particles.py", PARTICLES),
    ("parameters.py", PARAMETERS),
    ("couplings.py", COUPLINGS),
    ("lorentz.py", LORENTZ),
    ("vertices.py", VERTICES),
];

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
fn refusal(file: &str, from: &str, to: &str) -> String {
    let dir = model_with(&[(file, from, to)]);
    match ParsedModel::parse(dir.path()) {
        Ok(_) => panic!("{file}: `{from}` -> `{to}` loads"),
        Err(e) => e.to_string(),
    }
}

/// Asserts the refusal names the object (as `'<name>'`) and the field (as
/// `` `<field>` ``).
fn assert_names(message: &str, object: &str, field: &str) {
    assert!(
        message.contains(&format!("'{object}'")) && message.contains(&format!("`{field}`")),
        "expected '{object}' and `{field}` in: {message}"
    );
}

#[test]
fn the_fixture_loads_and_keeps_both_vertices() {
    let dir = model_with(&[]);
    let model = UFOModel::load(dir.path(), None).unwrap();
    assert!(model.vertices.contains_key("V_1"));
    assert!(model.vertices.contains_key("V_2"));
}

#[test]
fn every_required_particle_field_is_refused_by_name_when_missing() {
    for (field, text) in [
        ("pdg_code", "pdg_code = 9000001, "),
        ("antiname", "antiname = 'S', "),
        ("spin", "spin = 1, "),
        ("color", "color = 1,"),
        ("mass", "mass = Param.MS, "),
        ("width", "width = Param.WS, "),
        ("texname", "texname = 'S', antitexname"),
        ("antitexname", "antitexname = 'S',"),
        ("charge", "charge = 0, "),
    ] {
        let to = if field == "texname" {
            "antitexname"
        } else {
            ""
        };
        assert_names(&refusal("particles.py", text, to), "S", field);
    }
    assert_names(
        &refusal("particles.py", "name = 'S', antiname", "antiname"),
        "S",
        "name",
    );
}

#[test]
fn a_malformed_particle_field_is_refused_by_name() {
    for (field, from, to) in [
        ("pdg_code", "pdg_code = 9000001", "pdg_code = 'S'"),
        ("pdg_code", "pdg_code = 9000001", "pdg_code = 9000001.0"),
        ("spin", "spin = 1", "spin = '1'"),
        ("color", "color = 1", "color = 'singlet'"),
        ("mass", "mass = Param.MS", "mass = 'MS'"),
        ("charge", "charge = 0", "charge = 'neutral'"),
        ("name", "name = 'S', antiname", "name = S, antiname"),
        ("GhostNumber", "GhostNumber = 0", "GhostNumber = 'none'"),
    ] {
        assert_names(&refusal("particles.py", from, to), "S", field);
    }
}

#[test]
fn every_required_parameter_field_is_refused_by_name_when_missing() {
    for (field, from) in [
        ("name", "name = 'c1', "),
        (
            "nature",
            "nature = 'external', type = 'real', value = 1.0, texname = 'c_1'",
        ),
        ("type", "type = 'real', value = 1.0, texname = 'c_1'"),
        ("value", "value = 1.0, texname = 'c_1'"),
        ("texname", "texname = 'c_1',"),
        ("lhablock", "lhablock = 'NEWCOUP', lhacode = [ 1 ]"),
        ("lhacode", ", lhacode = [ 1 ]"),
    ] {
        // Each edit drops exactly the named field from `c1`.
        let to = match field {
            "nature" => "type = 'real', value = 1.0, texname = 'c_1'",
            "type" => "value = 1.0, texname = 'c_1'",
            "value" => "texname = 'c_1'",
            "lhablock" => "lhacode = [ 1 ]",
            _ => "",
        };
        assert_names(&refusal("parameters.py", from, to), "c1", field);
    }
    // An internal parameter's `value` is its expression: no default either.
    assert_names(
        &refusal("parameters.py", "value = '0.0', ", ""),
        "ZERO",
        "value",
    );
}

#[test]
fn a_malformed_parameter_field_is_refused_by_name() {
    for (object, field, from, to) in [
        // A misspelt nature is neither external nor internal; read as internal it
        // would make `c1` the expression `1.0` and drop it from every card.
        (
            "c1",
            "nature",
            "nature = 'external', type = 'real', value = 1.0, texname = 'c_1'",
            "nature = 'exernal', type = 'real', value = 1.0, texname = 'c_1'",
        ),
        (
            "c1",
            "type",
            "type = 'real', value = 1.0, texname = 'c_1'",
            "type = 'integer', value = 1.0, texname = 'c_1'",
        ),
        (
            "c1",
            "value",
            "value = 1.0, texname = 'c_1'",
            "value = 'one', texname = 'c_1'",
        ),
        ("ZERO", "value", "value = '0.0'", "value = 0.0"),
        ("c1", "lhacode", "lhacode = [ 1 ]", "lhacode = [ 'one' ]"),
        ("c1", "lhacode", "lhacode = [ 1 ]", "lhacode = 1"),
        (
            "c1",
            "lhablock",
            "lhablock = 'NEWCOUP', lhacode = [ 1 ]",
            "lhablock = NEWCOUP, lhacode = [ 1 ]",
        ),
    ] {
        assert_names(&refusal("parameters.py", from, to), object, field);
    }
}

/// MadGraph refuses a model defining one parameter name twice
/// (`import_ufo.py`, `UFOMG5Converter.load_model`: "define multiple time").
#[test]
fn a_parameter_defined_twice_is_refused_by_name() {
    let message = refusal(
        "parameters.py",
        "ZERO = Parameter(",
        "c1 = Parameter(name = 'c1', nature = 'internal', type = 'real', value = '2.0', texname = 'c')\n\
         ZERO = Parameter(",
    );
    assert!(
        message.contains("'c1'") && message.contains("twice"),
        "{message}"
    );
}

#[test]
fn every_required_coupling_field_is_refused_by_name() {
    for (field, from, to) in [
        ("name", "name = 'GC_1', ", ""),
        ("value", "value = 'c1', ", ""),
        ("order", ", order = {'QED':1}", ""),
        ("value", "value = 'c1'", "value = 1.0"),
        ("order", "order = {'QED':1}", "order = {'QED':'one'}"),
        ("order", "order = {'QED':1}", "order = ['QED']"),
    ] {
        assert_names(&refusal("couplings.py", from, to), "GC_1", field);
    }
}

#[test]
fn every_required_vertex_field_is_refused_by_name() {
    for (field, from, to) in [
        ("particles", "particles = [ P.S, P.S, P.S ], ", ""),
        (
            "color",
            "color = [ '1' ],\n             lorentz = [ L.SSS1 ]",
            "lorentz = [ L.SSS1 ]",
        ),
        ("lorentz", "lorentz = [ L.SSS1 ], ", ""),
        ("couplings", ", couplings = {(0,0):C.GC_1}", ""),
        (
            "particles",
            "particles = [ P.S, P.S, P.S ]",
            "particles = P.S",
        ),
        (
            "color",
            "color = [ '1' ],\n             lorentz = [ L.SSS1 ]",
            "color = [ 1 ],\n             lorentz = [ L.SSS1 ]",
        ),
        (
            "couplings",
            "couplings = {(0,0):C.GC_1}",
            "couplings = {(0,'a'):C.GC_1}",
        ),
        (
            "couplings",
            "couplings = {(0,0):C.GC_1}",
            "couplings = {(0,0):'GC_1'}",
        ),
    ] {
        assert_names(&refusal("vertices.py", from, to), "V_1", field);
    }
}

/// A vertex without a name, and two vertices with one name, are refused: the
/// loader keys vertices by name, so either would leave one silently replacing
/// the other.
#[test]
fn an_unnamed_or_duplicate_vertex_is_refused() {
    assert_names(&refusal("vertices.py", "name = 'V_1', ", ""), "V_1", "name");
    let message = refusal("vertices.py", "name = 'V_2'", "name = 'V_1'");
    assert!(
        message.contains("'V_1'") && message.contains("twice"),
        "{message}"
    );
}

fn restrict_card(dir: &Path, text: &str) -> std::path::PathBuf {
    let card = dir.join("restrict_test.dat");
    std::fs::write(&card, text).unwrap();
    card
}

/// MadGraph's restriction keeps a width given as `Auto` free and writes it to the
/// process card as `Auto`, for MadWidth to compute at launch
/// (`RestrictModel.restrict_model` and `modify_autowidth`, `import_ufo.py`). The
/// loader computes no width, and would otherwise leave the `parameters.py`
/// default in its place, so the restriction is refused naming the particle.
#[test]
fn a_restrict_card_auto_width_is_refused_naming_the_particle() {
    let dir = model_with(&[]);
    let card = restrict_card(dir.path(), "Block NEWCOUP\n 1 1.0\nDECAY 9000001 Auto\n");
    let message = match UFOModel::load(dir.path(), Some(&card)) {
        Ok(_) => panic!("a restrict card with DECAY 9000001 Auto loads"),
        Err(e) => e.to_string(),
    };
    assert!(
        message.contains("9000001") && message.contains("Auto"),
        "{message}"
    );
}

/// `EvaluatedModel::from_model_card` cannot read a width from an `Auto` row;
/// a card bound for a run is refused before it gets there (`ufo::bind`), and a
/// direct call is a caller's error.
#[test]
#[should_panic(expected = "Auto")]
fn evaluating_a_model_at_a_card_with_an_auto_width_panics() {
    let dir = model_with(&[]);
    let model = UFOModel::load(dir.path(), None).unwrap();
    let card: ParamCard = "DECAY 9000001 Auto\n".parse().unwrap();
    let _ = EvaluatedModel::from_model_card(model, &card);
}

/// Which of the fixture's two vertices survive a restriction setting
/// `(c1, c2, c3)`; `GC_3` is on no vertex.
fn survivors(c1: &str, c2: &str, c3: &str) -> (bool, bool) {
    let dir = model_with(&[]);
    let card = restrict_card(
        dir.path(),
        &format!("Block NEWCOUP\n 1 {c1}\n 2 {c2}\n 3 {c3}\n"),
    );
    let model = UFOModel::load(dir.path(), Some(&card)).unwrap();
    (
        model.vertices.contains_key("V_1"),
        model.vertices.contains_key("V_2"),
    )
}

/// MadGraph's rule (`RestrictModel.detect_identical_couplings`, `import_ufo.py`
/// L2549-L2558): a coupling with `|g| < 1e-13` is zero, unless some coupling of
/// the model lies in `[1e-13, 1e-10)`, in which case only exact zeros are.
#[test]
fn a_coupling_below_1e_13_is_pruned() {
    assert_eq!(survivors("1e-15", "0.5", "1.0"), (false, true));
    assert_eq!(survivors("9.9e-14", "0.5", "1.0"), (false, true));
    // Exact zero is pruned under either rule; a coupling at 1e-10 is not in the
    // window and leaves the threshold rule in force.
    assert_eq!(survivors("0.0", "0.5", "1.0"), (false, true));
    assert_eq!(survivors("1e-15", "1e-10", "1.0"), (false, true));
}

#[test]
fn a_coupling_in_the_window_makes_only_exact_zeros_vanish() {
    // A coupling far below either threshold survives once another sits in
    // [1e-13, 1e-10): on a vertex (c2), on no vertex (c3), or at the window's
    // lower edge.
    assert_eq!(survivors("1e-25", "1e-11", "1.0"), (true, true));
    assert_eq!(survivors("1e-25", "0.5", "1e-11"), (true, true));
    assert_eq!(survivors("1e-25", "0.5", "1e-13"), (true, true));
    // The strict rule still drops an exact zero.
    assert_eq!(survivors("0.0", "1e-11", "1.0"), (false, true));
}
