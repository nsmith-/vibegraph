//! The vendored SMEFTsim `topU3l_MwScheme` UFO through the loader.
//!
//! The first non-Standard-Model UFO this project reads end to end: 21 particle
//! definitions, 260 Lorentz structures, 904 vertices carrying couplings of many
//! different orders in one vertex, a `propagators.py`, and an input scheme
//! (`{m_W, m_Z, G_F}`) the SM UFO does not use. Everything here reads the committed copy under
//! `validation/ufo/` — no MadGraph run, no submodule — but it is registered in the
//! banked layer because the numbers it pins are reconciled against MadGraph's own,
//! not derived here.
//!
//! What each measurement is a falsifier for is written at the test, because most
//! of them exist to catch a *silent* change: a loader that stopped splitting
//! interactions, or started pruning one structure too many, still loads the model
//! and still enumerates diagrams — it just enumerates the wrong ones.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::ufo::identity::digest_bytes;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{expansion_order_caps, EvaluatedModel, ParsedModel, UFOModel};

fn model_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
    assert!(dir.is_dir(), "vendored UFO missing at {}", dir.display());
    dir
}

fn load(card: &str) -> std::sync::Arc<UFOModel> {
    let dir = model_dir();
    UFOModel::load(&dir, Some(&dir.join(card))).unwrap_or_else(|e| panic!("load {card}: {e}"))
}

/// Arity histogram of a vertex set: `n legs → count`.
fn arity(model: &UFOModel) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    for v in model.vertices.values() {
        *out.entry(v.particles.len()).or_insert(0) += 1;
    }
    out
}

fn diagram_count(model: &UFOModel, process: &str) -> usize {
    let opts = ParsingOptions::default();
    let pc = parse_proc_card(&format!("generate {process}"), &opts)
        .unwrap_or_else(|e| panic!("parse '{process}': {e}"));
    let sets = generate_from_proc_card(&pc, model)
        .unwrap_or_else(|e| panic!("enumerate '{process}': {e}"));
    sets.iter().map(|s| s.diagrams.len()).sum()
}

/// MadGraph's own post-restriction interaction count for every (model, card) pair
/// the validation manifest names, keyed `<model dir>-<restrict>`, with one row
/// generated under that pair.
fn banked_interaction_counts() -> BTreeMap<String, (usize, String)> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/interactions.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text).expect("parse interactions.json");
    json["models"]
        .as_object()
        .expect("interactions.json has a `models` table")
        .iter()
        .map(|(pair, entry)| {
            let count = entry["interactions"].as_u64().expect("interaction count") as usize;
            let row = entry["rows"][0]
                .as_str()
                .expect("every banked pair names a row")
                .to_owned();
            (pair.clone(), (count, row))
        })
        .collect()
}

/// MadGraph's own diagram count per manifest row, from the committed
/// `validation/madgraph/diagrams.json`.
fn banked_diagram_counts() -> BTreeMap<String, usize> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/diagrams.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text).expect("parse diagrams.json");
    json["processes"]
        .as_object()
        .expect("diagrams.json has a `processes` table")
        .iter()
        .map(|(key, entry)| {
            (
                key.clone(),
                entry["total_diagrams"].as_u64().expect("total_diagrams") as usize,
            )
        })
        .collect()
}

