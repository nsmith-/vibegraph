//! Per-helicity, per-flow amplitudes against MadGraph standalone output.
//!
//! Each table under `validation/madgraph/standalone/` (written by
//! `validation/madgraph/gen_standalone_jamps.py`) carries MadGraph's JAMP(1..NCOLOR)
//! and colour-summed |M|² for every helicity combination of its NHEL table, at a
//! few RAMBO points, together with the param card it computed them with. This side
//! binds the same card, evaluates the same helicities at the same momenta, and
//! compares:
//!
//! * the per-flow amplitudes, flow by flow in MadGraph's colour-basis order, under
//!   one complex constant fitted over every (point, helicity, flow) entry — the
//!   finest linear object MadGraph exposes that is one-to-one with ours;
//! * the per-helicity colour-summed |M|², which the fitted constant must also
//!   explain (it is `|G|²` there).
//!
//! The fitted constant must have modulus one — the two sides normalise amplitudes
//! alike — but its phase is free: external-wavefunction phase conventions differ
//! between the two programs by a process-dependent `±1`/`±i`.
//!
//! A single-flow table may also bank MadGraph's per-graph `AMP()` with JAMP(1)'s
//! coefficient on each. Such a row declares in [`MG_GRAPH_DIAGRAMS`] which of its
//! diagrams make up each MadGraph graph, and each group, compiled one diagram at a
//! time through the production evaluator and summed, must equal `G · c_k · AMP_k`
//! under the same `G`. A group is not always one diagram: see
//! [`MG_GRAPH_DIAGRAMS`] on the four-fermion contacts.
//!
//! What it cannot see: that global phase (a sign shared by every diagram of the
//! process); the split of a flow into diagrams, so two compensating errors in
//! diagrams that reach the same flows with the same weights at every point (closed
//! per MadGraph graph where `AMP()` is banked, but not inside a group); and any
//! helicity or kinematic region the few banked points do not visit.
//!
//! Needs only the committed tables, the interned SM model, the vendored SMEFTsim
//! UFO and the authored toy UFO.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{generate_with, sm_model};
use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// Relative deviation allowed per entry, against the largest MadGraph amplitude of
/// the table. Both sides round over the same few dozen diagrams, which sits near
/// 1e-15; a sign on one diagram class is O(1).
const REL_TOL: f64 = 1e-12;

fn table_path(key: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/standalone")
        .join(format!("{key}.json"))
}

