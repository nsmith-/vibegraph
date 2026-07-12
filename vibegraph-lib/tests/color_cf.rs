//! Colorize + CF-matrix fixtures: run the real diagram enumeration on the
//! interned SM model, colorize each subprocess, and assert the exact-rational
//! color-factor matrix against hand-derived / MadGraph-referenced values.

mod common;

use num_rational::Ratio;

use vibegraph::diagrams::Diagram;
use vibegraph::helas::color::{colorize_process, ColorBasis, TensorKind};
use vibegraph::ufo::UFOModel;

fn r(n: i64, d: i64) -> Ratio<i64> {
    Ratio::new(n, d)
}

/// Enumerate `process` on the interned SM model and colorize its first (only)
/// concrete subprocess.
fn colorize(process: &str) -> ColorBasis {
    let model: &UFOModel = &common::sm_model();
    let sets = common::generate_with(process, model);
    let with_diagrams: Vec<&Vec<Diagram>> = sets
        .iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|s| &s.diagrams)
        .collect();
    assert_eq!(
        with_diagrams.len(),
        1,
        "expected exactly one non-empty subprocess for '{process}', got {}",
        with_diagrams.len()
    );
    colorize_process(model, with_diagrams[0]).expect("colorize failed")
}

fn assert_cf(cb: &ColorBasis, expected: &[&[Ratio<i64>]]) {
    assert_eq!(cb.ncolor(), expected.len(), "NCOLOR mismatch");
    for (i, row) in expected.iter().enumerate() {
        assert_eq!(row.len(), expected.len(), "non-square expected matrix");
        for (j, &want) in row.iter().enumerate() {
            assert_eq!(
                cb.cf(i, j),
                want,
                "CF[{i}][{j}] = {} (want {want})",
                cb.cf(i, j)
            );
        }
    }
}

// ── NCOLOR = 1: colorless and single quark-line processes ────────────────────

/// Fully colorless: e+ e- > mu+ mu- reduces to the single `ColorOne` flow.
#[test]
fn ee_to_mumu_is_one() {
    assert_cf(&colorize("e+ e- > mu+ mu-"), &[&[r(1, 1)]]);
}

/// One quark line (u u~ → Z/γ → leptons): CF(1,1) = Nc = 3, computed.
#[test]
fn uux_to_ll_qcd0_is_three() {
    assert_cf(&colorize("u u~ > e+ e- QCD=0"), &[&[r(3, 1)]]);
}

/// Two independent quark lines: CF(1,1) = Nc² = 9. Same color content as the
/// validated `uux_to_ccx_emmm_qcd0` reference (the extra leptons are colorless).
#[test]
fn uux_to_ccx_qcd0_is_nine() {
    assert_cf(&colorize("u u~ > c c~ QCD=0"), &[&[r(9, 1)]]);
}

// ── NCOLOR = 2: two color flows ──────────────────────────────────────────────

/// `u u~ > u u~`: four external (anti)fundamentals give the two color-flow
/// basis {δδ, δδ}. In the fundamental basis the metric is
/// [[Nc², Nc],[Nc, Nc²]] = [[9,3],[3,9]] — one closed loop on the diagonal
/// gives Nc², the single-loop interference gives Nc. Matches MadGraph's
/// `pp_to_bb_qcd2/P1_qq_bbx` DATA CF (same external color content).
#[test]
fn uux_to_uux_two_flows() {
    assert_cf(
        &colorize("u u~ > u u~"),
        &[&[r(9, 1), r(3, 1)], &[r(3, 1), r(9, 1)]],
    );
}

/// `g g > b b~`: two external octets feeding a quark line via the `f(1,2,3)`
/// triple-gluon vertex (f → trace rules). NCOLOR = 2 with CF =
/// [[16/3, -2/3],[-2/3, 16/3]], matching MadGraph's `pp_to_bb_qcd2/P1_gg_bbx`
/// DATA CF (5.3333.. = 16/3, -0.6667.. = -2/3).
#[test]
fn gg_to_bbx_two_flows() {
    assert_cf(
        &colorize("g g > b b~"),
        &[&[r(16, 3), r(-2, 3)], &[r(-2, 3), r(16, 3)]],
    );
}

