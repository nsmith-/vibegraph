//! `W`, MadGraph's `get_lo_wgt` (`systematics.py:931-1034`) as a library
//! function over one event's [`LoRecord`] and its written momenta. A
//! variation's weight is `XWGTUP × W(variation) / W(nominal)`.

use super::record::{LoRecord, RecordKinematics};
use super::request::DynScale;

/// A PDF member as the weight reads it: its own strong coupling and its
/// densities, both at a scale `Q` in GeV.
pub trait VariationPdf {
    /// `αs(Q)`, LHAPDF's `alphasQ`.
    fn alphas_q(&self, q: f64) -> Result<f64, String>;
    /// `x·f(x, Q)` for PDG code `pdg`, LHAPDF's `xfxQ`; zero for a flavour
    /// the member does not carry.
    fn xfx_q(&self, pdg: i64, x: f64, q: f64) -> Result<f64, String>;
}

/// One variation: the factors, the dynamical choice and the PDF it is
/// evaluated with.
#[derive(Clone, Copy)]
pub struct Variation<'p> {
    pub mur: f64,
    pub muf: f64,
    pub alps: f64,
    pub dyn_scale: DynScale,
    pub pdf: &'p dyn VariationPdf,
}

/// The beam type multiplying a chain's PDG codes (`self.b1`, `self.b2`):
/// `1` for a proton, the only beam the systematics weights are written for.
const PROTON_BEAM: i64 = 1;

/// `Σ E·pT/√(pT² + pz²)` over the final state with `pT > 0`
/// (`get_et_scale`).
fn et_scale(k: &RecordKinematics) -> f64 {
    let mut scale = 0.0;
    for p in &k.final_state {
        let [e, px, py, pz] = *p;
        let pt = (px * px + py * py).max(0.0).sqrt();
        if pt > 0.0 {
            scale += e * pt / (pt * pt + pz * pz).sqrt();
        }
    }
    scale
}

/// `Σ √(m² + pT²)` over the final state with `m² = E² − |p|²` from the
/// momenta (`get_ht_scale`); a negative radicand has no reading.
fn ht_scale(k: &RecordKinematics) -> Option<f64> {
    let mut scale = 0.0;
    for p in &k.final_state {
        let [e, px, py, pz] = *p;
        let mass_sqr = e * e - px * px - py * py - pz * pz;
        let pt = (px * px + py * py).max(0.0).sqrt();
        let radicand = mass_sqr + pt * pt;
        if radicand < 0.0 {
            return None;
        }
        scale += radicand.sqrt();
    }
    Some(scale)
}

/// `√((p₁ + p₂)²)` of the incoming partons (`get_sqrts_scale`).
fn sqrts_scale(k: &RecordKinematics) -> Option<f64> {
    match k.incoming.as_slice() {
        [a, b] => {
            let s = [0, 1, 2, 3].map(|i| a[i] + b[i]);
            let m2 = s[0] * s[0] - s[1] * s[1] - s[2] * s[2] - s[3] * s[3];
            (m2 >= 0.0).then(|| m2.sqrt())
        }
        [a] => {
            let m2 = a[0] * a[0] - a[1] * a[1] - a[2] * a[2] - a[3] * a[3];
            Some(m2.max(0.0).sqrt())
        }
        _ => None,
    }
}

/// The scale a dynamical choice gives the written event, lab frame and no
/// `scalefact` (MadGraph's Python definitions, not `setscales.f`'s); `None`
/// for [`DynScale::Nominal`], which reads the record instead.
pub fn dynamical_scale(choice: DynScale, k: &RecordKinematics) -> Result<Option<f64>, String> {
    let value = match choice {
        DynScale::Nominal => return Ok(None),
        DynScale::SumEt => Some(et_scale(k)),
        DynScale::Ht => ht_scale(k),
        DynScale::HalfHt => ht_scale(k).map(|s| 0.5 * s),
        DynScale::Sqrts => sqrts_scale(k),
    };
    match value {
        Some(v) if v.is_finite() && v > 0.0 => Ok(Some(v)),
        Some(v) => Err(format!(
            "dynamical scale choice {} ({}) is {v} on this event, not a finite positive scale",
            choice.code(),
            choice.label()
        )),
        None => Err(format!(
            "dynamical scale choice {} ({}) has no real value on this event's momenta",
            choice.code(),
            choice.label()
        )),
    }
}

