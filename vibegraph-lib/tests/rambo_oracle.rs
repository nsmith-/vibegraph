//! RAMBO's `uniforms -> (momenta, weight)` map replayed against a committed
//! fixture.
//!
//! `validation/rambo/dump_rambo_fixture.py` dumps the exact uniforms and the
//! momenta and weight an independent implementation produced from them; this
//! feeds the same uniforms through the Rust `rambo` and matches per component.
//! Deterministic and committed, so it runs on a bare clone — no Monte-Carlo run
//! is involved, which is what the flat-MC checks in `rambo_flat_mc.rs` are for.

use std::path::Path;

use serde::Deserialize;
use vibegraph::phasespace::rambo;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    sqrt_s: f64,
    masses: Vec<f64>,
    uniforms: Vec<f64>,
    momenta: Vec<[f64; 4]>,
    xi: f64,
    weight: f64,
}

fn load_fixture() -> Fixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/rambo/rambo_fixture.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&content).expect("malformed rambo fixture")
}

#[test]
fn replay_matches_python() {
    let fixture = load_fixture();
    assert!(!fixture.cases.is_empty(), "empty fixture");

    let mut worst_mom = 0.0f64;
    let mut worst_weight = 0.0f64;
    for case in &fixture.cases {
        let pt = rambo(case.sqrt_s, &case.masses, &case.uniforms);
        assert_eq!(
            pt.momenta.len(),
            case.momenta.len(),
            "[{}] momentum count",
            case.name
        );

        // xi replay (exact map, so a tight bound).
        let xi_rel = (pt.xi - case.xi).abs() / case.xi.abs().max(1.0);
        assert!(xi_rel <= 1e-13, "[{}] xi rel {xi_rel:.3e}", case.name);

        for (i, (got, want)) in pt.momenta.iter().zip(&case.momenta).enumerate() {
            let g = [got.e(), got.px(), got.py(), got.pz()];
            for (c, (&gc, &wc)) in g.iter().zip(want).enumerate() {
                let scale = want.iter().fold(0.0f64, |m, x| m.max(x.abs())).max(1.0);
                let rel = (gc - wc).abs() / scale;
                worst_mom = worst_mom.max(rel);
                assert!(
                    rel <= 1e-13,
                    "[{}] leg {i} comp {c}: {gc} vs {wc} rel {rel:.3e}",
                    case.name
                );
            }
        }

        let w_rel = (pt.weight - case.weight).abs() / case.weight.abs();
        worst_weight = worst_weight.max(w_rel);
        assert!(
            w_rel <= 1e-12,
            "[{}] weight {} vs {} rel {w_rel:.3e}",
            case.name,
            pt.weight,
            case.weight
        );
    }
    eprintln!(
        "replay oracle: {} cases, worst momentum rel {worst_mom:.3e}, worst weight rel {worst_weight:.3e}",
        fixture.cases.len()
    );
}
