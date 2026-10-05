//! MLM's per-event reweighting of the matrix element: `reweight.f`'s `rewgt`
//! under `ickkw = 1` (`reweight.f:1333-1824`).
//!
//! A matched sample evaluates its matrix element at one renormalisation scale
//! and with its parton densities at one lowered factorisation scale, and then
//! corrects both toward what a parton shower would have used, vertex by vertex
//! along the clustering `setclscales` found:
//!
//! * **α_s.** Every clustering except the `2 → 2` core that produces a parton
//!   multiplies the weight by `αs(alpsfact·√pt2ijcl(n)) / αs(μR)`. An
//!   initial-state vertex counts when every line through it is a parton line
//!   (`goodjet`); a final-state one when it is a QCD vertex with a parton on it
//!   and at least one of its daughters is a parton line.
//! * **Parton densities** (`pdfwgt`). Each beam's line is walked up through its
//!   initial-state clusterings. The first carries no ratio, because the matrix
//!   element's density was already read there; every later one up to `jlast`
//!   at a rising scale multiplies by `f(x·z, q_now) / f(x·z, q_prev)`, with `f`
//!   the density of the line *entering* the vertex and the momentum fraction
//!   already reduced by the vertex's own `z`.
//!
//! [`rewgt`] returns every factor it multiplied in, with what decided it,
//! beside the product: a per-event product alone cannot tell a missing vertex
//! from a compensating density ratio.
//!
//! # What the weight reads besides the clustering
//!
//! MadEvent draws one flavour combination (`IPSEL`) of the subprocess per
//! point, `∝` its luminosity, and computes `rewgt` for it alone. The weight
//! reads that combination's incoming flavours and the flavours of the
//! final-state legs that are jets (`isjet`); every other line keeps the code
//! the scale walk left in the shared `ipdgcl` table. Summing every combination
//! with its own factor, as a caller of this function does, has the same
//! expectation as that draw.
//!
//! That table is carried across events, and this is where the port and
//! MadEvent part. A final-state leg that is not a jet keeps whatever code the
//! table last held for it — the subprocess's first combination until some
//! event of the run labels it a jet — and an internal line keeps whatever flavour
//! the last `ipartupdate` pass handed it. [`RewgtHistory::pdg`] is the table
//! as the scale walk of *this* event leaves it, which is MadEvent's state on
//! the first event of a run. The two agree whenever the jet-ness of every leg
//! and the flavour transmission of every vertex are the same across a
//! subprocess's combinations; where they are not, the difference is a
//! history-dependent defect of the reference, not a choice here.
//!
//! # `fake_id`
//!
//! `rewgt` skips a vertex whose mother carries `fake_id`, the code MadGraph's
//! exporter gives the propagator it invents when it splits a vertex of more
//! than three lines. The channel forests here never carry one: a process whose
//! every diagram would need the split is refused
//! ([`ConfigError::IrreducibleHigherVertex`](super::configs::ConfigError)), and
//! otherwise the higher-vertex diagrams are dropped, as MadGraph drops them.
//! No line can therefore carry `fake_id`, and the test is not made.

use super::graph::ColorTable;
use super::setclscales::ipartupdate;

/// Below this `pt2ijcl`, in GeV², a vertex that would be reweighted by `αs`
/// kills the event instead (`reweight.f:1597`).
pub const ALPHA_S_KILL_Q2: f64 = 4.0;

/// Below this density at the previous scale, a density ratio kills the event
/// instead (`reweight.f:1692`). It is a threshold on `f`, not on `x·f`.
pub const PDF_DENOMINATOR_FLOOR: f64 = 1e-10;

/// The run-card constants the reweighting branches on.
///
/// `maxjetflavor` is the colour table's own, which is what decides `isjet` in
/// the scale walk too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewgtSettings {
    pub ickkw: i64,
    pub alpsfact: f64,
    pub asrwgtflavor: i64,
    pub pdfwgt: bool,
}

/// One merge of the clustering, as the reweighting reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewgtMerge {
    /// `idacl(n, 1:2)`, in the clustering's order.
    pub daughters: [u32; 2],
    /// `imocl(n)`.
    pub mother: u32,
    /// `zcl(n)`.
    pub z: f64,
}

/// What `rewgt` reads of one event's clustering: the state MadEvent's second
/// `setclscales` call leaves in its common blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct RewgtHistory {
    pub n_external: usize,
    /// `nexternal − 2` merges; the last is the core.
    pub merges: Vec<RewgtMerge>,
    /// `pt2ijcl`, after every rewrite of the second call.
    pub pt2: Vec<f64>,
    /// `jlast` per beam, from `1`, `0` where the beam never split.
    pub jlast: [usize; 2],
    /// `iqjets` per external leg, leg `1` first.
    pub iqjets: Vec<i64>,
    /// `ipdgcl` by leg set (`2^nexternal` entries): every line's code as the
    /// scale walk left it.
    pub pdg: Vec<i64>,
    /// `q2bck`: the central factorisation scale per beam, GeV².
    pub q2bck: [f64; 2],
    /// `μR`, whose `αs` the matrix element was evaluated at (`asref`).
    pub mu_r: f64,
    /// The external momenta the clustering read, beams first, which
    /// `ipartupdate` compares transverse momenta of.
    pub momenta: Vec<[f64; 4]>,
}

/// How a clustering vertex fared in the `αs` reweighting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexClass {
    /// The `2 → 2` core, never reweighted.
    Core,
    /// An initial-state vertex on parton lines throughout.
    Isr,
    /// A final-state QCD vertex with a parton daughter line.
    Fsr,
    /// An initial-state vertex with a line through it that is not a parton line.
    IsrNotParton,
    /// A final-state vertex that is not a QCD vertex with a parton on it.
    FsrNotPartonVertex,
    /// A final-state parton vertex neither of whose daughters is a parton line.
    FsrNoPartonDaughter,
}

impl VertexClass {
    /// Whether the vertex carries an `αs` ratio.
    pub fn reweighted(self) -> bool {
        matches!(self, VertexClass::Isr | VertexClass::Fsr)
    }
}