/// `get_pdfQ`: the density `f = xfxQ/x` of `pdg` on a proton beam, with
/// `∓21/∓22` read as `21/22` and `0` as a density of one.
fn density(pdf: &dyn VariationPdf, pdg: i64, x: f64, q: f64) -> Result<f64, String> {
    let pdg = match pdg {
        -21 | -22 => -pdg,
        0 => return Ok(1.0),
        other => other,
    };
    Ok(pdf.xfx_q(pdg, x, q)? / x)
}

/// `W(variation)` on one event: `get_lo_wgt` term by term on a proton
/// beam pair.
pub fn lo_weight(
    record: &LoRecord,
    kinematics: &RecordKinematics,
    variation: &Variation<'_>,
) -> Result<f64, String> {
    let pdf = variation.pdf;
    let mut chains = record.beams.clone();
    let (mur, muf) = match dynamical_scale(variation.dyn_scale, kinematics)? {
        None => {
            let muf = chains
                .each_ref()
                .map(|c| c.entries.last().map_or(0.0, |e| e.2));
            (record.mu_r, muf)
        }
        Some(scale) => {
            let mut muf = [0.0; 2];
            for (b, chain) in chains.iter_mut().enumerate() {
                if let Some(last) = chain.entries.last_mut() {
                    last.2 = scale;
                    muf[b] = scale;
                }
            }
            (scale, muf)
        }
    };
    let n_rscale = i32::try_from(record.n_qcd).map_err(|e| e.to_string())?
        - i32::try_from(record.emission_scales.len()).map_err(|e| e.to_string())?;
    let mut wgt = pdf.alphas_q(variation.mur * mur)?.powi(n_rscale);
    for (b, chain) in chains.iter().enumerate() {
        if let Some(&(pdg, x, _)) = chain.entries.last() {
            if muf[b] != 0.0 {
                wgt *= density(pdf, PROTON_BEAM * pdg, x, variation.muf * muf[b])?;
            }
        }
    }
    for &scale in &record.emission_scales {
        wgt *= pdf.alphas_q(variation.alps * scale)?;
    }
    for (b, chain) in chains.iter().enumerate() {
        for pair in chain.entries.windows(2) {
            let (pdg, x, q) = pair[0];
            let x_next = pair[1].1;
            let scale = (variation.alps * q).min(variation.muf * muf[b]);
            wgt *= density(pdf, PROTON_BEAM * pdg, x, scale)?;
            wgt /= density(pdf, PROTON_BEAM * pdg, x_next, scale)?;
        }
    }
    Ok(wgt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systematics::record::PdfChain;

    /// A toy member: `αs(Q) = 1/ln Q` and `x·f = x(1 − x)·Q·(pdg + 30)`, so
    /// every term of `W` is a closed form.
    struct Toy;

    impl VariationPdf for Toy {
        fn alphas_q(&self, q: f64) -> Result<f64, String> {
            Ok(1.0 / q.ln())
        }
        fn xfx_q(&self, pdg: i64, x: f64, q: f64) -> Result<f64, String> {
            Ok(x * (1.0 - x) * q * (pdg as f64 + 30.0))
        }
    }

    fn record() -> (LoRecord, RecordKinematics) {
        let record = LoRecord {
            n_qcd: 3,
            mu_r: 50.0,
            emission_scales: vec![20.0],
            beams: [
                PdfChain {
                    entries: vec![(21, 0.2, 20.0), (2, 0.1, 40.0)],
                },
                PdfChain {
                    entries: vec![(-2, 0.3, 45.0)],
                },
            ],
            beam_order: [1, 2],
            tot_fact: 0.0,
        };
        let kinematics = RecordKinematics {
            incoming: vec![[100.0, 0.0, 0.0, 100.0], [60.0, 0.0, 0.0, -60.0]],
            final_state: vec![[80.0, 30.0, 40.0, 10.0], [80.0, -30.0, -40.0, 30.0]],
        };
        (record, kinematics)
    }

    /// Each term of `get_lo_wgt` on a record with an emission and a chain of
    /// two, against the closed form.
    #[test]
    fn the_weight_is_get_lo_wgt_term_by_term() {
        let (record, kinematics) = record();
        let variation = Variation {
            mur: 2.0,
            muf: 0.5,
            alps: 0.5,
            dyn_scale: DynScale::Nominal,
            pdf: &Toy,
        };
        let w = lo_weight(&record, &kinematics, &variation).unwrap();
        let f = |pdg: f64, x: f64, q: f64| (1.0 - x) * q * (pdg + 30.0);
        let expected = (1.0 / 100f64.ln()).powi(2)
            * f(2.0, 0.1, 20.0)
            * f(-2.0, 0.3, 22.5)
            * (1.0 / 10f64.ln())
            * f(21.0, 0.2, 10.0)
            / f(21.0, 0.1, 10.0);
        assert!((w / expected - 1.0).abs() < 1e-14, "{w} vs {expected}");
    }

    /// A dynamical choice replaces `μR` and each chain's last `μF` by the
    /// scale of the written momenta.
    #[test]
    fn a_dynamical_choice_reads_the_momenta() {
        let (mut record, kinematics) = record();
        record.emission_scales.clear();
        record.n_qcd = 2;
        record.beams[0].entries.remove(0);
        let et =
            80.0 * 50.0 / (2500.0f64 + 100.0).sqrt() + 80.0 * 50.0 / (2500.0f64 + 900.0).sqrt();
        assert_eq!(
            dynamical_scale(DynScale::SumEt, &kinematics).unwrap(),
            Some(et)
        );
        // m² + pT² per particle: 3800 + 2500 and 3000 + 2500.
        let ht_got = dynamical_scale(DynScale::Ht, &kinematics).unwrap().unwrap();
        assert!(
            ((ht_got - (6300.0f64.sqrt() + 5500.0f64.sqrt())) / ht_got).abs() < 1e-15,
            "{ht_got}"
        );
        assert_eq!(
            dynamical_scale(DynScale::Sqrts, &kinematics).unwrap(),
            Some((160.0f64 * 160.0 - 40.0 * 40.0).sqrt())
        );
        let variation = Variation {
            mur: 1.0,
            muf: 1.0,
            alps: 1.0,
            dyn_scale: DynScale::Sqrts,
            pdf: &Toy,
        };
        let q = (160.0f64 * 160.0 - 40.0 * 40.0).sqrt();
        let w = lo_weight(&record, &kinematics, &variation).unwrap();
        let expected = (1.0 / q.ln()).powi(2) * (1.0 - 0.1) * q * 32.0 * (1.0 - 0.3) * q * 28.0;
        assert!((w / expected - 1.0).abs() < 1e-14);
    }

    #[test]
    fn a_scale_with_no_reading_is_an_error() {
        let k = RecordKinematics {
            incoming: vec![[1.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, -1.0]],
            final_state: vec![[1.0, 0.0, 0.0, 0.5]],
        };
        assert!(dynamical_scale(DynScale::SumEt, &k).is_err());
        let tachyon = RecordKinematics {
            incoming: k.incoming.clone(),
            final_state: vec![[1.0, 0.0, 0.0, 2.0]],
        };
        assert!(dynamical_scale(DynScale::Ht, &tachyon).is_err());
    }
}
