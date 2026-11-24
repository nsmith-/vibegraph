//! UFO value forms the loader reads, each on a small authored model written to a
//! scratch directory so a result is the edit's and not the fixture's.
//!
//! Every form follows what MadGraph's UFO import does with it
//! (`mg5amcnlo/models/import_ufo.py`, `madgraph/core/base_objects.py`); a form
//! the loader does not read is still refused by name.

use num_complex::Complex64;
use std::collections::HashMap;
use vibegraph::ufo::{EvaluatedModel, ParsedModel, UFOModel, UfoError};

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

const COUPLING_ORDERS: &str = "\
from object_library import all_orders, CouplingOrder

QED = CouplingOrder(name = 'QED', expansion_order = 99, hierarchy = 2)
NP = CouplingOrder(name = 'NP', expansion_order = 2, hierarchy = 1)
";

const FILES: [(&str, &str); 6] = [
    ("particles.py", PARTICLES),
    ("parameters.py", PARAMETERS),
    ("couplings.py", COUPLINGS),
    ("lorentz.py", LORENTZ),
    ("vertices.py", VERTICES),
    ("coupling_orders.py", COUPLING_ORDERS),
];

/// The fixture written to a fresh directory: files in `omit` left out, `from`
/// replaced by `to` in `file` (each replacement asserted to happen exactly
/// once), then each `(file, text)` of `append` added to the end of its file
/// (created when it is not one of the fixture's).
fn build(
    omit: &[&str],
    edits: &[(&str, &str, &str)],
    append: &[(&str, &str)],
) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut files: Vec<(String, String)> = FILES
        .iter()
        .filter(|(name, _)| !omit.contains(name))
        .map(|&(n, t)| (n.to_owned(), t.to_owned()))
        .collect();
    for &(file, text) in append {
        match files.iter_mut().find(|(n, _)| n == file) {
            Some((_, t)) => t.push_str(text),
            None => files.push((file.to_owned(), text.to_owned())),
        }
    }
    for (name, text) in &mut files {
        for &(file, from, to) in edits {
            if file == name {
                assert_eq!(text.matches(from).count(), 1, "{file}: `{from}`");
                *text = text.replacen(from, to, 1);
            }
        }
        std::fs::write(dir.path().join(&*name), &*text).unwrap();
    }
    dir
}

fn model_with(omit: &[&str], edits: &[(&str, &str, &str)]) -> tempfile::TempDir {
    build(omit, edits, &[])
}

fn model_appending(append: &[(&str, &str)]) -> tempfile::TempDir {
    build(&[], &[], append)
}

/// The error loading the fixture with `edits` and `append`ed text gives; panics
/// if it loads.
fn refusal(edits: &[(&str, &str, &str)], append: &[(&str, &str)]) -> String {
    let dir = build(&[], edits, append);
    match ParsedModel::parse(dir.path()) {
        Ok(_) => panic!("{edits:?} {append:?} loads"),
        Err(e) => e.to_string(),
    }
}

/// Every parameter value of the fixture with `append`ed text, at its defaults.
fn values_of(append: &[(&str, &str)]) -> HashMap<String, Complex64> {
    let dir = model_appending(append);
    let model = UFOModel::load(dir.path(), None).unwrap();
    EvaluatedModel::from_model(model).param_values
}

#[test]
fn orders_come_from_the_couplings_when_coupling_orders_py_is_absent() {
    let dir = model_with(
        &["coupling_orders.py"],
        &[(
            "couplings.py",
            "order = {'QED':1}",
            "order = {'QAD':1, 'QED':1}",
        )],
    );
    let model = UFOModel::load(dir.path(), None).unwrap();
    let expected: std::collections::BTreeMap<String, u32> =
        [("QED".to_owned(), 1), ("QAD".to_owned(), 1)].into();
    assert_eq!(model.order_hierarchy, expected);
}

#[test]
fn a_model_with_exactly_qcd_and_qed_weights_qed_at_two_without_the_file() {
    let dir = model_with(
        &["coupling_orders.py"],
        &[("couplings.py", "order = {'QED':1}", "order = {'QCD':1}")],
    );
    let model = UFOModel::load(dir.path(), None).unwrap();
    let expected: std::collections::BTreeMap<String, u32> =
        [("QED".to_owned(), 2), ("QCD".to_owned(), 1)].into();
    assert_eq!(model.order_hierarchy, expected);
}

