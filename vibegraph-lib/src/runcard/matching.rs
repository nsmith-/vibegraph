//! The MLM block of a run card, resolved the way MadGraph resolves it before
//! anything reads it.
//!
//! MadGraph edits a matched card in three places: `banner.py`'s
//! `RunCardLO.check_validity` refuses some combinations and rewrites others
//! (`banner.py:4543-4577`), `setrun.f` forces `alpsfact` under systematics
//! (`setrun.f:151-159`), and `setcuts.f` rewrites the jet cuts once `xqcut` is
//! set (`setcuts.f:156-189`). Every consumer here — the cut filter, the scale
//! prescription, the artifact that records the card — reads the card after
//! these edits, so they are applied once, when the card is resolved.

use std::collections::BTreeMap;

use super::{ParamValue, RunCardError};

/// Apply MadGraph's matching-block edits to a card's values, or refuse the
/// combinations it refuses.
///
/// Each rewrite is idempotent, so a card resolved twice (a decay's derived
/// card, say) reads the same values.
pub(super) fn resolve(values: &mut BTreeMap<String, ParamValue>) -> Result<(), RunCardError> {
    let ickkw = int(values, "ickkw");
    if !matches!(ickkw, 0 | 1) {
        return Err(RunCardError::UnsupportedIckkw { ickkw });
    }
    if ickkw == 1 && int(values, "maxjetflavor") == 6 {
        return Err(RunCardError::MatchedTopJets);
    }

    // `setrun.f` applies this whenever `use_syst` is set; `banner.py` applies it
    // only under matching, which the Fortran then supersedes.
    if flag(values, "use_syst") && float(values, "alpsfact") != 1.0 {
        tracing::warn!(
            alpsfact = float(values, "alpsfact"),
            "use_syst = T: alpsfact set to 1, as setrun.f does"
        );
        values.insert("alpsfact".to_string(), ParamValue::Float(1.0));
    }

    let xqcut = float(values, "xqcut");
    if !(xqcut > 0.0) {
        return Ok(());
    }
    if ickkw == 0 {
        tracing::warn!(
            xqcut,
            "xqcut > 0 with ickkw = 0: applied as a pure cut, which MadGraph calls a \
             potentially inconsistent setup"
        );
    }

    let auto = flag(values, "auto_ptj_mjj");
    let ktscheme = int(values, "ktscheme");
    let ptj = float(values, "ptj");
    if auto && ptj >= 0.0 && ktscheme == 1 {
        values.insert("ptj".to_string(), ParamValue::Float(xqcut));
    } else if ptj > xqcut {
        values.insert("ptj".to_string(), ParamValue::Float(0.0));
    }
    let mmjj = float(values, "mmjj");
    if auto && mmjj >= 0.0 {
        values.insert("mmjj".to_string(), ParamValue::Float(xqcut));
    } else if mmjj > xqcut {
        values.insert("mmjj".to_string(), ParamValue::Float(0.0));
    }
    // `banner.py` zeroes any nonzero value before `setcuts.f`'s `> 0` test sees
    // it, so a negative separation is zeroed too.
    for name in ["drjj", "drjl"] {
        if float(values, name) != 0.0 {
            values.insert(name.to_string(), ParamValue::Float(0.0));
        }
    }

    // MadEvent's lower limit on τ is `(Σ xe)²/s`, where a jet's energy floor is
    // `max(ptj, sqrt(xqcut² − m²))` (`myamp.f:343-351`, `setxqcuts`), and pairs of
    // outgoing legs meeting in an s-channel take a further floor of `xqcut` on
    // their energy in the channels where they meet. With the jet threshold at
    // `xqcut` every such floor is already implied by the jet cut (a leg's energy
    // is at least its transverse momentum, and a pair holding a cut jet at least
    // `xqcut`), so the limit cuts nothing. Below it the limit is a cut of its own,
    // different in each integration channel.
    let resolved_ptj = float(values, "ptj");
    if resolved_ptj < xqcut {
        return Err(RunCardError::XqcutAboveJetThreshold {
            xqcut,
            ptj: resolved_ptj,
        });
    }
    Ok(())
}

fn float(values: &BTreeMap<String, ParamValue>, name: &str) -> f64 {
    values.get(name).expect("known param").as_f64()
}

fn int(values: &BTreeMap<String, ParamValue>, name: &str) -> i64 {
    values.get(name).expect("known param").as_i64()
}

fn flag(values: &BTreeMap<String, ParamValue>, name: &str) -> bool {
    values.get(name).expect("known param").as_bool()
}

#[cfg(test)]
mod tests {
    use crate::runcard::{RunCard, RunCardError};

    fn card(text: &str) -> RunCard {
        RunCard::parse(text).expect("run card")
    }

    fn cut(rc: &RunCard, name: &str) -> f64 {
        rc.float(name)
    }

    /// Without `xqcut` nothing moves: the defaults' jet cuts are the card's.
    #[test]
    fn no_xqcut_leaves_the_jet_cuts_alone() {
        let rc = card("25 = ptj\n0.3 = drjj\n0.2 = drjl\n15 = mmjj\n");
        assert_eq!(cut(&rc, "ptj"), 25.0);
        assert_eq!(cut(&rc, "drjj"), 0.3);
        assert_eq!(cut(&rc, "drjl"), 0.2);
        assert_eq!(cut(&rc, "mmjj"), 15.0);
    }

