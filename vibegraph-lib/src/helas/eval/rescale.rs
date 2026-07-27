//! Evaluating a bound amplitude at a per-event value of the strong coupling.
//!
//! Changing `αs` changes exactly one thing about a [`BoundAmplitude`]: the two numeric
//! constant pools. The compiled skeleton, the helicity expansion and the color-factor
//! matrix are all independent of it. [`ScaleAwareAmplitude`] therefore owns a private
//! copy of the pools and rewrites them per event, leaving the shared
//! [`AmplitudeEvaluator`](super::compile::AmplitudeEvaluator) untouched.
//!
//! Two ways to rewrite them:
//!
//! * The **reference path** re-evaluates the model — [`EvaluatedModel::set_alpha_s`]
//!   followed by [`Folded::pools`](super::fold::Folded::pools). Exact for any model and
//!   any parameter graph, and slow enough (the parameter graph is keyed by name) to
//!   dominate a cheap matrix element.
//! * The **scaling path** uses the fact that every tree-level coupling of a
//!   renormalisable model is a monomial `k·Gⁿ` in the strong coupling, with `k`
//!   independent of it. A product of monomials is a monomial, so the exponent survives
//!   the constant folding that collapses coupling products into single pool entries,
//!   and moving the pools is `consts[i] ← base[i]·rⁿⁱ` with `r = G(αs)/G(αs_ref)`.
//!
//! A sum of unequal exponents is *not* a monomial, and neither is anything reached
//! through a function of `G`. Those are detected, not assumed: an entry the analysis
//! cannot tag makes the whole amplitude take the reference path. The two paths agree
//! to within a rounding of each other — they are different floating-point realisations
//! of the same value, not the same expression — and they agree bit-for-bit at
//! `αs = αs_ref`, where `r` is exactly one and the pools are left as bound.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use num_traits::FromPrimitive;

use super::compile::AmplitudeEvaluator;
use super::fold::GPower;
use super::run::{BoundAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;
use crate::ufo::EvaluatedModel;

/// Widest span of `G` exponents the scaling path builds a factor table for. A
/// renormalisable tree amplitude folds to exponents in a range of a few; anything
/// wider is treated as untaggable rather than growing the table.
const MAX_POWER_SPAN: usize = 16;

/// Why an amplitude's constant pools cannot be moved to a new strong coupling by
/// scaling, so that it takes the reference path instead.
#[derive(Debug, Clone, PartialEq)]
pub enum RescaleFallback {
    /// The complex-pool entry at this index is not a monomial in `G`.
    ComplexEntry(usize),
    /// The real-pool entry at this index is not a monomial in `G`.
    RealEntry(usize),
    /// `G` does not follow the square root of the strong coupling, so no single ratio
    /// `r` moves the monomials. Carries the exponent measured from the model.
    CouplingExponent(f64),
    /// The card's own strong coupling is zero or absent, so there is no reference
    /// value to scale away from.
    NoReferenceCoupling,
    /// The tagged exponents span more than [`MAX_POWER_SPAN`].
    ExponentSpan { min: i32, max: i32 },
}

impl fmt::Display for RescaleFallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RescaleFallback::ComplexEntry(i) => {
                write!(f, "complex pool entry {i} is not a monomial in G")
            }
            RescaleFallback::RealEntry(i) => {
                write!(f, "real pool entry {i} is not a monomial in G")
            }
            RescaleFallback::CouplingExponent(p) => {
                write!(f, "G scales as alpha_s^{p}, not alpha_s^0.5")
            }
            RescaleFallback::NoReferenceCoupling => {
                write!(
                    f,
                    "the parameter card has no non-zero aS to scale away from"
                )
            }
            RescaleFallback::ExponentSpan { min, max } => {
                write!(f, "G exponents span {min}..={max}")
            }
        }
    }
}

/// How much of a bound amplitude's constant pools the `G`-power analysis could tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolTagCensus {
    /// Total constant-pool entries, complex and real.
    pub entries: usize,
    /// Entries carrying a known power of `G` (including power zero).
    pub tagged: usize,
    /// Entries carrying a non-zero power of `G` — the ones a scale change moves.
    pub scale_dependent: usize,
    /// Largest tagged power of `G`.
    pub max_power: i32,
}

