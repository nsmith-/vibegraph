//! Microbenchmarks of the Lorentz-algebra leaf kernels the evaluator is built from,
//! timed through their public API so a change to a kernel shows up here directly.
//!
//! Each kernel runs in two shapes, for scalar `f64` and a 4-wide `LaneField`:
//! - `throughput`: `M` independent calls, the way the dispatch loop presents
//!   independent instructions. Measures the kernel's instruction cost.
//! - `chain`: each call's result feeds the next call's input, so the loop time is
//!   the kernel's critical-path latency from that input. The dispatch loop mostly
//!   hides that latency behind independent work, so a `chain` win is a kernel
//!   property, not a promised end-to-end speedup.
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
    epsilon4, AsymRank2Tensor, Bispinor, Bra, ComplexVector, Contravariant, Covariant, Ket,
    LorentzVector, Multivector, SpinorRepr, VectorRepr,
};
use vibegraph::helas::repr::numbers::Chirality;
use vibegraph::helas::repr::{r, Real, C};

const M: usize = 1024;

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

fn complex<F: Field>(rng: &mut StdRng, half_width: f64) -> C<F> {
    C::new(F::draw(rng, half_width), F::draw(rng, half_width))
}

fn components<F: Field, const K: usize>(rng: &mut StdRng, half_width: f64) -> [C<F>; K] {
    std::array::from_fn(|_| complex(rng, half_width))
}

fn cvec<F: Field>(rng: &mut StdRng, half_width: f64) -> ComplexVector<F, Contravariant> {
    ComplexVector::new(components(rng, half_width))
}

fn tensor<F: Field>(rng: &mut StdRng, half_width: f64) -> AsymRank2Tensor<F> {
    AsymRank2Tensor::new(components(rng, half_width))
}

fn multivector<F: Field>(rng: &mut StdRng, half_width: f64) -> Multivector<F> {
    Multivector::new(
        complex(rng, half_width),
        cvec(rng, half_width),
        tensor(rng, half_width),
        cvec(rng, half_width),
        complex(rng, half_width),
    )
}

