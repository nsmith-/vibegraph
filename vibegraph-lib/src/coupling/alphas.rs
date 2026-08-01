//! Renormalisation-group running of the strong coupling, matching MadGraph's
//! `Source/alfas_functions.f` (`ALPHAS` / `NEWTON1`, attributed there to
//! R. K. Ellis).
//!
//! # The evolution
//!
//! `αs(Q)` is obtained by solving the `nloop`-order β function *implicitly*: the
//! integrated β function is inverted by Newton iteration ([`newton1`]) rather
//! than by an explicit expansion in `1/ln(Q²/Λ²)`. The iteration stops on a
//! **relative** step size below [`TOL`] `= 5e-4`, so the returned value is a
//! specific iterate rather than the exact root — reproducing MadGraph therefore
//! means reproducing the iteration, not just the underlying differential
//! equation.
//!
//! Flavour thresholds are fixed masses, not the model's quark masses:
//! [`CMASS`] `= 1.42`, [`BMASS`] `= 4.7`, [`ZMASS`] `= 91.188`. Evolution runs
//! from `M_Z` with `nf = 5` down to the b threshold, from there with `nf = 4`
//! down to the c threshold, and from there with `nf = 3`. The two threshold
//! values are computed once per `(asmz, nloop)` pair and cached in
//! [`RunningAlphaS`], as the Fortran caches them in `SAVE` variables.
//!
//! # Where the inputs come from
//!
//! `αs(M_Z)` and the loop order are *not* read from the parameter card in
//! general. `Source/setrun.f` recovers a candidate from the model's strong
//! coupling, and then, whenever a beam carries a PDF, `Source/PDF/pdfwrap.f`
//! **overrides** it with the value tabulated for the PDF label. See
//! [`RunningAlphaS::from_run_card`].
//!
//! # What this module deliberately does not cover
//!
//! With `pdlabel = lhapdf` MadGraph links `Source/alfas_functions_lhapdf.f`
//! instead, whose `ALPHAS(Q)` is a one-line forward to LHAPDF's `alphasPDF(Q)` —
//! the grid's own running, with the grid's own thresholds and order. Nothing in
//! this module applies to such a run, and [`RunningAlphaS::from_run_card`]
//! refuses it rather than returning a plausible wrong number.
//!
//! [`AlphaSSource`] is what a caller asks instead when it does not already know
//! which of the two a card selects: it resolves the same field the way MadGraph's
//! link step does, and hands the refused case to
//! [`GridAlphaS`](crate::pdf::alphas::GridAlphaS).

use thiserror::Error;

use crate::pdf::alphas::{GridAlphaS, GridAlphaSError};
use crate::pdf::grid::AlphaSInfo;
use crate::runcard::RunCard;

/// Charm threshold used for `nf = 4 → 3` switching (`COMMON/QMASS/`, `CMASS`).
pub const CMASS: f64 = 1.42;
/// Bottom threshold used for `nf = 5 → 4` switching (`COMMON/QMASS/`, `BMASS`).
pub const BMASS: f64 = 4.7;
/// Reference scale at which `asmz` is defined.
pub const ZMASS: f64 = 91.188;

/// Newton stopping criterion on `|Δa / a|`.
pub const TOL: f64 = 5e-4;

/// β-function coefficients indexed by `nf - 3`, for `nf ∈ {3, 4, 5}`.
///
/// These are transcribed as the literal decimals of the Fortran `DATA`
/// statements, not recomputed from `b0 = (11 − 2nf/3)/4π` and friends. The
/// literals are truncated well short of a double's precision, so recomputing
/// them would move the low bits of every evolved coupling away from the
/// reference.
const B0: [f64; 3] = [0.716197243913527, 0.66314559621623, 0.61009394851893];
const C1: [f64; 3] = [0.565884242104515, 0.49019722472304, 0.40134724779695];
const C2: [f64; 3] = [0.453013579178645, 0.30879037953664, 0.14942733137107];
/// `√(4·c2 − c1²)`, the discriminant of the three-loop integrated β function.
const DEL: [f64; 3] = [1.22140465909230, 0.99743079911360, 0.66077962451190];