fn param(name: &str, nature: &str, value: &str, code: u32) -> String {
    if nature == "external" {
        format!(
            "\n{name} = Parameter(name = '{name}', nature = 'external', type = 'real', \
             value = {value}, texname = '{name}', lhablock = 'NEWCOUP', lhacode = [ {code} ])\n"
        )
    } else {
        format!(
            "\n{name} = Parameter(name = '{name}', nature = 'internal', type = 'real', \
             value = {value}, texname = '{name}')\n"
        )
    }
}

#[test]
fn a_conditional_value_selects_by_the_default_of_its_condition() {
    let value = "'0.0 if aM1==0. else 1./aM1'";
    for (aem1, want) in [(0.0, 0.0), (125.0, 0.008)] {
        let text = param("aM1", "external", &format!("{aem1:?}"), 10)
            + &param("aEW", "internal", value, 0);
        let values = values_of(&[("parameters.py", &text)]);
        assert_eq!(values["aEW"], Complex64::new(want, 0.0), "aM1 = {aem1}");
    }
}

#[test]
fn a_width_written_with_max_re_and_cmath_sqrt_evaluates() {
    let text = param(
        "WX",
        "internal",
        "'max(1e-5, re(cmath.sqrt(MS**2 - 4.*c1)))/(8*cmath.pi)'",
        0,
    );
    let values = values_of(&[("parameters.py", &text)]);
    let want = (100.0_f64 - 4.0).sqrt().max(1e-5) / (8.0 * std::f64::consts::PI);
    assert!((values["WX"].re - want).abs() < 1e-14, "{}", values["WX"]);
}

#[test]
fn a_value_split_over_lines_with_plus_is_one_string() {
    let text = param("Y", "internal", "'2*' + 'MS'", 0);
    assert_eq!(values_of(&[("parameters.py", &text)])["Y"].re, 20.0);
}

#[test]
fn a_value_that_is_a_bare_name_reads_that_name() {
    // A module-level number, for an external parameter.
    let text = "\nMED = 91.5\n".to_owned() + &param("MZp", "external", "MED", 11);
    assert_eq!(values_of(&[("parameters.py", &text)])["MZp"].re, 91.5);
    // A parameter object, for an internal one: MadGraph writes its `name`.
    let text = param("Y", "internal", "c2", 0);
    assert_eq!(values_of(&[("parameters.py", &text)])["Y"].re, 1.0);
}

#[test]
fn a_bare_name_nothing_defines_is_refused_naming_it() {
    let text = param("MZp", "external", "MED", 11);
    let message = refusal(&[], &[("parameters.py", &text)]);
    assert!(
        message.contains("'MZp'") && message.contains("`MED`"),
        "{message}"
    );
    // Defined only after the parameter that reads it: Python raises there too.
    let text = param("MZp", "external", "MED", 11) + "\nMED = 91.5\n";
    let message = refusal(&[], &[("parameters.py", &text)]);
    assert!(message.contains("`MED`"), "{message}");
    // An internal parameter that is neither a number nor an earlier parameter.
    let text = param("Y", "internal", "LATER", 0) + &param("LATER", "internal", "'1.0'", 0);
    assert!(refusal(&[], &[("parameters.py", &text)]).contains("`LATER`"));
}

const GC4: &str = "
GC_4 = Coupling(name = 'GC_4', value = 'c1', order = {'NP':1,'QED':1})
";

#[test]
fn a_coupling_list_is_one_interaction_per_order() {
    let dir = build(
        &[],
        &[(
            "vertices.py",
            "couplings = {(0,0):C.GC_1}",
            "couplings = {(0,0):[ C.GC_1, C.GC_4 ]}",
        )],
        &[("couplings.py", GC4)],
    );
    let model = UFOModel::load(dir.path(), None).unwrap();
    let orders_of = |vertex: &str| {
        let v = &model.vertices[vertex];
        assert_eq!(v.couplings.len(), 1, "{vertex}");
        model.couplings[v.couplings[&(0, 0)]].orders.clone()
    };
    let orders = |pairs: &[(&str, i64)]| -> std::collections::BTreeMap<String, i64> {
        pairs.iter().map(|&(k, v)| (k.to_owned(), v)).collect()
    };
    assert_eq!(orders_of("V_1#1"), orders(&[("QED", 1)]));
    assert_eq!(orders_of("V_1#2"), orders(&[("NP", 1), ("QED", 1)]));
    assert!(!model.vertices.contains_key("V_1"));
    // The vertex that has no list is untouched.
    assert_eq!(orders_of("V_2"), orders(&[("QED", 2)]));
}

