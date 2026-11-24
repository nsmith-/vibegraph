//! `DECAY <pdg> Auto`: the tree-level two-body widths binding a card computes
//! ([`vibegraph::widths`]), against MadGraph's own `compute_widths` and against
//! analytic widths at the kinematic edges.
//!
//! **The MadGraph rows** (`validation/madgraph/auto_widths_reference.json`,
//! `gen_auto_widths.sh`) are MadGraph 3.7.1's `compute_widths` at
//! `--body_decay=2`, read from the param card it writes: each channel's partial
//! width at full double precision (the comment MadGraph writes beside the
//! branching ratio) and the total as the `DECAY` line prints it. Both models
//! ship `decays.py`, so MadGraph's two-body widths are its FeynRules formulas,
//! and this crate's are `Σ|M|²` from its own vertices: the comparison is per
//! channel, keyed by the daughters, so a total that agrees while two channels
//! trade width is caught, and so is a channel either side has and the other
//! lacks. The default `--body_decay=4.0025` run is banked too: every three-body
//! mode it adds must be among the modes the two-body analysis lists
//! ([`two_body_width`]'s `not_included`), and a particle with any such mode is
//! refused by binding (`BeyondTwoBody`): the SM top and W bind, the Z, the
//! Higgs boson and the g2HDM H2, H3 and H+ are refused.
//!
//! What the MadGraph rows cannot see: anything the two computations share by
//! construction. They read the same parameter card, so a wrong parameter
//! evaluation shared by both (none is shared: the formulas are MadGraph's
//! Python, the couplings this crate's) would pass; and an isotropy violation is
//! invisible, since a two-body width is evaluated at one orientation. The
//! amplitude-level checks of `decay_widths.rs` pin the orientation.
//!
//! **The analytic rows** pin the kinematic edges a reference cannot reach by
//! construction: a channel at threshold (closed at equality, open and on the
//! analytic `√(1-4x)` just above), identical daughters (`H > Z Z` is half of
//! `H > W+ W-` at the same couplings), massless daughters (`Z > ve ve~`) and a
//! closed channel. What they cannot see is a convention the analytic formula
//! and the evaluation share: both read the model's own `vev`, `ee`, `sw`.

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use std::collections::BTreeSet;

use vibegraph::ufo::bind::{bind_card, BoundCard, CardError};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::widths::{two_body_width, AutoWidthError, ComputedWidth};

fn madgraph_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bind(model: &Arc<UFOModel>, text: &str) -> Result<BoundCard, CardError> {
    let card: ParamCard = text.parse().unwrap();
    bind_card(model.clone(), &card)
}

fn width_of(bound: &BoundCard, pdg: i32) -> &ComputedWidth {
    bound
        .auto_widths
        .iter()
        .find(|w| w.pdg == pdg)
        .unwrap_or_else(|| panic!("no computed width for {pdg}"))
}

/// The `DECAY <pdg> Auto` codes of a card's text.
fn auto_codes(text: &str) -> Vec<i32> {
    text.lines()
        .filter_map(|l| match auto_line(l) {
            Some(code) => code.parse().ok(),
            None => None,
        })
        .collect()
}

/// The code of a `DECAY <code> Auto` line.
fn auto_line(line: &str) -> Option<&str> {
    let f: Vec<&str> = line.split('#').next()?.split_whitespace().collect();
    match f.as_slice() {
        [d, code, v] if d.eq_ignore_ascii_case("decay") && v.eq_ignore_ascii_case("auto") => {
            Some(code)
        }
        _ => None,
    }
}

/// `text` with every `Auto` width but `keep`'s set to one.
fn with_auto_only(text: &str, keep: Option<i32>) -> String {
    text.lines()
        .map(|l| match auto_line(l) {
            Some(code) if code.parse::<i32>().ok() != keep => format!("DECAY {code} 1.0\n"),
            _ => format!("{l}\n"),
        })
        .collect()
}

