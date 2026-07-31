//! The HELAS kernel chain against the original Fortran77 HELAS routines,
//! compiled through f2py by `validation/helas/gen_reference.py`.
//!
//! The Fortran here is the ancestor library, not MadGraph's generated code, so
//! this is the one comparison that pins the kernels themselves rather than their
//! composition. The composition is pinned separately and hermetically in
//! `helas_kernel_composition.rs`, against a hand-built chain of the same kernels.
//!
//! The reference grid (`validation/helas/reference.csv`) is committed, so this
//! runs on a bare clone and needs neither gfortran nor f2py. Regenerating it —
//! which recompiles the Fortran — is the one step that does:
//!
//!     pixi run -e helas-validation generate-helas

mod common;

// SM parameters matching MadGraph's default param_card.dat, as used by the
// reference generator.
const MDL_ME: f64 = 0.000_511;
const MDL_MMU: f64 = 0.105_658;

use std::path::Path;
use vibegraph::{helas::LorentzVector, ufo::EvaluatedModel};

fn compute_m2_ee_mumu_dynamic(sqrt_s: f64, cos_theta: f64) -> f64 {
    use common::{generate_with, sm_lepton_masses_model};
    use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
    use vibegraph::ufo::slha::ParamCard;

    // Same situation as test_eval_m2_ee_mumu_vs_hardcoded
    let model = sm_lepton_masses_model();
    let sets = generate_with("e+ e- > mu+ mu-", &model);
    let set = &sets[0];
    let card = format!(
        "Block MASS\n 11 {}\n 13 {}\nBlock YUKAWA\n 11 0.0\n 13 0.0\n",
        MDL_ME, MDL_MMU
    )
    .parse::<ParamCard>()
    .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let me = evaluated.mass(model.particle_id("e-").unwrap());
    let mmu = evaluated.mass(model.particle_id("mu-").unwrap());

    let evaluator =
        AmplitudeEvaluator::compile(set, &model).expect("failed to compile amplitude evaluator");

    let e_beam = sqrt_s / 2.0;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let p3_e = (e_beam * e_beam - me * me).max(0.0).sqrt();
    let p3_mu = (e_beam * e_beam - mmu * mmu).max(0.0).sqrt();
    let momenta = vec![
        LorentzVector::new(e_beam, 0.0, 0.0, -p3_e),
        LorentzVector::new(e_beam, 0.0, 0.0, p3_e),
        LorentzVector::new(e_beam, -p3_mu * sin_theta, 0.0, -p3_mu * cos_theta),
        LorentzVector::new(e_beam, p3_mu * sin_theta, 0.0, p3_mu * cos_theta),
    ];
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    bound.eval_m2(&momenta, &mut bound.scratch_space())
}

/// Relative tolerance for comparing Rust |M|² against Fortran reference.
/// TODO: Understand why we don't achieve machine precision as we do in validate_helas_mg
const REL_TOL: f64 = 3e-6;

/// Parse the CSV reference file and return rows as (sqrt_s, cos_theta, M2).
fn read_reference_csv(path: &Path) -> Vec<(f64, f64, f64)> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", path.display(), e));

    content
        .lines()
        .skip(1) // header: sqrt_s_GeV,cos_theta,M2_summed
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.split(',').collect();
            assert_eq!(cols.len(), 3, "Expected 3 columns in: {line}");
            let sqrt_s: f64 = cols[0].trim().parse().expect("bad sqrt_s");
            let cos_theta: f64 = cols[1].trim().parse().expect("bad cos_theta");
            let m2: f64 = cols[2].trim().parse().expect("bad M2");
            (sqrt_s, cos_theta, m2)
        })
        .collect()
}

#[test]
fn helas_matches_fortran_reference() {
    let csv_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/helas/reference.csv");

    let rows = read_reference_csv(&csv_path);
    assert!(!rows.is_empty(), "reference.csv is empty");

    let mut failures = 0usize;
    for (sqrt_s, cos_theta, m2_ref) in &rows {
        let m2_rust = compute_m2_ee_mumu_dynamic(*sqrt_s, *cos_theta);
        let rel_diff = if *m2_ref != 0.0 {
            (m2_rust - m2_ref).abs() / m2_ref.abs()
        } else {
            m2_rust.abs()
        };

        if rel_diff > REL_TOL {
            eprintln!(
                "FAIL  sqrt_s={sqrt_s:.3} cos_θ={cos_theta:.3}: \
                 Rust={m2_rust:.8e}  Fortran={m2_ref:.8e}  rel_diff={rel_diff:.2e}"
            );
            failures += 1;
        }
    }

    assert_eq!(
        failures,
        0,
        "{failures}/{} points exceeded relative tolerance {REL_TOL:.0e}",
        rows.len()
    );
}