fn complex(v: &serde_json::Value) -> C<f64> {
    C::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

/// For a row that banks MadGraph's `AMP()`, the MadGraph graph each of this side's
/// diagrams belongs to, in enumeration order: entry `d` is diagram `d`'s graph,
/// 0-based in `AMP()` order, the convention of `amplitude_oracle`'s
/// `MG_DIAGRAM_ORDER`. The diagrams that share a graph make it up together, and
/// every graph must have at least one.
///
/// `ee_to_ee_4f`: the three four-lepton contacts are three MadGraph graphs and six
/// diagrams here. MadGraph builds each contact on one canonical pairing of the
/// four legs and writes both spinor pairings of its Lorentz structures into that
/// one `AMP()` — its merged ALOHA routine carries the second pairing's terms with
/// the `-1` of `get_sign_flow` — while this side builds one diagram per pairing on
/// that pairing's own fermion lines, and the relative Fermi sign is the diagrams'.
/// Each contact's two diagrams sum to its graph. For the `cll1` and `cee`
/// contacts the two pairings are Fierz-equal, so each diagram is half the graph
/// and the wrong relative sign would cancel the contact; for `cle` they differ.
/// The photon and Z exchanges pair one to one. MadGraph's `AMP(5)`/`AMP(6)` and
/// `AMP(13)`/`AMP(14)` carry the Standard-Model and SMEFT photon couplings on the
/// two vertices in either order and are equal, so the values cannot tell which of
/// each pair is which: those two pairings are banked in enumeration order, and
/// swapping either is invisible here.
///
/// Every other row pairs one diagram to one graph. In `ttx_to_hh_chbox`,
/// `ht_to_ht_chbox` and `hh_to_hh_chbox` several graphs are equal at every point:
/// a Yukawa pair whose two couplings are swapped between its vertices, and the
/// O_Hbox shift of the cubic Higgs coupling against the same shift of the coupling
/// on the other vertex. Within each such set any assignment fits equally, so the
/// one banked is arbitrary and a swap inside the set is invisible.
const MG_GRAPH_DIAGRAMS: &[(&str, &[usize])] = &[
    ("ee_to_ttxh", &[3, 4, 1, 2, 0]),
    ("ee_to_ttxh_identity", &[3, 4, 1, 2, 0]),
    ("ll_to_qqxs_toy_yukawa", TOY_YUKAWA_GRAPHS),
    ("ll_to_ss_toy_dscalar", &[0, 1, 2, 3]),
    ("sl_to_sl_toy_dscalar", &[0, 1, 2, 3]),
    ("ttx_to_hh_chbox", TTX_TO_HH_GRAPHS),
    ("ht_to_ht_chbox", HT_TO_HT_GRAPHS),
    ("hh_to_hh_chbox", HH_TO_HH_GRAPHS),
    (
        "ee_to_ee_4f",
        &[
            0, 0, 1, 1, 2, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ],
    ),
];

const TOY_YUKAWA_GRAPHS: &[usize] = &[
    10, 11, 12, 14, 13, 15, 16, 18, 17, 19, 0, 1, 2, 4, 3, 5, 6, 8, 7, 9, 20, 21, 23, 22, 24, 25,
    26, 28, 27, 29, 30, 31, 33, 32, 34, 35, 36, 38, 37, 39,
];

const TTX_TO_HH_GRAPHS: &[usize] = &[0, 2, 1, 5, 6, 3, 4, 7, 10, 11, 8, 9, 12, 15, 16, 13, 14];

const HT_TO_HT_GRAPHS: &[usize] = &[0, 3, 4, 1, 2, 5, 10, 11, 9, 8, 6, 7, 12, 15, 16, 13, 14];

const HH_TO_HH_GRAPHS: &[usize] = &[
    0, 2, 1, 3, 4, 5, 11, 10, 12, 13, 7, 6, 8, 9, 14, 20, 19, 21, 22, 16, 15, 17, 18, 23, 29, 28,
    30, 31, 25, 24, 26, 27,
];

/// The per-graph level of a row that banks `AMP()`.
struct GraphLevel {
    /// JAMP(1)'s coefficient on each `AMP()`.
    coefs: Vec<f64>,
    /// The diagrams that make up each graph, in `AMP()` order.
    groups: Vec<Vec<usize>>,
    /// One evaluator per diagram, each compiled from a one-diagram set.
    per_diagram: Vec<AmplitudeEvaluator>,
}

impl GraphLevel {
    /// Reads the row's [`MG_GRAPH_DIAGRAMS`] entry against its table, two-way: a
    /// table that banks `AMP()` must be declared there, and a declared row must bank
    /// them. The entry must give every diagram a graph and every one of the NGRAPHS
    /// graphs a diagram, which is the diagram-count convention stated as a check.
    fn read(
        key: &str,
        table: &serde_json::Value,
        set: &DiagramSet,
        model: &UFOModel,
    ) -> Option<Self> {
        let declared = MG_GRAPH_DIAGRAMS.iter().find(|(k, _)| *k == key);
        let (coefs, graph_of) = match (table["jamp_coefficients"].as_array(), declared) {
            (None, None) => return None,
            (Some(coefs), Some((_, graph_of))) => (coefs, *graph_of),
            (Some(_), None) => panic!("{key}: banks AMP() but declares no MG_GRAPH_DIAGRAMS"),
            (None, Some(_)) => {
                panic!("{key}: declares MG_GRAPH_DIAGRAMS but its table banks no AMP()")
            }
        };
        let coefs: Vec<f64> = coefs.iter().map(|c| c.as_f64().unwrap()).collect();
        let n_graphs = table["n_graphs"].as_u64().unwrap() as usize;
        assert_eq!(coefs.len(), n_graphs, "{key}: one coefficient per AMP()");
        assert_eq!(
            graph_of.len(),
            set.diagrams.len(),
            "{key}: MG_GRAPH_DIAGRAMS must name one graph for each diagram"
        );
        let mut groups = vec![Vec::new(); n_graphs];
        for (d, &k) in graph_of.iter().enumerate() {
            assert!(
                k < n_graphs,
                "{key}: diagram {d} names graph {k} of {n_graphs}"
            );
            groups[k].push(d);
        }
        let empty: Vec<usize> = (0..n_graphs).filter(|&k| groups[k].is_empty()).collect();
        assert!(empty.is_empty(), "{key}: graphs {empty:?} have no diagram");
        let per_diagram = set
            .diagrams
            .iter()
            .map(|d| {
                AmplitudeEvaluator::compile(
                    &DiagramSet {
                        particles_in: set.particles_in.clone(),
                        particles_out: set.particles_out.clone(),
                        polarizations: set.polarizations.clone(),
                        diagrams: vec![d.clone()],
                    },
                    model,
                )
                .unwrap()
            })
            .collect();
        Some(GraphLevel {
            coefs,
            groups,
            per_diagram,
        })
    }

    /// `(c_k · AMP_k, the sum of graph k's diagrams, where)` for each graph of one
    /// helicity entry, from MadGraph's `amps` and this side's per-diagram values.
    fn entries(
        &self,
        key: &str,
        amps: &serde_json::Value,
        diagram_values: &[C<f64>],
        what: &str,
    ) -> Vec<(C<f64>, C<f64>, String)> {
        let amps = amps.as_array().unwrap();
        assert_eq!(
            amps.len(),
            self.coefs.len(),
            "{key}: {what}: one AMP() per graph"
        );
        amps.iter()
            .zip(&self.groups)
            .enumerate()
            .map(|(k, (amp, group))| {
                let mg = complex(amp) * self.coefs[k];
                let vg: C<f64> = group.iter().map(|&d| diagram_values[d]).sum();
                (
                    mg,
                    vg,
                    format!("{what}, AMP({}) = diagrams {group:?}", k + 1),
                )
            })
            .collect()
    }
}

/// The model a table was computed in: the interned SM, the model and restrict card
/// of the manifest row it names, or (`"ufo"`) the committed model directory and
/// restrict card it records itself, for a model card no manifest row carries.
fn table_model(key: &str, table: &serde_json::Value) -> Arc<UFOModel> {
    match table["model"].as_str().unwrap() {
        "sm" => sm_model(),
        "ufo" => {
            let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
            let ufo = &table["ufo"];
            let dir = repo.join(ufo["dir"].as_str().unwrap());
            let card = repo.join(ufo["restrict_card"].as_str().unwrap());
            UFOModel::load(&dir, Some(&card))
                .unwrap_or_else(|e| panic!("{key}: load {}: {e}", dir.display()))
        }
        row => common::model_for_row(row).unwrap_or_else(|e| panic!("{key}: {e}")),
    }
}

struct Measured {
    g: C<f64>,
    worst_flow: f64,
    worst_flow_where: String,
    worst_m2: f64,
    worst_m2_where: String,
    worst_graph: f64,
    worst_graph_where: String,
    n_entries: usize,
    n_graph_entries: usize,
}

impl Measured {
    /// Every way the comparison can disagree with MadGraph, empty when it agrees.
    fn disagreements(&self, key: &str) -> Vec<String> {
        let mut out = Vec::new();
        if self.worst_flow >= REL_TOL {
            out.push(format!(
                "{key}: {} ({:.3e} of the largest amplitude)",
                self.worst_flow_where, self.worst_flow
            ));
        }
        if self.worst_m2 >= REL_TOL {
            out.push(format!(
                "{key}: {} ({:.3e})",
                self.worst_m2_where, self.worst_m2
            ));
        }
        if self.worst_graph >= REL_TOL {
            out.push(format!(
                "{key}: {} ({:.3e} of the largest graph)",
                self.worst_graph_where, self.worst_graph
            ));
        }
        if (self.g.norm() - 1.0).abs() >= REL_TOL {
            out.push(format!(
                "{key}: the flows agree only up to |G| = {}",
                self.g.norm()
            ));
        }
        out
    }
}

/// Reads the table, binds its card and evaluates every listed entry. A table that
/// cannot be read, or a process whose colour or helicity structure is not the
/// table's, panics: those are not disagreements with MadGraph, and a row listed in
/// [`KNOWN_DISAGREEMENT`] must not absorb them. The deviations themselves are
/// returned for [`check`] to judge.
fn measure(key: &str) -> Measured {
    measure_with(key, |_| {})
}

/// [`measure`] on the table's model after `alter` has edited a private copy of it.
fn measure_with(key: &str, alter: impl FnOnce(&mut UFOModel)) -> Measured {
    let path = table_path(key);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let table: serde_json::Value = serde_json::from_str(&text).unwrap();
    let model = table_model(key, &table);
    let mut altered = (*model).clone();
    alter(&mut altered);
    let model = Arc::new(altered);
    let process = table["process"].as_str().unwrap();
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);
    let set = generate_with(process, model.as_ref()).remove(0);
    let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).unwrap();
    let n_flows = table["n_flows"].as_u64().unwrap() as usize;
    assert_eq!(evaluator.n_flows(), n_flows, "{key}: NCOLOR");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    let helicities: Vec<Vec<i32>> = table["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            h.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect()
        })
        .collect();
    let graphs = GraphLevel::read(key, &table, &set, model.as_ref());
    let per_diagram_bound: Vec<BoundAmplitude<f64>> = graphs
        .iter()
        .flat_map(|level| level.per_diagram.iter())
        .map(|ev| BoundAmplitude::<f64>::bind(ev, &evaluated))
        .collect();

    let ours: std::collections::BTreeSet<&Vec<i32>> = evaluator.helicities().iter().collect();
    let theirs: std::collections::BTreeSet<&Vec<i32>> = helicities.iter().collect();
    assert_eq!(
        ours, theirs,
        "{key}: the helicity sums run over different sets"
    );

    // (MadGraph, ours) per flow entry, and per-helicity |M|² pairs.
    let mut flows: Vec<(C<f64>, C<f64>, String)> = Vec::new();
    let mut m2: Vec<(f64, f64, String)> = Vec::new();
    // (c_k · AMP_k, the group's summed diagrams) per graph entry.
    let mut graph_entries: Vec<(C<f64>, C<f64>, String)> = Vec::new();
    for (pi, point) in table["points"].as_array().unwrap().iter().enumerate() {
        let momenta: Vec<LorentzVector<f64>> = point["momenta"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| {
                let c: Vec<f64> = k
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap())
                    .collect();
                LorentzVector::new(c[0], c[1], c[2], c[3])
            })
            .collect();
        for (hel, entry) in helicities.iter().zip(point["helicity"].as_array().unwrap()) {
            let mg: Vec<C<f64>> = entry["jamps"]
                .as_array()
                .unwrap()
                .iter()
                .map(complex)
                .collect();
            let vg = if n_flows == 1 {
                vec![bound.eval_amplitude(&momenta, hel, &mut scratch)]
            } else {
                bound.run_flows(&momenta, hel, &mut scratch)
            };
            for (f, (a, b)) in mg.iter().zip(&vg).enumerate() {
                flows.push((*a, *b, format!("point {pi}, helicities {hel:?}, flow {f}")));
            }
            let cf = evaluator.cf_matrix();
            let mut ours_m2 = 0.0;
            for i in 0..n_flows {
                for j in 0..n_flows {
                    let c =
                        *cf[i * n_flows + j].numer() as f64 / *cf[i * n_flows + j].denom() as f64;
                    ours_m2 += c * (vg[i].conj() * vg[j]).re;
                }
            }
            m2.push((
                entry["m2"].as_f64().unwrap(),
                ours_m2,
                format!("point {pi}, helicities {hel:?}"),
            ));
            if let Some(level) = &graphs {
                let diagram_values: Vec<C<f64>> = per_diagram_bound
                    .iter()
                    .map(|b| b.eval_amplitude(&momenta, hel, &mut b.scratch_space()))
                    .collect();
                graph_entries.extend(level.entries(
                    key,
                    &entry["amps"],
                    &diagram_values,
                    &format!("point {pi}, helicities {hel:?}"),
                ));
            }
        }
    }

    let (mut num, mut den, mut scale) = (C::new(0.0, 0.0), 0.0f64, 0.0f64);
    for (mg, vg, _) in &flows {
        num += mg.conj() * vg;
        den += mg.norm_sqr();
        scale = scale.max(mg.norm());
    }
    let g = num / den;
    let (mut worst_flow, mut worst_flow_where) = (0.0f64, String::new());
    for (mg, vg, what) in &flows {
        let dev = (vg - g * mg).norm() / scale;
        if dev > worst_flow {
            worst_flow = dev;
            worst_flow_where = format!(
                "{what}: vibegraph {vg:?} against G·MadGraph {:?} (G = {g:?})",
                g * mg
            );
        }
    }
    let m2_scale = m2.iter().map(|e| e.0.abs()).fold(0.0, f64::max);
    let (mut worst_m2, mut worst_m2_where) = (0.0f64, String::new());
    for (mg, vg, what) in &m2 {
        let dev = (vg - g.norm_sqr() * mg).abs() / m2_scale;
        if dev > worst_m2 {
            worst_m2 = dev;
            worst_m2_where = format!(
                "{what}: |M|² {vg:e} against |G|²·MadGraph {:e}",
                g.norm_sqr() * mg
            );
        }
    }
    let graph_scale = graph_entries.iter().map(|e| e.0.norm()).fold(0.0, f64::max);
    let (mut worst_graph, mut worst_graph_where) = (0.0f64, String::new());
    for (mg, vg, what) in &graph_entries {
        let dev = (vg - g * mg).norm() / graph_scale;
        if dev > worst_graph {
            worst_graph = dev;
            worst_graph_where = format!("{what}: vibegraph {vg:?} against G·c·AMP {:?}", g * mg);
        }
    }
    if graphs.is_some() {
        assert!(graph_scale > 0.0, "{key}: every banked AMP() vanishes");
    }
    Measured {
        g,
        worst_flow,
        worst_flow_where,
        worst_m2,
        worst_m2_where,
        worst_graph,
        worst_graph_where,
        n_entries: flows.len(),
        n_graph_entries: graph_entries.len(),
    }
}