#[test]
fn two_couplings_of_one_order_in_a_list_are_refused() {
    let same_order = "\nGC_5 = Coupling(name = 'GC_5', value = 'c2', order = {'QED':1})\n";
    let message = refusal(
        &[(
            "vertices.py",
            "couplings = {(0,0):C.GC_1}",
            "couplings = {(0,0):[ C.GC_1, C.GC_5 ]}",
        )],
        &[("couplings.py", same_order)],
    );
    assert!(
        message.contains("GC_1") && message.contains("GC_5") && message.contains("(0, 0)"),
        "{message}"
    );
}

const CONFIGURATION: &str = "flag = False\nother = True\ntext = 'x'\n";

fn orders_of_gc1(
    order: &str,
    configuration: Option<&str>,
) -> Result<std::collections::BTreeMap<String, i64>, String> {
    let mut edits = vec![(
        "couplings.py",
        "GC_1 = Coupling(name = 'GC_1', value = 'c1', order = {'QED':1})",
        order,
    )];
    let header = (
        "couplings.py",
        "from object_library import all_couplings, Coupling",
        "from object_library import all_couplings, Coupling\nimport configuration",
    );
    edits.push(header);
    let append: Vec<(&str, &str)> = configuration
        .map(|c| ("configuration.py", c))
        .into_iter()
        .collect();
    let dir = build(&[], &edits, &append);
    match ParsedModel::parse(dir.path()) {
        Ok(m) => Ok(m.couplings["GC_1"].orders.clone()),
        Err(e) => Err(e.to_string()),
    }
}

#[test]
fn a_coupling_order_may_switch_on_a_configuration_flag() {
    let order = |body: &str| {
        format!("GC_1 = Coupling(name = 'GC_1', value = 'c1', order = {{'QED':1, 'NP':{body}}})")
    };
    let cases = [
        ("2 if configuration.flag else 0", "flag = False\n", 0),
        ("2 if configuration.flag else 0", "flag = True\n", 2),
        ("1 if not configuration.flag else 3", "flag = False\n", 1),
        (
            "1 if configuration.a else 2 if configuration.b else 3",
            "a = False\nb = True\n",
            2,
        ),
        ("configuration.n", "n = 3\n", 3),
        ("-1 if configuration.other else 0", "other = True\n", -1),
        ("0 + configuration.n", "n = 3\n", 3),
        (
            "configuration.inc",
            "norm = True\ninc = 1 if norm else 0\n",
            1,
        ),
    ];
    for (body, config, want) in cases {
        let got =
            orders_of_gc1(&order(body), Some(config)).unwrap_or_else(|e| panic!("{body}: {e}"));
        assert_eq!(got["NP"], want, "{body} with {config:?}");
        assert_eq!(got["QED"], 1);
    }
}

#[test]
fn a_coupling_order_outside_the_read_forms_is_refused() {
    let order = |body: &str| {
        format!("GC_1 = Coupling(name = 'GC_1', value = 'c1', order = {{'QED':1, 'NP':{body}}})")
    };
    for (body, config) in [
        ("2 if configuration.missing else 0", Some(CONFIGURATION)),
        ("2 if configuration.text else 0", Some(CONFIGURATION)),
        ("2 if configuration.flag else 0", None),
        ("2 if flag else 0", Some(CONFIGURATION)),
        ("1 / 2", Some(CONFIGURATION)),
        ("abs(2)", Some(CONFIGURATION)),
        (
            "configuration.inc",
            Some("norm = 'x'\ninc = 1 if norm else 0\n"),
        ),
    ] {
        let message = orders_of_gc1(&order(body), config).expect_err(body);
        assert!(
            message.contains("'GC_1'") && message.contains("`order`"),
            "{body}: {message}"
        );
    }
}

