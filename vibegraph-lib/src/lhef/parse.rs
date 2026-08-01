//! Reading a Les Houches file back into the record types.
//!
//! The reader exists so that a file can be checked against the format oracle it
//! was written to match: MadGraph's own banked events parse into these types, and
//! re-serialising them has to reproduce the source bytes.
//!
//! # Two layers, two parsers
//!
//! The *document* is XML and is read with [`quick_xml`], which locates the
//! `<init>` and `<event>` elements and skips everything else — the banner, its
//! `CDATA` cards, the per-event reweighting blocks. The *bodies* of those two
//! elements are not XML content at all: they are Fortran fixed-format numeric
//! records, and are parsed here field by field.
//!
//! The reader is deliberately lenient about end-tag matching. Files in the wild
//! put unescaped text in the banner, and a banner defect must not cost us the
//! events.

use std::ops::Range;

use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;

use super::record::{BlockSource, LheEvent, LheInit, LheParticle, LheProcess, WeightStrategy};
use super::LhefError;

/// A parsed file: its `<init>` block and every `<event>` in order.
#[derive(Clone, Debug, PartialEq)]
pub struct LheFile {
    pub init: LheInit,
    pub events: Vec<LheEvent>,
}

impl LheFile {
    /// Parse a whole document.
    pub fn parse(text: &str) -> Result<Self, LhefError> {
        let mut reader = Reader::from_str(text);
        reader.config_mut().trim_text(false);
        reader.config_mut().check_end_names = false;

        let mut init = None;
        let mut events = Vec::new();
        loop {
            match reader.read_event().map_err(LhefError::xml)? {
                XmlEvent::Eof => break,
                XmlEvent::Start(start) => {
                    let block = match start.name().as_ref() {
                        b"init" => Block::Init,
                        b"event" => Block::Event,
                        _ => continue,
                    };
                    let span = reader.read_to_end(start.name()).map_err(LhefError::xml)?;
                    let body = &text[span.clone()];
                    match block {
                        Block::Init => {
                            init = Some(locate(text, &span, parse_init(body))?);
                        }
                        Block::Event => events.push(locate(text, &span, parse_event(body))?),
                    }
                }
                _ => {}
            }
        }
        Ok(LheFile {
            init: init.ok_or(LhefError::MissingInit)?,
            events,
        })
    }
}

enum Block {
    Init,
    Event,
}

/// A failure inside one element's body, at a line counted from the body's start.
struct RecordError {
    line: usize,
    reason: String,
}

fn malformed<T>(line: usize, reason: impl Into<String>) -> Result<T, RecordError> {
    Err(RecordError {
        line,
        reason: reason.into(),
    })
}

/// Lift a body-relative failure onto the source line it really sits on. The line
/// count walks the file and so is paid only when something is wrong.
fn locate<T>(
    text: &str,
    span: &Range<usize>,
    result: Result<T, RecordError>,
) -> Result<T, LhefError> {
    result.map_err(|e| LhefError::Malformed {
        line: text[..span.start].matches('\n').count() + e.line,
        reason: e.reason,
    })
}

/// A record line's whitespace-separated fields.
struct Fields<'a> {
    line: usize,
    items: Vec<&'a str>,
}

impl<'a> Fields<'a> {
    fn split((line, text): (usize, &'a str), want: usize, what: &str) -> Result<Self, RecordError> {
        let items: Vec<&str> = text.split_whitespace().collect();
        if items.len() == want {
            Ok(Fields { line, items })
        } else {
            malformed(
                line,
                format!("{what} needs {want} fields, found {}", items.len()),
            )
        }
    }

    fn int(&self, index: usize, name: &str) -> Result<i32, RecordError> {
        self.items[index]
            .parse()
            .or_else(|_| malformed(self.line, format!("{name}: {}", self.items[index])))
    }

    fn count(&self, index: usize, name: &str) -> Result<usize, RecordError> {
        let value = self.int(index, name)?;
        match usize::try_from(value) {
            Ok(n) => Ok(n),
            Err(_) => malformed(self.line, format!("{name} is negative: {value}")),
        }
    }

    /// A double, accepting the Fortran `D` exponent some generators still write.
    fn real(&self, index: usize, name: &str) -> Result<f64, RecordError> {
        let token = self.items[index];
        let normalised;
        let token = if token.contains(['D', 'd']) {
            normalised = token.replace(['D', 'd'], "e");
            normalised.as_str()
        } else {
            token
        };
        token
            .parse()
            .or_else(|_| malformed(self.line, format!("{name}: {token}")))
    }
}

/// An element body's lines, numbered from the element's start tag.
///
/// The body a well-formed block hands over begins with the newline that follows
/// its start tag, so the first content line is line 2 of the element and the
/// leading empty line is not a record.
///
/// The reader tracks how far the records reach so that the text they occupy can
/// be kept alongside the values it decoded to.
struct Body<'a> {
    text: &'a str,
    /// Byte offset of the first line not yet handed out.
    cursor: usize,
    /// The number of the last line handed out, counting from 1.
    number: usize,
    /// Byte offset just past the last line accepted as a record.
    records_end: usize,
}

