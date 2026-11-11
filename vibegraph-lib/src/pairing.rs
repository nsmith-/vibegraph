//! The amplitudes behind the pairing-weighted `cut_decays = F` exemption.
//!
//! Where identical final-state particles sit in a decay in some pairing and in
//! the core in another (`g g > z u u~, z > u u~`), every pairing is kept
//! coherently and each exempts its own decay products: the integrand is
//!
//! ```text
//! Σ_p w_p Θ_p |M|²,   w_p = |M_p|² / Σ_q |M_q|²
//! ```
//!
//! with `Θ_p` the filter exempting pairing `p`'s decay products
//! ([`Cuts::pass_pairings`]) and `|M_p|²` the colour- and helicity-summed square
//! of the amplitude over pairing `p`'s diagrams alone, the diagrams that force
//! `p`'s lines ([`ForcedResonances::pairings`]). The diagrams of a subprocess
//! partition exactly into its pairings, since each forces one set of lines.
//!
//! The weights sum to one, so a point every pairing's filter accepts carries
//! `|M|²` and one none accepts carries zero; only a point some filters accept
//! and others reject evaluates the per-pairing amplitudes, at about the cost of
//! one more `|M|²`. `|M_p|²` is not gauge invariant off the poles, so `w_p` is a
//! choice of how to share the interference between pairings; it is the only
//! part of the integrand it moves.
//!
//! IR safety: `|M|² ≤ N Σ_q |M_q|²` for `N` pairings (Cauchy–Schwarz), so
//! `w_p Θ_p |M|² ≤ N Θ_p |M_p|²`, finite wherever pairing `p`'s own `|M_p|²` is
//! integrable over `Θ_p`'s region. [`Cuts::compile_with`] checks that per
//! exemption with the limit guards.
//!
//! [`ForcedResonances::pairings`]: crate::cuts::ForcedResonances::pairings

use std::cell::RefCell;

use crate::cuts::{diagram_forced_lines, Cuts, ForcedResonances};
use crate::diagrams::DiagramSet;
use crate::hadronic::{compile_class, BoundSubprocess, HadronicError, SubprocessProto};
use crate::helas::eval::AmplitudeEvaluator;
use crate::helas::repr::lorentz::LorentzVector;
use crate::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

/// One subprocess's pairings under a pairing-weighted filter: per pairing, the
/// bit of [`Cuts::pass_pairings`] for its exemption, and, where the subprocess
/// has more than one pairing, the amplitude over its diagrams.
pub struct PairingSplit {
    bits: Vec<u64>,
    /// Parallel to `bits`; empty for a subprocess with one pairing, whose weight
    /// is its own filter's indicator.
    amplitudes: Vec<AmplitudeEvaluator>,
}

impl std::fmt::Debug for PairingSplit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingSplit")
            .field("bits", &self.bits)
            .field("amplitudes", &self.amplitudes.len())
            .finish()
    }
}

impl PairingSplit {
    /// The split of `set` under `cuts`, compiled from `set`'s own diagrams;
    /// `None` where `cuts` is not pairing-weighted.
    ///
    /// # Panics
    ///
    /// If a pairing of `set` exempts a leg set `cuts` has no filter for: `cuts`
    /// was not compiled from these diagrams.
    pub fn new(
        set: &DiagramSet,
        cuts: &Cuts,
        model: &UFOModel,
        evaluated: &EvaluatedModel,
    ) -> Result<Option<Self>, HadronicError> {
        if !cuts.is_pairing_weighted() {
            return Ok(None);
        }
        let forced = ForcedResonances::of(&set.diagrams, evaluated);
        let pairings = forced.pairings();
        let bits: Vec<u64> = pairings
            .iter()
            .map(|p| {
                cuts.pairing_bit(p.products())
                    .expect("every pairing's exemption has a filter in a filter compiled from it")
            })
            .collect();
        if pairings.len() < 2 {
            return Ok(Some(PairingSplit {
                bits,
                amplitudes: Vec::new(),
            }));
        }
        let of_diagram: Vec<usize> = set
            .diagrams
            .iter()
            .map(|d| {
                let lines = diagram_forced_lines(d, evaluated);
                pairings
                    .iter()
                    .position(|p| p.lines() == lines.as_slice())
                    .expect("every diagram is in the pairing of its own lines")
            })
            .collect();
        let amplitudes = (0..pairings.len())
            .map(|k| {
                let subset = DiagramSet {
                    particles_in: set.particles_in.clone(),
                    particles_out: set.particles_out.clone(),
                    polarizations: set.polarizations.clone(),
                    diagrams: set
                        .diagrams
                        .iter()
                        .zip(&of_diagram)
                        .filter(|(_, &p)| p == k)
                        .map(|(d, _)| d.clone())
                        .collect(),
                    mirrored: set.mirrored,
                };
                compile_class(&subset, model, evaluated)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(PairingSplit { bits, amplitudes }))
    }

    /// The number of pairings.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Whether the subprocess has no pairing, which a split never is.
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }
}