/// The two-body width of every `Auto` code of `text`, whether or not binding
/// would refuse it for a mode beyond two bodies ([`two_body_width`]), at the
/// card with each `Auto` width set to one, and that evaluation.
fn two_body(model: &Arc<UFOModel>, text: &str) -> (EvaluatedModel, Vec<ComputedWidth>) {
    let card: ParamCard = with_auto_only(text, None).parse().unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card).unwrap();
    let widths = auto_codes(text)
        .into_iter()
        .map(|pdg| two_body_width(model, &evaluated, pdg, &BTreeSet::new()).unwrap())
        .collect();
    (evaluated, widths)
}

fn find(widths: &[ComputedWidth], pdg: i32) -> &ComputedWidth {
    widths.iter().find(|w| w.pdg == pdg).unwrap()
}

/// The partial width of the open channel into `daughters` (any order), if any.
fn partial(width: &ComputedWidth, daughters: &[i32]) -> Option<f64> {
    let mut want = daughters.to_vec();
    want.sort_unstable();
    width
        .channels
        .iter()
        .find(|c| {
            let mut have: Vec<i32> = c.daughters.iter().map(|d| d.pdg).collect();
            have.sort_unstable();
            have == want
        })
        .map(|c| c.width)
}

fn param(evaluated: &EvaluatedModel, name: &str) -> f64 {
    evaluated.param_values[name].re
}

// ── MadGraph ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct Channel {
    daughters: Vec<i32>,
    width: f64,
}

#[derive(serde::Deserialize)]
struct Width {
    total: f64,
    total_text: String,
    channels: Vec<Channel>,
}

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    /// The card the widths are computed at, relative to the repository root;
    /// `null` for the restriction's defaults with the `Auto` lines alone.
    card: Option<String>,
    auto: Vec<i32>,
    two_body: BTreeMap<String, Width>,
    default: BTreeMap<String, Width>,
}

#[derive(serde::Deserialize)]
struct Reference {
    cases: Vec<Case>,
}