/// Order of the β function the evolution is carried out at (MadGraph's `nloop`).
///
/// The Fortran branches on `nloop ∈ {1, 2, 3}` and leaves its Newton residual
/// uninitialised for any other value, so the admissible orders are enumerated
/// here rather than carried as an integer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NLoop {
    One,
    Two,
    Three,
}

impl NLoop {
    /// The MadGraph `nloop` integer, or `None` if it is not one of 1, 2, 3.
    pub fn from_i64(nloop: i64) -> Option<Self> {
        match nloop {
            1 => Some(NLoop::One),
            2 => Some(NLoop::Two),
            3 => Some(NLoop::Three),
            _ => None,
        }
    }

    pub fn as_i64(self) -> i64 {
        match self {
            NLoop::One => 1,
            NLoop::Two => 2,
            NLoop::Three => 3,
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum AlphaSError {
    #[error("alpha_s(M_Z) must be positive, got {asmz}")]
    NonPositiveAsmz { asmz: f64 },
    #[error(
        "run card selects pdlabel = 'lhapdf' (lhaid {lhaid}): MadGraph then links \
         alfas_functions_lhapdf.f, whose ALPHAS(Q) forwards to LHAPDF's alphasPDF(Q) — \
         the grid's own running, not the beta-function solve implemented here"
    )]
    LhapdfRunning { lhaid: i64 },
    #[error(
        "run card selects pdlabel = '{label}', for which pdfwrap.f names no alpha_s(M_Z): \
         MadGraph falls back to 0.118 at two loops for any unrecognized label, which is a \
         plausible-looking value for an arbitrary set and is not adopted here"
    )]
    UnknownPdLabel { label: String },
    #[error(
        "run card selects pdlabel = 'lhapdf' (lhaid {lhaid}), so alpha_s belongs to the PDF set \
         the beams read, but no set's alpha_s metadata was supplied"
    )]
    GridUnavailable { lhaid: i64 },
    #[error("PDF set's tabulated alpha_s: {0}")]
    Grid(#[from] GridAlphaSError),
}

/// `αs` evolved from `M_Z` by MadGraph's `ALPHAS`, for one `(asmz, nloop)` pair.
///
/// The couplings at the b and c thresholds are resolved at construction; a
/// [`RunningAlphaS`] is therefore immutable and cheap to evaluate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunningAlphaS {
    asmz: f64,
    nloop: NLoop,
    /// `αs(BMASS)`, evolved down from `M_Z` with `nf = 5`.
    alpha_b: f64,
    /// `αs(CMASS)`, evolved down from the b threshold with `nf = 4`.
    alpha_c: f64,
}

impl RunningAlphaS {
    /// Evolution from `asmz = αs(M_Z)` at the given order.
    pub fn new(asmz: f64, nloop: NLoop) -> Result<Self, AlphaSError> {
        if !(asmz > 0.0) {
            return Err(AlphaSError::NonPositiveAsmz { asmz });
        }
        let alpha_b = newton1(2.0 * (BMASS / ZMASS).ln(), asmz, nloop, 5);
        let alpha_c = newton1(2.0 * (CMASS / BMASS).ln(), alpha_b, nloop, 4);
        Ok(RunningAlphaS {
            asmz,
            nloop,
            alpha_b,
            alpha_c,
        })
    }

    pub fn asmz(&self) -> f64 {
        self.asmz
    }

    pub fn nloop(&self) -> NLoop {
        self.nloop
    }

    /// `αs(BMASS)` and `αs(CMASS)`, the cached threshold values the low-scale
    /// branches evolve from.
    pub fn thresholds(&self) -> (f64, f64) {
        (self.alpha_b, self.alpha_c)
    }