/// A [`PairingSplit`]'s amplitudes, ready to bind per thread.
pub(crate) struct PairingAmplitudes<'a> {
    bits: &'a [u64],
    protos: Vec<SubprocessProto<'a>>,
}

impl<'a> PairingAmplitudes<'a> {
    /// Every pairing amplitude starts scale-aware at the parameter card's own
    /// coupling, and follows whatever coupling the subprocess is moved to.
    pub(crate) fn new(split: &'a PairingSplit, evaluated: &EvaluatedModel) -> Self {
        PairingAmplitudes {
            bits: &split.bits,
            protos: split
                .amplitudes
                .iter()
                .map(|eval| SubprocessProto::running(eval, evaluated))
                .collect(),
        }
    }

    pub(crate) fn bind(&self) -> BoundPairings<'a> {
        BoundPairings {
            bits: self.bits,
            subs: self.protos.iter().map(SubprocessProto::bind).collect(),
            m2: RefCell::new(vec![0.0; self.protos.len()]),
        }
    }
}

/// One thread's copy of a [`PairingAmplitudes`].
pub(crate) struct BoundPairings<'a> {
    bits: &'a [u64],
    subs: Vec<BoundSubprocess<'a>>,
    m2: RefCell<Vec<f64>>,
}

impl BoundPairings<'_> {
    /// The weight `Σ_p w_p Θ_p` of each of `passed`, a [`Cuts::pass_pairings`]
    /// mask per event, at the external momenta `ext` and `main`'s coupling.
    ///
    /// One where the mask holds every pairing's bit and zero where it holds none;
    /// otherwise the pairing amplitudes are evaluated once for all the masks. Zero
    /// where every `|M_p|²` vanishes, where `|M|²` does too.
    pub(crate) fn weights<const N: usize>(
        &self,
        passed: [u64; N],
        main: &BoundSubprocess<'_>,
        ext: &[V],
    ) -> [f64; N] {
        let mut evaluated = false;
        let mut m2 = self.m2.borrow_mut();
        passed.map(|mask| {
            let held = self.bits.iter().filter(|&&b| mask & b != 0).count();
            if held == 0 {
                return 0.0;
            }
            if held == self.bits.len() {
                return 1.0;
            }
            if !evaluated {
                let alpha_s = main.alpha_s();
                for (out, sub) in m2.iter_mut().zip(&self.subs) {
                    if let Some(alpha_s) = alpha_s {
                        sub.set_alpha_s(alpha_s);
                    }
                    *out = sub.eval_m2(ext);
                }
                evaluated = true;
            }
            let total: f64 = m2.iter().sum();
            if !(total > 0.0) {
                return 0.0;
            }
            let kept: f64 = m2
                .iter()
                .zip(self.bits)
                .filter(|(_, &b)| mask & b != 0)
                .map(|(m, _)| m)
                .sum();
            kept / total
        })
    }
}