/// Rows whose comparison is known to disagree with MadGraph, with the finding. A listed
/// row is measured in full and reported; the test fails if it starts agreeing, so the
/// exemption cannot outlive the defect. The exemption covers the measured deviations
/// only: a listed row whose table is missing or whose process stops matching the
/// table's colour and helicity structure fails like any other.
const KNOWN_DISAGREEMENT: &[(&str, &str)] = &[(
    "wpwm_to_epem",
    "a W pair at the anchor beside a final-state lepton line: the neutrino exchange \
     comes out with the wrong sign relative to the photon and Z s-channel, while \
     `e+ e- > w+ w-` (the same graphs crossed) agrees",
)];

fn check(key: &str) {
    let known = KNOWN_DISAGREEMENT.iter().find(|(k, _)| *k == key);
    let m = measure(key);
    println!(
        "{key}: {} flow entries, G = {:?}, worst flow {:.3e}, worst |M|² {:.3e}",
        m.n_entries, m.g, m.worst_flow, m.worst_m2
    );
    if m.n_graph_entries > 0 {
        println!(
            "{key}: {} graph entries, worst graph {:.3e}",
            m.n_graph_entries, m.worst_graph
        );
    }
    let disagreements = m.disagreements(key);
    match (disagreements.is_empty(), known) {
        (true, None) => {}
        (false, None) => panic!("{}", disagreements.join("\n")),
        (true, Some((_, why))) => panic!(
            "{key} is listed as disagreeing ({why}) but now agrees: G = {:?}, worst flow \
             {:.3e}; remove it from KNOWN_DISAGREEMENT",
            m.g, m.worst_flow
        ),
        (false, Some((_, why))) => println!(
            "{key}: informational, disagrees as listed: {why}\n  {}",
            disagreements.join("\n  ")
        ),
    }
}

