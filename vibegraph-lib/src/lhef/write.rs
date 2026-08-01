//! Serialising the record types into a Les Houches file.
//!
//! Two layers, matching [`parse`](super::parse). The *document* — the
//! `<LesHouchesEvents>` root, `<header>` and its comment, and the `<init>` and
//! `<event>` elements — goes through [`quick_xml`], so tag and attribute spelling
//! and the escaping of any authored text are the library's business. The *bodies*
//! of `<init>` and `<event>` are Fortran fixed-format numeric records rather than
//! XML content, and are written here column by column.

use std::io::{self, Write};

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event as XmlEvent};
use quick_xml::Writer;

use super::parse::{
    decode_event_info, decode_event_particle, decode_init_head, decode_init_process, EventInfo,
    InitHead,
};
use super::record::{BlockSource, LheEvent, LheInit, LheParticle, LheProcess};
use super::LHE_VERSION;

/// A C `%[+]w.pe` conversion.
///
/// Rust's `LowerExp` writes the exponent with neither a sign nor a minimum width,
/// so `2.5e2` has to become `2.50000000e+02` before any consumer written against
/// the accord's Fortran layout will read it. A non-finite value is written as
/// itself (`NaN`, `inf`), which no parser accepts — a record carrying one is a bug
/// upstream, and this makes it fail loudly at the file rather than silently at the
/// physics.
fn c_exponential(value: f64, precision: usize, force_sign: bool, width: usize) -> String {
    let formatted = if !value.is_finite() {
        value.to_string()
    } else {
        let mantissa_exp = if force_sign {
            format!("{value:+.precision$e}")
        } else {
            format!("{value:.precision$e}")
        };
        let (mantissa, exponent) = mantissa_exp
            .split_once('e')
            .expect("a finite float always formats with an exponent");
        let exponent: i32 = exponent.parse().expect("the exponent is an integer");
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exponent.abs())
    };
    pad(formatted, width)
}

fn pad(s: String, width: usize) -> String {
    if s.len() >= width {
        s
    } else {
        format!("{s:>width$}")
    }
}

/// A block's record lines: each one the source spelled and still decodes to,
/// re-emitted as written, and this writer's own layout for the rest.
///
/// The reuse is checked, not assumed. `decodes` reads a source line back and
/// asks whether it spells the record now being written, so a caller that edited
/// a field cannot get the stale spelling: that line falls to `render` while the
/// rest of the block keeps the file's own numeric dialect. A source whose
/// record-line count no longer matches the record — a leg added or dropped —
/// cannot be matched up line by line and is discarded whole.
///
/// A non-finite field never decodes back to itself, so a record carrying one
/// always takes the `render` path and fails loudly at the file.
fn record_block(
    source: Option<&BlockSource>,
    records: usize,
    render: impl Fn(usize) -> String,
    decodes: impl Fn(usize, &str) -> bool,
) -> String {
    let lines: Vec<&str> = match source.map(BlockSource::as_str) {
        Some(text) => text.lines().collect(),
        None => Vec::new(),
    };
    if lines.iter().filter(|l| !l.trim().is_empty()).count() != records {
        let mut body = String::from("\n");
        for index in 0..records {
            body.push_str(&render(index));
        }
        return body;
    }
    let mut body = String::new();
    let mut index = 0;
    for line in lines {
        if line.trim().is_empty() {
            body.push_str(line);
            body.push('\n');
            continue;
        }
        if decodes(index, line) {
            body.push_str(line);
            body.push('\n');
        } else {
            body.push_str(&render(index));
        }
        index += 1;
    }
    body
}

/// The `<init>` body: the beam line, one line per process, then whatever the
/// block carried after them.
fn init_body(init: &LheInit) -> String {
    let mut body = record_block(
        init.source.as_ref(),
        init.processes.len() + 1,
        |index| match index {
            0 => beam_line(init),
            i => process_line(&init.processes[i - 1]),
        },
        |index, line| match index {
            0 => decode_init_head(line) == Some(InitHead::of(init)),
            i => decode_init_process(line) == Some(init.processes[i - 1]),
        },
    );
    push_trailer(&mut body, &init.trailer);
    body
}