/// One [`PairingSplit`] per non-empty subprocess of `sets`, in the order
/// [`compile_subprocesses`](crate::hadronic::compile_subprocesses) keeps them,
/// for a fixed-beam integrand cut by the one filter `cuts` compiled from all of
/// their diagrams; empty where `cuts` is not pairing-weighted.
pub fn subprocess_pairing_splits(
    sets: &[DiagramSet],
    cuts: &Cuts,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<Vec<Option<PairingSplit>>, HadronicError> {
    if !cuts.is_pairing_weighted() {
        return Ok(Vec::new());
    }
    sets.iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|set| PairingSplit::new(set, cuts, model, evaluated))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::runcard::RunCard;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// The pole mass of the line `set`'s first pairing forces.
    fn forced_mass(set: &DiagramSet, evaluated: &EvaluatedModel) -> f64 {
        ForcedResonances::of(&set.diagrams, evaluated).pairings()[0].lines()[0].mass
    }

    /// Boost `p` by the velocity `beta`.
    fn boost(p: V, beta: [f64; 3]) -> V {
        let b2: f64 = beta.iter().map(|b| b * b).sum();
        if b2 == 0.0 {
            return p;
        }
        let gamma = 1.0 / (1.0 - b2).sqrt();
        let bp = beta[0] * p.px() + beta[1] * p.py() + beta[2] * p.pz();
        let k = (gamma - 1.0) * bp / b2 + gamma * p.e();
        V::new(
            gamma * (p.e() + bp),
            p.px() + k * beta[0],
            p.py() + k * beta[1],
            p.pz() + k * beta[2],
        )
    }

    /// A partonic-CM configuration, beams first, with outgoing leg `roles[0]`
    /// at energy `e0` and transverse momentum `eps` along beam `+z`, leg
    /// `roles[1]` at polar angle `th1` with the energy that puts the pair on
    /// the pole `mz`, and legs `roles[2]`, `roles[3]` back to back across the
    /// beam axis in the rest frame of the remainder, of mass `m23`.
    fn configuration(roles: [usize; 4], eps: f64, e0: f64, th1: f64, m23: f64, mz: f64) -> Vec<V> {
        let p0 = V::new(e0, eps, 0.0, (e0 * e0 - eps * eps).sqrt());
        let dir1 = [-th1.sin(), 0.0, th1.cos()];
        let cos_a = (p0.px() * dir1[0] + p0.pz() * dir1[2]) / e0;
        let e1 = mz * mz / (2.0 * e0 * (1.0 - cos_a));
        let p1 = V::new(e1, e1 * dir1[0], e1 * dir1[1], e1 * dir1[2]);
        let k = [p0.px() + p1.px(), p0.py() + p1.py(), p0.pz() + p1.pz()];
        let k2: f64 = k.iter().map(|x| x * x).sum();
        let sqrt_s = e0 + e1 + (m23 * m23 + k2).sqrt();
        let q0 = sqrt_s - e0 - e1;
        let beta = [-k[0] / q0, -k[1] / q0, -k[2] / q0];
        let half = 0.5 * m23;
        let p2 = boost(V::new(half, 0.0, half, 0.0), beta);
        let p3 = boost(V::new(half, 0.0, -half, 0.0), beta);
        let mut out = vec![V::new(0.0, 0.0, 0.0, 0.0); 4];
        for (role, p) in roles.into_iter().zip([p0, p1, p2, p3]) {
            out[role] = p;
        }
        let e = 0.5 * sqrt_s;
        let mut ext = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
        ext.extend(out);
        ext
    }

    struct Probe {
        set: DiagramSet,
        eval: AmplitudeEvaluator,
        cuts: Cuts,
        split: Option<PairingSplit>,
        /// Outgoing legs: the on-pole pair `roles[0..2]`, exempt in the pairing
        /// that decays it, and the other two.
        roles: [usize; 4],
    }

    fn probe(process: &str, model: &UFOModel, evaluated: &EvaluatedModel, rc: &RunCard) -> Probe {
        let card = parse_proc_card(process, &ParsingOptions::default()).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        assert_eq!(sets.len(), 1, "{process}");
        let set = sets.into_iter().next().unwrap();
        let eval = compile_class(&set, model, evaluated).unwrap();
        let legs = crate::hadronic::process_external_legs(&eval, model, evaluated);
        let forced = ForcedResonances::of(&set.diagrams, evaluated);
        let cuts = Cuts::compile_with(rc, &legs, &forced).unwrap();
        let split = PairingSplit::new(&set, &cuts, model, evaluated).unwrap();
        // The pair: a pairing's products whose first leg is a quark; the rest in
        // leg order.
        let products = forced
            .pairings()
            .iter()
            .map(|p| p.products())
            .find(|&m| {
                let first = m.trailing_zeros() as usize;
                m.count_ones() == 2 && legs[2 + first].pdg > 0
            })
            .expect("a two-leg decay whose first leg is a quark");
        let mut pair: Vec<usize> = (0..4).filter(|k| products & (1 << k) != 0).collect();
        let mut rest: Vec<usize> = (0..4).filter(|k| products & (1 << k) == 0).collect();
        pair.sort();
        rest.sort();
        Probe {
            set,
            eval,
            cuts,
            split,
            roles: [pair[0], pair[1], rest[0], rest[1]],
        }
    }

    /// I-C1's limit on the weighted integrand: on `g g > z u u~, z > u u~`, with
    /// the pair `(0, 1)` held on the `Z` pole and quark `0` driven collinear with
    /// a beam at `pT = ε`, `|M|² ε²` is constant (the other pairings' `g → u ū`
    /// splitting) and the union of the pairings' filters accepts the point, but
    /// `Σ_p w_p Θ_p |M|² ε² → 0`: every pairing that accepts it carries a weight
    /// that falls as `ε²`. On the distinct-flavour chain (`z > d d~`) there is no
    /// pole at all.
    #[test]
    fn the_weighted_integrand_has_no_beam_collinear_pole() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let rc = RunCard::parse("F = cut_decays\n20 = ptj\n5 = etaj\n0.4 = drjj\n").unwrap();
        let same = probe("generate g g > z u u~, z > u u~", &model, &evaluated, &rc);
        let control = probe("generate g g > z u u~, z > d d~", &model, &evaluated, &rc);
        let mz = forced_mass(&same.set, &evaluated);
        assert_eq!(mz, forced_mass(&control.set, &evaluated));
        assert!(same.cuts.is_pairing_weighted());
        assert_eq!(same.split.as_ref().map(PairingSplit::len), Some(4));
        assert!(!control.cuts.is_pairing_weighted());
        assert!(control.split.is_none());

        let m2_of = |eval: &AmplitudeEvaluator, ext: &[V]| {
            SubprocessProto::running(eval, &evaluated)
                .bind()
                .eval_m2(ext)
        };
        let weighted_of = |p: &Probe, ext: &[V]| -> (f64, f64, u64) {
            let main = SubprocessProto::running(&p.eval, &evaluated).bind();
            let m2 = main.eval_m2(ext);
            let passed = p.cuts.pass_pairings(ext);
            let w = match &p.split {
                Some(split) => {
                    PairingAmplitudes::new(split, &evaluated)
                        .bind()
                        .weights([passed], &main, ext)[0]
                }
                None => f64::from(u8::from(passed != 0)),
            };
            (m2, w, passed)
        };

        for (case, m23) in [(1, 160.0), (2, 111.19), (3, 180.0)] {
            let mut rows = Vec::new();
            for eps in [1e-2, 1e-3, 1e-4] {
                let ext = configuration(same.roles, eps, 40.0, 2.6, m23, mz);
                let (m2, w, passed) = weighted_of(&same, &ext);
                assert!(
                    same.cuts.pass(&ext),
                    "case {case}: the union accepts the point"
                );
                let ctl_ext = configuration(control.roles, eps, 40.0, 2.6, m23, mz);
                let (ctl_m2, ctl_w, _) = weighted_of(&control, &ctl_ext);
                assert_eq!(
                    ctl_w, 1.0,
                    "case {case}: the control's decay products are exempt"
                );
                eprintln!(
                    "case {case} eps {eps:7.0e} passed {passed:04b} |M|^2 eps^2 {:.6e} \
                     w {w:.3e} w|M|^2 eps^2 {:.6e} control |M|^2 eps^2 {:.3e}",
                    m2 * eps * eps,
                    w * m2 * eps * eps,
                    ctl_m2 * eps * eps
                );
                assert_eq!(m2, m2_of(&same.eval, &ext));
                rows.push((m2 * eps * eps, w * m2 * eps * eps, ctl_m2 * eps * eps));
            }
            let (first, last) = (rows[0], rows[2]);
            assert!(
                (last.0 / rows[1].0 - 1.0).abs() < 1e-3 && (last.0 / first.0 - 1.0).abs() < 1e-2,
                "case {case}: the unweighted |M|² ε² is constant, {} → {}",
                first.0,
                last.0
            );
            assert!(
                last.1 < 1e-3 * first.1,
                "case {case}: the weighted integrand × ε² falls, {} → {}",
                first.1,
                last.1
            );
            assert!(
                last.2 < 1e-3 * first.2,
                "case {case}: the control has no pole"
            );
        }
    }
}