/// The `G`-power tags and the reference coupling they are relative to, shared by every
/// thread evaluating one compiled amplitude.
#[derive(Debug)]
struct RescalePlan {
    alpha_s_ref: f64,
    powers_c: Box<[GPower]>,
    powers_f: Box<[GPower]>,
    min_power: i32,
    max_power: i32,
    /// Whether any entry carries a non-zero power — false for an amplitude with no
    /// strong coupling in it at all, whose pools a scale change never touches.
    scale_dependent: bool,
    fallback: Option<RescaleFallback>,
}

/// A bound amplitude that can be moved to a per-event value of the strong coupling.
///
/// Owns its constant pools, so it is mutable state: one instance per thread, obtained
/// by [`fork`](Self::fork). Nothing is shared mutably, so a parallel integrator cannot
/// have one thread read another's coupling.
///
/// Construction never needs a running-coupling object; the scale is supplied as a bare
/// `αs`, so a process with no strong coupling in its matrix element works without one.
pub struct ScaleAwareAmplitude<'a, F: Real + FromPrimitive> {
    /// The working amplitude: its pools are rewritten by every scale change.
    amp: BoundAmplitude<'a, F>,
    /// Pool values at the card's own `αs`. Every rescale starts here rather than from
    /// the previous scale, so a sequence of events cannot compound rounding.
    base_c: Box<[num_complex::Complex<F>]>,
    base_f: Box<[F]>,
    plan: Arc<RescalePlan>,
    /// A private copy of the evaluated model, present only when the plan needs the
    /// reference path.
    model: Option<EvaluatedModel>,
    alpha_s: f64,
}

impl<'a, F: Real + FromPrimitive> ScaleAwareAmplitude<'a, F> {
    /// Bind `eval` against `evaluated` and derive the plan for moving it to another
    /// strong coupling.
    ///
    /// The amplitude starts at the card's own `αs`, with pools bit-for-bit those of
    /// [`BoundAmplitude::bind`].
    pub fn new(eval: &'a AmplitudeEvaluator, evaluated: &EvaluatedModel) -> Self {
        let amp = BoundAmplitude::<F>::bind(eval, evaluated);
        let (base_c, base_f) = {
            let (c, f) = amp.pools();
            (c.to_vec().into_boxed_slice(), f.to_vec().into_boxed_slice())
        };

        // Parameters a change of aS moves, including aS itself: a pool entry that
        // reaches any of them other than G is not a power of G.
        let model = evaluated.model();
        let mut driven = model.params.dependents("aS");
        driven.insert("aS".to_owned());

        let powers = eval.folded().g_powers(model, &driven);
        let plan = build_plan(evaluated, &driven, powers.complex, powers.real);
        let alpha_s = plan.alpha_s_ref;
        let model = plan.fallback.is_some().then(|| evaluated.clone());

        ScaleAwareAmplitude {
            amp,
            base_c,
            base_f,
            plan: Arc::new(plan),
            model,
            alpha_s,
        }
    }

    /// An independent copy for another thread: same shared plan, its own pools.
    pub fn fork(&self) -> Self {
        ScaleAwareAmplitude {
            amp: self.amp.clone(),
            base_c: self.base_c.clone(),
            base_f: self.base_f.clone(),
            plan: Arc::clone(&self.plan),
            model: self.model.clone(),
            alpha_s: self.alpha_s,
        }
    }

    /// Move the amplitude to `alpha_s`.
    ///
    /// A repeat of the current value, and any value at all on an amplitude with no
    /// strong coupling in it, costs nothing.
    pub fn set_alpha_s(&mut self, alpha_s: f64) {
        if alpha_s == self.alpha_s || !self.plan.scale_dependent {
            self.alpha_s = alpha_s;
            return;
        }
        match self.model {
            Some(ref mut model) => {
                model.set_alpha_s(alpha_s);
                let (c, f) = self.amp.evaluator().folded().pools::<F>(model);
                self.amp.set_pools(c, f);
            }
            None => self.rescale_pools(alpha_s),
        }
        self.alpha_s = alpha_s;
    }