    /// `αs(q)`.
    ///
    /// Panics on a non-positive `q`, as the Fortran stops on one: a scale of
    /// zero means the caller's kinematics are degenerate, and continuing would
    /// silently feed a garbage coupling to the matrix element.
    pub fn eval(&self, q: f64) -> f64 {
        assert!(q > 0.0, "alpha_s evaluated at a non-positive scale q = {q}");
        if q < BMASS {
            if q < CMASS {
                newton1(2.0 * (q / CMASS).ln(), self.alpha_c, self.nloop, 3)
            } else {
                newton1(2.0 * (q / BMASS).ln(), self.alpha_b, self.nloop, 4)
            }
        } else {
            newton1(2.0 * (q / ZMASS).ln(), self.asmz, self.nloop, 5)
        }
    }

    /// The evolution MadGraph would run with for `card`, given the parameter
    /// card's `aS`.
    ///
    /// Mirrors `Source/setrun.f`: with no PDF on either beam the parameter card
    /// supplies `αs(M_Z)` and the order is forced to two loops (the card's
    /// `pdlabel` is ignored entirely — `setrun.f` overwrites it with `none`).
    /// As soon as either beam carries a PDF, `Source/PDF/pdfwrap.f` replaces the
    /// parameter-card value with the one tabulated for the PDF label, and may
    /// change the order with it.
    ///
    /// Note that the override is silent in MadGraph and worth roughly 10% on a
    /// QCD cross section: a hadronic run whose card says `aS = 0.118` while
    /// `pdlabel = nn23lo1` runs at `0.130`.
    pub fn from_run_card(card: &RunCard, param_card_as: f64) -> Result<Self, AlphaSError> {
        let param_card_asmz = asmz_from_param_card(param_card_as);
        if card.lpp1 == 0 && card.lpp2 == 0 {
            return RunningAlphaS::new(param_card_asmz, NLoop::Two);
        }
        let (asmz, nloop) = pdf_label_alpha_s(&card.pdlabel, card.lhaid, param_card_asmz)?;
        RunningAlphaS::new(asmz, nloop)
    }
}

/// The parameter card's `αs(M_Z)` as MadGraph recovers it.
///
/// `setrun.f` does not read the `aS` entry: it takes the model's strong coupling
/// out of the `coupl.inc` common block and undoes its definition, `asmz =
/// G²/(16·atan 1)`, where `Source/MODEL/couplings.f` had set `G = √(4π·aS)`.
/// The square root and the squaring do not cancel in double precision — for
/// `aS = 0.130` the round trip returns `0.13000000000000003` — so the shift is
/// reproduced here rather than short-circuited.
pub fn asmz_from_param_card(a_s: f64) -> f64 {
    // PI as spelled in Source/MODEL/couplings.f, so the literal tracks the
    // Fortran source it is transcribed from rather than Rust's own constant. The
    // two round to the same `f64`, so `std::f64::consts::PI` would be a style
    // change and not a numerical one, and it would lose that provenance.
    #[allow(clippy::approx_constant)]
    const PI: f64 = 3.141592653589793;
    let g = (4.0 * PI * a_s).sqrt();
    g * g / (16.0 * 1.0f64.atan())
}

/// `Source/PDF/pdfwrap.f`: the `(asmz, nloop)` a PDF label imposes once a beam
/// carries a PDF. `param_card_asmz` is returned for the labels that leave the
/// value alone.
///
/// Labels the Fortran does not name reach its `else` branch, which sets `0.118`
/// at two loops so that an arbitrary set added for a lepton collider does not
/// crash the run. That is a guess, not a property of the set, so it is reported
/// as [`AlphaSError::UnknownPdLabel`] instead of being adopted.
fn pdf_label_alpha_s(
    label: &str,
    lhaid: i64,
    param_card_asmz: f64,
) -> Result<(f64, NLoop), AlphaSError> {
    match label {
        "cteq6_m" | "cteq6_d" | "cteq6_l" => Ok((0.118, NLoop::Two)),
        "cteq6l1" => Ok((0.130, NLoop::One)),
        "nn23lo" | "nn23nlo" => Ok((0.119, NLoop::Two)),
        "nn23lo1" => Ok((0.130, NLoop::Two)),
        "eva" | "iww" | "none" => Ok((param_card_asmz, NLoop::Two)),
        "lhapdf" => Err(AlphaSError::LhapdfRunning { lhaid }),
        other => Err(AlphaSError::UnknownPdLabel {
            label: other.to_string(),
        }),
    }
}