/// One `αs` ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaSRatio {
    /// `pt2ijcl(n)`, GeV².
    pub q2: f64,
    /// `αs(alpsfact·√q2)`.
    pub numerator: f64,
    /// `αs(μR)`.
    pub denominator: f64,
    pub ratio: f64,
}

/// One clustering vertex's decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexDecision {
    /// The merge index, from `1`.
    pub n: usize,
    pub mother: u32,
    pub daughters: [u32; 2],
    /// The codes of mother and daughters once this vertex's `ipartupdate` ran.
    pub pdg: [i64; 3],
    /// `ipart(1, mother)`: a beam (`1`, `2`) on an initial-state vertex.
    pub ipart: usize,
    pub class: VertexClass,
    /// The ratio, on a reweighted vertex that did not kill the event.
    pub alpha_s: Option<AlphaSRatio>,
}

/// What one initial-state vertex did on its beam's density chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PdfOutcome {
    /// The beam's first vertex: no ratio, `q_now` becomes the chain's scale.
    First,
    /// A ratio was taken.
    Ratio {
        /// `f(x, q_now)`.
        numerator: f64,
        /// `f(x, q_prev)`.
        denominator: f64,
        ratio: f64,
    },
    /// `q_now` did not rise above `q_prev`: the chain keeps `q_prev`.
    NoRise,
    /// A scale past `jlast` that did not fall: no ratio, and no scale is handed
    /// on, so every later vertex on the beam lands here too with a NaN `q2_prev`.
    PastLast,
}

/// One entry of a beam's density chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfStep {
    /// The merge index, from `1`.
    pub n: usize,
    /// The beam line entering the vertex.
    pub line: u32,
    /// Its code, the flavour the density is read for.
    pub flavour: i64,
    /// The momentum fraction after this vertex's `z`.
    pub x: f64,
    /// The chain's scale on the line entering, GeV²; `0` before the first vertex.
    pub q2_prev: f64,
    /// `min(pt2ijcl(n), q2bck)`, or `q2bck` exactly at `jlast`, GeV².
    pub q2_now: f64,
    pub outcome: PdfOutcome,
}

/// Why the event's weight is zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RewgtKill {
    /// A reweighted vertex at `pt2ijcl ≤ 4 GeV²`.
    AlphaSScale { n: usize, q2: f64 },
    /// A density ratio whose denominator fell below `1e-10`.
    PdfDenominator {
        n: usize,
        beam: usize,
        flavour: i64,
        x: f64,
        q2: f64,
        value: f64,
    },
}

/// The reweighting factor with every factor it multiplied in.
#[derive(Clone, Debug, PartialEq)]
pub struct Rewgt {
    /// The product, `0` where the event was killed.
    pub weight: f64,
    /// One entry per merge reached, in order.
    pub vertices: Vec<VertexDecision>,
    /// Per beam, one entry per initial-state vertex on its line.
    pub pdf_chain: [Vec<PdfStep>; 2],
    pub kill: Option<RewgtKill>,
}

impl Rewgt {
    /// No reweighting: `ickkw ≤ 0`.
    pub fn unit() -> Self {
        Rewgt {
            weight: 1.0,
            vertices: Vec::new(),
            pdf_chain: [Vec::new(), Vec::new()],
            kill: None,
        }
    }

    /// The listed factors multiplied in the order `rewgt` multiplies them, `0`
    /// where a kill is recorded. Equal to [`Rewgt::weight`] bit for bit.
    pub fn product_of_factors(&self) -> f64 {
        if self.kill.is_some() {
            return 0.0;
        }
        let mut product = 1.0;
        let mut chain =
            [self.pdf_chain[0].iter(), self.pdf_chain[1].iter()].map(|steps| steps.peekable());
        for vertex in &self.vertices {
            if let Some(a) = vertex.alpha_s {
                product *= a.ratio;
            }
            // The density ratios of a merge follow its `αs` ratio; within a
            // merge, beam order is daughter order, which the chains record.
            let mut steps: Vec<&PdfStep> = Vec::new();
            for beam in &mut chain {
                while let Some(step) = beam.next_if(|s| s.n == vertex.n) {
                    steps.push(step);
                }
            }
            steps.sort_by_key(|s| {
                vertex
                    .daughters
                    .iter()
                    .position(|&d| d == s.line)
                    .unwrap_or(usize::MAX)
            });
            for step in steps {
                if let PdfOutcome::Ratio { ratio, .. } = step.outcome {
                    product *= ratio;
                }
            }
        }
        product
    }
}

/// The reweighting refused the event where `reweight.f` stops the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RewgtError {
    #[error(
        "the clustering's merge {n} joins lines {daughters:?} with a colour structure \
         reweight.f's ipartupdate does not name, where MadEvent stops the run"
    )]
    UnnamedColourStructure { n: usize, daughters: [u32; 2] },
    #[error("the flavour list has {got} entries for {want} external legs")]
    FlavourCount { got: usize, want: usize },
}

/// `isparton`: a gluon, or a quark up to `max(asrwgtflavor, maxjetflavor)`.
/// Like `isjet` it asks nothing about colour, so an unassigned line (`0`) is a
/// parton.
pub fn is_parton(colors: &ColorTable, asrwgtflavor: i64, pdg: i64) -> bool {
    let magnitude = pdg.abs();
    magnitude <= asrwgtflavor.max(colors.maxjetflavor()) || magnitude == 21
}

/// `ispartonvx`: a QCD vertex with a parton on it.
#[allow(clippy::too_many_arguments)]
fn is_parton_vertex(
    colors: &ColorTable,
    asrwgtflavor: i64,
    mother: usize,
    d1: usize,
    d2: usize,
    pdg: &[i64],
    ipart: &[[usize; 2]],
    islast: bool,
) -> bool {
    let (pdgm, pdg1, pdg2) = (pdg[mother], pdg[d1], pdg[d2]);
    if !colors.is_qcd(pdgm) || !colors.is_qcd(pdg1) || !colors.is_qcd(pdg2) {
        return false;
    }
    let beam1 = (1..=2).contains(&ipart[d1][0]);
    let beam2 = (1..=2).contains(&ipart[d2][0]);
    if beam1 || beam2 {
        return (!islast && beam2 && is_parton(colors, asrwgtflavor, pdg1))
            || (beam1 && is_parton(colors, asrwgtflavor, pdg2));
    }
    is_parton(colors, asrwgtflavor, pdg1) || is_parton(colors, asrwgtflavor, pdg2)
}