fn reference() -> Reference {
    let path = madgraph_dir().join("auto_widths_reference.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).expect("auto_widths_reference.json parses")
}

fn case(name: &str) -> Case {
    reference()
        .cases
        .into_iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"))
}

/// A partial width agrees when it is within this relative distance of
/// MadGraph's, or both are below [`NEGLIGIBLE`] of the total. Both sides are
/// tree level in double precision: the measured worst is printed, and sits
/// orders of magnitude below this.
const PER_CHANNEL_REL: f64 = 1e-9;

/// A partial width below this fraction of the parent's total is compared as
/// zero: a coupling that cancels analytically (`h2 > a a` in the g2HDM) is
/// rounding noise on either side, `1e-20` of the width.
const NEGLIGIBLE: f64 = 1e-12;

/// The total MadGraph prints is `%e`, seven significant digits: half a unit in
/// the last place.
const PRINTED_REL: f64 = 5e-7;

/// The sorted daughters of a channel, as a key both sides share.
fn key(daughters: impl IntoIterator<Item = i32>) -> Vec<i32> {
    let mut k: Vec<i32> = daughters.into_iter().collect();
    k.sort_unstable();
    k
}

/// Compare every computed width of `bound` with the case's MadGraph rows,
/// channel by channel; returns the worst relative difference over channels
/// above [`NEGLIGIBLE`].
fn compare(case: &Case, widths: &[ComputedWidth]) -> f64 {
    let mut worst = 0.0f64;
    for &pdg in &case.auto {
        let mg = &case.two_body[&pdg.to_string()];
        let ours = find(widths, pdg);
        let ours_by: BTreeMap<Vec<i32>, f64> = ours
            .channels
            .iter()
            .chain(&ours.below_qcd_scale)
            .map(|c| (key(c.daughters.iter().map(|d| d.pdg)), c.width))
            .collect();
        let mg_by: BTreeMap<Vec<i32>, f64> = mg
            .channels
            .iter()
            .map(|c| (key(c.daughters.iter().copied()), c.width))
            .collect();
        let floor = NEGLIGIBLE * mg.total;
        let keys: std::collections::BTreeSet<&Vec<i32>> =
            ours_by.keys().chain(mg_by.keys()).collect();
        for k in keys {
            let (a, b) = (ours_by.get(k).copied(), mg_by.get(k).copied());
            let (a0, b0) = (a.unwrap_or(0.0), b.unwrap_or(0.0));
            if a0.abs() < floor && b0.abs() < floor {
                continue;
            }
            let (Some(a), Some(b)) = (a, b) else {
                panic!("{} {pdg} > {k:?}: ours {a:?}, MadGraph {b:?}", case.name);
            };
            let rel = (a / b - 1.0).abs();
            worst = worst.max(rel);
            println!(
                "{:<12} {pdg:>3} > {k:<14?} ours {a:.12e}  MadGraph {b:.12e}  rel {rel:.1e}",
                case.name
            );
            assert!(rel < PER_CHANNEL_REL, "{} {pdg} > {k:?}", case.name);
        }
        let printed: f64 = mg.total_text.parse().unwrap();
        assert_eq!(printed, mg.total);
        let rel_total = (ours.total / printed - 1.0).abs();
        println!(
            "{:<12} {pdg:>3}   total ours {:.12e} (binds {:e})  MadGraph {}  rel {rel_total:.1e}",
            case.name, ours.total, ours.bound, mg.total_text
        );
        assert!(rel_total < PRINTED_REL, "{} {pdg} total", case.name);
        assert_eq!(
            format!("{:.6e}", ours.bound),
            format!("{:.6e}", printed),
            "{} {pdg}: the bound width is not the number MadGraph writes",
            case.name
        );
        check_beyond_two_body(case, pdg, ours);
    }
    worst
}

/// Every three-body mode MadGraph's default run adds must be listed as not
/// included; a mode with more bodies is counted, not compared.
fn check_beyond_two_body(case: &Case, pdg: i32, ours: &ComputedWidth) {
    let default = &case.default[&pdg.to_string()];
    let listed: Vec<Vec<i32>> = ours
        .not_included
        .iter()
        .map(|m| key(m.daughters.iter().map(|d| d.pdg)))
        .collect();
    let mut three = 0;
    let mut more = 0;
    let mut added = 0.0;
    for c in default.channels.iter().filter(|c| c.daughters.len() > 2) {
        added += c.width;
        if c.daughters.len() == 3 {
            three += 1;
            assert!(
                listed.contains(&key(c.daughters.iter().copied())),
                "{} {pdg}: MadGraph's default adds {:?}, which the width does not list",
                case.name,
                c.daughters
            );
        } else {
            more += 1;
        }
    }
    println!(
        "{:<12} {pdg:>3}   default run: total {} adds {three} three-body and {more} more-body \
         modes ({:.3}% of it); {} three-body modes listed as not included",
        case.name,
        default.total_text,
        100.0 * added / default.total,
        listed.len()
    );
}

/// Binding the card with each of its `Auto` codes alone left `Auto` binds
/// exactly `binds` and refuses the rest for modes beyond two bodies, naming
/// every one; a bound width is the two-body one with nothing listed as not
/// included.
fn check_decision(model: &Arc<UFOModel>, text: &str, widths: &[ComputedWidth], binds: &[i32]) {
    for pdg in auto_codes(text) {
        let two = find(widths, pdg);
        match bind(model, &with_auto_only(text, Some(pdg))) {
            Ok(bound) => {
                let w = width_of(&bound, pdg);
                println!("{pdg:>3} binds {:e}", w.bound);
                assert!(binds.contains(&pdg), "{pdg} binds; expected a refusal");
                assert!(w.not_included.is_empty() && two.not_included.is_empty());
                assert_eq!(w.total, two.total);
            }
            Err(CardError::AutoWidth {
                pdg: got,
                reason: reason @ AutoWidthError::BeyondTwoBody { .. },
            }) => {
                println!(
                    "{pdg:>3} refused, {} modes beyond two bodies named",
                    two.not_included.len()
                );
                assert_eq!(got, pdg);
                assert!(!binds.contains(&pdg), "{pdg} refused: {reason}");
                assert!(!two.not_included.is_empty());
                let text = reason.to_string();
                for m in &two.not_included {
                    let names: Vec<&str> = m.daughters.iter().map(|d| d.name.as_str()).collect();
                    let mode = format!("> {} (through {})", names.join(" "), m.through.name);
                    assert!(text.contains(&mode), "{pdg}: {mode} not named");
                }
            }
            Err(other) => panic!("{pdg}: {other}"),
        }
    }
}

/// The SM top, W, Z and Higgs widths, `import model sm`, against MadGraph's
/// `compute_widths 6 23 24 25 --body_decay=2`, channel by channel; and the
/// binding decision: t and W bind, Z and H are refused for modes beyond two
/// bodies.
#[test]
fn sm_auto_widths_match_madgraph_per_channel() {
    let case = case("sm");
    assert!(case.card.is_none());
    let text: String = case
        .auto
        .iter()
        .map(|p| format!("DECAY {p} Auto\n"))
        .collect();
    let model = common::sm_model();
    let (_, widths) = two_body(&model, &text);
    let worst = compare(&case, &widths);
    println!("sm: worst per-channel relative difference {worst:.1e}");
    check_decision(&model, &text, &widths, &[6, 24]);
}

/// The g2HDM neutral and charged heavy Higgs bosons at M = 1000 GeV against
/// MadGraph's `compute_widths 35 36 37 --body_decay=2` at the same card; all
/// three are refused for modes beyond two bodies.
///
/// The model is not in the repository (its UFO carries no licence to vendor
/// it under), so this runs where `VG_G2HDM_UFO` names the `g2HDM_UFO`
/// directory of the CMS model store's `g2HDM_UFO.tar.gz`:
/// `VG_G2HDM_UFO=… cargo test -p vibegraph-lib --test auto_widths -- --ignored`.
#[test]
#[ignore = "needs the g2HDM UFO at VG_G2HDM_UFO"]
fn g2hdm_auto_widths_match_madgraph_per_channel() {
    let dir = std::env::var("VG_G2HDM_UFO").expect("VG_G2HDM_UFO names the g2HDM_UFO directory");
    let model = UFOModel::load(std::path::Path::new(&dir), None).unwrap();
    let case = case("g2hdm_m1000");
    let card = std::fs::read_to_string(repo_root().join(case.card.as_deref().unwrap())).unwrap();
    let (_, widths) = two_body(&model, &card);
    let worst = compare(&case, &widths);
    println!("g2hdm_m1000: worst per-channel relative difference {worst:.1e}");
    check_decision(&model, &card, &widths, &[]);
}

// ── Analytic edges ───────────────────────────────────────────────────────────
//
// Two-body widths through `two_body_width`, which lists modes beyond two
// bodies rather than refusing, so the Z and the Higgs boson are reachable.

/// `Γ(H > V V) = δ M_H³ / (32π v²) · √(1-4x) (1-4x+12x²)`, `x = M_V²/M_H²`,
/// `δ = 2` for `W+ W-` and `1` for the identical `Z Z`.
fn h_to_vv(m_h: f64, m_v: f64, vev: f64, delta: f64) -> f64 {
    let x = m_v * m_v / (m_h * m_h);
    delta * m_h.powi(3) / (32.0 * std::f64::consts::PI * vev * vev)
        * (1.0 - 4.0 * x).sqrt()
        * (1.0 - 4.0 * x + 12.0 * x * x)
}

/// Identical daughters: at `M_H = 300 GeV`, `H > Z Z` is the analytic width
/// with the identical-particle `1/2`, and `H > W+ W-` the same formula without
/// it.
#[test]
fn identical_daughters_take_the_symmetry_factor() {
    let (ev, widths) = two_body(
        &common::sm_model(),
        "Block MASS\n 25 300.0\nDECAY 25 Auto\n",
    );
    let h = find(&widths, 25);
    let vev = param(&ev, "vev");
    let (mz, mw) = (param(&ev, "MZ"), param(&ev, "MW"));
    let zz = partial(h, &[23, 23]).unwrap();
    let ww = partial(h, &[24, -24]).unwrap();
    let (zz_a, ww_a) = (h_to_vv(300.0, mz, vev, 1.0), h_to_vv(300.0, mw, vev, 2.0));
    println!("H > Z Z {zz:e} analytic {zz_a:e}; H > W+ W- {ww:e} analytic {ww_a:e}");
    assert!((zz / zz_a - 1.0).abs() < 1e-12);
    assert!((ww / ww_a - 1.0).abs() < 1e-12);
}

/// Massless daughters: `Γ(Z > νν̄) = e² M_Z / (96π s_w² c_w²)`, one per
/// flavour, and `Γ(W+ > e+ ve) = e² M_W / (48π s_w²)`.
#[test]
fn massless_daughters_take_the_analytic_width() {
    let (ev, widths) = two_body(&common::sm_model(), "DECAY 23 Auto\nDECAY 24 Auto\n");
    let (ee, sw, cw) = (param(&ev, "ee"), param(&ev, "sw"), param(&ev, "cw"));
    let (mz, mw) = (param(&ev, "MZ"), param(&ev, "MW"));
    let pi = std::f64::consts::PI;
    let nu = ee * ee * mz / (96.0 * pi * sw * sw * cw * cw);
    let lep = ee * ee * mw / (48.0 * pi * sw * sw);
    for flavour in [12, 14, 16] {
        let got = partial(find(&widths, 23), &[flavour, -flavour]).unwrap();
        assert!(
            (got / nu - 1.0).abs() < 1e-12,
            "Z > {flavour}: {got} against {nu}"
        );
    }
    let got = partial(find(&widths, 24), &[12, -11]).unwrap();
    assert!(
        (got / lep - 1.0).abs() < 1e-12,
        "W+ > e+ ve: {got} against {lep}"
    );
}

/// A channel at threshold: `M_H = 2 M_Z` exactly closes `H > Z Z` (MadGraph
/// skips a channel whose mass difference is not positive), and `M_H` a part
/// in 10⁶ above opens it on the analytic `√(1-4x)` edge.
#[test]
fn a_channel_at_threshold_is_closed_and_opens_above_it() {
    let model = common::sm_model();
    let mz = 91.188;
    let (_, at) = two_body(
        &model,
        &format!("Block MASS\n 25 {}\nDECAY 25 Auto\n", 2.0 * mz),
    );
    let h = find(&at, 25);
    assert_eq!(partial(h, &[23, 23]), None);
    assert!(h
        .closed
        .iter()
        .any(|c| key(c.daughters.iter().map(|d| d.pdg)) == vec![23, 23]));

    let m_h = 2.0 * mz * (1.0 + 1e-6);
    let (ev, above) = two_body(&model, &format!("Block MASS\n 25 {m_h:e}\nDECAY 25 Auto\n"));
    let zz = partial(find(&above, 25), &[23, 23]).unwrap();
    let analytic = h_to_vv(m_h, mz, param(&ev, "vev"), 1.0);
    println!("H > Z Z a part in 1e6 above threshold: {zz:e}, analytic {analytic:e}");
    assert!(zz > 0.0);
    assert!(
        (zz / analytic - 1.0).abs() < 1e-8,
        "{zz} against {analytic}"
    );
}

/// A closed channel is listed as closed and contributes nothing: `Z > t t~`
/// and `H > t t~` at the default masses.
#[test]
fn a_closed_channel_is_listed_and_adds_nothing() {
    let (_, widths) = two_body(&common::sm_model(), "DECAY 23 Auto\nDECAY 25 Auto\n");
    for pdg in [23, 25] {
        let w = find(&widths, pdg);
        assert_eq!(partial(w, &[6, -6]), None);
        assert!(w
            .closed
            .iter()
            .any(|c| key(c.daughters.iter().map(|d| d.pdg)) == vec![-6, 6]));
        let sum: f64 = w.channels.iter().map(|c| c.width).sum();
        assert_eq!(sum, w.total);
    }
}

/// A coloured parent's open channel below 0.1 GeV is set to zero and left out
/// of the total and the decay table, as MadGraph sets it ("lower than QCD
/// scale"), and listed as such: under `restrict_ckm` the top's CKM-suppressed
/// `t > W+ s` and `t > W+ d` are, and `t > W+ b` carries the whole width.
/// (Binding refuses this top for its `t > W+ Z s` mode through `t`, so the
/// two-body width is read directly.)
#[test]
fn a_coloured_parents_channel_below_the_qcd_scale_is_left_out() {
    let model = vibegraph::ufo::sm::sm_model(vibegraph::ufo::sm::SMRestrict::Ckm);
    let (_, widths) = two_body(&model, "DECAY 6 Auto\n");
    let w = find(&widths, 6);
    let below: std::collections::BTreeSet<Vec<i32>> = w
        .below_qcd_scale
        .iter()
        .map(|c| key(c.daughters.iter().map(|d| d.pdg)))
        .collect();
    assert_eq!(
        below,
        [key([24, 3]), key([24, 1])].into_iter().collect(),
        "{w:?}"
    );
    for c in &w.below_qcd_scale {
        assert!(c.width > 0.0 && c.width < 0.1, "{c:?}");
    }
    assert_eq!(partial(w, &[24, 5]), Some(w.total), "{w:?}");
    let table = w.decay_table();
    assert_eq!(table.channels.len(), 1, "{table:?}");
    assert_eq!(
        key(table.channels[0].daughters.iter().copied()),
        key([24, 5])
    );
}

/// The SM particles that bind under `Auto` and those refused, each by name:
/// the top and the W bind with nothing listed as not included; the Z is
/// refused for `Z > W+ e- ve~` through `W-` (a mode MadGraph's default leaves
/// out at these masses, so lifting this refusal is a change of its own), and
/// the Higgs boson for `H > W+ e- ve~` through `W-` and the like.
#[test]
fn sm_top_and_w_bind_and_z_and_higgs_are_refused() {
    let model = common::sm_model();
    for pdg in [6, 24] {
        let bound = bind(&model, &format!("DECAY {pdg} Auto\n")).unwrap();
        let w = width_of(&bound, pdg);
        assert!(w.total > 0.0 && w.not_included.is_empty(), "{pdg}: {w:?}");
    }
    for (pdg, mode) in [
        (23, "Z > W+ e- ve~ (through W-)"),
        (25, "H > W+ e- ve~ (through W-)"),
    ] {
        match bind(&model, &format!("DECAY {pdg} Auto\n")) {
            Err(CardError::AutoWidth {
                pdg: got,
                reason: reason @ AutoWidthError::BeyondTwoBody { .. },
            }) => {
                assert_eq!(got, pdg);
                let text = reason.to_string();
                assert!(text.contains(mode), "{text}");
                assert!(text.contains("may include such modes"), "{text}");
            }
            other => panic!("{pdg}: {:?}", other.map(|b| b.auto_widths)),
        }
    }
}

// ── Refusals on a toy model ──────────────────────────────────────────────────

const TOY_PARTICLES: &str = "\
from object_library import all_particles, Particle
import parameters as Param

S = Particle(pdg_code = 9000001, name = 'S', antiname = 'S', spin = 1, color = 1,
             mass = Param.MS, width = Param.WS, texname = 'S', antitexname = 'S',
             charge = 0, GhostNumber = 0)
P = Particle(pdg_code = 9000002, name = 'phi', antiname = 'phi', spin = 1, color = 1,
             mass = Param.MP, width = Param.WP, texname = 'phi', antitexname = 'phi',
             charge = 0, GhostNumber = 0)
T = Particle(pdg_code = 9000003, name = 'T', antiname = 'T', spin = 1, color = 1,
             mass = Param.MT, width = Param.WT, texname = 'T', antitexname = 'T',
             charge = 0, GhostNumber = 0)
U = Particle(pdg_code = 9000004, name = 'U', antiname = 'U', spin = 1, color = 1,
             mass = Param.MU, width = Param.WU, texname = 'U', antitexname = 'U',
             charge = 0, GhostNumber = 0)
";

const TOY_PARAMETERS: &str = "\
from object_library import all_parameters, Parameter

ZERO = Parameter(name = 'ZERO', nature = 'internal', type = 'real', value = '0.0', texname = '0')
aS = Parameter(name = 'aS', nature = 'external', type = 'real', value = 0.118, texname = 'a_s',
               lhablock = 'SMINPUTS', lhacode = [ 3 ])
MS = Parameter(name = 'MS', nature = 'external', type = 'real', value = 100.0, texname = 'M_S',
               lhablock = 'MASS', lhacode = [ 9000001 ])
MP = Parameter(name = 'MP', nature = 'external', type = 'real', value = 10.0, texname = 'M_P',
               lhablock = 'MASS', lhacode = [ 9000002 ])
MT = Parameter(name = 'MT', nature = 'external', type = 'real', value = 50.0, texname = 'M_T',
               lhablock = 'MASS', lhacode = [ 9000003 ])
MU = Parameter(name = 'MU', nature = 'external', type = 'real', value = 60.0, texname = 'M_U',
               lhablock = 'MASS', lhacode = [ 9000004 ])
WS = Parameter(name = 'WS', nature = 'external', type = 'real', value = 1.0, texname = 'W_S',
               lhablock = 'DECAY', lhacode = [ 9000001 ])
WP = Parameter(name = 'WP', nature = 'external', type = 'real', value = 0.0, texname = 'W_P',
               lhablock = 'DECAY', lhacode = [ 9000002 ])
WT = Parameter(name = 'WT', nature = 'external', type = 'real', value = 1.0, texname = 'W_T',
               lhablock = 'DECAY', lhacode = [ 9000003 ])
WU = Parameter(name = 'WU', nature = 'external', type = 'real', value = 1.0, texname = 'W_U',
               lhablock = 'DECAY', lhacode = [ 9000004 ])
g1 = Parameter(name = 'g1', nature = 'external', type = 'real', value = 3.0, texname = 'g_1',
               lhablock = 'NEWCOUP', lhacode = [ 1 ])
";

const TOY_COUPLINGS: &str = "\
from object_library import all_couplings, Coupling

GC_1 = Coupling(name = 'GC_1', value = '-complex(0,1)*g1', order = {'QED':1})
GC_2 = Coupling(name = 'GC_2', value = '-complex(0,1)*g1*aS', order = {'QED':1})
GC_3 = Coupling(name = 'GC_3', value = '-complex(0,1)*g1*WS', order = {'QED':1})
";

const TOY_LORENTZ: &str = "\
from object_library import all_lorentz, Lorentz

SSS1 = Lorentz(name = 'SSS1', spins = [ 1, 1, 1 ], structure = '1')
";

const TOY_VERTICES: &str = "\
from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_1 = Vertex(name = 'V_1', particles = [ P.S, P.P, P.P ], color = [ '1' ],
             lorentz = [ L.SSS1 ], couplings = {(0,0):C.GC_1})