/// One `nloop`-order evolution step: given `a_in` at some scale and the
/// logarithmic separation `t = 2·ln(q_out/q_in)`, return `a_out` at `nf`
/// flavours (`NEWTON1`).
///
/// At one loop the integrated β function inverts in closed form. Beyond that the
/// closed form is only the starting guess for a Newton solve of
/// `b0·t + F(a_in) − F(a_out) = 0`, iterated until the relative step falls to
/// [`TOL`].
///
/// The arithmetic here is written to match the Fortran expression-by-expression,
/// including the two different orderings of the same one-loop product (`a·b0·t`
/// in the initial guess, `b0·a·t` in the two-loop guess) — floating-point
/// multiplication does not associate, and the reference is bit-comparable.
fn newton1(t: f64, a_in: f64, nloop: NLoop, nf: usize) -> f64 {
    let i = nf - 3;
    let (b0, c1, c2, del) = (B0[i], C1[i], C2[i], DEL[i]);

    let mut a_out = a_in / (1.0 + a_in * b0 * t);
    if nloop == NLoop::One {
        return a_out;
    }

    // Two-loop closed form, used as the Newton seed at both remaining orders.
    a_out = a_in / (1.0 + b0 * a_in * t + c1 * a_in * (1.0 + a_in * b0 * t).ln());

    // The Fortran follows this with `IF (A_OUT .LT. 0D0) AS=0.3D0`, but its loop
    // entry point is the very next statement and reassigns `AS = A_OUT`, so that
    // guard can never be read. There is no negative-seed fallback to reproduce.
    loop {
        let a = a_out;
        let (f, fp) = match nloop {
            NLoop::Two => (
                b0 * t + f2(a_in, c1) - f2(a, c1),
                1.0 / (a * a * (1.0 + c1 * a)),
            ),
            _ => (
                b0 * t + f3(a_in, c1, c2, del) - f3(a, c1, c2, del),
                1.0 / (a * a * (1.0 + c1 * a + c2 * (a * a))),
            ),
        };
        a_out = a - f / fp;
        let delta = (f / fp / a).abs();
        // Written as a negated `>` so that a NaN residual exits, as the Fortran
        // `IF (DELTA .GT. TOL) GO TO 30` does, instead of spinning forever.
        if !(delta > TOL) {
            return a_out;
        }
    }
}

/// Integrated two-loop β function, `F2` in `NEWTON1`.
fn f2(a: f64, c1: f64) -> f64 {
    1.0 / a + c1 * ((c1 * a) / (1.0 + c1 * a)).ln()
}

/// Integrated three-loop β function, `F3` in `NEWTON1`.
fn f3(a: f64, c1: f64, c2: f64, del: f64) -> f64 {
    let a2 = a * a;
    1.0 / a + 0.5 * c1 * ((c2 * a2) / (1.0 + c1 * a + c2 * a2)).ln()
        - (c1 * c1 - 2.0 * c2) / del * ((2.0 * c2 * a + c1) / del).atan()
}

/// Where a run's `αs(Q)` comes from: the β-function evolution of this module, or
/// the PDF set's own tabulation.
///
/// The choice is not a preference, it is a property of the run card, and MadGraph
/// makes it at link time: `pdlabel = lhapdf` links `alfas_functions_lhapdf.f` and
/// every `ALPHAS(Q)` in the run becomes LHAPDF's `alphasPDF(Q)`, while every other
/// label links `alfas_functions.f` and gets the solve implemented here.
/// [`AlphaSSource::from_run_card`] reproduces that decision from the same field,
/// so a run reads its coupling from the same place MadGraph read it.
///
/// The difference is small and systematic rather than negligible: for the banked
/// `NNPDF23_lo_as_0130_qed` run the two sources are `0.1300027` and `0.1300028` at
/// `M_Z`, which is `2×` the `<event>` line's printing budget for `AQCDUP` and grows
/// with any move off `M_Z`.
#[derive(Clone, Debug, PartialEq)]
pub enum AlphaSSource {
    Running(RunningAlphaS),
    Grid(GridAlphaS),
}

