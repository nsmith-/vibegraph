//! The triple-gluon vertex's `−1` against MadGraph, factor by factor.
//!
//! [`triple_gluon_sign`] multiplies a diagram by `−1` per triple-gluon vertex. These
//! tests split each diagram's per-flow value into three factors and pin each against
//! MadGraph's own per-graph `AMP()` and `JAMP()`, banked in
//! `validation/madgraph/amplitudes/`:
//!
//! * its colour coefficient, the `(diagram, flow)` weight of the colour basis;
//! * the triple-gluon sign;
//! * everything else (kernel, couplings, propagator phases, the rest of
//!   `fermi_sign`), the value with the other two divided out.
//!
//! On `g g > t t~` the colour coefficients are MadGraph's JAMP coefficients up to one
//! sign shared by every diagram, and the rest is MadGraph's `AMP()` up to one constant
//! on the two quark exchanges and minus that constant on the triple-gluon diagram:
//! the `−1` lies outside the colour factor, and the triple-gluon sign supplies it. On
//! `e+ e- > w+ w-`, the colourless control, the rest is MadGraph's `AMP()` up to one
//! constant on every diagram, the charged triple-vector one included: a vertex the
//! crossing exchanges two slots of carries its `−1` in its kernel (see
//! [`triple_gluon_sign`] for the derivation).
//!
//! What each check cannot see: a sign shared by every diagram of the process (the
//! fitted constants absorb it); a sign that moves between the colour and the rest by
//! the same amount on every diagram; a sign moved between the kernel, a coupling and
//! the other factors of `fermi_sign`, which the third factor lumps together; and any
//! process, helicity or point the two tables do not bank.

use std::path::Path;

use num_complex::Complex;
use num_traits::ToPrimitive;

use super::renumbering::single_diagram_amplitudes_at;
use super::root_diagram::triple_gluon_sign;
use crate::diagrams::diagram::{antiparticle, Ray};
use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use crate::helas::color::coeff::ColorCoeff;
use crate::helas::color::colorize::colorize_process;
use crate::helas::color::factor::ImmutableString;
use crate::helas::color::TensorKind;
use crate::helas::LorentzVector;
use crate::ufo::slha::ParamCard;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{EvaluatedModel, UFOModel};

type C = Complex<f64>;

/// Deviation allowed per entry, relative to the largest MadGraph value compared. Both
/// sides round over three diagrams, near 1e-15; a sign on one diagram is O(1).
const TOL: f64 = 1e-12;

const I: C = C::new(0.0, 1.0);
const ZERO: C = C::new(0.0, 0.0);

/// MadGraph's `JAMP(f) = Σ_k c[f][k] · AMP(k)` for `g g > t t~`, from the
/// `matrix1_orig.f` of its `P1_gg_ttx` export: `JAMP(1) = i·AMP(1) − AMP(2)`,
/// `JAMP(2) = −i·AMP(1) − AMP(3)`. The
/// coefficients carry MadGraph's colour and its fermion factor together, and
/// [`MadGraphRow::read`] checks them against the banked `JAMP()`.
const GG_TO_TTX_JAMP: [[C; 3]; 2] = [
    [I, C::new(-1.0, 0.0), ZERO],
    [C::new(0.0, -1.0), ZERO, C::new(-1.0, 0.0)],
];

/// One banked amplitude row: MadGraph's per-graph values and coefficients,
/// and the per-diagram, per-flow values of this side with each factor separated.
struct MadGraphRow {
    /// `jamp[f][k]`: graph `k`'s coefficient in MadGraph's flow `f`.
    jamp: Vec<Vec<C>>,
    /// Per banked (point, helicity): MadGraph's `AMP(k)`.
    amps: Vec<Vec<C>>,
    /// Per diagram (paired with graph `k = d`), per banked entry, per flow: this side's
    /// value of the diagram alone.
    values: Vec<Vec<Vec<C>>>,
    /// `colour[d][f]`: this side's colour coefficient of diagram `d` in flow `f`.
    colour: Vec<Vec<C>>,
    /// Per diagram: [`triple_gluon_sign`].
    triple_gluon: Vec<f64>,
}

