//! The per-event `<mgrwt>` record: what MadGraph's systematics module reads of
//! an event besides its momenta (`lhe_parser.py`'s `parse_lo_weight`,
//! `:1578-1640`), written as `unwgt.f` writes it (`:767-834`).

use super::format::{as_written_e15_8, fortran_e, fortran_i};
use crate::coupling::cluster::rewgt::{PdfOutcome, Rewgt};
use crate::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_OUTGOING};

/// One beam's density chain: `(pdg, x, q in GeV)` per entry. The last entry is
/// the one the weight reads as the density's flavour, `x` and `μF`; the
/// entries before it are MLM clustering steps.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PdfChain {
    pub entries: Vec<(i64, f64, f64)>,
}

/// What `parse_lo_weight` returns of one event.
#[derive(Clone, Debug, PartialEq)]
pub struct LoRecord {
    /// The event's total QCD coupling power: `<rscale>`'s integer plus the
    /// number of `<asrwt>` scales.
    pub n_qcd: u32,
    /// `<rscale>`'s scale, the event's `μR` in GeV.
    pub mu_r: f64,
    /// `<asrwt>`: the MLM emission scales in GeV, in clustering order.
    pub emission_scales: Vec<f64>,
    /// The chain of each physical beam, indexed by `<pdfrwt beam="b">`'s label
    /// minus one.
    pub beams: [PdfChain; 2],
    /// The beam labels in the order the two `<pdfrwt>` lines are written
    /// (the matrix element's leg order).
    pub beam_order: [u8; 2],
    /// `<totfact>`: written, never read by a weight.
    pub tot_fact: f64,
}

/// What MadEvent fills an event's `<mgrwt>` record from (`reweight.f`'s
/// `setclscales` and `rewgt`), per *clustering* beam: the matrix element's leg
/// order, beam `j` being its leg `j + 1`.
#[derive(Clone, Copy, Debug)]
pub struct RecordInputs<'a> {
    /// The event's QCD coupling power, `nqcd` of the configuration the
    /// clustering named first, or of the channel where nothing was clustered.
    pub n_qcd: u32,
    /// `<rscale>`'s scale, GeV: the `μR` of the last `setclscales` call.
    pub mu_r: f64,
    /// Each clustering beam's incoming code (`idup(j, IPSEL)`).
    pub incoming: [i64; 2],
    /// Each clustering beam's momentum fraction (`xbk(ib(j))`).
    pub x: [f64; 2],
    /// Each clustering beam's factorisation scale when the chain is seeded,
    /// GeV (`√q2fact(j)`).
    pub mu_f: [f64; 2],
    /// The physical beam (`1` or `2`) each clustering beam is written under:
    /// `[2, 1]` on an event whose matrix element saw the beams exchanged
    /// (`unwgt.f`'s `flip`).
    pub labels: [u8; 2],
    /// `asref`, the coupling the matrix element ran at, which `<totfact>`
    /// carries to the event's QCD power; `0` without matching
    /// (`reweight.f:1460`), where it carries none.
    pub asref: f64,
    /// The matched reweighting, factor by factor, in clustering order; `None`
    /// without matching, where `rewgt` returns `1` before reading anything.
    pub rewgt: Option<&'a Rewgt>,
}

/// The written event's momenta a dynamical scale is taken from, lab frame,
/// `[E, px, py, pz]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecordKinematics {
    /// Status `-1` particles.
    pub incoming: Vec<[f64; 4]>,
    /// Status `1` particles.
    pub final_state: Vec<[f64; 4]>,
}

impl RecordKinematics {
    /// The momenta of a written event: its status `-1` and status `1`
    /// particles, in record order.
    pub fn of(event: &LheEvent) -> Self {
        let mut kinematics = RecordKinematics::default();
        for p in &event.particles {
            if p.status == STATUS_INCOMING {
                kinematics.incoming.push(p.momentum);
            } else if p.status == STATUS_OUTGOING {
                kinematics.final_state.push(p.momentum);
            }
        }
        kinematics
    }
}