fn beam_line(init: &LheInit) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {}\n",
        init.beam_pdg[0],
        init.beam_pdg[1],
        c_exponential(init.beam_energy[0], 6, false, 0),
        c_exponential(init.beam_energy[1], 6, false, 0),
        init.pdf_group[0],
        init.pdf_group[1],
        init.pdf_set[0],
        init.pdf_set[1],
        init.weight_strategy.as_i32(),
        init.processes.len(),
    )
}

fn process_line(process: &LheProcess) -> String {
    format!(
        "{} {} {} {}\n",
        c_exponential(process.xsec_pb, 6, false, 0),
        c_exponential(process.xerr_pb, 6, false, 0),
        c_exponential(process.xmax, 6, false, 0),
        process.id,
    )
}

/// The `<event>` body: the info line, one line per leg, then whatever the block
/// carried after them.
fn event_body(event: &LheEvent) -> String {
    let mut body = record_block(
        event.source.as_ref(),
        event.nup() + 1,
        |index| match index {
            0 => info_line(event),
            i => particle_line(&event.particles[i - 1]),
        },
        |index, line| match index {
            0 => decode_event_info(line) == Some(EventInfo::of(event)),
            i => decode_event_particle(line) == Some(event.particles[i - 1]),
        },
    );
    push_trailer(&mut body, &event.trailer);
    body
}

fn info_line(event: &LheEvent) -> String {
    format!(
        "{} {} {} {} {} {}\n",
        pad(event.nup().to_string(), 2),
        pad(event.process_id.to_string(), 6),
        c_exponential(event.weight, 7, true, 13),
        c_exponential(event.scale, 8, false, 14),
        c_exponential(event.alpha_qed, 8, false, 14),
        c_exponential(event.alpha_qcd, 8, false, 14),
    )
}

fn particle_line(p: &LheParticle) -> String {
    let [e, px, py, pz] = p.momentum;
    format!(
        " {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
        pad(p.pdg.to_string(), 8),
        pad(p.status.to_string(), 2),
        pad(p.mothers[0].to_string(), 4),
        pad(p.mothers[1].to_string(), 4),
        pad(p.color[0].to_string(), 4),
        pad(p.color[1].to_string(), 4),
        c_exponential(px, 10, true, 13),
        c_exponential(py, 10, true, 13),
        c_exponential(pz, 10, true, 13),
        c_exponential(e, 10, false, 14),
        c_exponential(p.mass, 10, false, 14),
        c_exponential(p.lifetime, 4, false, 10),
        c_exponential(p.spin, 4, false, 10),
    )
}

/// Lines the block carried past its records, re-emitted as written. They are
/// markup the record types do not model — MadGraph's `<generator>` tag, its
/// per-event `<mgrwt>` and `<rwgt>` blocks — so reproducing them means copying
/// them, not reformatting them.
fn push_trailer(body: &mut String, trailer: &[String]) {
    for line in trailer {
        body.push_str(line);
        body.push('\n');
    }
}

fn to_io(error: quick_xml::Error) -> io::Error {
    match error {
        quick_xml::Error::Io(err) => io::Error::new(err.kind(), err.to_string()),
        other => io::Error::other(other.to_string()),
    }
}

/// An element whose body is a fixed-format record block, plus the newline that
/// separates it from the next element.
fn write_block(out: &mut impl Write, name: &str, body: &str) -> io::Result<()> {
    let mut writer = Writer::new(out);
    for event in [
        XmlEvent::Start(BytesStart::new(name)),
        XmlEvent::Text(BytesText::from_escaped(body)),
        XmlEvent::End(BytesEnd::new(name)),
        XmlEvent::Text(BytesText::from_escaped("\n")),
    ] {
        writer.write_event(event).map_err(to_io)?;
    }
    Ok(())
}

/// Write the `<init>` block, tags included.
pub fn write_init(out: &mut impl Write, init: &LheInit) -> io::Result<()> {
    write_block(out, "init", &init_body(init))
}

