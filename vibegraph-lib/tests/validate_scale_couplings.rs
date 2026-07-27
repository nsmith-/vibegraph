//! Per-event strong coupling: the scaling path against the reference path, entry by
//! entry, for every process the MadGraph amplitude gate covers.
//!
//! [`ScaleAwareAmplitude`] moves a bound amplitude to a new `αs` either by scaling the
//! tagged powers of `G` in its constant pools or by re-evaluating the model through the
//! UFO parameter graph. The second is exact by construction and needs no reference data
//! to be trusted; the first is an optimisation of it. So the oracle here is internal and
//! as fine as the quantity allows: every constant-pool entry, at every sampled coupling,
//! rather than the |M|² the entries eventually feed.
//!
//! What this comparison cannot see: an error shared by both paths. The powers of `G` are
//! read off the same UFO expressions the reference path evaluates, so a *model* that
//! stated the wrong coupling would satisfy both. That class is covered by the amplitude
//! gate against MadGraph, which pins the pools at the card's own coupling.
//!
//! Each process is also checked to still bind bit-for-bit at the card's own coupling
//! after a round trip through other values, which is what keeps the amplitude gate
//! exactly where it was.

mod common;

use std::path::Path;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude, ScaleAwareAmplitude};
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// The processes the MadGraph amplitude gate covers, as (banked-output name, process).
const PROCESSES: [(&str, &str); 14] = [
    ("ee_to_mumu", "e+ e- > mu+ mu-"),
    ("pp_to_ll_qcd0", "u u~ > mu+ mu-"),
    ("ee_to_ee", "e+ e- > e+ e-"),
    ("ee_to_mumua", "e+ e- > mu+ mu- a"),
    ("ee_to_ttx", "e+ e- > t t~"),
    ("ee_to_wpwm", "e+ e- > w+ w-"),
    ("ee_to_zh", "e+ e- > z h"),
    ("ee_to_tatah", "e+ e- > ta+ ta- h"),
    ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
    ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
    ("bbx_to_ccx_emmm_qcd0", "b b~ > c c~ e+ e- mu+ mu- QCD=0"),
    ("uux_to_uux", "u u~ > u u~"),
    ("gg_to_ttx", "g g > t t~"),
    ("gg_to_gg", "g g > g g"),
];

/// Couplings sampled per process.
const ALPHA_S_SAMPLES: usize = 100;

/// The two paths are different floating-point routes to the same value, not the same
/// expression: the reference path rebuilds `-G`, `i·G²`, … from a freshly computed `G`,
/// while the scaling path multiplies the already-rounded entry by an already-rounded
/// ratio. Bit equality is therefore unreachable, and the comparison is in units of the
/// last place. It loses nothing: the error this oracle exists to catch — an entry
/// tagged with the wrong power of `G` — is wrong by a whole factor of the ratio, some
/// fifteen orders of magnitude above this bound. The observed worst over the sampled
/// range is 5, on the one entry carrying `G²`.
const MAX_ULP: i64 = 8;

/// Largest relative disagreement tolerated between the two paths' |M|².
const M2_REL_TOL: f64 = 1e-13;

fn param_card(name: &str) -> ParamCard {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/output")
        .join(name)
        .join("Cards/param_card.dat");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .unwrap_or_else(|| "".parse::<ParamCard>().unwrap())
}

/// Distance in units of the last place, with a large sentinel for a sign or
/// zero/non-zero mismatch that no ulp count describes.
fn ulps(a: f64, b: f64) -> i64 {
    if a.to_bits() == b.to_bits() {
        return 0;
    }
    if a.is_sign_negative() != b.is_sign_negative() {
        return i64::MAX;
    }
    (a.to_bits() as i64 - b.to_bits() as i64).abs()
}