/// Why an `<mgrwt>` block could not be read.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[error("<mgrwt> record not parsed: {0}")]
pub struct RecordParseError(pub String);

impl LoRecord {
    /// The record MadEvent writes for an event, from what `rewgt` leaves in
    /// its systematics buffers (`reweight.f:1450-1482, 1603-1606, 1715-1724,
    /// 1803-1818`):
    ///
    /// * each beam's chain starts at its incoming parton, at its own `x` and
    ///   the factorisation scale the chain is seeded at, and gains one entry
    ///   per density ratio `rewgt` took on that beam: the flavour of the line
    ///   entering the vertex, its momentum fraction there, and the ratio's
    ///   upper scale;
    /// * one emission scale per `αs` ratio, `√pt2ijcl`, in vertex order;
    /// * `<totfact>` is `rewgt × f₁ f₂ × asref^n_qcd`, the densities at each
    ///   chain's first entry.
    ///
    /// `xfx(pdg, x, q2)` is `x·f(x, Q²)` of the run's central member.
    pub fn fill<E>(
        inputs: &RecordInputs<'_>,
        xfx: impl Fn(i64, f64, f64) -> Result<f64, E>,
    ) -> Result<LoRecord, E> {
        let mut beams = [PdfChain::default(), PdfChain::default()];
        let mut tot_fact = inputs.rewgt.map_or(1.0, |r| r.weight);
        for j in 0..2 {
            let (pdg, x, q) = (inputs.incoming[j], inputs.x[j], inputs.mu_f[j]);
            tot_fact *= xfx(pdg, x, q * q)? / x;
            let chain = &mut beams[usize::from(inputs.labels[j]) - 1];
            chain.entries.push((pdg, x, q));
            if let Some(rewgt) = inputs.rewgt {
                for step in &rewgt.pdf_chain[j] {
                    if let PdfOutcome::Ratio { .. } = step.outcome {
                        chain
                            .entries
                            .push((step.flavour, step.x_entering, step.q2_now.sqrt()));
                    }
                }
            }
        }
        if inputs.asref > 0.0 {
            tot_fact *= inputs.asref.powi(inputs.n_qcd as i32);
        }
        let emission_scales = inputs.rewgt.map_or_else(Vec::new, |r| {
            r.vertices
                .iter()
                .filter_map(|v| v.alpha_s.map(|a| a.q2.sqrt()))
                .collect()
        });
        Ok(LoRecord {
            n_qcd: inputs.n_qcd,
            mu_r: inputs.mu_r,
            emission_scales,
            beams,
            beam_order: inputs.labels,
            tot_fact,
        })
    }

    /// The `<mgrwt>` block's lines, as `unwgt.f` writes them.
    pub fn lines(&self) -> Vec<String> {
        let n_em = self.emission_scales.len() as i64;
        let mut lines = vec!["<mgrwt>".to_string()];
        lines.push(format!(
            "<rscale>{}{}</rscale>",
            fortran_i(i64::from(self.n_qcd) - n_em, 3),
            fortran_e(self.mu_r, 15, 8)
        ));
        if n_em > 0 {
            let scales: String = self
                .emission_scales
                .iter()
                .map(|&q| fortran_e(q, 15, 8))
                .collect();
            lines.push(format!("<asrwt>{}{scales}</asrwt>", fortran_i(n_em, 3)));
        } else {
            lines.push("<asrwt>0</asrwt>".to_string());
        }
        for &label in &self.beam_order {
            let chain = &self.beams[usize::from(label) - 1];
            if chain.entries.is_empty() {
                lines.push(format!("<pdfrwt beam=\"{label}\">0</pdfrwt>"));
                continue;
            }
            let mut body = fortran_i(chain.entries.len() as i64, 3);
            for &(pdg, _, _) in &chain.entries {
                body.push_str(&fortran_i(pdg, 9));
            }
            for &(_, x, _) in &chain.entries {
                body.push_str(&fortran_e(x, 15, 8));
            }
            for &(_, _, q) in &chain.entries {
                body.push_str(&fortran_e(q, 15, 8));
            }
            lines.push(format!("<pdfrwt beam=\"{label}\">{body}</pdfrwt>"));
        }
        lines.push(format!(
            "<totfact>{}</totfact>",
            fortran_e(self.tot_fact, 15, 8)
        ));
        lines.push("</mgrwt>".to_string());
        lines
    }