/// Five gluons: ten of the 25 diagrams join a four-gluon contact to a triple-gluon
/// vertex, the smallest multiplicity where a contact is not the whole diagram.
#[test]
fn gg_to_ggg() {
    check("gg_to_ggg");
}

/// A quark-line anchor with triple-gluon vertices and a four-gluon contact as sources.
#[test]
fn uux_to_ggg() {
    check("uux_to_ggg");
}

/// Quark-gluon scattering with the quark first: a triple-gluon source beside the
/// quark-line anchor.
#[test]
fn ug_to_ug() {
    check("ug_to_ug");
}

/// Colourless four-vector contacts as sources beside a lepton-line anchor.
#[test]
fn ee_to_wpwmz() {
    check("ee_to_wpwmz");
}

/// A colourless four-vector contact as the anchor, beside photon, Z and Higgs exchange.
#[test]
fn wpwm_to_wpwm() {
    check("wpwm_to_wpwm");
}

/// O_HG's gluon-pair–scalar vertex producing the s-channel Higgs current, interfering
/// with QCD.
#[test]
fn ttx_to_gg_chg() {
    check("ttx_to_gg_chg");
}

/// Known disagreement, kept running.
#[test]
fn wpwm_to_epem() {
    check("wpwm_to_epem");
}

/// The tau Yukawa's s-channel Higgs beside Higgsstrahlung and the top Yukawa: the
/// diagrams carrying both Yukawas, one of them through the Higgs self-coupling.
#[test]
fn tata_to_ttxh() {
    check("tata_to_ttxh");
}

