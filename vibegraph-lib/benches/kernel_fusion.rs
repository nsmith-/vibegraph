//! Operator-fusion microbenchmark: the FFV `[g_L, g_R]` vertex evaluated as the
//! interpreter's generic kernel chain vs the production fused kernels.
//!
//! Two vertex rootings are measured, each in three strategies:
//! - `generic_res` — the kernel chain with every intermediate materialized in a
//!   result buffer and children read back by index, mimicking the forward scan's
//!   per-node slot traffic (9 nodes; the real lowered vertex is ~14 because the
//!   per-structure `Coeff` factors and the two-level coupling sum are separate
//!   nodes here folded into `g_L`/`g_R`, so this chain *understates* the gap).
//! - `generic_direct` — the same kernel calls nested without materialization:
//!   isolates kernel arithmetic + call overhead from slot traffic.
//! - `fused` — the production `ffv_vout`/`ffv_iout` kernels: `g_L·(left term) +
//!   g_R·(right term)` in one node. Besides collapsing the `Mul`/`Add`
//!   scaffolding, they skip the arithmetic the generic chain wastes on
//!   structurally-zero chiral halves (`GammaVout` computes both currents even
//!   when its input was just projected).
//!
//! Run: `cargo bench -p vibegraph-lib --features bench-internals --bench kernel_fusion`

use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::Zero;

use vibegraph::helas::eval::bench_internals::{
    ffv_iout, ffv_vout, gamma_iout, gamma_vout, mul_apply, proj_m, proj_p, rand_bra, rand_c,
    rand_ket, rand_vector, seeded_rng, slots_approx_eq, WaveformSlot,
};
use vibegraph::helas::repr::lorentz::{LorentzVector, VectorRepr};
use vibegraph::helas::repr::C;
use vibegraph::helas::wavefn::ScalarWf;

type Slot = WaveformSlot<f64>;

/// One FFV vertex input set: bra/ket fermions, a vector, and the per-chirality
/// effective couplings as scalar slots (`Σ coupling·coeff` per chiral structure,
/// as constant folding would produce them at bind time).
struct FfvInput {
    bra: Slot,
    ket: Slot,
    v: Slot,
    gl: Slot,
    gr: Slot,
}

fn coupling_slot(g: C<f64>) -> Slot {
    WaveformSlot::Scalar(ScalarWf {
        value: g,
        momentum: LorentzVector::zero(),
    })
}

fn gen_inputs(n: usize, seed: u64) -> Vec<FfvInput> {
    let mut rng = seeded_rng(seed);
    (0..n)
        .map(|_| FfvInput {
            bra: rand_bra(&mut rng),
            ket: rand_ket(&mut rng),
            v: rand_vector(&mut rng),
            gl: coupling_slot(rand_c(&mut rng)),
            gr: coupling_slot(rand_c(&mut rng)),
        })
        .collect()
}

// ─────────────────────────── FFV → off-shell vector current ───────────────────────────
//
// The `Add(Mul(g_L, GammaVout(bra, ProjM(ket))), Mul(g_R, GammaVout(bra, ProjP(ket))))`
// subgraph (e.g. the Z current in ee→μμ).

fn vout_generic_res(inp: &FfvInput, res: &mut Vec<Slot>) -> Slot {
    res.clear();
    res.push(inp.gl); // 0: Coupling g_L
    res.push(inp.gr); // 1: Coupling g_R
    let n = proj_m(&inp.ket);
    res.push(n); // 2
    let n = proj_p(&inp.ket);
    res.push(n); // 3
    let n = gamma_vout(&inp.bra, &res[2]);
    res.push(n); // 4
    let n = gamma_vout(&inp.bra, &res[3]);
    res.push(n); // 5
    let n = mul_apply([res[0], res[4]]);
    res.push(n); // 6
    let n = mul_apply([res[1], res[5]]);
    res.push(n); // 7
    let n = res[6] + res[7];
    res.push(n); // 8: Add
    res[8]
}

fn vout_generic_direct(inp: &FfvInput) -> Slot {
    let tl = mul_apply([inp.gl, gamma_vout(&inp.bra, &proj_m(&inp.ket))]);
    let tr = mul_apply([inp.gr, gamma_vout(&inp.bra, &proj_p(&inp.ket))]);
    tl + tr
}

// ─────────────────────────── FFV → continuing fermion (ket) current ───────────────────────────
//
// The `Add(Mul(g_L, GammaIout(v, ProjM(ket))), Mul(g_R, GammaIout(v, ProjP(ket))))`
// subgraph (a fermion line absorbing a vector).

fn iout_generic_res(inp: &FfvInput, res: &mut Vec<Slot>) -> Slot {
    res.clear();
    res.push(inp.gl); // 0
    res.push(inp.gr); // 1
    let n = proj_m(&inp.ket);
    res.push(n); // 2
    let n = proj_p(&inp.ket);
    res.push(n); // 3
    let n = gamma_iout(&inp.v, &res[2]);
    res.push(n); // 4
    let n = gamma_iout(&inp.v, &res[3]);
    res.push(n); // 5
    let n = mul_apply([res[0], res[4]]);
    res.push(n); // 6
    let n = mul_apply([res[1], res[5]]);
    res.push(n); // 7
    let n = res[6] + res[7];
    res.push(n); // 8
    res[8]
}

fn iout_generic_direct(inp: &FfvInput) -> Slot {
    let tl = mul_apply([inp.gl, gamma_iout(&inp.v, &proj_m(&inp.ket))]);
    let tr = mul_apply([inp.gr, gamma_iout(&inp.v, &proj_p(&inp.ket))]);
    tl + tr
}

fn bench_ffv(c: &mut Criterion) {
    const N: usize = 512;
    let inputs = gen_inputs(N, 0xF05ED);

    // The fused kernels must agree with the generic chains before their timings
    // mean anything (the adoption-grade oracle lives in the kernel tests).
    let mut res = Vec::with_capacity(9);
    for inp in &inputs {
        slots_approx_eq(
            &ffv_vout(&inp.bra, &inp.ket, &inp.gl, &inp.gr),
            &vout_generic_res(inp, &mut res),
            1e-13,
        )
        .expect("vout fused == generic");
        slots_approx_eq(
            &ffv_iout(&inp.v, &inp.ket, &inp.gl, &inp.gr),
            &iout_generic_res(inp, &mut res),
            1e-13,
        )
        .expect("iout fused == generic");
    }

    let mut group = c.benchmark_group("ffv_vout");
    group.bench_function("generic_res", |b| {
        let mut res = Vec::with_capacity(9);
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match vout_generic_res(inp, &mut res) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("generic_direct", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match vout_generic_direct(inp) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("fused", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match ffv_vout(&inp.bra, &inp.ket, &inp.gl, &inp.gr) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.finish();

    let mut group = c.benchmark_group("ffv_iout");
    group.bench_function("generic_res", |b| {
        let mut res = Vec::with_capacity(9);
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match iout_generic_res(inp, &mut res) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("generic_direct", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match iout_generic_direct(inp) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("fused", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match ffv_iout(&inp.v, &inp.ket, &inp.gl, &inp.gr) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_ffv);
criterion_main!(benches);