V_2 = Vertex(name = 'V_2', particles = [ P.T, P.P, P.P ], color = [ '1' ],
             lorentz = [ L.SSS1 ], couplings = {(0,0):C.GC_2})
V_3 = Vertex(name = 'V_3', particles = [ P.U, P.P, P.P ], color = [ '1' ],
             lorentz = [ L.SSS1 ], couplings = {(0,0):C.GC_3})
";

const TOY_ORDERS: &str = "\
from object_library import all_orders, CouplingOrder

QED = CouplingOrder(name = 'QED', expansion_order = 99, hierarchy = 2)
";

fn toy() -> (tempfile::TempDir, Arc<UFOModel>) {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in [
        ("particles.py", TOY_PARTICLES),
        ("parameters.py", TOY_PARAMETERS),
        ("couplings.py", TOY_COUPLINGS),
        ("lorentz.py", TOY_LORENTZ),
        ("vertices.py", TOY_VERTICES),
        ("coupling_orders.py", TOY_ORDERS),
    ] {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    let model = UFOModel::load(dir.path(), None).unwrap();
    (dir, model)
}

/// A cubic scalar coupling `g` into two identical scalars:
/// `Γ = g² p* / (8π M²) / 2`, the `1/2` for the identical pair.
#[test]
fn a_scalar_into_identical_scalars_takes_the_analytic_width() {
    let (_dir, model) = toy();
    let bound = bind(&model, "DECAY 9000001 Auto\n").unwrap();
    let w = width_of(&bound, 9000001);
    let (m, mp, g) = (100.0f64, 10.0f64, 3.0f64);
    let p_star = (m * m / 4.0 - mp * mp).sqrt();
    let analytic = g * g * p_star / (8.0 * std::f64::consts::PI * m * m) / 2.0;
    assert_eq!(w.channels.len(), 1);
    assert!(
        (w.total / analytic - 1.0).abs() < 1e-12,
        "{} against {analytic}",
        w.total
    );
}

/// A channel whose coupling depends on `α_s` is refused: MadGraph runs `α_s` to
/// the parent's mass first.
#[test]
fn a_channel_running_with_alpha_s_is_refused() {
    let (_dir, model) = toy();
    let err = bind(&model, "DECAY 9000003 Auto\n").err().unwrap();
    assert!(
        matches!(
            &err,
            CardError::AutoWidth { pdg: 9000003, reason: AutoWidthError::RunsWithAlphaS { channel } }
                if channel == "T > phi phi"
        ),
        "{err:?}"
    );
}

/// A channel whose coupling depends on another `Auto` width is refused, since
/// MadGraph computes it with that width at one; with that width a number it
/// binds.
#[test]
fn a_channel_reading_another_auto_width_is_refused() {
    let (_dir, model) = toy();
    let err = bind(&model, "DECAY 9000001 Auto\nDECAY 9000004 Auto\n")
        .err()
        .unwrap();
    assert!(
        matches!(
            &err,
            CardError::AutoWidth { pdg: 9000004, reason: AutoWidthError::DependsOnAutoWidth { width, .. } }
                if width == "WS"
        ),
        "{err:?}"
    );
    let bound = bind(&model, "DECAY 9000001 2.0\nDECAY 9000004 Auto\n").unwrap();
    let (m, mp, g) = (60.0f64, 10.0f64, 3.0f64 * 2.0);
    let p_star = (m * m / 4.0 - mp * mp).sqrt();
    let analytic = g * g * p_star / (8.0 * std::f64::consts::PI * m * m) / 2.0;
    let w = width_of(&bound, 9000004);
    assert!(
        (w.total / analytic - 1.0).abs() < 1e-12,
        "{} against {analytic}",
        w.total
    );
}