const PARTICLE_CHARGE: &str = "charge = 0,";

#[test]
fn a_particle_charge_that_is_a_parameters_value_loads() {
    for charge in ["charge = Param.MS.value,", "charge = MS.value,"] {
        let dir = build(
            &[],
            &[
                ("particles.py", PARTICLE_CHARGE, charge),
                (
                    "particles.py",
                    "import parameters as Param",
                    "import parameters as Param\nfrom parameters import MS",
                ),
            ],
            &[],
        );
        UFOModel::load(dir.path(), None).unwrap_or_else(|e| panic!("{charge}: {e}"));
    }
}

/// An optional source file that is present but unreadable (here, not UTF-8) is
/// refused naming it, never read as absent: an unreadable `function_library.py`
/// would otherwise leave a model's own definitions unread.
#[test]
fn an_unreadable_optional_source_file_is_refused_naming_it() {
    for file in [
        "function_library.py",
        "configuration.py",
        "coupling_orders.py",
        "propagators.py",
    ] {
        let dir = model_with(&[], &[]);
        std::fs::write(dir.path().join(file), b"\xff\xfe not UTF-8\n").unwrap();
        match ParsedModel::parse(dir.path()) {
            Err(UfoError::Io { file: path, .. }) => assert!(path.ends_with(file), "{path}"),
            other => panic!("{file}: {:?}", other.map(|_| ())),
        }
    }
}

#[test]
fn a_particle_charge_naming_nothing_is_refused() {
    for (charge, import) in [
        ("charge = MS.value,", ""),
        ("charge = Param.NOPE.value,", ""),
        ("charge = Param.ZERO.value,", ""),
        ("charge = MS.vaule,", "from parameters import MS"),
        ("charge = MS,", "from parameters import MS"),
    ] {
        let message = refusal(
            &[
                ("particles.py", PARTICLE_CHARGE, charge),
                (
                    "particles.py",
                    "import parameters as Param",
                    &format!("import parameters as Param\n{import}"),
                ),
            ],
            &[],
        );
        assert!(
            message.contains("'S'") && message.contains("charge"),
            "{charge}: {message}"
        );
    }
}

const PARAMETERS_IMPORTING_CONFIGURATION: &str =
    "from object_library import all_parameters, Parameter\n";

/// `parameters.py` of the fixture with a leading `import configuration` and
/// `extra` appended, loaded with `configuration.py` as `config`.
fn conditional_values(config: &str, extra: &str) -> Result<HashMap<String, Complex64>, String> {
    let dir = build(
        &[],
        &[(
            "parameters.py",
            PARAMETERS_IMPORTING_CONFIGURATION,
            "from object_library import all_parameters, Parameter\nimport configuration\n",
        )],
        &[("parameters.py", extra), ("configuration.py", config)],
    );
    UFOModel::load(dir.path(), None)
        .map(|model| EvaluatedModel::from_model(model).param_values)
        .map_err(|e| e.to_string())
}

/// MadGraph imports `parameters.py` as Python, so `if configuration.flag:`
/// around a definition and a `'A' if configuration.flag else 'B'` value both
/// resolve against `configuration.py`.
#[test]
fn definitions_and_values_switched_on_a_configuration_flag_follow_the_flag() {
    let extra = "
if configuration.fixed:
    EWK = Parameter(name = 'EWK', nature = 'external', type = 'real', value = 79.5, texname = 'E',
                    lhablock = 'NEWCOUP', lhacode = [ 20 ])
else:
    EWM1 = Parameter(name = 'EWM1', nature = 'external', type = 'real', value = 128.0, texname = 'E',
                     lhablock = 'NEWCOUP', lhacode = [ 21 ])
    EWK = Parameter(name = 'EWK', nature = 'internal', type = 'real', value = '1./EWM1', texname = 'E')
GS = Parameter(name = 'GS', nature = 'internal', type = 'real',
               value = 'MS' if configuration.norm else '1.', texname = 'G')
";
    let off = conditional_values("fixed = False\nnorm = False\n", extra).unwrap();
    assert_eq!(off["EWK"], Complex64::new(1.0 / 128.0, 0.0));
    assert_eq!(off["GS"], Complex64::new(1.0, 0.0));
    let on = conditional_values("fixed = True\nnorm = True\n", extra).unwrap();
    assert_eq!(on["EWK"], Complex64::new(79.5, 0.0));
    assert_eq!(on["GS"], Complex64::new(10.0, 0.0));
    assert!(!on.contains_key("EWM1"));
}