    /// `setcuts.f:156-189` under `auto_ptj_mjj = T`: `ptj` and `mmjj` become
    /// `xqcut`, the separations are zeroed.
    #[test]
    fn xqcut_sets_the_jet_thresholds_under_auto_ptj_mjj() {
        for ickkw in [0, 1] {
            let rc = card(&format!(
                "{ickkw} = ickkw\n30 = xqcut\n0.4 = drjj\n0.4 = drjl\n"
            ));
            assert_eq!(cut(&rc, "ptj"), 30.0);
            assert_eq!(cut(&rc, "mmjj"), 30.0);
            assert_eq!(cut(&rc, "drjj"), 0.0);
            assert_eq!(cut(&rc, "drjl"), 0.0);
        }
        // A negative separation is zeroed too: banner.py zeroes any nonzero one.
        let rc = card("30 = xqcut\n-1 = drjj\n");
        assert_eq!(cut(&rc, "drjj"), 0.0);
        // Resolving twice reads the same card.
        let twice = RunCard::parse(
            &rc.iter()
                .filter_map(|(k, v)| match v {
                    crate::runcard::ParamValue::Float(x) => Some(format!("{x} = {k}\n")),
                    _ => None,
                })
                .collect::<String>(),
        )
        .expect("re-parse");
        for name in ["ptj", "mmjj", "drjj", "drjl"] {
            assert_eq!(cut(&twice, name), cut(&rc, name));
        }
    }

    /// Off `auto_ptj_mjj` a threshold above `xqcut` is zeroed and one below is
    /// kept; a jet threshold below `xqcut` is then refused, because MadEvent's
    /// τ floor becomes a cut of its own.
    #[test]
    fn without_auto_ptj_mjj_thresholds_above_xqcut_are_zeroed() {
        let rc = card("F = auto_ptj_mjj\n30 = xqcut\n30 = ptj\n50 = mmjj\n");
        assert_eq!(cut(&rc, "ptj"), 30.0);
        assert_eq!(cut(&rc, "mmjj"), 0.0);
        let rc = card("F = auto_ptj_mjj\n30 = xqcut\n30 = ptj\n10 = mmjj\n");
        assert_eq!(cut(&rc, "mmjj"), 10.0);
        for text in [
            "F = auto_ptj_mjj\n30 = xqcut\n50 = ptj\n",
            "F = auto_ptj_mjj\n30 = xqcut\n10 = ptj\n",
            "30 = xqcut\n-1 = ptj\n",
        ] {
            assert!(
                matches!(
                    RunCard::parse(text),
                    Err(RunCardError::XqcutAboveJetThreshold { xqcut, .. }) if xqcut == 30.0
                ),
                "{text}"
            );
        }
    }

    /// `ickkw` admits MadGraph's `[0, 1]`, and matching refuses top jets.
    #[test]
    fn ickkw_and_maxjetflavor_refusals() {
        for ickkw in [2, -1, 3] {
            assert!(matches!(
                RunCard::parse(&format!("{ickkw} = ickkw\n")),
                Err(RunCardError::UnsupportedIckkw { ickkw: got }) if got == ickkw
            ));
        }
        assert!(matches!(
            RunCard::parse("1 = ickkw\n6 = maxjetflavor\n"),
            Err(RunCardError::MatchedTopJets)
        ));
        assert!(RunCard::parse("0 = ickkw\n6 = maxjetflavor\n").is_ok());
        assert!(RunCard::parse("1 = ickkw\n5 = maxjetflavor\n").is_ok());
    }

    /// `use_syst = T` forces `alpsfact = 1` (`setrun.f:151-159`), matched or not;
    /// with systematics off the card's value stands.
    #[test]
    fn use_syst_forces_alpsfact_to_one() {
        assert_eq!(card("2 = alpsfact\n").float("alpsfact"), 1.0);
        assert_eq!(
            card("1 = ickkw\n2 = alpsfact\nT = use_syst\n").float("alpsfact"),
            1.0
        );
        assert_eq!(
            card("1 = ickkw\n2 = alpsfact\nF = use_syst\n").float("alpsfact"),
            2.0
        );
    }

    /// Per-beam PDF labels at proton beams set `pdlabel` when they agree, as
    /// `banner.py`'s `PDLabelBlock` does, and are refused when they do not; a
    /// card that never names them keeps its `pdlabel`.
    #[test]
    fn per_beam_pdf_labels_resolve_into_pdlabel() {
        let rc = card("lhapdf = pdlabel1\nlhapdf = pdlabel2\n247000 = lhaid\n");
        assert_eq!(rc.pdlabel, "lhapdf");
        assert!(matches!(
            RunCard::parse("lhapdf = pdlabel1\nnn23lo1 = pdlabel2\n"),
            Err(RunCardError::AsymmetricBeamPdf { .. })
        ));
        assert_eq!(card("cteq6l1 = pdlabel\n").pdlabel, "cteq6l1");
        // Fixed-energy beams read no density: the labels stay inert.
        let fixed = card("0 = lpp1\n0 = lpp2\nnone = pdlabel1\nnone = pdlabel2\n");
        assert_eq!(fixed.pdlabel, "nn23lo1");
    }
}
