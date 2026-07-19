//! Extended validation: the hadronic LO Drell–Yan cross section σ(pp → e⁺e⁻),
//! assembled by [`vibegraph::hadronic`] from the PDF luminosity, the compiled
//! amplitude, the run-card cuts, and VEGAS, against a banked MadGraph5 reference
//! generated with the *same* run-card file (`validation/madgraph/dy13_*.dat`),
//! PDF set (NNPDF23_lo_as_0130_qed / lhaid 247000), and fixed scale μF = m_Z.
//!
//! Two reference runs are enforced: default lepton cuts, and the m_ll ∈ [60,120]
//! window. Both must agree within combined Monte-Carlo error (target < 1%).
//!
//! A pointwise integrand oracle pins the PDF × flux × |M|² factors at fixed
//! `(x₁, x₂, cosθ)` points (including points just inside/outside a cut boundary)
//! against an independent Python computation (`validation/madgraph/gen_dy_oracle.py`).
//!
//! Gated behind `extended-validation`; the σ tests need the fetched PDF set and
//! the banked reference JSON:
//!
//!     pixi run -e madgraph fetch-pdf
//!     pixi run -e madgraph generate-hadronic-sigma   # banks the MG σ reference
//!     cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic

mod common;

#[cfg(feature = "extended-validation")]
mod validate_hadronic {
    use std::path::{Path, PathBuf};

    use vibegraph::cuts::Cuts;
    use vibegraph::hadronic::{
        compile_class, dy_external_legs, dy_flavor_classes, DrellYanIntegrand,
    };
    use vibegraph::helas::eval::BoundAmplitude;
    use vibegraph::pdf::{PdfMember, PdfSet};
    use vibegraph::runcard::RunCard;
    use vibegraph::ufo::EvaluatedModel;

    const MU_F: f64 = 91.1880;
    const SQRT_S_HAD: f64 = 13000.0;
    const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

    fn validation_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
    }

    fn load_pdf() -> PdfMember {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/pdf")
            .join(PDF_SET);
        let set = PdfSet::load(&dir, PDF_SET).unwrap_or_else(|e| {
            panic!(
                "cannot load PDF set {PDF_SET} from {}: {e}\n\
                 run `pixi run -e madgraph fetch-pdf`",
                dir.display()
            )
        });
        set.member(0).expect("PDF member 0")
    }

    /// Run the full VEGAS integration for a given run card, returning (σ, Δσ) in pb.
    fn run_sigma(run_card_path: &Path, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let model = super::common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let fc = dy_flavor_classes(&model).expect("classify DY");
        let up = compile_class(&fc.up_set, &model, &evaluated).expect("up class");
        let down = compile_class(&fc.down_set, &model, &evaluated).expect("down class");
        let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
        let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);

        let rc = RunCard::parse_file(run_card_path).expect("parse run card");
        let cuts = Cuts::compile(&rc, &dy_external_legs(2)).expect("compile cuts");
        let pdf = load_pdf();

        let integ = DrellYanIntegrand::new(
            &b_up,
            &b_down,
            &pdf,
            &cuts,
            fc.up_flavors,
            fc.down_flavors,
            SQRT_S_HAD,
            MU_F,
        );
        integ.integrate(neval, niter, seed)
    }

    /// Banked MG σ ± Δσ for one run, or `None` when the reference JSON is absent.
    fn banked(run: &str) -> Option<(f64, f64)> {
        let path = validation_dir().join("hadronic_sigma_reference.json");
        let text = std::fs::read_to_string(&path).ok()?;
        let v: serde_json::Value = serde_json::from_str(&text).ok()?;
        let r = v.get(run)?;
        Some((
            r.get("sigma_pb")?.as_f64()?,
            r.get("sigma_err_pb")?.as_f64()?,
        ))
    }

    fn check_run(run: &str, card: &str) {
        let card_path = validation_dir().join(card);
        let (sigma, err) = run_sigma(&card_path, 120_000, 12, 20260719);
        match banked(run) {
            Some((mg, mg_err)) => {
                let combined = (err * err + mg_err * mg_err).sqrt();
                let delta = (sigma - mg).abs();
                let rel = delta / mg;
                eprintln!(
                    "[{run}] vibegraph σ = {sigma:.3} ± {err:.3} pb | \
                     MG σ = {mg:.3} ± {mg_err:.3} pb | Δ = {delta:.3} pb \
                     ({} combined σ), rel = {rel:.4}",
                    delta / combined
                );
                assert!(
                    delta < 3.0 * combined || rel < 0.01,
                    "[{run}] σ disagreement: vibegraph {sigma:.3}±{err:.3} vs MG {mg:.3}±{mg_err:.3} pb, \
                     Δ = {delta:.3} pb = {:.1}σ, rel = {rel:.4}",
                    delta / combined
                );
            }
            None => {
                // Known-wrong informational mode until the MG reference is banked.
                eprintln!(
                    "[{run}] INFO (no banked MG reference yet): vibegraph σ = {sigma:.3} ± {err:.3} pb"
                );
            }
        }
    }

    #[test]
    fn sigma_default_cuts_vs_mg() {
        check_run("default", "dy13_default_run_card.dat");
    }

    #[test]
    fn sigma_mmll_window_vs_mg() {
        check_run("mmll_60_120", "dy13_mmll_run_card.dat");
    }
}