/// Two Higgs bosons: final-state top lines built of Yukawa vertices alone, across
/// zero, one and two internal propagators, interfering in one sum.
#[test]
fn tata_to_ttxhh() {
    check("tata_to_ttxhh");
}

/// The all-Yukawa `b` line: t/u-channel exchange across one propagator beside the
/// triple-Higgs annihilation, which must interfere at MadGraph's relative sign.
#[test]
fn bbx_to_hh() {
    check("bbx_to_hh");
}

/// Bhabha scattering through the four-lepton contacts: each contact's two spinor
/// pairings interfere with each other and with photon and Z exchange in both
/// channels, compared per MadGraph graph as well as per helicity.
#[test]
fn ee_to_ee_4f() {
    check("ee_to_ee_4f");
}

/// The top Yukawa as MadGraph's `sm` writes it, `FFS4 = ProjM + ProjP`, with the
/// Higgs emitted off the final-state top pair beside Higgsstrahlung: the control
/// for the bare-`Identity` spelling of the same Yukawa.
#[test]
fn ee_to_ttxh() {
    check("ee_to_ttxh");
}

/// SMEFTsim's bare `Identity(2,1)` top Yukawa rooted as a fermion current on the
/// crossed top line: the four Higgs-off-top graphs carry the crossed-line −1 that the
/// `ProjM + ProjP` spelling of [`ee_to_ttxh`] takes, relative to Higgsstrahlung.
#[test]
fn ee_to_ttxh_identity() {
    check("ee_to_ttxh_identity");
}