impl AlphaSSource {
    /// The `αs` source MadGraph would link for `card`.
    ///
    /// `grid` is the `AlphaS_*` metadata of the set the run's beams read their
    /// densities from, and is consulted only on the branch that needs it — a card
    /// whose label names its own `αs(M_Z)` resolves without one. When that branch is
    /// taken and no set was supplied, the result is
    /// [`AlphaSError::GridUnavailable`] rather than a fall back to the
    /// beta-function solve, which would silently substitute a different coupling
    /// than the one the densities were fitted with.
    pub fn from_run_card(
        card: &RunCard,
        param_card_as: f64,
        grid: Option<&AlphaSInfo>,
    ) -> Result<Self, AlphaSError> {
        match RunningAlphaS::from_run_card(card, param_card_as) {
            Ok(running) => Ok(AlphaSSource::Running(running)),
            Err(AlphaSError::LhapdfRunning { lhaid }) => {
                let info = grid.ok_or(AlphaSError::GridUnavailable { lhaid })?;
                Ok(AlphaSSource::Grid(GridAlphaS::from_info(info)?))
            }
            Err(other) => Err(other),
        }
    }

    /// `αs(q)`.
    pub fn eval(&self, q: f64) -> f64 {
        match self {
            AlphaSSource::Running(running) => running.eval(q),
            AlphaSSource::Grid(grid) => grid.eval(q),
        }
    }

    /// The evolution, when the source is one. `None` for a tabulated source, whose
    /// `asmz`/`nloop` describe a fit rather than a solve this module would run.
    pub fn running(&self) -> Option<&RunningAlphaS> {
        match self {
            AlphaSSource::Running(running) => Some(running),
            AlphaSSource::Grid(_) => None,
        }
    }

