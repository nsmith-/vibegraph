//! Microbenchmarks of the Lorentz-algebra leaf kernels the evaluator is built from,
//! timed through their public API so a change to a kernel shows up here directly.
//!
//! Each kernel runs in two shapes, for scalar `f64` and a 4-wide `LaneField`:
//! - `throughput`: `M` independent calls, the way the dispatch loop presents
//!   independent instructions. Measures the kernel's instruction cost.
//! - `chain`: each call's result feeds every component of the next call's input,
//!   so the loop time is the kernel's critical-path latency. The dispatch loop
//!   mostly hides that latency behind independent work, so a `chain` win is a
//!   kernel property, not a promised end-to-end speedup.
//!
//! Each chain is built to stay at O(1) magnitude over all `M` steps (asserted), so
//! it never times overflowed or subnormal arithmetic.
//!
//! Run: `cargo bench -p vibegraph-lib --bench lorentz_kernels`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use vibegraph::helas::eval::LaneField;
use vibegraph::helas::repr::lorentz::{
    Bispinor, ComplexVector, Contravariant, Covariant, Ket, SpinorRepr, VectorRepr,
};
use vibegraph::helas::repr::{r, Real, C};

const M: usize = 1024;

/// One `dot` input: the contravariant side's components (the chain perturbs
/// them), and the covariant side.
type DotPair<F> = ([C<F>; 4], ComplexVector<F, Covariant>);

/// A scalar field the bench draws inputs for, with its lanes read back as `f64`
/// for the chain's range check.
trait Field: Real {
    fn draw(rng: &mut StdRng, half_width: f64) -> Self;
    fn lanes(self) -> Vec<f64>;
}

impl Field for f64 {
    fn draw(rng: &mut StdRng, half_width: f64) -> Self {
        rng.random_range(-half_width..half_width)
    }
    fn lanes(self) -> Vec<f64> {
        vec![self]
    }
}

impl Field for LaneField<4> {
    fn draw(rng: &mut StdRng, half_width: f64) -> Self {
        LaneField::from_array(std::array::from_fn(|_| {
            rng.random_range(-half_width..half_width)
        }))
    }
    fn lanes(self) -> Vec<f64> {
        self.to_array().to_vec()
    }
}

fn components<F: Field>(rng: &mut StdRng, half_width: f64) -> [C<F>; 4] {
    std::array::from_fn(|_| C::new(F::draw(rng, half_width), F::draw(rng, half_width)))
}

/// Panics unless every lane of `x` is a normal float of moderate size.
fn assert_moderate<F: Field>(what: &str, x: F) {
    for v in x.lanes() {
        assert!(
            v.is_finite() && (1e-100..1e100).contains(&v),
            "{what}: chain left the normal range ({v:e})"
        );
    }
}

fn bench_dot<F: Field>(c: &mut Criterion, name: &str) {
    let mut rng = StdRng::seed_from_u64(0xD07);
    // |b_j| ≤ 0.5·√2, so the chain map `a ↦ base + S·(a·b)` has gain below 1 and
    // contracts to O(1) values.
    const S: f64 = 0.25;
    let data: Vec<DotPair<F>> = (0..M)
        .map(|_| {
            let base = components(&mut rng, 1.0);
            (base, ComplexVector::new(components(&mut rng, 0.5)))
        })
        .collect();
    let s = r(F::from(S).unwrap());
    let chain = || {
        let mut z = C::new(F::zero(), F::zero());
        for (base, b) in &data {
            let a = ComplexVector::<F, Contravariant>::new(base.map(|x| x + s * z));
            z = black_box(&a).dot(b);
        }
        z
    };
    assert_moderate(&format!("dot/{name}"), chain().norm_sqr());

    let mut g = c.benchmark_group(format!("dot/{name}"));
    g.bench_function("throughput", |bch| {
        bch.iter(|| {
            for (a, b) in black_box(&data) {
                black_box(ComplexVector::<F, Contravariant>::new(*a).dot(b));
            }
        })
    });
    g.bench_function("chain", |bch| bch.iter(chain));
    g.finish();
}

fn bench_slash<F: Field>(c: &mut Criterion, name: &str) {
    let mut rng = StdRng::seed_from_u64(0x5_1A54);
    let psis: Vec<Bispinor<F, Ket>> = (0..M)
        .map(|_| Bispinor::from_components(components(&mut rng, 1.0)))
        .collect();
    let raw: Vec<[C<F>; 4]> = (0..M).map(|_| components(&mut rng, 1.0)).collect();
    let vs: Vec<ComplexVector<F, Contravariant>> =
        raw.iter().map(|v| ComplexVector::new(*v)).collect();

    // The chain `ψ ↦ v̸ ψ` multiplies |ψ| by a varying factor each step. Scale the
    // chain's vectors by the inverse of the measured mean growth, so |ψ| performs
    // a random walk around 1 instead of drifting to overflow or subnormals.
    let norm = |p: &Bispinor<F, Ket>| {
        (0..4)
            .map(|k| p.component(k).norm_sqr())
            .fold(F::zero(), |a, b| a + b)
            .sqrt()
    };
    let (mut psi, mut log_growth) = (psis[0], 0.0);
    for v in &vs {
        psi = psi.slash(v);
        let n = norm(&psi);
        log_growth += n.lanes()[0].ln();
        psi = Bispinor::from_components(std::array::from_fn(|k| psi.component(k) / r(n)));
    }
    let scale = r(F::from((-log_growth / M as f64).exp()).unwrap());
    let chain_vs: Vec<ComplexVector<F, Contravariant>> = raw
        .iter()
        .map(|v| ComplexVector::new(v.map(|x| x * scale)))
        .collect();
    let chain = || {
        let mut psi = black_box(psis[0]);
        for v in &chain_vs {
            psi = psi.slash(v);
        }
        psi
    };
    assert_moderate(&format!("slash/{name}"), norm(&chain()));

    let mut g = c.benchmark_group(format!("slash/{name}"));
    g.bench_function("throughput", |bch| {
        bch.iter(|| {
            for (psi, v) in black_box(&psis).iter().zip(black_box(&vs)) {
                black_box(psi.slash(v));
            }
        })
    });
    g.bench_function("chain", |bch| bch.iter(chain));
    g.finish();
}

fn benches(c: &mut Criterion) {
    bench_dot::<f64>(c, "f64");
    bench_dot::<LaneField<4>>(c, "lanes4");
    bench_slash::<f64>(c, "f64");
    bench_slash::<LaneField<4>>(c, "lanes4");
}

criterion_group!(lorentz_kernels, benches);
criterion_main!(lorentz_kernels);
