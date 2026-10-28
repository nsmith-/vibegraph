//! The per-event `<mgrwt>` record: what MadGraph's systematics module reads of
//! an event besides its momenta (`lhe_parser.py`'s `parse_lo_weight`,
//! `:1578-1640`), written as `unwgt.f` writes it (`:767-834`).

use super::format::{as_written_e15_8, fortran_e, fortran_i};
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
