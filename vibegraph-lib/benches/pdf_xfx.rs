//! Parton-density interpolation microbenchmark: one all-flavor reading against
//! the per-flavor calls a luminosity sum is otherwise made of.
//!
//! The grid is synthetic but production-shaped — `NNPDF23_lo_as_0130_qed`'s
//! 100 × 50 knots × 14 flavors in one Q² band, so the coefficient table is the
//! same 2.2 MB and the knot searches the same depth — which keeps the benchmark
//! hermetic (no fetched set) while measuring the same memory behaviour. The
//! probe points cycle through the grid rather than repeating one cell, so the
//! index lookups are not perfectly predicted.
//!
//! Run: `cargo bench -p vibegraph-lib --bench pdf_xfx`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use vibegraph::pdf::grid::SubGrid;
use vibegraph::pdf::{PdfMember, FLAVOR_SLOTS};

/// The flavor list an `lhagrid1` QED set carries.
const FLAVORS: [i32; 14] = [-6, -5, -4, -3, -2, -1, 21, 1, 2, 3, 4, 5, 6, 22];

const NX: usize = 100;
const NQ: usize = 50;

fn geomspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    let (la, lb) = (a.ln(), b.ln());
    (0..n)
        .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
        .collect()
}

fn member() -> PdfMember {
    let x = geomspace(1e-9, 1.0, NX);
    let q2 = geomspace(1.0, 1e8, NQ);
    let mut xf = vec![0.0; NX * NQ * FLAVORS.len()];
    for (ix, &xv) in x.iter().enumerate() {
        for (iq, &q2v) in q2.iter().enumerate() {
            for (ifl, &fl) in FLAVORS.iter().enumerate() {
                let (lx, lq) = (xv.ln(), q2v.ln());
                xf[(ix * NQ + iq) * FLAVORS.len() + ifl] =
                    (1.0 - xv).powi(3) * (0.2 * lx - 0.03 * lq + 0.01 * fl as f64).exp();
            }
        }
    }
    PdfMember::from_subgrids(vec![SubGrid {
        x,
        q2,
        flavors: FLAVORS.to_vec(),
        xf,
    }])
}

/// Probe points spread over the grid's interior, deterministic and off-knot.
fn probes(n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let a = (i as f64 + 0.37) / n as f64;
            let b = ((i * 7 + 3) % n) as f64 / n as f64;
            (
                1e-9_f64.powf(1.0 - a) * 0.5_f64.powf(a),
                10.0_f64.powf(0.3 + 7.0 * b),
            )
        })
        .collect()
}

fn bench_xfx(c: &mut Criterion) {
    let pdf = member();
    let points = probes(512);

    let mut group = c.benchmark_group("pdf_xfx");

    let mut row = [0.0; FLAVOR_SLOTS];
    let mut i = 0usize;
    group.bench_function("xfx_all", |b| {
        b.iter(|| {
            let (x, q2) = points[i % points.len()];
            i += 1;
            pdf.xfx_all(black_box(x), black_box(q2), &mut row);
            black_box(row[12])
        })
    });

    let mut j = 0usize;
    group.bench_function("xfx_q2_all_14_flavors", |b| {
        b.iter(|| {
            let (x, q2) = points[j % points.len()];
            j += 1;
            let mut acc = 0.0;
            for &fl in &FLAVORS {
                acc += pdf.xfx_q2(fl, black_box(x), black_box(q2));
            }
            acc
        })
    });

    let mut k = 0usize;
    group.bench_function("xfx_q2_single_flavor", |b| {
        b.iter(|| {
            let (x, q2) = points[k % points.len()];
            k += 1;
            pdf.xfx_q2(21, black_box(x), black_box(q2))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_xfx);
criterion_main!(benches);