    /// `consts[i] ← base[i]·rⁿⁱ` over both pools.
    fn rescale_pools(&mut self, alpha_s: f64) {
        let plan = &self.plan;
        let r = (alpha_s / plan.alpha_s_ref).sqrt();
        let r_f = F::from_f64(r).expect("scale ratio convertible to the scalar field");
        let mut factor = [F::one(); MAX_POWER_SPAN];
        for (slot, n) in factor.iter_mut().zip(plan.min_power..=plan.max_power) {
            *slot = r_f.powi(n);
        }
        let base = plan.min_power;

        let base_c = &self.base_c;
        let base_f = &self.base_f;
        let (consts_c, consts_f) = self.amp.pools_mut();
        for (i, power) in plan.powers_c.iter().enumerate() {
            let n = power.expect("tagged pool on the scaling path");
            if n != 0 {
                consts_c[i] = base_c[i] * factor[(n - base) as usize];
            }
        }
        for (i, power) in plan.powers_f.iter().enumerate() {
            let n = power.expect("tagged pool on the scaling path");
            if n != 0 {
                consts_f[i] = base_f[i] * factor[(n - base) as usize];
            }
        }
    }

    /// The amplitude at the current coupling, for evaluation.
    pub fn amplitude(&self) -> &BoundAmplitude<'a, F> {
        &self.amp
    }

    /// A workspace sized for this amplitude (see [`BoundAmplitude::scratch_space`]).
    pub fn scratch_space(&self) -> ScratchSpace<F> {
        self.amp.scratch_space()
    }

    /// Color- and helicity-summed |M|² at the current coupling.
    pub fn eval_m2(&self, momenta: &[LorentzVector<F>], scratch: &mut ScratchSpace<F>) -> F {
        self.amp.eval_m2(momenta, scratch)
    }

    /// The strong coupling the pools currently hold.
    pub fn alpha_s(&self) -> f64 {
        self.alpha_s
    }

    /// The parameter card's own strong coupling — the value the pools are exact at.
    pub fn alpha_s_ref(&self) -> f64 {
        self.plan.alpha_s_ref
    }

    /// Whether any constant of this amplitude moves with the strong coupling. `false`
    /// for a matrix element with no QCD coupling in it, whose caller then needs no
    /// running coupling at all.
    pub fn depends_on_alpha_s(&self) -> bool {
        self.plan.scale_dependent
    }

    /// Why this amplitude takes the reference path, or `None` if it scales its pools.
    pub fn fallback(&self) -> Option<&RescaleFallback> {
        self.plan.fallback.as_ref()
    }

    /// How much of the constant pools carries a known power of `G`.
    pub fn census(&self) -> PoolTagCensus {
        let all = || self.plan.powers_c.iter().chain(self.plan.powers_f.iter());
        PoolTagCensus {
            entries: all().count(),
            tagged: all().filter(|p| p.is_some()).count(),
            scale_dependent: all().filter(|p| !matches!(p, Some(0))).count(),
            max_power: self.plan.max_power,
        }
    }
}

/// Derive the plan from the tagged pools: the exponent range, whether the scaling path
/// is usable at all, and — when it is — that `G` really is the square root of `αs` it
/// is assumed to be.
fn build_plan(
    evaluated: &EvaluatedModel,
    driven: &HashSet<String>,
    powers_c: Vec<GPower>,
    powers_f: Vec<GPower>,
) -> RescalePlan {
    let alpha_s_ref = evaluated.alpha_s().unwrap_or(0.0);
    let scale_dependent = powers_c
        .iter()
        .chain(powers_f.iter())
        .any(|p| !matches!(p, Some(0)));
    let (min_power, max_power) = powers_c
        .iter()
        .chain(powers_f.iter())
        .flatten()
        .fold((0, 0), |(lo, hi), &p| (lo.min(p), hi.max(p)));

    let untagged = powers_c
        .iter()
        .position(Option::is_none)
        .map(RescaleFallback::ComplexEntry)
        .or_else(|| {
            powers_f
                .iter()
                .position(Option::is_none)
                .map(RescaleFallback::RealEntry)
        });

    let fallback = if !scale_dependent {
        None
    } else if let Some(reason) = untagged {
        Some(reason)
    } else if alpha_s_ref <= 0.0 {
        Some(RescaleFallback::NoReferenceCoupling)
    } else if (max_power - min_power) as usize >= MAX_POWER_SPAN {
        Some(RescaleFallback::ExponentSpan {
            min: min_power,
            max: max_power,
        })
    } else {
        sqrt_law_violation(evaluated, driven, alpha_s_ref)
    };

    RescalePlan {
        alpha_s_ref,
        powers_c: powers_c.into_boxed_slice(),
        powers_f: powers_f.into_boxed_slice(),
        min_power,
        max_power,
        scale_dependent,
        fallback,
    }
}