    /// This record as a reader of its written lines sees it: every real at
    /// the eight significant digits `E15.8` keeps.
    pub fn as_written(&self) -> LoRecord {
        LoRecord {
            n_qcd: self.n_qcd,
            mu_r: as_written_e15_8(self.mu_r),
            emission_scales: self
                .emission_scales
                .iter()
                .map(|&q| as_written_e15_8(q))
                .collect(),
            beams: self.beams.clone().map(|chain| PdfChain {
                entries: chain
                    .entries
                    .into_iter()
                    .map(|(pdg, x, q)| (pdg, as_written_e15_8(x), as_written_e15_8(q)))
                    .collect(),
            }),
            beam_order: self.beam_order,
            tot_fact: as_written_e15_8(self.tot_fact),
        }
    }

    /// Parse the `<mgrwt>` block among `lines` (an event's trailer), as
    /// `parse_lo_weight` does: `None` when the event has none.
    pub fn parse(lines: &[String]) -> Result<Option<LoRecord>, RecordParseError> {
        let Some(start) = lines.iter().position(|l| l.trim() == "<mgrwt>") else {
            return Ok(None);
        };
        let text: String = lines[start + 1..]
            .iter()
            .take_while(|l| l.trim() != "</mgrwt>")
            .map(|l| format!("{l}\n"))
            .collect();
        let err = |why: &str| RecordParseError(format!("{why}: {text}"));
        let inner = |tag: &str| -> Result<String, RecordParseError> {
            let open = format!("<{tag}>");
            let close = format!("</{tag}>");
            let a = text
                .find(&open)
                .ok_or_else(|| err(&format!("no <{tag}>")))?
                + open.len();
            let b = text[a..]
                .find(&close)
                .ok_or_else(|| err(&format!("no </{tag}>")))?;
            Ok(text[a..a + b].to_string())
        };
        let num = |s: &str| -> Result<f64, RecordParseError> {
            s.parse()
                .map_err(|_| err(&format!("'{s}' is not a number")))
        };
        let int = |s: &str| -> Result<i64, RecordParseError> {
            s.parse()
                .map_err(|_| err(&format!("'{s}' is not an integer")))
        };

        let rscale = inner("rscale")?;
        let fields: Vec<&str> = rscale.split_whitespace().collect();
        let [n, mu_r] = fields.as_slice() else {
            return Err(err("<rscale> holds two fields"));
        };
        let rscale_n = int(n)?;
        let mu_r = num(mu_r)?;
        let asrwt = inner("asrwt")?;
        let emission_scales = asrwt
            .split_whitespace()
            .skip(1)
            .map(num)
            .collect::<Result<Vec<_>, _>>()?;
        let tot_fact = num(inner("totfact")?.trim())?;

        let mut beams = [PdfChain::default(), PdfChain::default()];
        let mut order = Vec::new();
        let mut rest = text.as_str();
        while let Some(at) = rest.find("<pdfrwt") {
            rest = &rest[at..];
            let label_start = rest
                .find("beam=")
                .ok_or_else(|| err("<pdfrwt> without a beam"))?
                + 5;
            let label: String = rest[label_start..]
                .chars()
                .filter(|c| !matches!(c, '"' | '\''))
                .take_while(char::is_ascii_digit)
                .collect();
            let label: u8 = match label.as_str() {
                "1" => 1,
                "2" => 2,
                _ => return Err(err("a <pdfrwt> beam is 1 or 2")),
            };
            let body_start = rest.find('>').ok_or_else(|| err("unclosed <pdfrwt>"))? + 1;
            let body_end = rest.find("</pdfrwt>").ok_or_else(|| err("no </pdfrwt>"))?;
            let fields: Vec<&str> = rest[body_start..body_end].split_whitespace().collect();
            let n = int(fields.first().ok_or_else(|| err("empty <pdfrwt>"))?)?;
            let n = usize::try_from(n).map_err(|_| err("negative <pdfrwt> count"))?;
            if fields.len() != 1 + 3 * n {
                return Err(err("a <pdfrwt> holds 1 + 3n fields"));
            }
            let mut entries = Vec::with_capacity(n);
            for i in 0..n {
                entries.push((
                    int(fields[1 + i])?,
                    num(fields[1 + n + i])?,
                    num(fields[1 + 2 * n + i])?,
                ));
            }
            beams[usize::from(label) - 1] = PdfChain { entries };
            order.push(label);
            rest = &rest[body_end + 9..];
        }
        let beam_order: [u8; 2] = match order.as_slice() {
            [a, b] if a != b => [*a, *b],
            _ => return Err(err("two <pdfrwt> lines, one per beam")),
        };
        let n_qcd = u32::try_from(rscale_n + emission_scales.len() as i64)
            .map_err(|_| err("a negative QCD power"))?;
        Ok(Some(LoRecord {
            n_qcd,
            mu_r,
            emission_scales,
            beams,
            beam_order,
            tot_fact,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::cluster::rewgt::{AlphaSRatio, PdfStep, VertexClass, VertexDecision};

    /// A toy `x·f`: a different power of `x` and `Q²` per flavour, so a density
    /// read for the wrong flavour, at the wrong fraction or scale differs.
    fn xfx(pdg: i64, x: f64, q2: f64) -> Result<f64, ()> {
        let k = (pdg.unsigned_abs() % 7) as f64 + 1.0;
        Ok(x.powf(0.1 * k) * (1.0 - x) * q2.ln() * (1.0 + 0.01 * pdg as f64))
    }

    fn step(
        n: usize,
        flavour: i64,
        x: f64,
        x_entering: f64,
        q2_now: f64,
        o: PdfOutcome,
    ) -> PdfStep {
        PdfStep {
            n,
            line: 0,
            flavour,
            x,
            x_entering,
            q2_prev: 1.0,
            q2_now,
            outcome: o,
        }
    }

    fn vertex(n: usize, class: VertexClass, q2: Option<f64>) -> VertexDecision {
        VertexDecision {
            n,
            mother: 0,
            daughters: [0, 0],
            pdg: [0; 3],
            ipart: 1,
            class,
            alpha_s: q2.map(|q2| AlphaSRatio {
                q2,
                numerator: 0.2,
                denominator: 0.1,
                ratio: 2.0,
            }),
        }
    }

    const RATIO: PdfOutcome = PdfOutcome::Ratio {
        numerator: 1.0,
        denominator: 1.0,
        ratio: 1.0,
    };

    /// The banked `pp_to_ll_0j2j_mlm` event `g u → e⁺e⁻ u` (its first),
    /// rebuilt from what `rewgt` left: one ISR `αs` ratio at 51.098 GeV and,
    /// on the gluon beam, one density ratio for the `ū` line entering the
    /// core, recorded at that line's fraction and the ratio's upper scale.
    /// The beam's first vertex and a vertex whose scale did not rise take no
    /// ratio and add no entry. Every line but `<totfact>` is MadGraph's own;
    /// `<totfact>` is `rewgt · f₁ f₂ · asref^n` at the chains' first entries.
    #[test]
    fn a_matched_record_is_what_rewgt_left_in_its_buffers() {
        let (q_isr, q_core) = (51.098437, 104.78542);
        let rewgt = Rewgt {
            weight: 0.75,
            vertices: vec![
                vertex(1, VertexClass::Isr, Some(q_isr * q_isr)),
                vertex(2, VertexClass::FsrNotPartonVertex, None),
                vertex(3, VertexClass::Core, None),
            ],
            pdf_chain: [
                vec![
                    step(
                        1,
                        21,
                        0.0012,
                        0.0085277455,
                        q_isr * q_isr,
                        PdfOutcome::First,
                    ),
                    step(3, -2, 0.0005, 0.00083878028, q_core * q_core, RATIO),
                ],
                vec![step(
                    3,
                    2,
                    0.059038495,
                    0.059038495,
                    900.0,
                    PdfOutcome::NoRise,
                )],
            ],
            kill: None,
        };
        let inputs = RecordInputs {
            n_qcd: 1,
            mu_r: q_core,
            incoming: [21, 2],
            x: [0.0085277455, 0.059038495],
            mu_f: [q_isr, q_isr],
            labels: [1, 2],
            asref: 0.13,
            rewgt: Some(&rewgt),
        };
        let record = LoRecord::fill(&inputs, xfx).unwrap();
        let lines = record.lines();
        assert_eq!(
            lines[..5],
            [
                "<mgrwt>",
                "<rscale>  0 0.10478542E+03</rscale>",
                "<asrwt>  1 0.51098437E+02</asrwt>",
                "<pdfrwt beam=\"1\">  2       21       -2 0.85277455E-02 0.83878028E-03 \
                 0.51098437E+02 0.10478542E+03</pdfrwt>",
                "<pdfrwt beam=\"2\">  1        2 0.59038495E-01 0.51098437E+02</pdfrwt>",
            ]
        );
        let f = |pdg: i64, x: f64, q: f64| xfx(pdg, x, q * q).unwrap() / x;
        let expected = 0.75 * f(21, 0.0085277455, q_isr) * f(2, 0.059038495, q_isr) * 0.13;
        assert_eq!(record.tot_fact, expected);
        assert_eq!(record.n_qcd, 1);
    }

    /// On an event whose matrix element saw the beams exchanged, clustering
    /// beam 1 is the physical beam 2: its chain is written first, labelled
    /// `beam="2"`, and read back under that label.
    #[test]
    fn an_exchanged_event_writes_each_chain_under_its_physical_beam() {
        let rewgt = Rewgt {
            weight: 1.0,
            vertices: vec![vertex(1, VertexClass::Core, None)],
            pdf_chain: [vec![step(1, 3, 0.02, 0.04, 400.0, RATIO)], Vec::new()],
            kill: None,
        };
        let inputs = RecordInputs {
            n_qcd: 0,
            mu_r: 88.932649,
            incoming: [3, -3],
            x: [0.0042275231, 0.011070055],
            mu_f: [10.0, 88.932649],
            labels: [2, 1],
            asref: 0.11,
            rewgt: Some(&rewgt),
        };
        let record = LoRecord::fill(&inputs, xfx).unwrap();
        let lines = record.lines();
        assert_eq!(
            lines[3],
            "<pdfrwt beam=\"2\">  2        3        3 0.42275231E-02 0.40000000E-01 \
             0.10000000E+02 0.20000000E+02</pdfrwt>"
        );
        assert_eq!(
            lines[4],
            "<pdfrwt beam=\"1\">  1       -3 0.11070055E-01 0.88932649E+02</pdfrwt>"
        );
        let back = LoRecord::parse(&lines).unwrap().unwrap();
        assert_eq!(back.beams[1].entries[0].0, 3);
        assert_eq!(back.beams[0].entries, [(-3, 0.011070055, 88.932649)]);
    }

    /// Without matching the chain is the incoming parton alone and the total
    /// factor `f₁ f₂`, with no coupling whatever the QCD power (`asref = 0`).
    #[test]
    fn an_unmatched_record_carries_one_entry_per_beam_and_no_coupling() {
        let inputs = RecordInputs {
            n_qcd: 2,
            mu_r: 91.188,
            incoming: [21, -1],
            x: [0.1, 0.2],
            mu_f: [45.0, 46.0],
            labels: [1, 2],
            asref: 0.0,
            rewgt: None,
        };
        let record = LoRecord::fill(&inputs, xfx).unwrap();
        assert_eq!(record.emission_scales, Vec::<f64>::new());
        assert_eq!(record.beams[0].entries, [(21, 0.1, 45.0)]);
        assert_eq!(record.beams[1].entries, [(-1, 0.2, 46.0)]);
        let f = |pdg: i64, x: f64, q: f64| xfx(pdg, x, q * q).unwrap() / x;
        assert_eq!(record.tot_fact, f(21, 0.1, 45.0) * f(-1, 0.2, 46.0));
        assert_eq!(record.lines()[1], "<rscale>  2 0.91188000E+02</rscale>");
    }

    /// A banked `pp_to_ll` event's record, as MadGraph wrote it.
    const BANKED: &[&str] = &[
        "<mgrwt>",
        "<rscale>  0 0.88513229E+02</rscale>",
        "<asrwt>0</asrwt>",
        "<pdfrwt beam=\"1\">  1        1 0.18237367E-02 0.88513229E+02</pdfrwt>",
        "<pdfrwt beam=\"2\">  1       -1 0.25419530E-01 0.88513229E+02</pdfrwt>",
        "<totfact> 0.78916645E+04</totfact>",
        "</mgrwt>",
    ];

    /// MadGraph's own lines parse, and writing the parsed record gives the
    /// same bytes back.
    #[test]
    fn a_banked_record_round_trips_byte_for_byte() {
        let lines: Vec<String> = BANKED.iter().map(|l| l.to_string()).collect();
        let record = LoRecord::parse(&lines).unwrap().unwrap();
        assert_eq!(record.n_qcd, 0);
        assert_eq!(record.mu_r, 88.513229);
        assert_eq!(record.beams[1].entries, [(-1, 0.02541953, 88.513229)]);
        assert_eq!(record.lines(), lines);
        assert_eq!(record.as_written(), record);
    }

    /// A chain of several entries, emission scales, and a flipped event whose
    /// beam 2 line comes first: written in `unwgt.f`'s layout and read back by
    /// label.
    #[test]
    fn a_matched_record_writes_and_reads_by_label() {
        let record = LoRecord {
            n_qcd: 4,
            mu_r: 91.188,
            emission_scales: vec![24.18643, 129.76006],
            beams: [
                PdfChain {
                    entries: vec![(21, 0.0011720939, 24.18643), (4, 0.00079266617, 129.76006)],
                },
                PdfChain {
                    entries: vec![(21, 0.01, 129.76006)],
                },
            ],
            beam_order: [2, 1],
            tot_fact: 1.5,
        };
        let lines = record.lines();
        assert_eq!(lines[1], "<rscale>  2 0.91188000E+02</rscale>");
        assert_eq!(lines[2], "<asrwt>  2 0.24186430E+02 0.12976006E+03</asrwt>");
        assert!(lines[3].starts_with("<pdfrwt beam=\"2\">  1       21"));
        assert_eq!(
            lines[4],
            "<pdfrwt beam=\"1\">  2       21        4 0.11720939E-02 0.79266617E-03 \
             0.24186430E+02 0.12976006E+03</pdfrwt>"
        );
        let back = LoRecord::parse(&lines).unwrap().unwrap();
        assert_eq!(back, record.as_written());
        assert!(LoRecord::parse(&["<event>".to_string()]).unwrap().is_none());
    }
}