/// Write one `<event>` block, tags included.
pub fn write_event(out: &mut impl Write, event: &LheEvent) -> io::Result<()> {
    write_block(out, "event", &event_body(event))
}

/// The `<generator>` element the accord expects inside `<init>`, ready to be put
/// in [`LheInit::trailer`](super::record::LheInit::trailer).
///
/// Built through the XML writer, so `note` is escaped rather than pasted.
pub fn generator_element(name: &str, version: &str, note: &str) -> String {
    let mut writer = Writer::new(Vec::new());
    let mut tag = BytesStart::new("generator");
    tag.push_attribute(("name", name));
    tag.push_attribute(("version", version));
    for event in [
        XmlEvent::Start(tag),
        XmlEvent::Text(BytesText::new(note)),
        XmlEvent::End(BytesEnd::new("generator")),
    ] {
        writer
            .write_event(event)
            .expect("writing to a Vec cannot fail");
    }
    String::from_utf8(writer.into_inner()).expect("the generator element is ASCII")
}

/// Streaming writer for a whole file: root tag, header, `<init>`, then events one
/// at a time.
///
/// Events are written as they arrive rather than collected, so a run's length is
/// bounded by disk and not by memory. That does mean `<init>` — and so `XSECUP`,
/// `XERRUP` and `XMAXUP` — is fixed before the first event exists, which is the
/// right way round: those come from the integration the generation replays, not
/// from the sample it produces.
pub struct LheWriter<W: Write> {
    out: W,
    events: u64,
}

impl<W: Write> LheWriter<W> {
    /// Open the document and write the header and `<init>` block.
    ///
    /// `header` is embedded as an XML comment inside `<header>`; the accord leaves
    /// that block free-form and consumers ignore it, so it is where a run's
    /// provenance goes.
    pub fn begin(mut out: W, init: &LheInit, header: Option<&str>) -> io::Result<Self> {
        {
            let mut writer = Writer::new(&mut out);
            let mut root = BytesStart::new("LesHouchesEvents");
            root.push_attribute(("version", LHE_VERSION));
            let mut events = vec![
                XmlEvent::Start(root),
                XmlEvent::Text(BytesText::from_escaped("\n")),
                XmlEvent::Start(BytesStart::new("header")),
                XmlEvent::Text(BytesText::from_escaped("\n")),
            ];
            if let Some(header) = header {
                // A comment body cannot contain `--`, so a header that does would
                // produce a file no parser accepts; blanking the pair keeps the
                // document well-formed and the provenance readable.
                let body = format!("\n{}\n", header.replace("--", "- -"));
                events.push(XmlEvent::Comment(BytesText::from_escaped(body)));
                events.push(XmlEvent::Text(BytesText::from_escaped("\n")));
            }
            events.push(XmlEvent::End(BytesEnd::new("header")));
            events.push(XmlEvent::Text(BytesText::from_escaped("\n")));
            for event in events {
                writer.write_event(event).map_err(to_io)?;
            }
        }
        write_init(&mut out, init)?;
        Ok(LheWriter { out, events: 0 })
    }

    /// Append one event.
    pub fn write_event(&mut self, event: &LheEvent) -> io::Result<()> {
        write_event(&mut self.out, event)?;
        self.events += 1;
        Ok(())
    }

    /// How many events have been written.
    pub fn events_written(&self) -> u64 {
        self.events
    }