#[test]
fn scaling_path_reproduces_the_reference_path() {
    let model = common::sm_model();
    let mut worst_overall = 0i64;

    for (name, process) in PROCESSES {
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(name));
        let alpha_s_ref = evaluated.alpha_s().expect("the SM carries aS");
        let sets = common::generate_with(process, model.as_ref());
        assert!(!sets.is_empty(), "{name}: no diagrams generated");
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).expect("compile");

        let mut fast = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
        let census = fast.census();
        assert!(
            fast.fallback().is_none(),
            "{name}: no Standard Model amplitude should need the reference path, got {}",
            fast.fallback().unwrap()
        );

        let n_ext = sets[0].particles_in.len() + sets[0].particles_out.len();
        let mut rng = StdRng::seed_from_u64(0xD3_5CA1E);
        let momenta: Vec<Vec<LorentzVector<f64>>> = (0..2)
            .map(|_| rambo_massless(500.0, n_ext - 2, &mut rng))
            .collect();
        let mut scratch = fast.scratch_space();

        let mut worst_ulp = 0i64;
        let mut worst_m2_rel = 0.0f64;
        for i in 0..ALPHA_S_SAMPLES {
            let alpha_s = rng.random_range(0.05..0.5);
            fast.set_alpha_s(alpha_s);

            let mut reference_model = evaluated.clone();
            reference_model.set_alpha_s(alpha_s);
            let reference = BoundAmplitude::<f64>::bind(&eval, &reference_model);

            let (fc, ff) = fast.amplitude().pools();
            let (rc, rf) = reference.pools();
            assert_eq!((fc.len(), ff.len()), (rc.len(), rf.len()));
            for (k, (a, b)) in fc.iter().zip(rc.iter()).enumerate() {
                let d = ulps(a.re, b.re).max(ulps(a.im, b.im));
                worst_ulp = worst_ulp.max(d);
                assert!(
                    d <= MAX_ULP,
                    "{name}: complex pool entry {k} at aS={alpha_s}: {a} vs {b} ({d} ulp)"
                );
            }
            for (k, (a, b)) in ff.iter().zip(rf.iter()).enumerate() {
                let d = ulps(*a, *b);
                worst_ulp = worst_ulp.max(d);
                assert!(
                    d <= MAX_ULP,
                    "{name}: real pool entry {k} at aS={alpha_s}: {a} vs {b} ({d} ulp)"
                );
            }

            // The pools are what |M|² is built from, so this adds no independent
            // information; it is the check that nothing else in the binding moved.
            if i < 2 {
                let p = &momenta[i];
                let got = fast.eval_m2(p, &mut scratch);
                let want = reference.eval_m2(p, &mut scratch);
                let rel = (got - want).abs() / want.abs().max(f64::MIN_POSITIVE);
                worst_m2_rel = worst_m2_rel.max(rel);
                assert!(rel <= M2_REL_TOL, "{name}: |M|^2 {got:e} vs {want:e}");
            }
        }

        // Back at the card's own coupling, the pools must be the bound ones bit for bit.
        fast.set_alpha_s(alpha_s_ref);
        let at_card = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let (fc, ff) = fast.amplitude().pools();
        let (bc, bf) = at_card.pools();
        for (k, (a, b)) in fc.iter().zip(bc.iter()).enumerate() {
            assert_eq!(
                (a.re.to_bits(), a.im.to_bits()),
                (b.re.to_bits(), b.im.to_bits()),
                "{name}: complex pool entry {k} moved at the card's own aS"
            );
        }
        for (k, (a, b)) in ff.iter().zip(bf.iter()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "{name}: real pool entry {k} moved at the card's own aS"
            );
        }

        worst_overall = worst_overall.max(worst_ulp);
        println!(
            "  [{name}] pool {}/{} tagged, {} carry a power of G (max {}), \
             aS_ref={alpha_s_ref}, worst {worst_ulp} ulp, |M|^2 rel {worst_m2_rel:.2e}",
            census.tagged, census.entries, census.scale_dependent, census.max_power
        );
    }

    println!("worst pool disagreement over all processes: {worst_overall} ulp");
}