impl<'a> Body<'a> {
    fn new(body: &'a str) -> Self {
        Body {
            text: body,
            cursor: 0,
            number: 0,
            records_end: 0,
        }
    }

    fn next_line(&mut self) -> Option<&'a str> {
        if self.cursor >= self.text.len() {
            return None;
        }
        let rest = &self.text[self.cursor..];
        let (line, step) = match rest.find('\n') {
            Some(end) => (&rest[..end], end + 1),
            None => (rest, rest.len()),
        };
        self.cursor += step;
        self.number += 1;
        Some(line.strip_suffix('\r').unwrap_or(line))
    }

    /// The next line that carries anything, with its number.
    fn record(&mut self, what: &str) -> Result<(usize, &'a str), RecordError> {
        while let Some(line) = self.next_line() {
            if !line.trim().is_empty() {
                self.records_end = self.cursor;
                return Ok((self.number, line));
            }
        }
        malformed(0, format!("the element ends where {what} was expected"))
    }

    /// The text the records occupy, from the body's start through the newline
    /// that ends the last of them.
    fn source(&self) -> BlockSource {
        BlockSource::new(&self.text[..self.records_end])
    }

    /// Everything left, as written.
    fn rest(mut self) -> Vec<String> {
        let mut rest = Vec::new();
        while let Some(line) = self.next_line() {
            rest.push(line.to_string());
        }
        rest
    }
}

/// The `<init>` beam line's fields, `NPRUP` included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InitHead {
    pub beam_pdg: [i32; 2],
    pub beam_energy: [f64; 2],
    pub pdf_group: [i32; 2],
    pub pdf_set: [i32; 2],
    pub weight_strategy: WeightStrategy,
    pub n_processes: usize,
}

impl InitHead {
    /// The beam line a block's records spell.
    pub(super) fn of(init: &LheInit) -> Self {
        InitHead {
            beam_pdg: init.beam_pdg,
            beam_energy: init.beam_energy,
            pdf_group: init.pdf_group,
            pdf_set: init.pdf_set,
            weight_strategy: init.weight_strategy,
            n_processes: init.processes.len(),
        }
    }
}

/// The `<event>` info line's fields, `NUP` included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EventInfo {
    pub nup: usize,
    pub process_id: i32,
    pub weight: f64,
    pub scale: f64,
    pub alpha_qed: f64,
    pub alpha_qcd: f64,
}

impl EventInfo {
    /// The info line an event's records spell.
    pub(super) fn of(event: &LheEvent) -> Self {
        EventInfo {
            nup: event.nup(),
            process_id: event.process_id,
            weight: event.weight,
            scale: event.scale,
            alpha_qed: event.alpha_qed,
            alpha_qcd: event.alpha_qcd,
        }
    }
}

fn init_head(line: (usize, &str)) -> Result<InitHead, RecordError> {
    let f = Fields::split(line, 10, "the <init> beam line")?;
    Ok(InitHead {
        beam_pdg: [f.int(0, "IDBMUP(1)")?, f.int(1, "IDBMUP(2)")?],
        beam_energy: [f.real(2, "EBMUP(1)")?, f.real(3, "EBMUP(2)")?],
        pdf_group: [f.int(4, "PDFGUP(1)")?, f.int(5, "PDFGUP(2)")?],
        pdf_set: [f.int(6, "PDFSUP(1)")?, f.int(7, "PDFSUP(2)")?],
        weight_strategy: WeightStrategy::from_i32(f.int(8, "IDWTUP")?),
        n_processes: f.count(9, "NPRUP")?,
    })
}

fn init_process(line: (usize, &str)) -> Result<LheProcess, RecordError> {
    let f = Fields::split(line, 4, "an <init> process entry")?;
    Ok(LheProcess {
        xsec_pb: f.real(0, "XSECUP")?,
        xerr_pb: f.real(1, "XERRUP")?,
        xmax: f.real(2, "XMAXUP")?,
        id: f.int(3, "LPRUP")?,
    })
}