/// Measure the exponent in `G ∝ αs^p` through the model itself, and report a violation
/// of `p = 1/2`.
///
/// The tags say how many powers of `G` an entry carries; turning that into a factor
/// needs `G(αs)/G(αs_ref)`, which the scaling path computes as `√(αs/αs_ref)` rather
/// than by re-evaluating the model. That is a claim about the model, so it is measured
/// rather than assumed — at two ratios, since one can be matched by coincidence.
fn sqrt_law_violation(
    evaluated: &EvaluatedModel,
    driven: &HashSet<String>,
    alpha_s_ref: f64,
) -> Option<RescaleFallback> {
    if !driven.contains("G") {
        // Nothing was tagged with a non-zero power in that case, so this is unreachable
        // from `build_plan`; treated as a violation rather than assumed away.
        return Some(RescaleFallback::CouplingExponent(0.0));
    }
    let g_ref = evaluated.param_values.get("G").copied()?;
    if g_ref.im != 0.0 || g_ref.re <= 0.0 {
        return Some(RescaleFallback::CouplingExponent(f64::NAN));
    }
    let mut probe = evaluated.clone();
    for ratio in [4.0, 0.37] {
        probe.set_alpha_s(alpha_s_ref * ratio);
        let g = probe.param_values["G"];
        if g.im != 0.0 {
            return Some(RescaleFallback::CouplingExponent(f64::NAN));
        }
        let exponent = (g.re / g_ref.re).ln() / ratio.ln();
        if (exponent - 0.5).abs() > 1e-9 {
            return Some(RescaleFallback::CouplingExponent(exponent));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::helas::repr::C;
    use crate::phasespace::rambo_massless;
    use crate::ufo::expr::parse_expr;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::{ParsedModel, UFOModel};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::sync::Arc;

    fn driven_set() -> HashSet<String> {
        let model = sm_model(SMRestrict::Default);
        let mut d = model.params.dependents("aS");
        d.insert("aS".to_owned());
        d
    }

    fn power_of(src: &str) -> GPower {
        let driven = driven_set();
        super::super::fold::g_power_expr(&parse_expr(src).unwrap(), &driven, true)
    }

    #[test]
    fn monomials_in_g_are_tagged_and_non_monomials_are_not() {
        assert_eq!(power_of("-G"), Some(1));
        assert_eq!(power_of("complex(0,1)*G**2"), Some(2));
        assert_eq!(power_of("-(ee*complex(0,1))/3"), Some(0));
        assert_eq!(power_of("G*G*G"), Some(3));
        assert_eq!(power_of("1/G"), Some(-1));
        assert_eq!(power_of("G + G"), Some(1));
        // The cases the scaling path must refuse rather than guess at.
        assert_eq!(power_of("G + G**2"), None);
        assert_eq!(power_of("1 + G"), None);
        assert_eq!(power_of("cmath.sqrt(G)"), None);
        assert_eq!(power_of("G**2 - G"), None);
        assert_eq!(power_of("aS"), None);
        assert_eq!(power_of("complex(0,1)*aS"), None);
    }

    #[test]
    fn the_strong_coupling_is_the_only_parameter_alpha_s_drives() {
        // The scaling path's factor is `√(αs/αs_ref)`, which is only the ratio of
        // couplings because `G` is the whole of `aS`'s influence on the model.
        let model = sm_model(SMRestrict::Default);
        let driven = model.params.dependents("aS");
        assert_eq!(
            driven.iter().map(String::as_str).collect::<Vec<_>>(),
            ["G"],
            "a second aS-driven parameter would invalidate every G-power tag"
        );
    }

    fn compile(process: &str, model: &UFOModel) -> AmplitudeEvaluator {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let sets: Vec<DiagramSet> = generate_from_proc_card(&card, model).unwrap();
        AmplitudeEvaluator::compile(&sets[0], model).unwrap()
    }

    fn points(n_ext: usize, count: usize) -> Vec<Vec<LorentzVector<f64>>> {
        let mut rng = StdRng::seed_from_u64(0x5CA1E);
        (0..count)
            .map(|_| rambo_massless(500.0, n_ext - 2, &mut rng))
            .collect()
    }

    /// The scaling path must reproduce a full model re-evaluation, entry by entry.
    #[test]
    fn scaling_reproduces_the_reference_path() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        for process in ["g g > g g", "g g > t t~", "u u~ > u u~"] {
            let eval = compile(process, &model);
            let mut fast = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
            assert!(
                fast.fallback().is_none(),
                "{process}: {:?}",
                fast.fallback()
            );
            assert!(
                fast.depends_on_alpha_s(),
                "{process} has no strong coupling"
            );

            for k in 1..=20 {
                let alpha_s = 0.05 + 0.01 * k as f64;
                fast.set_alpha_s(alpha_s);
                let mut reference = evaluated.clone();
                reference.set_alpha_s(alpha_s);
                let bound = BoundAmplitude::<f64>::bind(&eval, &reference);
                let (fc, ff) = fast.amplitude().pools();
                let (rc, rf) = bound.pools();
                for (i, (a, b)) in fc.iter().zip(rc.iter()).enumerate() {
                    assert!(
                        close(a.re, b.re) && close(a.im, b.im),
                        "{process}: complex entry {i} at aS={alpha_s}: {a} vs {b}"
                    );
                }
                for (i, (a, b)) in ff.iter().zip(rf.iter()).enumerate() {
                    assert!(close(*a, *b), "{process}: real entry {i}: {a} vs {b}");
                }
            }
        }
    }

    /// At the card's own coupling the pools must be untouched, bit for bit — this is
    /// what keeps every existing amplitude comparison exactly where it was.
    #[test]
    fn returning_to_the_reference_coupling_restores_the_pools_exactly() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let eval = compile("g g > g g", &model);
        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let mut fast = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);

        fast.set_alpha_s(0.37);
        fast.set_alpha_s(evaluated.alpha_s().unwrap());
        let (fc, ff) = fast.amplitude().pools();
        let (bc, bf) = bound.pools();
        for (a, b) in fc.iter().zip(bc.iter()) {
            assert_eq!(
                (a.re.to_bits(), a.im.to_bits()),
                (b.re.to_bits(), b.im.to_bits())
            );
        }
        for (a, b) in ff.iter().zip(bf.iter()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    /// A process with no strong coupling never needs a coupling value, and its pools
    /// stay where they were bound whatever it is told.
    #[test]
    fn a_qcd_free_amplitude_ignores_the_coupling() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let eval = compile("e+ e- > mu+ mu-", &model);
        let mut amp = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
        assert!(!amp.depends_on_alpha_s());
        assert!(amp.fallback().is_none());
        let before: Vec<C<f64>> = amp.amplitude().pools().0.to_vec();
        amp.set_alpha_s(0.42);
        assert_eq!(amp.amplitude().pools().0, before.as_slice());
    }

    /// The SM with one coupling rewritten to a sum of two powers of `G`: the analysis
    /// must refuse to tag it, and the amplitude must then track a full re-evaluation
    /// that a single-power rescale would miss by far more than rounding.
    #[test]
    fn a_non_monomial_coupling_forces_the_reference_path() {
        let mut parsed: ParsedModel = crate::ufo::sm::sm_parsed_model();
        let gc10 = parsed.couplings.get_mut("GC_10").expect("GC_10");
        assert_eq!(gc10.value, parse_expr("-G").unwrap());
        gc10.value = parse_expr("-G - G**2/10").unwrap();
        let card = SMRestrict::Default.restrict_card_text().parse().unwrap();
        let doctored: Arc<UFOModel> = Arc::new(parsed.into_model(Some(&card)).unwrap());

        let evaluated = EvaluatedModel::from_model(doctored.clone());
        let eval = compile("g g > t t~", &doctored);
        let mut amp = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
        assert_eq!(
            amp.fallback(),
            Some(&RescaleFallback::ComplexEntry(
                first_untagged_complex(&amp).expect("a complex entry must be untagged")
            )),
            "a sum of unequal powers of G must not be tagged"
        );

        // What the reference path gives, and what a monomial rescale would have given.
        let alpha_s = 0.30;
        let ratio = (alpha_s / evaluated.alpha_s().unwrap()).sqrt();
        let base: Vec<C<f64>> = amp.amplitude().pools().0.to_vec();
        amp.set_alpha_s(alpha_s);
        let mut reference = evaluated.clone();
        reference.set_alpha_s(alpha_s);
        let reference_bound = BoundAmplitude::<f64>::bind(&eval, &reference);
        let exact: Vec<C<f64>> = reference_bound.pools().0.to_vec();

        let (got, _) = amp.amplitude().pools();
        assert_eq!(got, exact.as_slice(), "the reference path must be exact");

        let worst = base
            .iter()
            .zip(exact.iter())
            .filter(|(b, _): &(&C<f64>, &C<f64>)| b.norm() > 0.0)
            .map(|(b, e): (&C<f64>, &C<f64>)| {
                let guess = *b * ratio;
                (guess - *e).norm() / e.norm().max(f64::MIN_POSITIVE)
            })
            .fold(0.0f64, f64::max);
        assert!(
            worst > 1e-3,
            "the fallback must be load-bearing: a power-1 rescale is off by only {worst:e}"
        );
    }

    fn first_untagged_complex<F: Real + FromPrimitive>(
        amp: &ScaleAwareAmplitude<'_, F>,
    ) -> Option<usize> {
        amp.plan.powers_c.iter().position(Option::is_none)
    }

    /// Per-thread ownership: each thread forks its own pools, sets its own coupling,
    /// and gets the answer it would have got alone.
    #[test]
    fn forks_do_not_share_pools() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let eval = compile("g g > g g", &model);
        let seed = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
        let pts = points(4, 4);

        let alphas = [0.09, 0.118, 0.15, 0.22];
        let expected: Vec<f64> = alphas
            .iter()
            .map(|&a| {
                let mut solo = seed.fork();
                solo.set_alpha_s(a);
                let mut scratch = solo.scratch_space();
                solo.eval_m2(&pts[0], &mut scratch)
            })
            .collect();

        let got: Vec<f64> = std::thread::scope(|s| {
            let handles: Vec<_> = alphas
                .iter()
                .map(|&a| {
                    let mut mine = seed.fork();
                    let pts = &pts;
                    s.spawn(move || {
                        let mut scratch = mine.scratch_space();
                        let mut last = 0.0;
                        for _ in 0..200 {
                            mine.set_alpha_s(a);
                            last = mine.eval_m2(&pts[0], &mut scratch);
                            mine.set_alpha_s(a * 1.7);
                            mine.eval_m2(&pts[1], &mut scratch);
                        }
                        last
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert_eq!(got, expected);

        // The amplitude the forks came from never moved.
        assert_eq!(seed.alpha_s(), seed.alpha_s_ref());
    }

    /// The real pool is not assumed to be independent of the strong coupling.
    ///
    /// Nothing in the Standard Model's `consts_f` moves with `aS`, so every comparison
    /// against the reference path would pass even if the real pool were never tagged.
    /// Declaring a mass parameter driven by the coupling is what shows the analysis
    /// actually looks at it: the entry must go untagged and the amplitude must fall
    /// back to re-evaluating the model rather than leave the mass where it was.
    #[test]
    fn a_coupling_driven_mass_is_refused_rather_than_left_behind() {
        let model = sm_model(SMRestrict::Default);
        let eval = compile("g g > t t~", &model);
        let honest = {
            let mut driven = model.params.dependents("aS");
            driven.insert("aS".to_owned());
            eval.folded().g_powers(&model, &driven)
        };
        assert!(
            honest.real.iter().all(Option::is_some),
            "no Standard Model mass or width moves with the coupling"
        );

        let mut driven = model.params.dependents("aS");
        driven.extend(["aS".to_owned(), "MT".to_owned()]);
        let moved = eval.folded().g_powers(&model, &driven);
        assert!(
            moved.real.iter().any(Option::is_none),
            "a mass parameter driven by the coupling must leave its pool entry untagged"
        );
    }

    #[test]
    fn a_scale_aware_amplitude_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ScaleAwareAmplitude<'static, f64>>();
    }

    /// Two pool values agree to within a rounding of each other. The two paths
    /// evaluate the same quantity by different floating-point routes, so they are not
    /// bit-identical; a mistagged power would be wrong by the ratio itself.
    fn close(a: f64, b: f64) -> bool {
        if a == b {
            return true;
        }
        (a - b).abs() <= 4.0 * f64::EPSILON * a.abs().max(b.abs())
    }
}