/// The toy model's bare `Identity` and `Gamma5` Yukawas emitted off a final-state
/// quark pair and off the initial lepton line: both bilinears take the same
/// crossed-line −1 as fermion currents.
#[test]
fn ll_to_qqxs_toy_yukawa() {
    check("ll_to_qqxs_toy_yukawa");
}

/// The derivative cubic scalar vertex as a scalar current, beside its plain
/// partner: both take the all-scalar −1.
#[test]
fn ll_to_ss_toy_dscalar() {
    check("ll_to_ss_toy_dscalar");
}

/// The derivative cubic scalar vertex as the amplitude, with a scalar on leg 0:
/// both it and its plain partner take the all-scalar −1.
#[test]
fn sl_to_sl_toy_dscalar() {
    check("sl_to_sl_toy_dscalar");
}

/// SMEFTsim's derivative Higgs self-couplings (`SSS2`, `SSS3`) as the s-channel
/// Higgs current off an initial top pair.
#[test]
fn ttx_to_hh_chbox() {
    check("ttx_to_hh_chbox");
}

/// SMEFTsim's derivative Higgs self-couplings (`SSS2`, `SSS3`) as the amplitude of
/// the t-channel Higgs exchange, with a Higgs on leg 0.
#[test]
fn ht_to_ht_chbox() {
    check("ht_to_ht_chbox");
}