/// `g g > t t~`: the `f(1,2,3)` triple-gluon vertex feeding a quark line, mixing
/// an `f`-derived (imaginary) color structure with pure T-chain (rational) ones.
/// The two flows must come out in MadGraph's convention `T(1,2,3,4)` /
/// `T(2,1,3,4)` — with the external fundamental index (`t`, leg 3) before the
/// antifundamental (`t~`, leg 4). Reading the string off feyngraph's all-incoming
/// crossing instead transposes each `T` to `T(1,2,4,3)` / `T(2,1,4,3)`; that
/// transpose complex-conjugates the string, silently flipping the sign of the
/// imaginary `f → trace` coefficient relative to the real T-chain terms and
/// giving a wrong (color-summed) |M|². The CF matrix is transpose-invariant and
/// cannot catch this, so it is pinned here at the basis-structure level.
#[test]
fn gg_to_ttx_flow_structures_untransposed() {
    let cb = colorize("g g > t t~");
    assert_eq!(cb.ncolor(), 2, "gg→tt~ must have NCOLOR = 2");
    let structures: Vec<Vec<(TensorKind, Vec<i32>)>> =
        cb.elements.iter().map(|e| e.structure.clone()).collect();
    assert_eq!(
        structures,
        vec![
            vec![(TensorKind::T, vec![1, 2, 3, 4])],
            vec![(TensorKind::T, vec![2, 1, 3, 4])],
        ],
        "gg→tt~ flows must be MadGraph's (untransposed) T(1,2,3,4) / T(2,1,3,4)"
    );

    // Each flow carries one imaginary (f-derived, s-channel) contribution and one
    // rational (T-chain) contribution; the f→trace sign is what the |M|² gate
    // (validate_helas_mg) locks down bit-for-bit.
    for el in &cb.elements {
        assert_eq!(el.contributions.len(), 2, "each flow has two contributions");
        assert_eq!(
            el.contributions.iter().filter(|c| c.coeff.imag).count(),
            1,
            "exactly one f-derived (imaginary) contribution per flow"
        );
    }
}

// ── NCOLOR = 6: the 4-gluon multi-structure stress test ──────────────────────

/// `g g > g g`: four external gluons, exercising the 4-gluon vertex (the only
/// SM vertex with k > 1 color structures, expanded ×3 per color-index chain)
/// and pure-`f` algebra. `full_simplify` reduces every `f`-product to the
/// 6-element single-trace basis `{ Tr(1, σ) : σ ∈ perms(2,3,4) }`.
///
/// The 6×6 CF is the standard SU(3) four-gluon color matrix: diagonal
/// `⟨Tr(1234)|Tr(1234)⟩ = 19/6`, `2/3` between a trace and its reverse, and
/// `-1/3` otherwise. The diagonal is hand-checked below; the whole pipeline is
/// independently pinned to real MadGraph DATA CF by `uux_to_uux` / `gg_to_bbx`.
///
/// Diagonal derivation (`Nc = 3`, `Tr[T^aT^b] = δ^{ab}/2`, generators summed):
/// contracting `Tr(1,2,3,4)·Tr(4,3,2,1)` with the trace Fierz rules gives
/// `½·Tr(4,3,2,2,3,4) − 1/(2Nc)·Tr(2,3,4)·Tr(4,3,2)`
/// `= ½·(Nc²−1)³/(8Nc²) − 1/(2Nc)·7/3 = 32/9 − 7/18 = 19/6`.
#[test]
fn gg_to_gg_six_flows() {
    let cb = colorize("g g > g g");
    assert_eq!(cb.ncolor(), 6, "gg→gg must have NCOLOR = 6");

    let d = r(19, 6);
    let o = r(-1, 3);
    let x = r(2, 3);
    assert_cf(
        &cb,
        &[
            &[d, o, o, o, o, x],
            &[o, d, o, x, o, o],
            &[o, o, d, o, x, o],
            &[o, x, o, d, o, o],
            &[o, o, x, o, d, o],
            &[x, o, o, o, o, d],
        ],
    );
}