/// The vendored copy against its own `SHA256SUMS`, both ways: every listed file
/// has the recorded digest, and every file in the directory is listed. A drifted
/// or extended copy fails here rather than silently changing what every gate
/// below measures.
#[test]
fn vendored_copy_matches_its_manifest() {
    let dir = model_dir();
    let manifest = std::fs::read_to_string(dir.join("SHA256SUMS")).expect("read SHA256SUMS");

    let mut listed: Vec<String> = Vec::new();
    for line in manifest.lines().filter(|l| !l.trim().is_empty()) {
        let (want, name) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("malformed SHA256SUMS line: {line:?}"));
        let bytes = std::fs::read(dir.join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"));
        assert_eq!(
            digest_bytes(&bytes),
            want,
            "{name} does not match SHA256SUMS"
        );
        listed.push(name.to_owned());
    }
    listed.sort();

    let mut present: Vec<String> = std::fs::read_dir(&dir)
        .expect("read model dir")
        .map(|e| {
            e.expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n != "SHA256SUMS")
        .collect();
    present.sort();
    assert_eq!(listed, present, "SHA256SUMS does not cover the directory");
}

/// The pre-restriction model as the loader sees it, against a direct reading of
/// the UFO's Python: 904 `Vertex(...)` entries carrying 1278 couplings become
/// 1985 interactions once split by coupling-order tuple, and no vertex is lost.
///
/// The split count is the number MadGraph's `add_interaction` produces from the
/// same file, so it is what a per-interaction diagram count is comparable
/// against. It is also the falsifier for the splitting itself: a loader that
/// stopped splitting would report 904 here and would then read every SMEFTsim
/// `FFV` vertex as carrying `NP`.
#[test]
fn interaction_splitting_matches_madgraph() {
    let parsed = ParsedModel::parse(&model_dir()).expect("parse SMEFTsim");

    // 21 `Particle(...)` entries plus the antiparticle each non-self-conjugate one
    // implies, which the loader materialises so a vertex can name either.
    assert_eq!(parsed.particles.len(), 36);
    assert_eq!(parsed.lorentz.len(), 260);
    assert_eq!(parsed.couplings.len(), 1278);
    assert_eq!(parsed.vertices.len(), 1985, "split interaction count");

    // Every split's couplings agree on their order tuple — the property the whole
    // split exists to establish, and what `topo::build_feyngraph_model` relies on.
    for (name, vertex) in &parsed.vertices {
        let mut tuples: Vec<_> = vertex
            .couplings
            .values()
            .map(|&id| &parsed.couplings[id].orders)
            .collect();
        tuples.dedup();
        assert_eq!(
            tuples.len(),
            1,
            "interaction '{name}' mixes coupling orders"
        );
    }

    let mut split_arity: BTreeMap<usize, usize> = BTreeMap::new();
    for v in parsed.vertices.values() {
        *split_arity.entry(v.particles.len()).or_insert(0) += 1;
    }
    assert_eq!(
        split_arity,
        [(3, 527), (4, 1234), (5, 212), (6, 12)]
            .into_iter()
            .collect::<BTreeMap<usize, usize>>(),
        "post-split arity histogram"
    );

    // Splitting partitions each UFO vertex's coupling entries: the 2737
    // `(color, lorentz) -> coupling` entries of the 904 `Vertex(...)` definitions
    // are spread across the 1985 interactions with none lost or duplicated. (A
    // coupling *constant* is shared across vertices, so it is the entries that are
    // counted, not the distinct coupling ids.)
    let entries: usize = parsed.vertices.values().map(|v| v.couplings.len()).sum();
    assert_eq!(entries, 2737, "coupling entries after splitting");
}

/// Every one of the 260 Lorentz structures parses, with the operator mix the
/// model actually writes. `Gamma5` and the `**` powers are the two the parser
/// gained for this model: `Gamma5` used to fail the whole load as an
/// `UnknownOperator`, and `P(-1,a)**2` used to fail as a syntax error. The `P`
/// count is *after* power expansion, so it exceeds the 3189 textual `P(` uses by
/// the 123 squared momenta, each of which becomes two contracted copies.
#[test]
fn every_lorentz_structure_parses() {
    use vibegraph::ufo::lorentz::LorentzOp;

    let parsed = ParsedModel::parse(&model_dir()).expect("parse SMEFTsim");
    let mut ops: BTreeMap<&'static str, usize> = BTreeMap::new();
    for structure in parsed.lorentz.values() {
        for term in &structure.expr {
            for op in &term.ops {
                let name = match op {
                    LorentzOp::Gamma { .. } => "Gamma",
                    LorentzOp::Gamma5 { .. } => "Gamma5",
                    LorentzOp::Sigma { .. } => "Sigma",
                    LorentzOp::Identity { .. } => "Identity",
                    LorentzOp::ProjM { .. } => "ProjM",
                    LorentzOp::ProjP { .. } => "ProjP",
                    LorentzOp::Metric { .. } => "Metric",
                    LorentzOp::P { .. } => "P",
                    LorentzOp::Epsilon { .. } => "Epsilon",
                    LorentzOp::C { .. } => "C",
                };
                *ops.entry(name).or_insert(0) += 1;
            }
        }
    }
    println!("SMEFTsim Lorentz operator uses: {ops:?}");
    assert_eq!(ops.get("Gamma5"), Some(&13));
    assert!(
        !ops.contains_key("Sigma"),
        "SMEFTsim emits no literal Sigma"
    );
    assert!(
        !ops.contains_key("C"),
        "SMEFTsim emits no charge conjugation"
    );
}

/// `propagators.py` is read rather than refused: the four width-corrected
/// auxiliary fields keep their propagator forms verbatim, and only they carry
/// one. The refusal now lives where such a particle actually propagates
/// (`diagrams::diagram::ConvertError::CustomPropagator`).
#[test]
fn custom_propagators_are_read_and_attached() {
    let parsed = ParsedModel::parse(&model_dir()).expect("parse SMEFTsim");

    let mut named: Vec<(&str, &str)> = parsed
        .particles
        .values()
        .filter_map(|p| p.propagator.as_deref().map(|prop| (p.name.as_str(), prop)))
        .collect();
    named.sort();
    assert_eq!(
        named,
        [
            ("H1", "H1"),
            ("W1+", "W1"),
            ("W1-", "W1"),
            ("Z1", "Z1"),
            ("t1", "T1"),
            ("t1~", "T1"),
        ]
    );

    // The forms are kept as written, including the `denominator + "**2"`
    // concatenation the file builds them from.
    let z1 = &parsed.propagators["Z1"];
    assert!(
        z1.numerator.contains("dWZ"),
        "Z1 numerator: {}",
        z1.numerator
    );
    assert!(
        z1.denominator.ends_with("**2"),
        "Z1 denominator: {}",
        z1.denominator
    );
    assert_eq!(parsed.propagators["V2"].numerator, "- Metric(1, 2)");
}

/// SMEFTsim declares an `expansion_order` for every coupling order, and exactly
/// one of them is not 99: `NPprop = 0`. MadGraph's window is `0 < v < 99`, so
/// *none* of them caps anything — the auxiliary fields are kept out of a default
/// process by their hierarchy-99 orders under the WEIGHTED search, not by
/// `expansion_order`. Recorded here because the opposite is the natural reading.
#[test]
fn expansion_order_is_declared_but_caps_nothing() {
    let parsed = ParsedModel::parse(&model_dir()).expect("parse SMEFTsim");
    assert_eq!(parsed.expansion_order.get("NPprop"), Some(&0));
    assert_eq!(parsed.expansion_order.get("QED"), Some(&99));
    assert_eq!(parsed.expansion_order.get("NP"), Some(&99));
    assert_eq!(
        parsed
            .expansion_order
            .values()
            .filter(|&&v| v != 99)
            .count(),
        1,
        "only NPprop departs from 99"
    );
    assert!(
        expansion_order_caps(&parsed.expansion_order).is_empty(),
        "nothing in SMEFTsim falls inside MadGraph's 0 < expansion_order < 99 window"
    );
    assert_eq!(parsed.order_hierarchy.get("NPprop"), Some(&99));
    assert_eq!(parsed.order_hierarchy.get("QCD"), Some(&1));
    assert_eq!(parsed.order_hierarchy.get("QED"), Some(&2));
}

/// Every restrict card the manifest names, against MadGraph's own count of what
/// survives it.
///
/// The reference is `display interactions` in each row's build log, read into
/// `validation/madgraph/interactions.json`: the number of interactions MadGraph
/// holds after `add_interaction` has split every UFO vertex by coupling-order
/// tuple and the restriction has dropped the ones whose couplings all vanish. Two
/// programs arriving at the same number from the same file is what says the split
/// and the pruning are MadGraph's, and it is checked on all twelve pairs rather
/// than the two shipped cards, because the per-class cards are where a card zeroes
/// most of the model and a pruning error has room to show.
#[test]
fn restricted_interaction_counts_match_madgraph() {
    let banked = banked_interaction_counts();
    assert!(!banked.is_empty(), "no banked interaction counts");
    for (pair, (want, row)) in &banked {
        let model = common::model_for_row(row)
            .unwrap_or_else(|e| panic!("[{pair}] load the model row '{row}' names: {e}"));
        assert_eq!(
            model.vertices.len(),
            *want,
            "[{pair}] interactions after restriction"
        );
    }
}

/// The two shipped restrict cards, after zero couplings, empty interactions and
/// unreferenced Lorentz structures are removed in MadGraph's order.
///
/// `SMlimit_massless` zeroes every Wilson coefficient, so what survives is the
/// Standard Model as SMEFTsim writes it; `massless` sets every real coefficient
/// to a fixed non-zero value and keeps a vertex from every structure class,
/// including the five- and six-leg field-strength contact terms. The counts
/// themselves are MadGraph's ([`restricted_interaction_counts_match_madgraph`]);
/// what is asserted here is how they are distributed over vertex arities, which
/// MadGraph's log does not report and which is this side's own measurement.
#[test]
fn both_restrict_cards_prune_to_a_workable_model() {
    let banked = banked_interaction_counts();
    let sm_limit = load("restrict_SMlimit_massless.dat");
    assert_eq!(
        sm_limit.vertices.len(),
        banked["SMEFTsim_topU3l_MwScheme_UFO-SMlimit_massless"].0
    );
    assert_eq!(
        arity(&sm_limit),
        [(3, 50), (4, 10), (5, 2)]
            .into_iter()
            .collect::<BTreeMap<usize, usize>>()
    );

    let all_on = load("restrict_massless.dat");
    assert_eq!(
        all_on.vertices.len(),
        banked["SMEFTsim_topU3l_MwScheme_UFO-massless"].0
    );
    assert_eq!(
        arity(&all_on),
        [(3, 256), (4, 564), (5, 82), (6, 11)]
            .into_iter()
            .collect::<BTreeMap<usize, usize>>()
    );

    // Pruning is exactly the removal of vanishing couplings: nothing survives
    // holding a Lorentz structure no coupling of its own refers to, which is what
    // used to send the evaluator into a zero-coupling dipole chain.
    for model in [&sm_limit, &all_on] {
        for (name, vertex) in &model.vertices {
            let used: std::collections::BTreeSet<usize> =
                vertex.couplings.keys().map(|&(_, l)| l).collect();
            assert_eq!(
                used.len(),
                vertex.lorentz.len(),
                "interaction '{name}' kept an unreferenced Lorentz structure"
            );
            assert!(!vertex.couplings.is_empty());
        }
    }
}

/// The `{m_W, m_Z, G_F}` input scheme evaluates to the values SMEFTsim's own
/// documentation quotes, under both cards — the derived electroweak parameters
/// are what every amplitude below is built from, and the Wilson coefficients do
/// not shift them at this order in the `massless` card either.
#[test]
fn mw_scheme_derived_parameters() {
    for card in ["restrict_SMlimit_massless.dat", "restrict_massless.dat"] {
        let model = load(card);
        let ev = EvaluatedModel::from_model_card(model, &ParamCard::default());
        let at = |name: &str| ev.param_values[name].re;
        assert!(
            (at("ee") - 0.30825).abs() < 5e-6,
            "[{card}] ee = {}",
            at("ee")
        );
        assert!(
            (at("sth") - 0.47208).abs() < 5e-6,
            "[{card}] sth = {}",
            at("sth")
        );
        assert!(
            (at("vevhat") - 246.22).abs() < 5e-3,
            "[{card}] vevhat = {}",
            at("vevhat")
        );
        assert!(
            (at("yt") - 0.99228).abs() < 5e-6,
            "[{card}] yt = {}",
            at("yt")
        );
        // The scheme's own inputs, straight from the card.
        assert!((at("MW") - 80.387).abs() < 1e-9);
        assert!((at("MZ") - 91.1876).abs() < 1e-9);
        assert!((at("LambdaSMEFT") - 1000.0).abs() < 1e-9);
    }
}

/// The SMEFTsim rows the manifest gates, with the process each one enumerates.
///
/// Order bounds are part of the process string: SMEFTsim gives `NP` hierarchy 99,
/// so MadGraph's default WEIGHTED search drops every diagram carrying a Wilson
/// coefficient and `b b~ > h` is a different process of this model from
/// `b b~ > h NP<=1`. The same is true of `SMHLOOP`, which is what separates the
/// two `g g > t t~` rows.
///
/// This list is the model's coverage instrument twice over: the diagram counts
/// below and the op census at the end of the file both run on it.
const GATED_ROWS: [(&str, &str); 12] = [
    ("ee_to_mumu_smlimit", "e+ e- > mu+ mu-"),
    ("gg_to_ttx_smlimit", "g g > t t~"),
    ("gg_to_ttx_smlimit_qcd2", "g g > t t~ QCD<=2"),
    ("ee_to_ttx_smlimit", "e+ e- > t t~"),
    ("bbx_to_h_identity", "b b~ > h NP<=1"),
    ("gg_to_h_cpeven", "g g > h NP<=1"),
    ("gg_to_h_cpodd", "g g > h NP<=1"),
    ("ee_to_ttx_dipole", "e+ e- > t t~ NP<=1"),
    ("gg_to_gg_cg", "g g > g g NP<=1"),
    ("ee_to_mumu_4f", "e+ e- > mu+ mu- NP<=1"),
    ("uux_to_ttx_4f", "u u~ > t t~ NP<=1"),
    ("ee_to_ttx_smeft", "e+ e- > t t~ NP<=1"),
];

/// [`GATED_ROWS`] is exactly the set of SMEFTsim rows the manifest declares
/// `amplitudes = gate`.
///
/// The census and the diagram counts below are only a coverage statement if the
/// list they run on is the gated set; a row promoted in the manifest without being
/// added here would silently leave the census, and an op it newly covers would
/// stay on the allowlist.
#[test]
fn gated_rows_are_the_manifest_s_gated_smeftsim_rows() {
    let modes = common::manifest::category_modes("amplitudes");
    let declared: std::collections::BTreeSet<String> = common::manifest::row_models()
        .into_iter()
        .filter(|(key, row)| {
            row.dir.ends_with("SMEFTsim_topU3l_MwScheme_UFO")
                && modes.get(key).map(String::as_str) == Some("gate")
        })
        .map(|(key, _)| key)
        .collect();
    let listed: std::collections::BTreeSet<String> = GATED_ROWS
        .iter()
        .map(|(key, _)| (*key).to_owned())
        .collect();
    assert_eq!(listed, declared);
}

/// Every gated row's diagram count against MadGraph's own, from the committed
/// `validation/madgraph/diagrams.json`.
///
/// This is what the interaction splitting exists for. Before it, `e+ e- > mu+ mu-`
/// enumerated **0** — every SMEFTsim `FFV` vertex bundles the SM current with
/// dipole and current-shift couplings, so the union of their orders made the
/// photon vertex read as `NP = 1` and two of them exceeded any bound. And
/// `g g > t t~` enumerated **4**, the extra one being the `g g > H > t t~`
/// s-channel through SMEFTsim's effective `SMHLOOP` `ggH` vertex; the WEIGHTED
/// default costs that diagram 99 per `SMHLOOP` power and drops it, leaving
/// MadGraph's 3. It is a real diagram of this model, not a spurious one, which is
/// what the `QCD<=2` row is here to show: bounding `QCD` while leaving `SMHLOOP`
/// free brings it back, on both sides.
///
/// A row whose `diagrams` cell the manifest declares informational is counted and
/// reported rather than asserted, exactly as `validate_madgraph_diagrams` does and
/// for the same reason it does: `g g > g g` under a four-gluon vertex with several
/// colour structures is one diagram here and one MadGraph graph per colour-ordered
/// structure, a counting convention rather than a missing diagram. The amplitude
/// gate compares those rows at the per-flow and configuration levels instead.
#[test]
fn gated_row_diagram_counts_match_madgraph() {
    let banked = banked_diagram_counts();
    let modes = common::manifest::category_modes("diagrams");
    for (key, process) in GATED_ROWS {
        let model = common::model_for_row(key)
            .unwrap_or_else(|e| panic!("[{key}] load the row's model: {e}"));
        let want = banked
            .get(key)
            .unwrap_or_else(|| panic!("[{key}] has no entry in diagrams.json"));
        let ours = diagram_count(&model, process);
        if modes.get(key).map(String::as_str) == Some("info") {
            println!(
                "[{key}] '{process}': {ours} diagrams against MadGraph's {want} (informational)"
            );
            continue;
        }
        assert_eq!(ours, *want, "[{key}] '{process}'");
    }
}

/// No process of the coverage table selects a diagram in which one of the four
/// width-corrected auxiliary fields propagates, so none of them meets the
/// custom-propagator refusal. Their `NPprop` order carries hierarchy 99, which is
/// what the WEIGHTED default charges them; `expansion_order` does not (see
/// [`expansion_order_is_declared_but_caps_nothing`]).
///
/// That the refusal *fires* when it should is pinned separately, on a model built
/// to trigger it (`diagrams::diagram`), because a refusal nothing reaches is a
/// check that proves nothing.
#[test]
fn no_coverage_row_propagates_an_auxiliary_field() {
    let all_on = load("restrict_massless.dat");
    let opts = ParsingOptions::default();
    for process in [
        "e+ e- > mu+ mu- NP<=1",
        "e+ e- > t t~ NP<=1",
        "e+ e- > W+ W- NP<=1",
        "e+ e- > Z h NP<=1",
    ] {
        let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &all_on)
            .unwrap_or_else(|e| panic!("'{process}' reached the custom-propagator refusal: {e}"));
        for set in &sets {
            for diagram in &set.diagrams {
                for prop in &diagram.props {
                    assert!(
                        all_on.particle(prop.particle).propagator.is_none(),
                        "'{process}' propagates {}",
                        all_on.particle(prop.particle).name
                    );
                }
            }
        }
    }
}