fn event_info(line: (usize, &str)) -> Result<EventInfo, RecordError> {
    let f = Fields::split(line, 6, "the <event> info line")?;
    Ok(EventInfo {
        nup: f.count(0, "NUP")?,
        process_id: f.int(1, "IDPRUP")?,
        weight: f.real(2, "XWGTUP")?,
        scale: f.real(3, "SCALUP")?,
        alpha_qed: f.real(4, "AQEDUP")?,
        alpha_qcd: f.real(5, "AQCDUP")?,
    })
}

fn event_particle(line: (usize, &str)) -> Result<LheParticle, RecordError> {
    let f = Fields::split(line, 13, "an <event> particle line")?;
    Ok(LheParticle {
        pdg: f.int(0, "IDUP")?,
        status: f.int(1, "ISTUP")?,
        mothers: [f.int(2, "MOTHUP(1)")?, f.int(3, "MOTHUP(2)")?],
        color: [f.int(4, "ICOLUP(1)")?, f.int(5, "ICOLUP(2)")?],
        // The file writes `px py pz E`; the crate's layout is energy first.
        momentum: [
            f.real(9, "E")?,
            f.real(6, "px")?,
            f.real(7, "py")?,
            f.real(8, "pz")?,
        ],
        mass: f.real(10, "mass")?,
        lifetime: f.real(11, "VTIMUP")?,
        spin: f.real(12, "SPINUP")?,
    })
}

/// What one record line spells, or `None` when it does not spell a record of
/// that kind at all. The writer asks so that it can reuse a source line only
/// where the line still decodes to what it is being asked to write.
pub(super) fn decode_init_head(line: &str) -> Option<InitHead> {
    init_head((0, line)).ok()
}

pub(super) fn decode_init_process(line: &str) -> Option<LheProcess> {
    init_process((0, line)).ok()
}

pub(super) fn decode_event_info(line: &str) -> Option<EventInfo> {
    event_info((0, line)).ok()
}

pub(super) fn decode_event_particle(line: &str) -> Option<LheParticle> {
    event_particle((0, line)).ok()
}

fn parse_init(body: &str) -> Result<LheInit, RecordError> {
    let mut lines = Body::new(body);
    let head = init_head(lines.record("the <init> beam line")?)?;
    let mut processes = Vec::with_capacity(head.n_processes);
    for _ in 0..head.n_processes {
        processes.push(init_process(lines.record("an <init> process entry")?)?);
    }
    let source = lines.source();
    Ok(LheInit {
        beam_pdg: head.beam_pdg,
        beam_energy: head.beam_energy,
        pdf_group: head.pdf_group,
        pdf_set: head.pdf_set,
        weight_strategy: head.weight_strategy,
        processes,
        trailer: lines.rest(),
        source: Some(source),
    })
}