/// Adds `z` to every component: how a chain feeds one call's scalar result into
/// the next call's input.
fn shift<F: Real, const K: usize>(x: &[C<F>; K], z: C<F>) -> [C<F>; K] {
    x.map(|c| c + z)
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

/// A kernel with a complex-scalar result. The chain maps `z ↦ S · kernel(x + z)`,
/// with `perturb` adding `z` to the inputs the chain runs through; `S` is chosen
/// per kernel so the map contracts to O(1) values.
fn bench_scalar<F: Field, X>(
    c: &mut Criterion,
    name: &str,
    data: &[X],
    s: f64,
    perturb: impl Fn(&X, C<F>) -> X,
    kernel: impl Fn(&X) -> C<F>,
) {
    let s = r(F::from(s).unwrap());
    let chain = || {
        let mut z = C::new(F::zero(), F::zero());
        for x in data {
            z = kernel(black_box(&perturb(x, z))) * s;
        }
        z
    };
    assert_moderate(name, chain().norm_sqr());

    let mut g = c.benchmark_group(name);
    g.bench_function("throughput", |bch| {
        bch.iter(|| {
            for x in black_box(data) {
                black_box(kernel(x));
            }
        })
    });
    g.bench_function("chain", |bch| bch.iter(chain));
    g.finish();
}

/// A kernel mapping a state through an operator, `ψ ↦ op(ψ)`. The chain applies
/// the `M` operators in turn, each multiplying |ψ| by a varying factor; the
/// operators are scaled by the inverse of the measured mean growth, so |ψ|
/// performs a random walk around 1 instead of drifting to overflow or subnormals.
fn bench_map<F: Field, S: Copy, P: Copy + std::ops::Mul<F, Output = P>>(
    c: &mut Criterion,
    name: &str,
    states: &[S],
    ops: &[P],
    norm: impl Fn(&S) -> F,
    renormalize: impl Fn(&S, F) -> S,
    kernel: impl Fn(&S, &P) -> S,
) {
    let (mut st, mut log_growth) = (states[0], 0.0);
    for op in ops {
        st = kernel(&st, op);
        let n = norm(&st);
        log_growth += n.lanes()[0].ln();
        st = renormalize(&st, n);
    }
    let scale = F::from((-log_growth / ops.len() as f64).exp()).unwrap();
    let chain_ops: Vec<P> = ops.iter().map(|&op| op * scale).collect();
    let chain = || {
        let mut st = black_box(states[0]);
        for op in &chain_ops {
            st = kernel(&st, op);
        }
        st
    };
    assert_moderate(name, norm(&chain()));

    let mut g = c.benchmark_group(name);
    g.bench_function("throughput", |bch| {
        bch.iter(|| {
            for (st, op) in black_box(states).iter().zip(black_box(ops)) {
                black_box(kernel(st, op));
            }
        })
    });
    g.bench_function("chain", |bch| bch.iter(chain));
    g.finish();
}

// Each kernel's inputs are a one-off tuple, spelled out where it is drawn.
#[allow(clippy::type_complexity)]
fn bench_field<F: Field>(c: &mut Criterion, field: &str) {
    let mut rng = StdRng::seed_from_u64(0xD07);
    let name = |kernel: &str| format!("{kernel}/{field}");

    // `ComplexVector::dot`: the chain runs through the contravariant side.
    let data: Vec<([C<F>; 4], ComplexVector<F, Covariant>)> = (0..M)
        .map(|_| {
            (
                components(&mut rng, 1.0),
                ComplexVector::new(components(&mut rng, 0.5)),
            )
        })
        .collect();
    bench_scalar(
        c,
        &name("dot"),
        &data,
        0.25,
        |(a, b), z| (shift(a, z), *b),
        |(a, b)| ComplexVector::<F, Contravariant>::new(*a).dot(b),
    );

    // `ComplexVector::dot_lorentz`: the chain runs through the complex side.
    let data: Vec<([C<F>; 4], LorentzVector<F>)> = (0..M)
        .map(|_| {
            let p: [F; 4] = std::array::from_fn(|_| F::draw(&mut rng, 0.5));
            (
                components(&mut rng, 1.0),
                LorentzVector::new(p[0], p[1], p[2], p[3]),
            )
        })
        .collect();
    bench_scalar(
        c,
        &name("dot_lorentz"),
        &data,
        0.25,
        |(a, p), z| (shift(a, z), *p),
        |(a, p)| ComplexVector::new(*a).dot_lorentz(p),
    );

    // `scalar_bilinear(Both)`: the chain runs through the ket.
    let data: Vec<(Bispinor<F, Bra>, [C<F>; 4])> = (0..M)
        .map(|_| {
            let fo = Bispinor::<F, Ket>::from_components(components(&mut rng, 0.5)).bar();
            (fo, components(&mut rng, 1.0))
        })
        .collect();
    bench_scalar(
        c,
        &name("scalar_bilinear"),
        &data,
        0.25,
        |(fo, fi), z| (*fo, shift(fi, z)),
        |(fo, fi)| fo.scalar_bilinear(&Bispinor::from_components(*fi), Chirality::Both),
    );

    // `epsilon4`: the chain runs through the last vector only, so its latency is
    // that of the final contraction against the three-vector cofactors.
    let data: Vec<([ComplexVector<F>; 3], [C<F>; 4])> = (0..M)
        .map(|_| {
            let abc = std::array::from_fn(|_| cvec(&mut rng, 0.5));
            (abc, components(&mut rng, 1.0))
        })
        .collect();
    bench_scalar(
        c,
        &name("epsilon4"),
        &data,
        0.25,
        |(abc, d), z| (*abc, shift(d, z)),
        |([a, b, cc], d)| epsilon4(a, b, cc, &ComplexVector::new(*d)),
    );

    // `AsymRank2Tensor::contract`: the chain runs through one tensor.
    let data: Vec<([C<F>; 6], AsymRank2Tensor<F>)> = (0..M)
        .map(|_| (components(&mut rng, 1.0), tensor(&mut rng, 0.5)))
        .collect();
    bench_scalar(
        c,
        &name("tensor_contract"),
        &data,
        0.1,
        |(t, u), z| (shift(t, z), *u),
        |(t, u)| AsymRank2Tensor::new(*t).contract(u),
    );

    // `AsymRank2Tensor::contract_vectors`: the chain runs through both vectors.
    let data: Vec<(AsymRank2Tensor<F>, [C<F>; 4], [C<F>; 4])> = (0..M)
        .map(|_| {
            let t = tensor(&mut rng, 0.5);
            (t, components(&mut rng, 1.0), components(&mut rng, 1.0))
        })
        .collect();
    bench_scalar(
        c,
        &name("tensor_contract_vectors"),
        &data,
        0.05,
        |(t, a, b), z| (*t, shift(a, z), shift(b, z)),
        |(t, a, b)| t.contract_vectors(&ComplexVector::new(*a), &ComplexVector::new(*b)),
    );

    // `Multivector::fierz_pairing`: the chain runs through one multivector's
    // sixteen coefficients.
    let data: Vec<([C<F>; 16], Multivector<F>)> = (0..M)
        .map(|_| (components(&mut rng, 1.0), multivector(&mut rng, 0.5)))
        .collect();
    let mv = |x: &[C<F>; 16]| {
        let v = |o: usize| ComplexVector::new(std::array::from_fn(|k| x[o + k]));
        let t = AsymRank2Tensor::new(std::array::from_fn(|k| x[9 + k]));
        Multivector::new(x[0], v(1), t, v(5), x[15])
    };
    bench_scalar(
        c,
        &name("fierz_pairing"),
        &data,
        0.05,
        |(x, n), z| (shift(x, z), *n),
        |(x, n)| mv(x).fierz_pairing(n),
    );

    // `SpinorRepr::slash` on a ket.
    let psis: Vec<Bispinor<F, Ket>> = (0..M)
        .map(|_| Bispinor::from_components(components(&mut rng, 1.0)))
        .collect();
    let spinor_norm = |p: &Bispinor<F, Ket>| {
        (0..4)
            .map(|k| p.component(k).norm_sqr())
            .fold(F::zero(), |a, b| a + b)
            .sqrt()
    };
    let spinor_renormalize = |p: &Bispinor<F, Ket>, n: F| {
        Bispinor::from_components(std::array::from_fn(|k| p.component(k) / r(n)))
    };
    let vs: Vec<ComplexVector<F>> = (0..M).map(|_| cvec(&mut rng, 1.0)).collect();
    bench_map(
        c,
        &name("slash"),
        &psis,
        &vs,
        spinor_norm,
        spinor_renormalize,
        |psi, v| psi.slash(v),
    );

    // `SpinorRepr::apply` on a ket: a Weyl-matrix build off the chain, then four
    // row contractions on it.
    let ms: Vec<Multivector<F>> = (0..M).map(|_| multivector(&mut rng, 0.5)).collect();
    bench_map(
        c,
        &name("apply"),
        &psis,
        &ms,
        spinor_norm,
        spinor_renormalize,
        |psi, m| psi.apply(m),
    );

    // `AsymRank2Tensor::contract_vector`: the chain runs through the vector.
    let vec_norm = |v: &ComplexVector<F>| {
        (0..4)
            .map(|k| v.component(k).norm_sqr())
            .fold(F::zero(), |a, b| a + b)
            .sqrt()
    };
    let ts: Vec<AsymRank2Tensor<F>> = (0..M).map(|_| tensor(&mut rng, 1.0)).collect();
    bench_map(
        c,
        &name("tensor_contract_vector"),
        &vs,
        &ts,
        vec_norm,
        |v, n| *v * (F::one() / n),
        |v, t| t.contract_vector(v),
    );
}

fn benches(c: &mut Criterion) {
    bench_field::<f64>(c, "f64");
    bench_field::<LaneField<4>>(c, "lanes4");
}

criterion_group!(lorentz_kernels, benches);
criterion_main!(lorentz_kernels);