/// A colour coefficient at `Nc = 3`.
fn coeff(c: &ColorCoeff) -> C {
    let v = c.eval_nc(3).to_f64().expect("rational in f64 range");
    if c.imag {
        C::new(0.0, v)
    } else {
        C::new(v, 0.0)
    }
}

/// A basis key in MadGraph's notation, `T(1,2,3,4)`.
fn key_name(key: &ImmutableString) -> String {
    key.iter()
        .map(|(kind, idx)| {
            let idx: Vec<String> = idx.iter().map(i32::to_string).collect();
            let kind = match kind {
                TensorKind::One => "ColorOne".to_owned(),
                other => format!("{other:?}"),
            };
            format!("{kind}({})", idx.join(","))
        })
        .collect()
}

/// One banked `(point, helicity)` entry: MadGraph's per-graph `AMP()` and its `JAMP()`.
struct Entry {
    helicity: Vec<i32>,
    amp: Vec<C>,
    jamps: Vec<C>,
}

fn complex(v: &serde_json::Value) -> C {
    C::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

fn complexes(v: &serde_json::Value) -> Vec<C> {
    v.as_array().unwrap().iter().map(complex).collect()
}

fn read_table(key: &str) -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/amplitudes")
        .join(format!("{key}.json"));
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn param_card(table: &serde_json::Value) -> ParamCard {
    table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .unwrap()
}

/// The table's points that carry a `detail`, the only ones that bank MadGraph's
/// per-graph `AMP()`: each point's momenta and its banked entries.
fn detailed_points(table: &serde_json::Value) -> Vec<(Vec<LorentzVector<f64>>, Vec<Entry>)> {
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
    table["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p.get("detail").is_some())
        .map(|point| {
            let momenta = point["momenta"]
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
            let detail = &point["detail"];
            let entries = detail["helicities"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(e, h)| Entry {
                    helicity: helicities[h.as_u64().unwrap() as usize].clone(),
                    amp: complexes(&detail["amps"][e]),
                    jamps: complexes(&detail["jamps"][e]),
                })
                .collect();
            (momenta, entries)
        })
        .collect()
}

/// Per diagram, the position in the process's colour basis `flows` of each flow the
/// diagram reaches when compiled alone.
fn flow_maps(key: &str, model: &UFOModel, set: &DiagramSet, flows: &[String]) -> Vec<Vec<usize>> {
    set.diagrams
        .iter()
        .enumerate()
        .map(|(d, diagram)| {
            let own = colorize_process(model, std::slice::from_ref(diagram)).unwrap();
            own.elements
                .iter()
                .map(|e| {
                    let name = key_name(&e.structure);
                    flows.iter().position(|k| *k == name).unwrap_or_else(|| {
                        panic!("{key}: diagram {d}'s flow {name} is not in the basis {flows:?}")
                    })
                })
                .collect()
        })
        .collect()
}

/// Checks that `jamp` rebuilds each entry's banked `JAMP()` from its banked `AMP()`.
fn check_transcription(key: &str, jamp: &[Vec<C>], entries: &[Entry]) {
    for entry in entries {
        let scale = entry.amp.iter().map(|a| a.norm()).fold(0.0, f64::max);
        for (f, row) in jamp.iter().enumerate() {
            let sum: C = row.iter().zip(&entry.amp).map(|(c, a)| c * a).sum();
            assert!(
                (sum - entry.jamps[f]).norm() <= TOL * scale.max(f64::MIN_POSITIVE),
                "{key}: the transcribed JAMP coefficients do not rebuild MadGraph's flow {f}"
            );
        }
    }
}

impl MadGraphRow {
    fn read(process: &str, key: &str, jamp: Option<Vec<Vec<C>>>) -> (Self, DiagramSet) {
        let table = read_table(key);
        let n_graphs = table["n_graphs"].as_u64().unwrap() as usize;
        let jamp = jamp.unwrap_or_else(|| vec![complexes(&table["jamp_coefficients"])]);
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(&table));

        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let set = generate_from_proc_card(&card, &model).unwrap().remove(0);
        assert_eq!(
            set.diagrams.len(),
            n_graphs,
            "{key}: one diagram per MadGraph graph"
        );
        let basis = colorize_process(&model, &set.diagrams).unwrap();
        let flows: Vec<String> = basis
            .elements
            .iter()
            .map(|e| key_name(&e.structure))
            .collect();
        let theirs: Vec<String> = table["flow_structures"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            flows, theirs,
            "{key}: the colour basis is MadGraph's, in its order"
        );
        assert_eq!(
            jamp.len(),
            flows.len(),
            "{key}: one coefficient row per flow"
        );

        let mut colour = vec![vec![ZERO; flows.len()]; n_graphs];
        for (f, el) in basis.elements.iter().enumerate() {
            for c in &el.contributions {
                colour[c.diagram][f] += coeff(&c.coeff);
            }
        }

        let maps = flow_maps(key, &model, &set, &flows);

        let points = detailed_points(&table);
        assert!(
            !points.is_empty(),
            "{key}: no banked point carries MadGraph's per-graph AMP()"
        );
        let mut amps = Vec::new();
        let mut values = vec![Vec::new(); n_graphs];
        for (momenta, entries) in &points {
            check_transcription(key, &jamp, entries);
            let helicities: Vec<Vec<i32>> = entries.iter().map(|e| e.helicity.clone()).collect();
            for (d, (diagram, map)) in set.diagrams.iter().zip(&maps).enumerate() {
                let own = single_diagram_amplitudes_at(
                    &set,
                    diagram,
                    &model,
                    &evaluated,
                    momenta,
                    Some(&helicities),
                );
                assert_eq!(
                    own.len(),
                    helicities.len() * map.len(),
                    "{key}: diagram {d} evaluates one value per own flow and helicity"
                );
                for per_flow in own.chunks(map.len()) {
                    let mut full = vec![ZERO; flows.len()];
                    for (v, &f) in per_flow.iter().zip(map) {
                        full[f] = *v;
                    }
                    values[d].push(full);
                }
            }
            amps.extend(entries.iter().map(|e| e.amp.clone()));
        }
        let triple_gluon = set
            .diagrams
            .iter()
            .map(|d| f64::from(triple_gluon_sign(d, &model)))
            .collect();
        (
            MadGraphRow {
                jamp,
                amps,
                values,
                colour,
                triple_gluon,
            },
            set,
        )
    }

    /// The largest `|MadGraph value|` over every (entry, flow, graph).
    fn scale(&self) -> f64 {
        self.amps
            .iter()
            .flat_map(|amp| {
                self.jamp
                    .iter()
                    .flat_map(move |row| row.iter().zip(amp).map(|(c, a)| (c * a).norm()))
            })
            .fold(0.0, f64::max)
    }

    /// Fits `ours = K · theirs` over the given pairs and returns `K` and the largest
    /// `|ours − K · theirs|` relative to [`Self::scale`].
    fn fit(&self, pairs: &[(C, C)]) -> (C, f64) {
        let num: C = pairs.iter().map(|(o, t)| t.conj() * o).sum();
        let den: f64 = pairs.iter().map(|(_, t)| t.norm_sqr()).sum();
        let k = num / den;
        let dev = pairs
            .iter()
            .map(|(o, t)| (o - k * t).norm())
            .fold(0.0, f64::max);
        (k, dev / self.scale())
    }

    /// `(ours, MadGraph)` per (diagram in `diagrams`, entry, flow) for each diagram's
    /// whole value: `ours` against `c[f][d] · AMP(d)`.
    fn totals(&self, diagrams: &[usize]) -> Vec<(C, C)> {
        self.pairs(diagrams, |_, _, v| v, true)
    }

    /// `(ours, MadGraph)` per (diagram in `diagrams`, entry, flow the diagram reaches)
    /// for the rest: `ours / (colour · triple-gluon sign)` against `AMP(d)`.
    fn rest(&self, diagrams: &[usize]) -> Vec<(C, C)> {
        self.pairs(
            diagrams,
            |d, f, v| v / (self.colour[d][f] * self.triple_gluon[d]),
            false,
        )
    }

    fn pairs(
        &self,
        diagrams: &[usize],
        ours: impl Fn(usize, usize, C) -> C,
        whole: bool,
    ) -> Vec<(C, C)> {
        let mut out = Vec::new();
        for &d in diagrams {
            for (e, amp) in self.amps.iter().enumerate() {
                for f in 0..self.jamp.len() {
                    if self.colour[d][f] == ZERO {
                        assert!(
                            self.values[d][e][f] == ZERO && self.jamp[f][d] == ZERO,
                            "diagram {d} reaches flow {f} on one side only"
                        );
                        continue;
                    }
                    let theirs = if whole {
                        self.jamp[f][d] * amp[d]
                    } else {
                        amp[d]
                    };
                    out.push((ours(d, f, self.values[d][e][f]), theirs));
                }
            }
        }
        out
    }
}

/// The `g g > t t~` diagram with the triple-gluon vertex, and the two quark exchanges.
fn gg_to_ttx() -> (MadGraphRow, usize, Vec<usize>) {
    let (row, set) = MadGraphRow::read(
        "g g > t t~",
        "gg_to_ttx",
        Some(GG_TO_TTX_JAMP.iter().map(|r| r.to_vec()).collect()),
    );
    let model = sm_model(SMRestrict::Default);
    let gluon = model.particle_id("g").unwrap();
    let (with, without): (Vec<usize>, Vec<usize>) = (0..set.diagrams.len()).partition(|&d| {
        set.diagrams[d].vertices.iter().any(|v| {
            let def = model.vertex_def(v.interaction);
            def.particles.len() == 3 && def.particles.iter().all(|&p| p == gluon)
        })
    });
    assert_eq!(
        (with.len(), without.len()),
        (1, 2),
        "one s-channel gluon, two quark exchanges"
    );
    (row, with[0], without)
}

/// Each diagram's per-flow value is MadGraph's `c[f][d] · AMP(d)` under one constant,
/// so the triple-gluon sign is on: dropping it flips the triple-gluon diagram against
/// the quark exchanges.
#[test]
fn the_triple_gluon_diagram_agrees_with_madgraph() {
    let (row, s, t) = gg_to_ttx();
    let all: Vec<usize> = std::iter::once(s).chain(t).collect();
    let (g, dev) = row.fit(&row.totals(&all));
    println!("g g > t t~, each diagram against G·c·AMP: worst {dev:.3e}, G = {g:.15}");
    assert!(
        (g.norm() - 1.0).abs() < TOL,
        "g g > t t~: the fitted G = {g} is not a phase (|G| = {})",
        g.norm()
    );
    assert!(
        dev < TOL,
        "a diagram of g g > t t~ disagrees with MadGraph's c·AMP: worst {dev:.3e} (G = {g})"
    );
    assert_eq!(
        row.triple_gluon[s], -1.0,
        "triple_gluon_sign on the triple-gluon diagram {s} of g g > t t~"
    );
}

/// The colour coefficients are MadGraph's JAMP coefficients up to one sign shared by
/// every diagram (MadGraph's fermion factor rides in its coefficients): the
/// triple-gluon diagram's `∓i` stand against the quark exchanges' `±1` as MadGraph's
/// do, so the `−1` is not in the colour factor.
#[test]
fn the_triple_gluon_diagram_has_madgraphs_colour() {
    let (row, s, t) = gg_to_ttx();
    let mut ratios = Vec::new();
    for d in std::iter::once(s).chain(t) {
        for f in 0..row.jamp.len() {
            match (row.colour[d][f] == ZERO, row.jamp[f][d] == ZERO) {
                (true, true) => {}
                (false, false) => ratios.push((d, f, row.colour[d][f] / row.jamp[f][d])),
                _ => panic!("diagram {d} reaches flow {f} on one side only"),
            }
        }
    }
    let phi = ratios[0].2;
    println!("g g > t t~, colour over MadGraph's coefficient: {ratios:?}");
    assert!(
        (phi.norm() - 1.0).abs() < 1e-15,
        "diagram {}, flow {}: colour coefficient over MadGraph's is {phi}, not a phase",
        ratios[0].0,
        ratios[0].1
    );
    for (d, f, r) in &ratios {
        assert!(
            (r - phi).norm() < 1e-15,
            "diagram {d}, flow {f}: colour coefficient over MadGraph's is {r}, against {phi} \
             on diagram {}, flow {}",
            ratios[0].0,
            ratios[0].1
        );
    }
}

/// With its colour coefficient and the triple-gluon sign divided out, each diagram is
/// MadGraph's `AMP()` under one constant on the quark exchanges, and minus that
/// constant on the triple-gluon diagram: the kernel, the couplings and the rest of
/// `fermi_sign` carry `−1` against MadGraph there, and the triple-gluon sign is what
/// cancels it.
#[test]
fn the_triple_gluon_diagram_reads_minus_one_outside_its_colour() {
    let (row, s, t) = gg_to_ttx();
    let (k, dev) = row.fit(&row.rest(&t));
    let (k_s, _) = row.fit(&row.rest(&[s]));
    println!(
        "g g > t t~, colour and triple-gluon sign divided out, against AMP: quark \
         exchanges K = {k:.15} (worst {dev:.3e}), triple-gluon diagram {k_s:.15}"
    );
    assert!(
        dev < TOL,
        "the quark exchanges do not share one constant: {dev:.3e}"
    );
    let flipped: Vec<(C, C)> = row.rest(&[s]).into_iter().map(|(o, a)| (-o, a)).collect();
    let worst = flipped
        .iter()
        .map(|(o, a)| (o - k * a).norm())
        .fold(0.0, f64::max)
        / row.scale();
    assert!(
        worst < TOL,
        "the triple-gluon diagram, colour and triple-gluon sign divided out, is not \
         −K·AMP(1) (K = {k}): worst {worst:.3e}"
    );
}

/// The colourless control. On `e+ e- > w+ w-` the s-channel photon and Z each reach a
/// charged triple-vector vertex, and every diagram with its colour divided out is
/// MadGraph's `AMP()` under one constant: no triple-vector vertex takes a sign there,
/// because the crossing puts each final-state W in its antiparticle's slot (checked
/// here), which exchanges the two W slots against MadGraph's reading and makes the
/// kernel `−1`.
#[test]
fn a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel() {
    let (row, set) = MadGraphRow::read("e+ e- > w+ w-", "ee_to_wpwm", None);
    let model = sm_model(SMRestrict::Default);
    let all: Vec<usize> = (0..set.diagrams.len()).collect();
    let (g, total) = row.fit(&row.totals(&all));
    println!("e+ e- > w+ w-, each diagram against G·c·AMP: worst {total:.3e}, G = {g:.15}");
    assert!(
        total < TOL,
        "a diagram of e+ e- > w+ w- disagrees with MadGraph's c·AMP: worst {total:.3e} (G = {g})"
    );
    assert!(
        row.triple_gluon.iter().all(|&s| s == 1.0),
        "triple_gluon_sign fires on e+ e- > w+ w-: {:?}",
        row.triple_gluon
    );
    let (k, dev) = row.fit(&row.rest(&all));
    println!("e+ e- > w+ w-, colour divided out, against AMP: K = {k:.15}, worst {dev:.3e}");
    assert!(
        dev < TOL,
        "a diagram of e+ e- > w+ w- disagrees with MadGraph's AMP: worst {dev:.3e} (K = {k})"
    );

    let mut charged_vertices = 0;
    for diagram in &set.diagrams {
        for v in &diagram.vertices {
            let def = model.vertex_def(v.interaction);
            if !def.particles.iter().all(|&p| model.particle(p).spin == 3) {
                continue;
            }
            charged_vertices += 1;
            for (slot, ray) in v.rays.iter().enumerate() {
                let Ray::Leg(l) = ray else { continue };
                // The leg's particle as the process names it, outgoing for a final leg,
                // which is the slot MadGraph puts it in.
                let named = set
                    .particles_in
                    .iter()
                    .chain(&set.particles_out)
                    .nth(l.0)
                    .unwrap();
                let named = model.particle_id(named).unwrap();
                let incoming_identity = if l.0 < set.particles_in.len() {
                    named
                } else {
                    antiparticle(&model, named)
                };
                assert_eq!(
                    def.particles[slot], incoming_identity,
                    "leg {} stands in slot {slot} of {}",
                    l.0, def.name
                );
                assert_ne!(
                    def.particles[slot], named,
                    "the final-state W stands in its own slot of {}, as MadGraph reads it",
                    def.name
                );
            }
        }
    }
    assert_eq!(
        charged_vertices, 2,
        "the photon and Z s-channels each reach one"
    );
}