#[test]
fn a_parameter_condition_outside_the_subset_is_refused_by_name() {
    let extra = "
GS = Parameter(name = 'GS', nature = 'internal', type = 'real',
               value = 'MS' if configuration.mystery else '1.', texname = 'G')
";
    let message = conditional_values("fixed = False\n", extra).unwrap_err();
    assert!(
        message.contains("'GS'") && message.contains("configuration.py"),
        "{message}"
    );
    let extra = "
if configuration.fixed > 0:
    GS = Parameter(name = 'GS', nature = 'internal', type = 'real', value = '1.', texname = 'G')
";
    let message = conditional_values("fixed = 1\n", extra).unwrap_err();
    assert!(message.contains("outside the subset"), "{message}");
}

/// An `if` with no `Parameter` in any branch defines nothing the loader reads,
/// so its condition is not evaluated: a test outside the subset (here a
/// comparison) loads, and every value is the model's without it.
#[test]
fn an_if_outside_the_subset_that_defines_no_parameter_loads() {
    let extra = "
if configuration.x > 1:
    plain = 1.0
";
    let with_if = conditional_values("x = 3\n", extra).unwrap();
    let without = conditional_values("x = 3\n", "").unwrap();
    assert_eq!(with_if, without);
}

/// A refused `if` names the `Parameter`s in its own branches, not those under
/// another `if` of the file.
#[test]
fn a_refused_if_names_only_the_parameters_under_it() {
    let extra = "
if configuration.fixed:
    EWK = Parameter(name = 'EWK', nature = 'external', type = 'real', value = 79.5, texname = 'E',
                    lhablock = 'NEWCOUP', lhacode = [ 20 ])
if configuration.x > 1:
    extra = Parameter(name = 'extra', nature = 'internal', type = 'real', value = '1.0',
                      texname = 'extra')
else:
    other = Parameter(name = 'other', nature = 'internal', type = 'real', value = '2.0',
                      texname = 'other')
";
    let message = conditional_values("fixed = True\nx = 3\n", extra).unwrap_err();
    assert!(
        message.contains("parameter(s) 'extra', 'other' are defined under an `if`")
            && !message.contains("'EWK'"),
        "{message}"
    );
}

/// A name bound under a skipped `if` has the value Python gives it there, which
/// the loader does not know: a parameter whose `value` is that name is refused
/// naming both, whether or not the name was bound before the `if`.
#[test]
fn a_value_naming_a_name_bound_under_a_skipped_if_is_refused() {
    let skipped = "
if configuration.x > 1:
    plain = 2.0
";
    for (before, nature, code) in [
        ("", "external", 11),
        ("", "internal", 0),
        ("\nplain = 1.0\n", "external", 11),
        ("\nplain = 1.0\n", "internal", 0),
    ] {
        let extra = before.to_owned() + skipped + &param("P", nature, "plain", code);
        let message = conditional_values("x = 3\n", &extra).unwrap_err();
        assert!(
            message.contains("'P'") && message.contains("`plain`"),
            "{before:?} {nature}: {message}"
        );
    }
    // Bound again at module level after the `if`, the name reads that number.
    let extra = skipped.to_owned() + "\nplain = 3.0\n" + &param("P", "external", "plain", 11);
    assert_eq!(conditional_values("x = 3\n", &extra).unwrap()["P"].re, 3.0);
    // Under an `if` the subset evaluates, the branch that runs binds it.
    let extra = "
plain = 1.0
if configuration.flag:
    plain = 2.0
"
    .to_owned()
        + &param("P", "external", "plain", 11);
    assert_eq!(
        conditional_values("flag = True\n", &extra).unwrap()["P"].re,
        2.0
    );
    assert_eq!(
        conditional_values("flag = False\n", &extra).unwrap()["P"].re,
        1.0
    );
}