/// SMEFTsim's derivative quartic Higgs contacts (`SSSS2`, `SSSS3`) and derivative
/// cubic vertices beside their plain partners, in four-Higgs scattering.
#[test]
fn hh_to_hh_chbox() {
    check("hh_to_hh_chbox");
}

/// The gluon-initiated `t t~ + jet` subprocess: a four-gluon contact beside the top
/// line, whose three colour structures each multiply one of its Lorentz structures.
#[test]
fn gg_to_ttxg() {
    check("gg_to_ttxg");
}

/// Quark-gluon `t t~ + jet` with the quark first: the light-quark line and the top
/// pair joined by a gluon, the radiated gluon on either line or at a triple-gluon
/// vertex.
#[test]
fn ug_to_ttxu() {
    check("ug_to_ttxu");
}

/// The same channel in MadEvent's own leg order, the gluon first.
#[test]
fn gu_to_ttxu() {
    check("gu_to_ttxu");
}

/// The antiquark-gluon channel in MadEvent's leg order.
#[test]
fn gux_to_ttxux() {
    check("gux_to_ttxux");
}

/// Quark annihilation into `t t~ + g`: the gluon off either quark line or off the
/// s-channel gluon.
#[test]
fn uux_to_ttxg() {
    check("uux_to_ttxg");
}

/// The gate is sensitive to which colour structure of the four-gluon contact
/// multiplies which Lorentz structure: each of the five non-identity assignments
/// disagrees with `gg_to_ttxg` far above [`REL_TOL`]. Measured over the five, the
/// weakest worst-flow deviation is 4.4e-2 and the weakest |M|² deviation 2.9e-2, so
/// the 1e-2 floor is a factor of three below the weakest. The identity assignment
/// is the banked agreement at 1e-15.
#[test]
fn gg_to_ttxg_sees_the_contact_colour_assignment() {
    let permutations = [[1, 2, 0], [2, 0, 1], [1, 0, 2], [0, 2, 1], [2, 1, 0]];
    for perm in permutations {
        let m = measure_with("gg_to_ttxg", |model| {
            let gluon = model.particle_id("g").unwrap();
            let contacts: Vec<_> = model
                .vertices
                .values_mut()
                .filter(|v| v.particles.len() == 4 && v.particles.iter().all(|&p| p == gluon))
                .collect();
            assert_eq!(contacts.len(), 1, "the SM has one four-gluon contact");
            let contact = &mut *contacts.into_iter().next().unwrap();
            contact.couplings = std::mem::take(&mut contact.couplings)
                .into_iter()
                .map(|((c, l), id)| ((perm[c], l), id))
                .collect();
        });
        assert!(
            m.worst_flow > 1e-2 && m.worst_m2 > 1e-2,
            "colour permutation {perm:?}: worst flow {:.3e}, worst |M|² {:.3e}",
            m.worst_flow,
            m.worst_m2
        );
    }
}
