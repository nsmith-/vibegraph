//! The running strong coupling ([`vibegraph::coupling::alphas`]) against
//! MadGraph's own Fortran, iterate-for-iterate.
//!
//! [`fortran_reference_matches_the_iterate`] replays
//! `validation/alphas/reference.csv`, produced by linking MadGraph's unmodified
//! `Source/alfas_functions.f` against a driver
//! (`pixi run -e madgraph generate-alphas-reference`). Both sides run the same
//! Newton iteration to the same `TOL = 5e-4`, and the comparison is at the
//! *iterate* level: a wrong branch, iteration count, or coefficient shows up at
//! the Newton tolerance scale (~1e-4 relative), which the few-ulp bound here
//! sits eleven orders of magnitude below. What the bound tolerates is only the
//! transcendentals' last-ulp dependence on the host libm — the committed grid
//! was generated on one platform, and pinning its exact bits would make this a
//! test of the machine, not of the coupling. The grid is committed, so this
//! runs on a bare clone.
//!
//! **What this cannot see.** Nothing about where `asmz` and `nloop` come from —
//! the grid supplies them directly. That is the job of the per-event oracles in
//! `validate_alphas.rs` and of the unit tests in the module itself.

use std::path::{Path, PathBuf};

use vibegraph::coupling::alphas::{NLoop, RunningAlphaS, BMASS, CMASS, ZMASS};

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation")
}

/// One row of the Fortran-generated grid.
struct Row {
    asmz: f64,
    nloop: NLoop,
    q: f64,
    alphas: f64,
}

fn load_reference() -> Vec<Row> {
    let path = validation_dir().join("alphas/reference.csv");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e}\nregenerate with: pixi run -e madgraph generate-alphas-reference",
            path.display()
        )
    });
    let mut rows = Vec::new();
    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        assert_eq!(fields.len(), 4, "malformed reference row: {line}");
        rows.push(Row {
            asmz: fields[0].parse().expect("asmz"),
            nloop: NLoop::from_i64(fields[1].parse().expect("nloop")).expect("nloop in 1..=3"),
            q: fields[2].parse().expect("q"),
            alphas: fields[3].parse().expect("alphas"),
        });
    }
    assert!(!rows.is_empty(), "reference grid is empty");
    rows
}

/// The grid is only a net if it actually straddles what it claims to. A grid
/// confined to `Q > BMASS` would pass the comparison below while leaving both
/// flavour-threshold branches untested.
#[test]
fn reference_grid_straddles_every_branch() {
    let rows = load_reference();
    let count = |f: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count();

    assert!(count(&|r| r.q < CMASS) >= 4, "no nf = 3 coverage");
    assert!(
        count(&|r| r.q >= CMASS && r.q < BMASS) >= 4,
        "no nf = 4 coverage"
    );
    assert!(
        count(&|r| r.q >= BMASS && r.q < ZMASS) >= 4,
        "no nf = 5 coverage below M_Z"
    );
    assert!(count(&|r| r.q > ZMASS) >= 4, "no coverage above M_Z");
    // The two flavour thresholds are the branch points, so they are sampled from
    // immediately either side: a branch keyed on the wrong comparison would
    // otherwise have a whole decade to hide in.
    for threshold in [CMASS, BMASS] {
        assert!(
            count(&|r| (r.q - threshold).abs() < 1e-6 * threshold && r.q < threshold) >= 1
                && count(&|r| (r.q - threshold).abs() < 1e-6 * threshold && r.q > threshold) >= 1,
            "threshold {threshold} is not sampled from immediately either side"
        );
    }
    assert!(
        count(&|r| r.q == ZMASS) >= 1,
        "the reference scale itself is not sampled"
    );
    for nloop in [NLoop::One, NLoop::Two, NLoop::Three] {
        assert!(
            count(&|r| r.nloop == nloop) >= 4,
            "no coverage at nloop = {}",
            nloop.as_i64()
        );
    }
}

/// Same Newton iterate as MadGraph's own `ALPHAS`, modulo host-libm ulps.
///
/// Measured cross-platform drift on the committed grid: 2 of 792 points at
/// 1 ulp. The bound leaves headroom over that while staying ~11 orders below
/// the ~1e-4-relative signature of a different iterate.
#[test]
fn fortran_reference_matches_the_iterate() {
    const MAX_ULPS: i64 = 4;

    let rows = load_reference();
    let mut mismatches = Vec::new();
    let mut worst_ulps = 0i64;

    for row in &rows {
        let running = RunningAlphaS::new(row.asmz, row.nloop).expect("positive asmz");
        let got = running.eval(row.q);
        let ulps = (got.to_bits() as i64 - row.alphas.to_bits() as i64).abs();
        worst_ulps = worst_ulps.max(ulps);
        if ulps > MAX_ULPS && mismatches.len() < 10 {
            mismatches.push(format!(
                "asmz={} nloop={} q={}: fortran {:.17e}, rust {:.17e} ({ulps} ulp)",
                row.asmz,
                row.nloop.as_i64(),
                row.q,
                row.alphas,
                got
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "{} of {} grid points differ beyond {MAX_ULPS} ulp (worst {worst_ulps}):\n{}",
        mismatches.len(),
        rows.len(),
        mismatches.join("\n")
    );
    println!(
        "alpha_s grid: {} points within {worst_ulps} ulp of the Fortran iterate",
        rows.len()
    );
}