/// `rewgt` for one flavour combination of one clustered event.
///
/// * `flavours` — the combination's external codes (`idup(:, IPSEL)`), in the
///   clustering's leg order, beams first.
/// * `x` — the momentum fraction of each clustering beam (`xbk(ib(j))`).
/// * `alpha_s` — `αs(Q)` at a scale in GeV, the running the matrix element's
///   coupling came from.
/// * `xfx` — `x·f(x, Q²)` of a flavour in the beam hadron, the densities the
///   matrix element reads.
// `beam` and `leg` are physical indices into several parallel per-beam and
// per-leg arrays at once, so an iterator over one of them would keep the index.
#[allow(clippy::needless_range_loop)]
pub fn rewgt(
    history: &RewgtHistory,
    colors: &ColorTable,
    settings: &RewgtSettings,
    flavours: &[i64],
    x: [f64; 2],
    alpha_s: impl Fn(f64) -> f64,
    xfx: impl Fn(i64, f64, f64) -> f64,
) -> Result<Rewgt, RewgtError> {
    if settings.ickkw <= 0 {
        return Ok(Rewgt::unit());
    }
    let n = history.n_external;
    if flavours.len() != n {
        return Err(RewgtError::FlavourCount {
            got: flavours.len(),
            want: n,
        });
    }
    let core = n - 2;
    let parton = |pdg: i64| is_parton(colors, settings.asrwgtflavor, pdg);

    let mut pdg = history.pdg.clone();
    let n_masks = pdg.len();
    // The combination's incoming codes, and its jet codes on the final state
    // (`reweight.f:1531-1537`); every other line keeps the walk's code.
    for leg in 1..=n {
        let mask = 1usize << (leg - 1);
        if leg <= 2 || colors.is_jet(flavours[leg - 1]) {
            pdg[mask] = flavours[leg - 1];
        }
    }
    let mut goodjet = vec![false; n_masks];
    let mut ipart = vec![[0usize; 2]; n_masks];
    // `pt2pdf`: the density chain's scale per line, zero on every external leg.
    let mut pt2pdf = vec![0.0f64; n_masks];
    for leg in 1..=n {
        let mask = 1usize << (leg - 1);
        let code = pdg[mask];
        goodjet[mask] = if leg <= 2 {
            parton(code)
        } else {
            history.iqjets[leg - 1] > 0 || (parton(code) && !colors.is_jet(code))
        };
        ipart[mask] = [leg, 0];
    }

    let mut out = Rewgt::unit();
    let mut ibeam = [1u32, 2u32];
    let mut xnow = x;
    let asref = alpha_s(history.mu_r);

    for (index, merge) in history.merges.iter().enumerate() {
        let step = index + 1;
        let mother = merge.mother as usize;
        let [d1, d2] = merge.daughters.map(|d| d as usize);
        if !ipartupdate(
            colors,
            &history.momenta,
            merge.mother,
            merge.daughters,
            &mut pdg,
            &mut ipart,
        ) {
            return Err(RewgtError::UnnamedColourStructure {
                n: step,
                daughters: merge.daughters,
            });
        }

        let mut decision = VertexDecision {
            n: step,
            mother: merge.mother,
            daughters: merge.daughters,
            pdg: [pdg[mother], pdg[d1], pdg[d2]],
            ipart: ipart[mother][0],
            class: VertexClass::Core,
            alpha_s: None,
        };
        if step < core {
            goodjet[mother] = parton(pdg[mother]) && goodjet[d1] && goodjet[d2];
            // An unset provenance (`0`) reads as a beam, as the reference's
            // `ipart(1, imo) .le. 2` does.
            decision.class = if ipart[mother][0] <= 2 {
                if goodjet[mother] {
                    VertexClass::Isr
                } else {
                    VertexClass::IsrNotParton
                }
            } else if !is_parton_vertex(
                colors,
                settings.asrwgtflavor,
                mother,
                d1,
                d2,
                &pdg,
                &ipart,
                false,
            ) {
                VertexClass::FsrNotPartonVertex
            } else if goodjet[d1] || goodjet[d2] {
                VertexClass::Fsr
            } else {
                VertexClass::FsrNoPartonDaughter
            };
            if decision.class.reweighted() {
                let q2 = history.pt2[index];
                if q2 <= ALPHA_S_KILL_Q2 {
                    out.vertices.push(decision);
                    out.kill = Some(RewgtKill::AlphaSScale { n: step, q2 });
                    out.weight = 0.0;
                    return Ok(out);
                }
                let numerator = alpha_s(settings.alpsfact * q2.sqrt());
                let ratio = numerator / asref;
                out.weight *= ratio;
                decision.alpha_s = Some(AlphaSRatio {
                    q2,
                    numerator,
                    denominator: asref,
                    ratio,
                });
            }
        }
        out.vertices.push(decision);

        if !settings.pdfwgt {
            continue;
        }
        for daughter in merge.daughters {
            let d = daughter as usize;
            if !colors.is_qcd(pdg[d]) {
                continue;
            }
            for beam in 0..2 {
                if !(parton(pdg[d]) && daughter == ibeam[beam]) {
                    continue;
                }
                ibeam[beam] = merge.mother;
                if merge.z > 0.0 && merge.z < 1.0 {
                    xnow[beam] *= merge.z;
                }
                let q2_now = if step == history.jlast[beam] {
                    history.q2bck[beam]
                } else {
                    history.pt2[index].min(history.q2bck[beam])
                };
                let q2_prev = pt2pdf[d];
                let flavour = pdg[d];
                let outcome = if q2_prev == 0.0 {
                    pt2pdf[mother] = q2_now;
                    PdfOutcome::First
                } else if q2_prev < q2_now && step <= history.jlast[beam] {
                    let density = |q2: f64| xfx(flavour, xnow[beam], q2) / xnow[beam];
                    let numerator = density(q2_now);
                    let denominator = density(q2_prev);
                    if denominator < PDF_DENOMINATOR_FLOOR {
                        out.pdf_chain[beam].push(PdfStep {
                            n: step,
                            line: daughter,
                            flavour,
                            x: xnow[beam],
                            q2_prev,
                            q2_now,
                            outcome: PdfOutcome::Ratio {
                                numerator,
                                denominator,
                                ratio: numerator / denominator,
                            },
                        });
                        out.kill = Some(RewgtKill::PdfDenominator {
                            n: step,
                            beam,
                            flavour,
                            x: xnow[beam],
                            q2: q2_prev,
                            value: denominator,
                        });
                        out.weight = 0.0;
                        return Ok(out);
                    }
                    let ratio = numerator / denominator;
                    out.weight *= ratio;
                    pt2pdf[mother] = q2_now;
                    PdfOutcome::Ratio {
                        numerator,
                        denominator,
                        ratio,
                    }
                } else if q2_prev >= q2_now {
                    pt2pdf[mother] = q2_prev;
                    PdfOutcome::NoRise
                } else {
                    // `reweight.f` leaves the mother's scale unset here. Every
                    // later vertex on this beam is past `jlast` too, so it can
                    // take no ratio whatever it reads; NaN keeps it on this arm.
                    pt2pdf[mother] = f64::NAN;
                    PdfOutcome::PastLast
                };
                out.pdf_chain[beam].push(PdfStep {
                    n: step,
                    line: daughter,
                    flavour,
                    x: xnow[beam],
                    q2_prev,
                    q2_now,
                    outcome,
                });
                break;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::alphas::{NLoop, RunningAlphaS};

    const U: i64 = 2;
    const UB: i64 = -2;
    const G: i64 = 21;
    const Z: i64 = 23;

    fn colors(maxjetflavor: i64) -> ColorTable {
        let mut table = vec![(21, 8), (11, 1), (-11, 1), (22, 1), (23, 1)];
        for q in 1..=6 {
            table.push((q, 3));
            table.push((-q, -3));
        }
        ColorTable::new(table, maxjetflavor)
    }

    fn settings() -> RewgtSettings {
        RewgtSettings {
            ickkw: 1,
            alpsfact: 1.0,
            asrwgtflavor: 5,
            pdfwgt: true,
        }
    }

    /// A toy running coupling, monotone and far from constant, so a ratio taken
    /// at the wrong scale is visible.
    fn toy_alpha_s(q: f64) -> f64 {
        1.0 / q.ln()
    }

    /// A toy density whose `Q²`-evolution depends on both `x` and the flavour, so
    /// a ratio read at the wrong momentum fraction or for the wrong flavour
    /// differs from the right one. Every code gets its own exponent.
    fn toy_xfx(pdg: i64, x: f64, q2: f64) -> f64 {
        let b = 0.01 * (pdg + 30) as f64;
        x.powf(b * q2.ln()) * (1.0 - x).powi(3)
    }

    fn toy_f(pdg: i64, x: f64, q2: f64) -> f64 {
        toy_xfx(pdg, x, q2) / x
    }

    /// Boost-free momenta for `n` legs; only their transverse momenta are read,
    /// by `ipartupdate`, to order a final-state splitting's daughters.
    fn momenta(pts: &[f64]) -> Vec<[f64; 4]> {
        let mut p = vec![[500.0, 0.0, 0.0, 500.0], [500.0, 0.0, 0.0, -500.0]];
        p.extend(pts.iter().map(|&pt| [pt.max(1.0), pt, 0.0, 0.0]));
        p
    }

    fn merge(daughters: [u32; 2], mother: u32, z: f64) -> RewgtMerge {
        RewgtMerge {
            daughters,
            mother,
            z,
        }
    }

    // One argument per field of the history a test sets by hand.
    #[allow(clippy::too_many_arguments)]
    fn history(
        n: usize,
        merges: Vec<RewgtMerge>,
        pt2: Vec<f64>,
        jlast: [usize; 2],
        iqjets: Vec<i64>,
        lines: &[(u32, i64)],
        q2bck: [f64; 2],
        mu_r: f64,
        pts: &[f64],
    ) -> RewgtHistory {
        let mut pdg = vec![0; 1 << n];
        for &(mask, code) in lines {
            pdg[mask as usize] = code;
        }
        RewgtHistory {
            n_external: n,
            merges,
            pt2,
            jlast,
            iqjets,
            pdg,
            q2bck,
            mu_r,
            momenta: momenta(pts),
        }
    }

    const X: [f64; 2] = [0.2, 0.05];

    /// `u g → e+ e- u`, clustered as the scale walk leaves it: the lepton pair
    /// into a `Z` (FS, no colour), the outgoing quark with the incoming gluon into
    /// the spacelike `ū` line (ISR), then the core. The walk has already handed
    /// the spacelike line `−2` (`ipartupdate`'s transmission from the gluon).
    fn ug_to_eeu(core_pt2: f64, isr_pt2: f64) -> RewgtHistory {
        history(
            5,
            vec![
                merge([4, 8], 12, 0.0),
                merge([2, 16], 18, 0.4),
                merge([1, 18], 12, 1.0),
            ],
            vec![8000.0, isr_pt2, core_pt2],
            [3, 3],
            vec![0, 0, 0, 0, 1],
            &[
                (1, U),
                (2, G),
                (4, -11),
                (8, 11),
                (16, U),
                (12, Z),
                (18, UB),
            ],
            [8100.0, 8100.0],
            60.0,
            &[40.0, 40.0, 20.0],
        )
    }

    fn check_product(r: &Rewgt) {
        assert_eq!(r.product_of_factors().to_bits(), r.weight.to_bits());
    }

    fn run(h: &RewgtHistory, colors: &ColorTable, s: &RewgtSettings, flavours: &[i64]) -> Rewgt {
        let r = rewgt(h, colors, s, flavours, X, toy_alpha_s, toy_xfx).expect("reweights");
        check_product(&r);
        r
    }

    #[test]
    fn without_matching_the_weight_is_one_and_lists_nothing() {
        let h = ug_to_eeu(8100.0, 400.0);
        let s = RewgtSettings {
            ickkw: 0,
            ..settings()
        };
        let r = run(&h, &colors(4), &s, &[U, G, -11, 11, U]);
        assert_eq!(r, Rewgt::unit());
    }

    /// One ISR vertex: an `αs` ratio at its `pt2`, no density ratio at either
    /// beam's first vertex, and one at the core on the gluon beam, for the line
    /// entering it (`ū`), at `x₂·z` and between the first vertex's scale and
    /// `q2bck`. The core is never reweighted by `αs`, and the core's scale
    /// below `q2bck` does not lower the `jlast` scale, which is `q2bck` exactly.
    #[test]
    fn a_single_isr_vertex_carries_one_alpha_s_and_one_density_ratio() {
        let h = ug_to_eeu(5000.0, 400.0);
        let r = run(&h, &colors(4), &settings(), &[U, G, -11, 11, U]);
        assert_eq!(r.kill, None);
        let classes: Vec<VertexClass> = r.vertices.iter().map(|v| v.class).collect();
        assert_eq!(
            classes,
            [
                VertexClass::FsrNotPartonVertex,
                VertexClass::Isr,
                VertexClass::Core
            ]
        );
        let a = r.vertices[1].alpha_s.expect("the ISR vertex is reweighted");
        assert_eq!(a.q2, 400.0);
        assert_eq!(a.numerator, toy_alpha_s(20.0));
        assert_eq!(a.denominator, toy_alpha_s(60.0));
        assert!(r.vertices[0].alpha_s.is_none() && r.vertices[2].alpha_s.is_none());

        assert_eq!(r.pdf_chain[0].len(), 1);
        let first = r.pdf_chain[0][0];
        assert_eq!((first.n, first.flavour, first.x), (3, U, X[0]));
        assert_eq!(first.q2_now, 8100.0);
        assert_eq!(first.outcome, PdfOutcome::First);

        let chain = &r.pdf_chain[1];
        assert_eq!(chain.len(), 2);
        assert_eq!((chain[0].n, chain[0].flavour), (2, G));
        assert_eq!(chain[0].outcome, PdfOutcome::First);
        assert_eq!(chain[0].q2_now, 400.0);
        let x = X[1] * 0.4;
        assert_eq!(chain[0].x, x);
        assert_eq!((chain[1].n, chain[1].flavour, chain[1].x), (3, UB, x));
        assert_eq!((chain[1].q2_prev, chain[1].q2_now), (400.0, 8100.0));
        let ratio = toy_f(UB, x, 8100.0) / toy_f(UB, x, 400.0);
        assert_eq!(
            chain[1].outcome,
            PdfOutcome::Ratio {
                numerator: toy_f(UB, x, 8100.0),
                denominator: toy_f(UB, x, 400.0),
                ratio,
            }
        );
        let expected = toy_alpha_s(20.0) / toy_alpha_s(60.0) * ratio;
        assert!((r.weight - expected).abs() <= 1e-15 * expected);

        // The ratio at the unreduced momentum fraction, or for the flavour the
        // line leaves the vertex with, is a different number.
        let unreduced = toy_f(UB, X[1], 8100.0) / toy_f(UB, X[1], 400.0);
        let outgoing_flavour = toy_f(G, x, 8100.0) / toy_f(G, x, 400.0);
        assert!((unreduced / ratio - 1.0).abs() > 1e-2);
        assert!((outgoing_flavour / ratio - 1.0).abs() > 1e-2);
    }

    /// The core's own scale is never tested against the kill floor: it is not
    /// reweighted.
    #[test]
    fn the_core_is_never_reweighted_whatever_its_scale() {
        let h = ug_to_eeu(1.0, 400.0);
        let r = run(&h, &colors(4), &settings(), &[U, G, -11, 11, U]);
        assert_eq!(r.kill, None);
        assert_eq!(r.vertices[2].class, VertexClass::Core);
        assert!(r.vertices[2].alpha_s.is_none());
    }

    /// `alpsfact` multiplies the scale the numerator is read at, and nothing
    /// else: the denominator stays `αs(μR)` and the density ratios do not move.
    #[test]
    fn alpsfact_scales_the_numerator_scale_only() {
        let h = ug_to_eeu(8100.0, 400.0);
        let one = run(&h, &colors(4), &settings(), &[U, G, -11, 11, U]);
        let two = run(
            &h,
            &colors(4),
            &RewgtSettings {
                alpsfact: 2.0,
                ..settings()
            },
            &[U, G, -11, 11, U],
        );
        let a = two.vertices[1].alpha_s.expect("reweighted");
        assert_eq!(a.q2, 400.0);
        assert_eq!(a.numerator, toy_alpha_s(40.0));
        assert_eq!(a.denominator, toy_alpha_s(60.0));
        assert_eq!(one.pdf_chain, two.pdf_chain);
        let expected = one.weight * toy_alpha_s(40.0) / toy_alpha_s(20.0);
        assert!((two.weight - expected).abs() <= 1e-14 * expected);
    }

    /// Without `pdfwgt` no chain is walked; the `αs` ratios are unchanged.
    #[test]
    fn without_pdfwgt_only_the_alpha_s_ratios_remain() {
        let h = ug_to_eeu(8100.0, 400.0);
        let r = run(
            &h,
            &colors(4),
            &RewgtSettings {
                pdfwgt: false,
                ..settings()
            },
            &[U, G, -11, 11, U],
        );
        assert!(r.pdf_chain.iter().all(Vec::is_empty));
        assert_eq!(r.weight, toy_alpha_s(20.0) / toy_alpha_s(60.0));
    }

    /// A reweighted vertex at `pt2 ≤ 4 GeV²` kills the event, and at the
    /// boundary itself too.
    #[test]
    fn a_reweighted_vertex_at_or_below_four_gev2_kills_the_event() {
        for pt2 in [3.9, 4.0] {
            let h = ug_to_eeu(8100.0, pt2);
            let r = run(&h, &colors(4), &settings(), &[U, G, -11, 11, U]);
            assert_eq!(r.weight, 0.0);
            assert_eq!(r.kill, Some(RewgtKill::AlphaSScale { n: 2, q2: pt2 }));
            assert_eq!(r.vertices.len(), 2);
        }
        let h = ug_to_eeu(8100.0, 4.000001);
        assert!(run(&h, &colors(4), &settings(), &[U, G, -11, 11, U]).weight > 0.0);
    }

    /// A density below `1e-10` at the previous scale kills the event. The floor
    /// is on `f`, not on `x·f`: a value of `x·f` above it whose `f` is below it
    /// still kills.
    #[test]
    fn a_vanishing_density_denominator_kills_the_event() {
        let h = ug_to_eeu(8100.0, 400.0);
        let x = X[1] * 0.4;
        // `x·f = 1.5e-12` at the lower scale, so `f = 7.5e-11`.
        let xfx = |pdg: i64, x: f64, q2: f64| {
            if q2 == 400.0 {
                1.5e-12
            } else {
                toy_xfx(pdg, x, q2)
            }
        };
        let r = rewgt(
            &h,
            &colors(4),
            &settings(),
            &[U, G, -11, 11, U],
            X,
            toy_alpha_s,
            xfx,
        )
        .expect("reweights");
        assert_eq!(r.weight, 0.0);
        check_product(&r);
        match r.kill {
            Some(RewgtKill::PdfDenominator {
                n,
                beam,
                flavour,
                x: at,
                q2,
                value,
            }) => {
                assert_eq!((n, beam, flavour, q2), (3, 1, UB, 400.0));
                assert_eq!(at, x);
                assert!((value - 1.5e-12 / x).abs() < 1e-24);
            }
            other => panic!("expected a density kill, got {other:?}"),
        }
        // An `x·f` of `1e-10` is an `f` above the floor and does not kill.
        let xfx = |pdg: i64, x: f64, q2: f64| {
            if q2 == 400.0 {
                1e-10
            } else {
                toy_xfx(pdg, x, q2)
            }
        };
        let r = rewgt(
            &h,
            &colors(4),
            &settings(),
            &[U, G, -11, 11, U],
            X,
            toy_alpha_s,
            xfx,
        )
        .expect("reweights");
        assert_eq!(r.kill, None);
    }

    /// The flavour combination's own codes replace the walk's on the beams and
    /// on the final-state jets, and the spacelike line takes its flavour from
    /// them: a `c g` member of a group whose walk ran on `u g` reads the `c̄`
    /// density.
    #[test]
    fn the_combination_s_codes_replace_the_walk_s_on_beams_and_jets() {
        let h = ug_to_eeu(8100.0, 400.0);
        let r = run(&h, &colors(4), &settings(), &[4, G, -11, 11, 4]);
        assert_eq!(r.pdf_chain[0][0].flavour, 4);
        assert_eq!(r.pdf_chain[1][1].flavour, -4);
        assert_eq!(r.vertices[1].pdg, [-4, G, 4]);
    }

    /// `u u~ → e+ e- u u~` with the outgoing pair from an s-channel gluon: an FSR
    /// vertex, then that gluon absorbed by beam 1 (ISR), the leptons, and the
    /// core. Two `αs` ratios, both at their own vertex's scale.
    fn uux_to_eeuux(iqjets: Vec<i64>) -> RewgtHistory {
        history(
            6,
            vec![
                merge([16, 32], 48, 0.0),
                merge([1, 48], 49, 0.3),
                merge([4, 8], 12, 0.0),
                merge([49, 2], 12, 1.0),
            ],
            vec![225.0, 900.0, 8000.0, 8100.0],
            [4, 4],
            iqjets,
            &[
                (1, U),
                (2, UB),
                (4, -11),
                (8, 11),
                (16, U),
                (32, UB),
                (48, G),
                (49, U),
                (12, Z),
            ],
            [8100.0, 8100.0],
            60.0,
            &[40.0, 40.0, 30.0, 20.0],
        )
    }

    #[test]
    fn fsr_and_isr_vertices_each_take_their_own_scale() {
        let h = uux_to_eeuux(vec![0, 0, 0, 0, 1, 1]);
        let r = run(&h, &colors(4), &settings(), &[U, UB, -11, 11, U, UB]);
        let classes: Vec<VertexClass> = r.vertices.iter().map(|v| v.class).collect();
        assert_eq!(
            classes,
            [
                VertexClass::Fsr,
                VertexClass::Isr,
                VertexClass::FsrNotPartonVertex,
                VertexClass::Core
            ]
        );
        assert_eq!(r.vertices[0].alpha_s.expect("FSR").q2, 225.0);
        assert_eq!(r.vertices[1].alpha_s.expect("ISR").q2, 900.0);
        // The s-channel gluon is the hardest daughter's line, a final-state one.
        assert_eq!(r.vertices[0].ipart, 5);
        assert_eq!(r.vertices[1].ipart, 1);
        let x = X[0] * 0.3;
        let chain = &r.pdf_chain[0];
        assert_eq!(chain.len(), 2);
        assert_eq!(
            (chain[0].n, chain[0].flavour, chain[0].q2_now),
            (2, U, 900.0)
        );
        assert_eq!((chain[1].n, chain[1].flavour, chain[1].x), (4, U, x));
        let ratio = toy_f(U, x, 8100.0) / toy_f(U, x, 900.0);
        let expected =
            toy_alpha_s(15.0) / toy_alpha_s(60.0) * toy_alpha_s(30.0) / toy_alpha_s(60.0) * ratio;
        assert!((r.weight - expected).abs() <= 1e-14 * expected);
        // Beam 2's only vertex is the core: its first, with no ratio.
        assert_eq!(r.pdf_chain[1].len(), 1);
        assert_eq!(r.pdf_chain[1][0].outcome, PdfOutcome::First);
    }

    /// Final-state partons that are jets but were not tagged as jets
    /// (`iqjets = 0`) are not parton lines: their splitting is an FSR vertex with
    /// no parton daughter, and the gluon it leaves is not a parton line either,
    /// so the ISR vertex that absorbs it is not reweighted — `goodjet`
    /// propagating up the clustering.
    #[test]
    fn goodjet_propagates_from_untagged_daughters_to_the_isr_vertex() {
        let h = uux_to_eeuux(vec![0; 6]);
        let r = run(&h, &colors(4), &settings(), &[U, UB, -11, 11, U, UB]);
        assert_eq!(r.vertices[0].class, VertexClass::FsrNoPartonDaughter);
        assert_eq!(r.vertices[1].class, VertexClass::IsrNotParton);
        assert!(r.vertices.iter().all(|v| v.alpha_s.is_none()));
        // The density chain does not ask about jets: beam 1 still takes its ratio.
        assert!(matches!(
            r.pdf_chain[0][1].outcome,
            PdfOutcome::Ratio { .. }
        ));
    }

    /// `g b → e+ e- b` at `maxjetflavor = 4`, the outgoing `b` joining the
    /// incoming gluon into the spacelike `b` line (the walk's code: a `b` is no
    /// jet, so no flavour is transmitted).
    fn gb_to_eeb() -> RewgtHistory {
        history(
            5,
            vec![
                merge([1, 16], 17, 0.5),
                merge([4, 8], 12, 0.0),
                merge([17, 2], 12, 1.0),
            ],
            vec![400.0, 8000.0, 8100.0],
            [1, 3],
            vec![0; 5],
            &[(1, G), (2, 5), (4, -11), (8, 11), (16, 5), (17, 5), (12, Z)],
            [8100.0, 8100.0],
            60.0,
            &[40.0, 40.0, 20.0],
        )
    }

    /// With `asrwgtflavor = 5` a `b` is a parton though not a jet: the outgoing
    /// `b` is a parton line, the ISR vertex is reweighted, and the `b` line is
    /// walked. With `asrwgtflavor = 4` neither: the `b` line is no parton line
    /// and beam 1's chain stops at the gluon.
    #[test]
    fn asrwgtflavor_decides_whether_a_b_quark_is_a_parton_line() {
        let h = gb_to_eeb();
        let flavours = [G, 5, -11, 11, 5];
        let five = run(&h, &colors(4), &settings(), &flavours);
        assert_eq!(five.vertices[0].class, VertexClass::Isr);
        assert_eq!(five.vertices[0].alpha_s.expect("reweighted").q2, 400.0);
        assert_eq!(five.pdf_chain[0].len(), 2);
        // `jlast` for beam 1 is its first vertex here, whose scale is therefore
        // `q2bck` and not the vertex's own; nothing later on the beam rises
        // above it.
        assert_eq!(five.pdf_chain[0][0].q2_now, 8100.0);
        assert_eq!(five.pdf_chain[0][1].flavour, 5);
        assert_eq!(five.pdf_chain[0][1].outcome, PdfOutcome::NoRise);

        let four = run(
            &h,
            &colors(4),
            &RewgtSettings {
                asrwgtflavor: 4,
                ..settings()
            },
            &flavours,
        );
        assert_eq!(four.vertices[0].class, VertexClass::IsrNotParton);
        assert_eq!(four.pdf_chain[0].len(), 1);
        // Beam 2's `b` is no parton either, so its line is never walked.
        assert!(four.pdf_chain[1].is_empty());
        assert_eq!(four.weight, 1.0);
    }

    /// With `jlast` at the core, the `b` line takes its ratio, for the code the
    /// line carries (`+5`: the walk's, since a non-jet quark transmits nothing).
    #[test]
    fn a_b_quark_line_reads_its_own_density_up_to_jlast() {
        let mut h = gb_to_eeb();
        h.jlast = [3, 3];
        let r = run(&h, &colors(4), &settings(), &[G, 5, -11, 11, 5]);
        let step = r.pdf_chain[0][1];
        assert_eq!((step.n, step.flavour, step.x), (3, 5, X[0] * 0.5));
        let PdfOutcome::Ratio { ratio, .. } = step.outcome else {
            panic!("expected a ratio, got {:?}", step.outcome)
        };
        assert_eq!(
            ratio,
            toy_f(5, X[0] * 0.5, 8100.0) / toy_f(5, X[0] * 0.5, 400.0)
        );
    }

    /// A final-state leg that is not a jet keeps the walk's code, not the
    /// combination's: a `b g` member of a group whose walk ran on `d g`, at
    /// `maxjetflavor = 4`, still sees `d` on its outgoing leg, so the spacelike
    /// line is handed `d̄` and its density is read for `d̄`. This is MadEvent's
    /// state on the first event of a run.
    #[test]
    fn a_non_jet_final_state_leg_keeps_the_walk_s_code() {
        let h = history(
            5,
            vec![
                merge([2, 16], 18, 0.4),
                merge([4, 8], 12, 0.0),
                merge([1, 18], 12, 1.0),
            ],
            vec![400.0, 8000.0, 8100.0],
            [3, 3],
            vec![0, 0, 0, 0, 1],
            &[
                (1, 1),
                (2, G),
                (4, -11),
                (8, 11),
                (16, 1),
                (18, -1),
                (12, Z),
            ],
            [8100.0, 8100.0],
            60.0,
            &[40.0, 40.0, 20.0],
        );
        let r = run(&h, &colors(4), &settings(), &[5, G, -11, 11, 5]);
        assert_eq!(r.pdf_chain[0][0].flavour, 5);
        assert_eq!(r.pdf_chain[1][1].flavour, -1);
    }

    /// Three initial-state vertices on beam 1. The middle one reads
    /// `min(pt2, q2bck)`; the core, at `jlast`, reads `q2bck` exactly. With the
    /// middle vertex's `pt2` above `q2bck`, its scale is capped at `q2bck` and
    /// the core no longer rises, so it takes no ratio.
    fn uux_to_eegg(middle_pt2: f64, jlast: usize) -> RewgtHistory {
        history(
            6,
            vec![
                merge([1, 16], 17, 0.5),
                merge([17, 32], 49, 0.25),
                merge([4, 8], 12, 0.0),
                merge([49, 2], 12, 1.0),
            ],
            vec![100.0, middle_pt2, 8000.0, 8100.0],
            [jlast, 4],
            vec![0, 0, 0, 0, 1, 1],
            &[
                (1, U),
                (2, UB),
                (4, -11),
                (8, 11),
                (16, G),
                (32, G),
                (17, U),
                (49, U),
                (12, Z),
            ],
            [2500.0, 2500.0],
            60.0,
            &[40.0, 40.0, 30.0, 20.0],
        )
    }

    #[test]
    fn the_chain_scale_is_capped_at_q2bck_and_is_q2bck_at_jlast() {
        let flavours = [U, UB, -11, 11, G, G];
        let r = run(&uux_to_eegg(900.0, 4), &colors(4), &settings(), &flavours);
        let chain = &r.pdf_chain[0];
        let q2: Vec<(f64, f64)> = chain.iter().map(|s| (s.q2_prev, s.q2_now)).collect();
        assert_eq!(q2, [(0.0, 100.0), (100.0, 900.0), (900.0, 2500.0)]);
        assert_eq!(chain[1].x, X[0] * 0.5 * 0.25);
        assert_eq!(chain[2].x, X[0] * 0.5 * 0.25);
        assert!(chain[1..]
            .iter()
            .all(|s| matches!(s.outcome, PdfOutcome::Ratio { .. })));

        let r = run(&uux_to_eegg(4900.0, 4), &colors(4), &settings(), &flavours);
        let chain = &r.pdf_chain[0];
        let q2: Vec<(f64, f64)> = chain.iter().map(|s| (s.q2_prev, s.q2_now)).collect();
        assert_eq!(q2, [(0.0, 100.0), (100.0, 2500.0), (2500.0, 2500.0)]);
        assert_eq!(chain[2].outcome, PdfOutcome::NoRise);
    }

    /// Past `jlast` a rising scale takes no ratio and hands none on. A walk
    /// never produces this: the chain's scale is `q2bck` from `jlast` on and
    /// nothing later exceeds it. The arm is the source's, and is reached here
    /// only by a history whose `jlast` precedes the chain.
    #[test]
    fn past_jlast_no_ratio_is_taken() {
        let flavours = [U, UB, -11, 11, G, G];
        let r = run(&uux_to_eegg(900.0, 0), &colors(4), &settings(), &flavours);
        let chain = &r.pdf_chain[0];
        assert_eq!(chain[0].outcome, PdfOutcome::First);
        assert_eq!(chain[1].outcome, PdfOutcome::PastLast);
        assert_eq!(chain[2].outcome, PdfOutcome::PastLast);
        assert!(chain[2].q2_prev.is_nan());
        assert_eq!(
            r.weight,
            r.vertices
                .iter()
                .filter_map(|v| v.alpha_s)
                .map(|a| a.ratio)
                .product::<f64>()
        );
    }

    /// The listed factors reproduce the product bit for bit on every history
    /// above, whatever their order of multiplication across beams.
    #[test]
    fn the_product_is_the_listed_factors() {
        let cases: Vec<(RewgtHistory, Vec<i64>)> = vec![
            (ug_to_eeu(5000.0, 400.0), vec![U, G, -11, 11, U]),
            (
                uux_to_eeuux(vec![0, 0, 0, 0, 1, 1]),
                vec![U, UB, -11, 11, U, UB],
            ),
            (uux_to_eegg(900.0, 4), vec![U, UB, -11, 11, G, G]),
        ];
        for (h, flavours) in cases {
            let r = run(&h, &colors(4), &settings(), &flavours);
            let mut by_hand = 1.0;
            for v in &r.vertices {
                if let Some(a) = v.alpha_s {
                    by_hand *= a.ratio;
                }
                for beam in 0..2 {
                    for s in r.pdf_chain[beam].iter().filter(|s| s.n == v.n) {
                        if let PdfOutcome::Ratio { ratio, .. } = s.outcome {
                            by_hand *= ratio;
                        }
                    }
                }
            }
            assert!((by_hand - r.weight).abs() <= 1e-15 * r.weight);
            assert!(r.weight != 1.0);
        }
    }

    /// Negative control for the `αs` factor. A σ-like integral over the ISR
    /// vertex's `kT` from 20 to 200 GeV, with a falling spectrum, the real
    /// two-loop running and a slowly evolving density, computed from `rewgt` and by hand
    /// from the two formulas: the two agree, and dropping the `αs` ratio moves
    /// the integral by more than ten times a 1% gate.
    #[test]
    fn dropping_the_alpha_s_factor_moves_a_sigma_like_integral_far_outside_tolerance() {
        let running = RunningAlphaS::new(0.118, NLoop::Two).expect("running coupling");
        let alpha_s = |q: f64| running.eval(q);
        // A density evolving as slowly as a sea quark's at moderate `x`, so the
        // spectrum is not dominated by the density ratio.
        let xfx = |_: i64, x: f64, q2: f64| (1.0 - x).powi(3) * (1.0 + 0.05 * q2.ln());
        let f = |x: f64, q2: f64| xfx(UB, x, q2) / x;
        // The core's scale, near the lepton pair's mass, as on a matched
        // `p p → ℓℓj` event.
        let mu_r = 91.188;
        let (lo, hi, bins) = (20.0f64, 200.0f64, 400);
        let width = (hi - lo) / bins as f64;
        let (mut sigma, mut by_hand, mut no_alpha_s) = (0.0, 0.0, 0.0);
        for i in 0..bins {
            let kt = lo + (i as f64 + 0.5) * width;
            let spectrum = width / (kt * kt);
            let mut h = ug_to_eeu(8100.0, kt * kt);
            h.mu_r = mu_r;
            let r = rewgt(
                &h,
                &colors(4),
                &settings(),
                &[U, G, -11, 11, U],
                X,
                alpha_s,
                xfx,
            )
            .expect("reweights");
            sigma += spectrum * r.weight;
            let x = X[1] * 0.4;
            let pdf = f(x, 8100.0) / f(x, (kt * kt).min(8100.0));
            by_hand += spectrum * alpha_s(kt) / alpha_s(mu_r) * pdf;
            let pdf_only: f64 = r.pdf_chain[1]
                .iter()
                .map(|s| match s.outcome {
                    PdfOutcome::Ratio { ratio, .. } => ratio,
                    _ => 1.0,
                })
                .product();
            no_alpha_s += spectrum * pdf_only;
        }
        let tolerance = 0.01;
        assert!((sigma / by_hand - 1.0).abs() < 1e-12);
        let shift = (no_alpha_s / sigma - 1.0).abs();
        assert!(shift > 10.0 * tolerance, "dropping αs moves σ by {shift}");
    }
}