    /// The tabulation, when the source is one.
    pub fn grid(&self) -> Option<&GridAlphaS> {
        match self {
            AlphaSSource::Running(_) => None,
            AlphaSSource::Grid(grid) => Some(grid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runcard::RunCard;

    /// A run card built from MadGraph's defaults with the listed overrides.
    fn card(overrides: &str) -> RunCard {
        RunCard::parse(overrides).expect("run card parses")
    }

    #[test]
    fn threshold_values_are_the_cached_ones() {
        let a = RunningAlphaS::new(0.130, NLoop::Two).unwrap();
        let (alpha_b, alpha_c) = a.thresholds();
        // `q = BMASS` takes the nf = 5 branch with exactly the separation the
        // cached b-threshold value was built from, so the two must agree bit for
        // bit; `q = CMASS` takes the nf = 4 branch at zero separation.
        assert_eq!(a.eval(BMASS), alpha_b);
        assert!((a.eval(CMASS) - alpha_c).abs() < 1e-12 * alpha_c);
        assert!((a.eval(ZMASS) - a.asmz()).abs() < 1e-12 * a.asmz());
    }

    #[test]
    fn evolution_is_monotonically_falling() {
        let a = RunningAlphaS::new(0.130, NLoop::Two).unwrap();
        let mut previous = f64::INFINITY;
        for q in [1.5, 2.0, 4.0, 5.0, 10.0, 91.188, 250.0, 1000.0, 13000.0] {
            let value = a.eval(q);
            assert!(value < previous, "alpha_s({q}) = {value} did not fall");
            previous = value;
        }
    }

    #[test]
    fn nf_switching_happens_at_the_fixed_quark_masses() {
        // The branch taken must key on CMASS/BMASS, not on the model's quark
        // masses: evaluating just either side of a threshold must cross between
        // two different flavour numbers, which shows up as a kink in the local
        // slope of ln alpha_s vs ln q.
        let a = RunningAlphaS::new(0.130, NLoop::Two).unwrap();
        for &m in &[CMASS, BMASS] {
            let below = slope(&a, m * (1.0 - 1e-3), m * (1.0 - 1e-6));
            let above = slope(&a, m * (1.0 + 1e-6), m * (1.0 + 1e-3));
            assert!(
                (below - above).abs() > 1e-3,
                "no flavour-number kink at {m}: slopes {below} vs {above}"
            );
        }
    }

    fn slope(a: &RunningAlphaS, q1: f64, q2: f64) -> f64 {
        (a.eval(q2).ln() - a.eval(q1).ln()) / (q2.ln() - q1.ln())
    }

    #[test]
    fn one_loop_matches_the_closed_form() {
        let asmz = 0.130;
        let a = RunningAlphaS::new(asmz, NLoop::One).unwrap();
        let q = 250.0;
        let t = 2.0 * (q / ZMASS).ln();
        assert_eq!(a.eval(q), asmz / (1.0 + asmz * B0[2] * t));
    }

    #[test]
    fn non_positive_asmz_is_rejected() {
        assert_eq!(
            RunningAlphaS::new(0.0, NLoop::Two),
            Err(AlphaSError::NonPositiveAsmz { asmz: 0.0 })
        );
    }

    #[test]
    #[should_panic(expected = "non-positive scale")]
    fn non_positive_scale_panics() {
        RunningAlphaS::new(0.130, NLoop::Two).unwrap().eval(0.0);
    }

    /// The parameter-card round trip through `G` is not the identity: MadGraph's
    /// own run log prints `0.13000000000000003` for a card holding `0.130`, and
    /// `0.11799999999999999` (the exact double nearest `0.118`) for `0.118`.
    #[test]
    fn param_card_asmz_reproduces_madgraphs_round_trip() {
        assert_eq!(asmz_from_param_card(0.130), 0.13000000000000003);
        assert_eq!(asmz_from_param_card(0.118), 0.118);
    }

    /// The PDF override is the sprint's named trap, and every banked MadGraph run
    /// has a parameter card already holding the PDF's value — so no banked event
    /// can see it. This is the test that would fail if the override were dropped.
    #[test]
    fn a_pdf_beam_overrides_the_param_card_alpha_s() {
        let hadronic = card("1 = lpp1\n1 = lpp2\nnn23lo1 = pdlabel\n");
        let running = RunningAlphaS::from_run_card(&hadronic, 0.118).unwrap();
        assert_eq!(running.asmz(), 0.130);
        assert_eq!(running.nloop(), NLoop::Two);

        // The same parameter card without a PDF keeps its own value.
        let partonic = card("0 = lpp1\n0 = lpp2\nnn23lo1 = pdlabel\n");
        let running = RunningAlphaS::from_run_card(&partonic, 0.118).unwrap();
        assert_eq!(running.asmz(), 0.118);
        assert_eq!(running.nloop(), NLoop::Two);
    }

    /// Without a PDF, `setrun.f` overwrites `pdlabel` with `none` before anything
    /// reads it, so a card naming a PDF set must not reach the override table.
    #[test]
    fn no_pdf_ignores_the_cards_pdf_label() {
        for label in ["nn23lo1", "cteq6l1", "lhapdf", "some_unknown_set"] {
            let c = card(&format!("0 = lpp1\n0 = lpp2\n{label} = pdlabel\n"));
            let running = RunningAlphaS::from_run_card(&c, 0.118).unwrap();
            assert_eq!(running.asmz(), 0.118, "label {label}");
            assert_eq!(running.nloop(), NLoop::Two, "label {label}");
        }
    }

    #[test]
    fn cteq6l1_also_drops_the_running_to_one_loop() {
        let c = card("1 = lpp1\n1 = lpp2\ncteq6l1 = pdlabel\n");
        let running = RunningAlphaS::from_run_card(&c, 0.118).unwrap();
        assert_eq!(running.asmz(), 0.130);
        assert_eq!(running.nloop(), NLoop::One);
    }

    #[test]
    fn lhapdf_is_refused_rather_than_run_through_this_rge() {
        let c = card("1 = lpp1\n1 = lpp2\nlhapdf = pdlabel\n230000 = lhaid\n");
        assert_eq!(
            RunningAlphaS::from_run_card(&c, 0.118),
            Err(AlphaSError::LhapdfRunning { lhaid: 230000 })
        );
    }

    #[test]
    fn an_unnamed_pdf_label_is_refused_rather_than_defaulted() {
        let c = card("1 = lpp1\n1 = lpp2\nct14lo = pdlabel\n");
        assert_eq!(
            RunningAlphaS::from_run_card(&c, 0.118),
            Err(AlphaSError::UnknownPdLabel {
                label: "ct14lo".to_string()
            })
        );
    }

    /// A two-knot stand-in for a set's `AlphaS_*` block, bracketing `M_Z`.
    fn grid_info() -> AlphaSInfo {
        AlphaSInfo {
            mz: 0.130,
            order_qcd: 0,
            kind: "ipol".to_string(),
            qs: vec![ZMASS, 2.0 * ZMASS],
            vals: vec![0.13, 0.12],
            lambda4: 0.276,
            lambda5: 0.166,
        }
    }

    /// The label MadGraph links `alfas_functions_lhapdf.f` for is the label that
    /// sends this to the set's own table — and the value that comes back is the
    /// table's, not the parameter card's.
    #[test]
    fn the_lhapdf_label_takes_alpha_s_from_the_set() {
        let c = card("1 = lpp1\n1 = lpp2\nlhapdf = pdlabel\n247000 = lhaid\n");
        let info = grid_info();
        let source = AlphaSSource::from_run_card(&c, 0.118, Some(&info)).unwrap();
        assert!(source.running().is_none());
        assert_eq!(source.eval(ZMASS), 0.13);
        assert_eq!(source.grid().unwrap().knots(), 2);
    }

    /// Every other label keeps the beta-function solve, and supplying a set does
    /// not divert it: the run card decides the source, not what the caller has
    /// loaded.
    #[test]
    fn a_named_label_keeps_the_beta_function_solve_even_with_a_set_at_hand() {
        let c = card("1 = lpp1\n1 = lpp2\nnn23lo1 = pdlabel\n");
        let info = grid_info();
        for grid in [None, Some(&info)] {
            let source = AlphaSSource::from_run_card(&c, 0.118, grid).unwrap();
            assert_eq!(source.running().unwrap().asmz(), 0.130);
            assert_eq!(
                source.eval(ZMASS),
                RunningAlphaS::new(0.130, NLoop::Two).unwrap().eval(ZMASS)
            );
        }
    }

    /// With no PDF on either beam there is no set to read, and `setrun.f` has
    /// already overwritten the label — so `lhapdf` on a partonic card resolves
    /// without a grid rather than demanding one.
    #[test]
    fn a_partonic_card_needs_no_set_whatever_its_label_says() {
        let c = card("0 = lpp1\n0 = lpp2\nlhapdf = pdlabel\n247000 = lhaid\n");
        let source = AlphaSSource::from_run_card(&c, 0.118, None).unwrap();
        assert_eq!(source.running().unwrap().asmz(), 0.118);
    }

    /// The branch that needs a set and has none stops, rather than falling back to
    /// the evolution — which would run the set's densities against a coupling the
    /// set was not fitted with.
    #[test]
    fn a_missing_set_is_refused_rather_than_evolved_around() {
        let c = card("1 = lpp1\n1 = lpp2\nlhapdf = pdlabel\n247000 = lhaid\n");
        assert_eq!(
            AlphaSSource::from_run_card(&c, 0.118, None),
            Err(AlphaSError::GridUnavailable { lhaid: 247000 })
        );
    }

    /// A set whose table this cannot read is refused with the reason, not read
    /// anyway.
    #[test]
    fn an_unreadable_table_propagates_its_own_error() {
        let c = card("1 = lpp1\n1 = lpp2\nlhapdf = pdlabel\n247000 = lhaid\n");
        let mut info = grid_info();
        info.kind = "analytic".to_string();
        assert_eq!(
            AlphaSSource::from_run_card(&c, 0.118, Some(&info)),
            Err(AlphaSError::Grid(GridAlphaSError::UnsupportedType {
                kind: "analytic".to_string()
            }))
        );
    }
}