fn parse_event(body: &str) -> Result<LheEvent, RecordError> {
    let mut lines = Body::new(body);
    let head = event_info(lines.record("the <event> info line")?)?;
    let mut particles = Vec::with_capacity(head.nup);
    for _ in 0..head.nup {
        particles.push(event_particle(lines.record("an <event> particle line")?)?);
    }
    let source = lines.source();
    Ok(LheEvent {
        process_id: head.process_id,
        weight: head.weight,
        scale: head.scale,
        alpha_qed: head.alpha_qed,
        alpha_qcd: head.alpha_qcd,
        particles,
        trailer: lines.rest(),
        source: Some(source),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhef::record::{STATUS_INCOMING, STATUS_OUTGOING};
    use crate::lhef::write::{write_event, write_init};

    /// One event of a banked `g g > t t~` run plus the surrounding document,
    /// verbatim.
    const SAMPLE: &str = "\
<LesHouchesEvents version=\"3.0\">
<header>
<!-- a banner with an unescaped & and a < in it -->
<MGGenerationInfo>
#  Number of Events        :       10000
</MGGenerationInfo>
</header>
<init>
21 21 2.500000e+02 2.500000e+02 0 0 0 0 -4 1
1.595319e+01 3.742484e-02 1.595319e+01 1
<generator name='MadGraph5_aMC@NLO' version='3.5.7'>please cite 1405.0301 </generator>
</init>
<event>
 4      1 +1.5953190e+01 2.50000000e+02 7.54677100e-03 1.11330500e-01
       21 -1    0    0  503  502 +0.0000000000e+00 +0.0000000000e+00 +2.5000000000e+02 2.5000000000e+02 0.0000000000e+00 0.0000e+00 1.0000e+00
       21 -1    0    0  501  503 -0.0000000000e+00 -0.0000000000e+00 -2.5000000000e+02 2.5000000000e+02 0.0000000000e+00 0.0000e+00 1.0000e+00
        6  1    1    2  501    0 +8.1272219179e+01 +1.8563195547e+01 -1.6006634300e+02 2.5000000000e+02 1.7300000000e+02 0.0000e+00 1.0000e+00
       -6  1    1    2    0  502 -8.1272219179e+01 -1.8563195547e+01 +1.6006634300e+02 2.5000000000e+02 1.7300000000e+02 0.0000e+00 1.0000e+00
<mgrwt>
<rscale>  0 0.25000000E+03</rscale>
</mgrwt>
</event>
</LesHouchesEvents>
";

    fn parsed() -> LheFile {
        LheFile::parse(SAMPLE).expect("sample parses")
    }

    #[test]
    fn init_fields_land_where_the_accord_puts_them() {
        let init = parsed().init;
        assert_eq!(init.beam_pdg, [21, 21]);
        assert_eq!(init.beam_energy, [250.0, 250.0]);
        assert_eq!(init.pdf_group, [0, 0]);
        assert_eq!(init.pdf_set, [0, 0]);
        assert_eq!(init.weight_strategy.as_i32(), -4);
        assert_eq!(init.processes.len(), 1);
        assert_eq!(init.processes[0].id, 1);
        assert_eq!(init.processes[0].xsec_pb, 15.95319);
        assert_eq!(init.trailer.len(), 1);
    }

    /// The one field permutation in the whole format: the file writes momenta as
    /// `px py pz E` while everything in this crate is `[E, px, py, pz]`. Pinned
    /// with four components that are all different, so a rotation of the tuple
    /// cannot pass.
    #[test]
    fn momentum_components_are_reordered_on_the_way_in() {
        let event = &parsed().events[0];
        let top = event.particles[2];
        assert_eq!(
            top.momentum,
            [250.0, 81.272219179, 18.563195547, -160.066343]
        );
        // The incoming legs keep their physical signs: beam 2 runs down the axis.
        assert_eq!(event.particles[0].momentum[3], 250.0);
        assert_eq!(event.particles[1].momentum[3], -250.0);
        assert_eq!(event.particles[0].status, STATUS_INCOMING);
        assert_eq!(event.particles[2].status, STATUS_OUTGOING);
        assert_eq!(event.particles[2].mothers, [1, 2]);
        assert_eq!(event.particles[0].mothers, [0, 0]);
        // The reweighting block is carried through rather than interpreted.
        assert_eq!(event.trailer.len(), 3);
    }

    /// The reader and writer are inverse on MadGraph's own bytes — the property
    /// the banked-file oracle rests on, checked here on a sample small enough to
    /// read.
    #[test]
    fn madgraph_blocks_round_trip_byte_for_byte() {
        let file = parsed();
        let mut out = Vec::new();
        write_init(&mut out, &file.init).expect("write");
        for event in &file.events {
            write_event(&mut out, event).expect("write");
        }
        let rendered = String::from_utf8(out).expect("ASCII");
        let expected: String = SAMPLE
            .lines()
            .skip_while(|l| l.trim() != "<init>")
            .take_while(|l| l.trim() != "</LesHouchesEvents>")
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(rendered, expected);
    }

    /// A banner defect must not cost the events: the reader walks past unescaped
    /// text and mismatched tags in the header rather than refusing the file.
    #[test]
    fn a_malformed_banner_does_not_hide_the_events() {
        let text = SAMPLE.replace(
            "<MGGenerationInfo>",
            "<MGGenerationInfo>\nptl > 0 & pta < 5\n<unclosed>",
        );
        let file = LheFile::parse(&text).expect("the banner is not the events");
        assert_eq!(file.events.len(), 1);
    }

    #[test]
    fn a_truncated_event_is_an_error_rather_than_a_short_record() {
        let text = SAMPLE.replace(
            "       -6  1    1    2    0  502 -8.1272219179e+01 -1.8563195547e+01 \
             +1.6006634300e+02 2.5000000000e+02 1.7300000000e+02 0.0000e+00 1.0000e+00\n",
            "",
        );
        let err = LheFile::parse(&text).expect_err("NUP promised four legs");
        // The `<mgrwt>` line is where the fourth particle should have been, and the
        // reported line is the source line, not an offset into the element.
        assert!(
            matches!(&err, LhefError::Malformed { line, .. } if *line == 18),
            "unexpected error: {err}"
        );
    }
}