    /// Close the document and give the sink back.
    pub fn finish(mut self) -> io::Result<W> {
        {
            let mut writer = Writer::new(&mut self.out);
            for event in [
                XmlEvent::End(BytesEnd::new("LesHouchesEvents")),
                XmlEvent::Text(BytesText::from_escaped("\n")),
            ] {
                writer.write_event(event).map_err(to_io)?;
            }
        }
        self.out.flush()?;
        Ok(self.out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhef::parse::LheFile;
    use crate::lhef::record::{
        LheProcess, WeightStrategy, SPIN_UNKNOWN, STATUS_INCOMING, STATUS_OUTGOING,
    };

    fn rendered(f: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        f(&mut buf).expect("writing to a Vec cannot fail");
        String::from_utf8(buf).expect("ASCII")
    }

    /// The exponent conversion, against the C/Python spellings the layout is
    /// defined in. Rust's own `{:e}` produces none of these.
    #[test]
    fn exponentials_match_the_c_conversions() {
        assert_eq!(c_exponential(250.0, 8, false, 14), "2.50000000e+02");
        assert_eq!(c_exponential(15.95319, 7, true, 13), "+1.5953190e+01");
        assert_eq!(c_exponential(0.1113305, 8, false, 14), "1.11330500e-01");
        assert_eq!(c_exponential(1.0, 4, false, 10), "1.0000e+00");
        assert_eq!(c_exponential(-1.0, 4, false, 10), "-1.0000e+00");
        assert_eq!(c_exponential(0.0, 10, true, 13), "+0.0000000000e+00");
        assert_eq!(
            c_exponential(81.272219179, 10, true, 13),
            "+8.1272219179e+01"
        );
        // Three-digit exponents keep all their digits rather than being truncated
        // to the two the padding guarantees.
        assert_eq!(c_exponential(1e-300, 4, false, 10), "1.0000e-300");
        // The width pads only when the value is narrower than the field, which for
        // these conversions happens for nothing but a short integer-like value.
        assert_eq!(c_exponential(1.0, 4, false, 12), "  1.0000e+00");
    }

    /// A negative zero is a real distinction in a momentum column: MadGraph writes
    /// the second beam's transverse components as `-0.0`, and a formatter that
    /// normalised the sign away would silently rewrite every such line.
    #[test]
    fn negative_zero_keeps_its_sign() {
        assert_eq!(c_exponential(-0.0, 10, true, 13), "-0.0000000000e+00");
        assert_eq!(c_exponential(0.0, 10, true, 13), "+0.0000000000e+00");
    }

    fn init() -> LheInit {
        LheInit {
            beam_pdg: [21, 21],
            beam_energy: [250.0, 250.0],
            pdf_group: [0, 0],
            pdf_set: [0, 0],
            weight_strategy: WeightStrategy::MeanCrossSectionPb,
            processes: vec![LheProcess {
                xsec_pb: 15.95319,
                xerr_pb: 0.03742484,
                xmax: 15.95319,
                id: 1,
            }],
            trailer: vec!["<generator name='vibegraph'>x</generator>".to_string()],
            source: None,
        }
    }

    fn event() -> LheEvent {
        let beam = |pz: f64, sign: f64| LheParticle {
            pdg: 21,
            status: STATUS_INCOMING,
            mothers: [0, 0],
            color: [503, 502],
            momentum: [250.0, sign * 0.0, sign * 0.0, pz],
            mass: 0.0,
            lifetime: 0.0,
            spin: 1.0,
        };
        LheEvent {
            process_id: 1,
            weight: 15.95319,
            scale: 250.0,
            alpha_qed: 0.007546771,
            alpha_qcd: 0.1113305,
            particles: vec![
                beam(250.0, 1.0),
                beam(-250.0, -1.0),
                LheParticle {
                    pdg: 6,
                    status: STATUS_OUTGOING,
                    mothers: [1, 2],
                    color: [501, 0],
                    momentum: [250.0, 81.272219179, 18.563195547, -160.066343],
                    mass: 173.0,
                    lifetime: 0.0,
                    spin: SPIN_UNKNOWN,
                },
            ],
            trailer: Vec::new(),
            source: None,
        }
    }

    #[test]
    fn init_block_matches_the_madgraph_layout() {
        assert_eq!(
            rendered(|out| write_init(out, &init())),
            "<init>\n\
             21 21 2.500000e+02 2.500000e+02 0 0 0 0 -4 1\n\
             1.595319e+01 3.742484e-02 1.595319e+01 1\n\
             <generator name='vibegraph'>x</generator>\n\
             </init>\n"
        );
    }

    #[test]
    fn event_block_matches_the_madgraph_layout() {
        assert_eq!(
            rendered(|out| write_event(out, &event())),
            "<event>\n\
             \x203      1 +1.5953190e+01 2.50000000e+02 7.54677100e-03 1.11330500e-01\n\
             \x20      21 -1    0    0  503  502 +0.0000000000e+00 +0.0000000000e+00 \
             +2.5000000000e+02 2.5000000000e+02 0.0000000000e+00 0.0000e+00 1.0000e+00\n\
             \x20      21 -1    0    0  503  502 -0.0000000000e+00 -0.0000000000e+00 \
             -2.5000000000e+02 2.5000000000e+02 0.0000000000e+00 0.0000e+00 1.0000e+00\n\
             \x20       6  1    1    2  501    0 +8.1272219179e+01 +1.8563195547e+01 \
             -1.6006634300e+02 2.5000000000e+02 1.7300000000e+02 0.0000e+00 9.0000e+00\n\
             </event>\n"
        );
    }

    /// The document a full run produces, read back by the same reader that reads
    /// MadGraph's files.
    #[test]
    fn a_written_document_parses_back_into_the_records_it_came_from() {
        let init = init();
        let event = event();
        let text = rendered(|out| {
            let mut writer = LheWriter::begin(out, &init, Some("vibegraph run\nseed 1"))?;
            writer.write_event(&event)?;
            writer.write_event(&event)?;
            assert_eq!(writer.events_written(), 2);
            writer.finish().map(|_| ())
        });
        assert!(text.starts_with("<LesHouchesEvents version=\"3.0\">\n<header>\n<!--\n"));
        assert!(text.ends_with("</LesHouchesEvents>\n"));
        let file = LheFile::parse(&text).expect("our own output parses");
        assert_eq!(file.init, init);
        assert_eq!(file.events, vec![event.clone(), event]);
    }

    /// A `--` in the provenance text would close the header comment early and
    /// break the document; it is neutralised rather than allowed to escape.
    #[test]
    fn a_header_comment_cannot_be_closed_early_by_its_own_text() {
        let text = rendered(|out| {
            let writer = LheWriter::begin(out, &init(), Some("run --nevents 10 --seed 3"))?;
            writer.finish().map(|_| ())
        });
        assert!(text.contains("run - -nevents 10 - -seed 3"), "{text}");
        assert!(LheFile::parse(&text).is_ok());
    }

    /// One `g g > t t~` event as MadGraph 3.7.1 delivers it when its own
    /// post-processing took the fast path that never converts the numbers:
    /// three-digit-mantissa Fortran momenta, `0.` and `-1.` where the converted
    /// spelling has `0.0000e+00` and `-1.0000e+00`, and an info line with no
    /// column padding at all. Nothing in this crate emits any of it.
    const PASS_THROUGH: &str = "\
<LesHouchesEvents version=\"3.0\">
<init>
21 21 2.500000e+02 2.500000e+02 0 0 247000 247000 -4 1
1.351348e+01 1.628904e-02 1.351348e+01 1
</init>
<event>
4 1 +1.3513480e+01 0.2500000E+03 0.7546771E-02 0.1024649E+00
21   -1    0    0  501  502  0.00000000000E+00  0.00000000000E+00  0.25000000000E+03  0.25000000000E+03  0.00000000000E+00 0. -1.
21   -1    0    0  502  503 -0.00000000000E+00 -0.00000000000E+00 -0.25000000000E+03  0.25000000000E+03  0.00000000000E+00 0.  1.
6    1    1    2  501    0  0.13402070006E+03 -0.68879185628E+02  0.99323258824E+02  0.25000000000E+03  0.17300000000E+03 0. -1.
-6    1    1    2    0  503 -0.13402070006E+03  0.68879185628E+02 -0.99323258824E+02  0.25000000000E+03  0.17300000000E+03 0.  1.
</event>
</LesHouchesEvents>
";

    /// The record span of [`PASS_THROUGH`]: what re-serialising it has to
    /// reproduce.
    fn pass_through_records() -> String {
        PASS_THROUGH
            .lines()
            .skip_while(|l| l.trim() != "<init>")
            .take_while(|l| l.trim() != "</LesHouchesEvents>")
            .map(|l| format!("{l}\n"))
            .collect()
    }

    fn serialise(file: &LheFile) -> String {
        rendered(|out| {
            write_init(out, &file.init)?;
            for event in &file.events {
                write_event(out, event)?;
            }
            Ok(())
        })
    }

    /// A file spelled in a dialect this writer does not emit still re-serialises
    /// to its own bytes, because what was read back is what is written out.
    #[test]
    fn a_file_is_re_emitted_in_its_own_dialect_and_not_in_this_writers() {
        let file = LheFile::parse(PASS_THROUGH).expect("the sample parses");
        assert_eq!(serialise(&file), pass_through_records());
    }

    /// The other half of the claim, and the one a verbatim copy would fake: the
    /// source text is handed back only for the lines whose values are untouched.
    /// Editing one leg's mass must move that line — and only that line — into
    /// this writer's own layout.
    #[test]
    fn an_edited_field_reformats_its_own_line_and_leaves_the_rest_spelled_as_read() {
        let mut file = LheFile::parse(PASS_THROUGH).expect("the sample parses");
        file.events[0].particles[2].mass = 175.0;
        let out = serialise(&file);

        let records = pass_through_records();
        let source: Vec<&str> = records.lines().collect();
        let got: Vec<&str> = out.lines().collect();
        assert_eq!(got.len(), source.len(), "{out}");
        for index in [0, 1, 2, 3, 4, 5, 6, 7, 9, 10] {
            assert_eq!(got[index], source[index], "line {index} was rewritten");
        }
        assert_eq!(
            got[8],
            "        6  1    1    2  501    0 +1.3402070006e+02 -6.8879185628e+01 \
             +9.9323258824e+01 2.5000000000e+02 1.7500000000e+02 0.0000e+00 -1.0000e+00"
        );
    }

    /// A source that no longer has one line per record cannot be matched up with
    /// it, and is dropped rather than guessed at: the whole block comes back in
    /// this writer's layout, `NUP` included.
    #[test]
    fn dropping_a_leg_discards_the_source_text_for_the_whole_block() {
        let mut file = LheFile::parse(PASS_THROUGH).expect("the sample parses");
        file.events[0].particles.pop();
        let out = serialise(&file);
        let info = out
            .lines()
            .nth(5)
            .expect("the <event> info line follows the <init> block and the <event> tag");
        assert_eq!(
            info,
            " 3      1 +1.3513480e+01 2.50000000e+02 7.54677100e-03 1.02464900e-01"
        );
    }

    /// The reuse rule is a value comparison, and a non-finite value compares
    /// equal to nothing — including the text it came from. The claim that a
    /// record carrying one still reaches the writer's loud spelling rather than
    /// being papered over by the source line is worth its own check, because it
    /// is the one case where "the text still decodes to this" is false of a
    /// record nothing edited.
    #[test]
    fn a_non_finite_field_is_written_out_rather_than_hidden_by_the_source_line() {
        let mut file = LheFile::parse(PASS_THROUGH).expect("the sample parses");
        file.events[0].particles[0].lifetime = f64::NAN;
        let out = serialise(&file);
        assert!(
            out.lines()
                .nth(6)
                .expect("the first particle line")
                .ends_with("NaN -1.0000e+00"),
            "{out}"
        );
    }

    /// The source text says how one file spelled a record, not what the record
    /// is, so a parsed block has to compare equal to the same block built from
    /// scratch — otherwise every consumer that checks a round trip by value
    /// would start failing on files it reads correctly.
    #[test]
    fn a_parsed_record_equals_the_same_record_built_from_scratch() {
        let file = LheFile::parse(PASS_THROUGH).expect("the sample parses");
        let mut rebuilt = file.clone();
        rebuilt.init.source = None;
        for event in rebuilt.events.iter_mut() {
            event.source = None;
        }
        assert!(file.init.source.is_some() && file.events[0].source.is_some());
        assert_eq!(file, rebuilt);
    }

    /// Authored element content is escaped by the XML writer rather than pasted.
    #[test]
    fn the_generator_element_escapes_what_it_is_given() {
        let element = generator_element("vibegraph", "0.1", "a & b < c");
        assert_eq!(
            element,
            "<generator name=\"vibegraph\" version=\"0.1\">a &amp; b &lt; c</generator>"
        );
    }
}