/// The gated rows as this model's op-coverage instrument.
///
/// The same two-way census the Standard Model's MG-validated suite runs
/// (`helas::eval::op_census`), instantiated on a second model and over more than
/// one restrict card: the ops the gated rows compile to are the ops MadGraph's
/// reference actually exercises here, everything else is listed, and an op the
/// list starts covering must be struck from the allowlist. The list is short
/// because only these rows compile today; it grows as the primitives the full
/// SMEFT rows need arrive, and the allowlist shrinks with it.
#[test]
fn gated_rows_op_census() {
    use vibegraph::helas::eval::op_census::{assert_op_coverage_across, Op};

    // Every op the gated rows do not reach. `Hels` is never emitted at compile time
    // in any model (the helicity expansion derives it). The chiral projector ops are
    // absent because SMEFTsim writes its scalar four-fermion contacts with Wilson
    // coefficients this ladder's cards leave at zero, so every contact that survives
    // is the vector⊗vector shape, whose projectors sit inside a gamma chain. The
    // fermion-continuing fused forms `FfvIout`/`FfvOout` need a chiral *pair* on a
    // vertex rooted at a fermion leg, which SMEFTsim's restricted vertices do not
    // keep — `FfvVout`, the pair rooted at the vector leg, is reached by the
    // capstone's `e+ e- > t t~` currents. `Gamma5Amp` (a bare pseudoscalar bilinear)
    // exists in this model but not in these rows: the dipole reaches γ⁵ only inside
    // a chain, never as a bilinear of its own. The four-gluon row is what covers
    // `EpsilonVout` (a Levi-Civita tensor rooted at a vector leg rather than closed
    // into the amplitude) and `MetricVout`: its gluon-exchange diagrams root both at
    // the off-shell gluon.
    const KNOWN_UNCOVERED: [Op; 6] = [
        Op::Hels,
        Op::ProjMAmp,
        Op::ProjPAmp,
        Op::Gamma5Amp,
        Op::FfvIout,
        Op::FfvOout,
    ];

    let models: Vec<_> = GATED_ROWS
        .iter()
        .map(|(key, process)| {
            let model = common::model_for_row(key)
                .unwrap_or_else(|e| panic!("[{key}] load the row's model: {e}"));
            (*key, model, [*process])
        })
        .collect();
    let instances: Vec<_> = models
        .iter()
        .map(|(key, model, processes)| (*key, model.as_ref(), &processes[..]))
        .collect();
    assert_op_coverage_across(&instances, &KNOWN_UNCOVERED);
}
