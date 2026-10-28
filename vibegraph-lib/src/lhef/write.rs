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
use crate::artifact::PdfInput;
use crate::cache::lhapdf::Tier;
use crate::generate::GenerateSettings;
use crate::runcard::{card_text, RunCard};
use crate::ufo::parameters::ParamNature;
use crate::ufo::EvaluatedModel;

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

/// The `<MGRunCard>` element MadGraph puts in an event file's header
/// (`banner.py:69-86`): the run card, one `value = name` line per parameter,
/// as MadGraph records it. The card is the integration's, joined with the
/// generation's `settings` and its `seed` as `iseed` (`RunCard::banner_with`),
/// so the record is the run as it was drawn.
///
/// The body is element text, with `<`, `>` and `&` escaped, and not the CDATA
/// section MadGraph wraps it in: Pythia 8.312's reader drops a CDATA
/// section's content, so
/// `Info::header("MGRunCard")` of MadGraph's own file is empty and
/// `JetMatching:setMad = on` finds none of its parameters there. Plain text is
/// the spelling it reads.
///
/// A shower reads it back: Pythia's `JetMatching:setMad = on` takes `ickkw`,
/// `xqcut`, `maxjetflavor` and `alpsfact` from it, parsing each line as a
/// number to the left of the first `=` and a name up to a `!`, and skipping any
/// line holding a `#`. Every parameter the card resolves is written once,
/// under the name this crate knows it by; a real is written in its shortest
/// round-tripping form with a decimal point (`20.0`), a flag as `True` or
/// `False`, and a list or string as its text; a parameter whose value is
/// empty (an unset list) is left out.
pub fn mg_run_card(card: &RunCard, settings: &GenerateSettings, seed: u64) -> String {
    let mut body = String::from("\n");
    for (name, value) in &card.banner_with(settings, seed) {
        let text = card_text(value);
        if text.trim().is_empty() {
            continue;
        }
        body.push_str(&format!("  {text} = {name}\n"));
    }
    let mut writer = Writer::new(Vec::new());
    for event in [
        XmlEvent::Start(BytesStart::new("MGRunCard")),
        XmlEvent::Text(BytesText::from_escaped("\n")),
        // `<`, `>` and `&` only: a reader that does not unescape (Pythia's
        // does not) then sees every other character as the card spells it.
        XmlEvent::Text(BytesText::from_escaped(quick_xml::escape::partial_escape(
            &body,
        ))),
        XmlEvent::Text(BytesText::from_escaped("\n")),
        XmlEvent::End(BytesEnd::new("MGRunCard")),
    ] {
        writer
            .write_event(event)
            .expect("writing to a Vec cannot fail");
    }
    String::from_utf8(writer.into_inner()).expect("a run card is text")
}

/// PDG codes MadGraph's card writer counts as Standard Model particles and
/// writes no `QNUMBERS` block for (`write_param_card.py`, `sm_pdg`).
const SM_PDG: [i64; 17] = [1, 2, 3, 4, 5, 6, 11, 12, 13, 14, 15, 16, 21, 22, 23, 24, 25];

/// A card value as MadGraph's writer spells it (`%e`), or with the fewest
/// further digits that read back as the value: a card moved to more digits
/// than MadGraph prints is written as it was bound.
fn slha_number(value: f64) -> String {
    (6..=16)
        .map(|digits| c_exponential(value, digits, false, 0))
        .find(|text| text.parse::<f64>() == Ok(value))
        .unwrap_or_else(|| c_exponential(value, 16, false, 0))
}

/// The `<slha>` element MadGraph puts in an event file's header: the parameter
/// card the run used, in the layout of MadGraph's `ParamCardWriter`
/// (`models/write_param_card.py`), so a shower reading SLHA from the event
/// file (Pythia's `SLHA:readFrom`) takes the run's masses and widths.
///
/// * The free external parameters, at the values `evaluated` binds them to,
///   ordered as `order_param` orders them: by block name with `DECAY` last,
///   then by code. A parameter the restriction fixed to zero is not written,
///   as MadGraph's restricted model no longer has it.
/// * After the `MASS` and after the `DECAY` entries, the masses and widths
///   that are not free parameters (computed, `ZERO`, or fixed to zero), one per
///   physical particle by ascending PDG code, then any particle whose mass or
///   width is another particle's parameter, each with `%e` and the particle's
///   name lower-cased, as MadGraph's model names it. A computed entry's
///   comment names the parameter, where MadGraph prints its expression.
/// * A `QNUMBERS` block for every physical particle outside MadGraph's SM list.
pub fn slha_block(evaluated: &EvaluatedModel) -> String {
    let model = evaluated.model();
    let params = &model.params;
    let value = |name: &str| evaluated.param_values.get(name).map_or(0.0, |v| v.re);
    let free_at = |name: &str| -> Option<&[i32]> {
        if params.zeros.contains(name) {
            return None;
        }
        params.externals.iter().find_map(|p| match &p.nature {
            ParamNature::External { lha_code, .. } if p.name == name => Some(lha_code.as_slice()),
            _ => None,
        })
    };

    let mut externals: Vec<(String, &[i32], &str)> = params
        .externals
        .iter()
        .filter(|p| !params.zeros.contains(&p.name))
        .filter_map(|p| match &p.nature {
            ParamNature::External {
                lha_block,
                lha_code,
                ..
            } => Some((
                lha_block.to_uppercase(),
                lha_code.as_slice(),
                p.name.as_str(),
            )),
            ParamNature::Internal { .. } => None,
        })
        .collect();
    externals.sort_by(|a, b| (a.0 == "DECAY", &a.0, a.1).cmp(&(b.0 == "DECAY", &b.0, b.1)));

    let particles: Vec<_> = {
        let mut seen = Vec::new();
        model
            .particles
            .values()
            .filter(|p| p.pdg_code > 0 && !p.is_unphysical())
            .filter(|p| {
                let new = !seen.contains(&p.pdg_code);
                seen.push(p.pdg_code);
                new
            })
            .collect()
    };
    // MadGraph's `define_not_dep_param`: a particle whose mass (width) is a free
    // parameter at another particle's code, or one an earlier particle already
    // uses, is a duplicate; one whose parameter is not free is dependent.
    let derived = |width: bool| {
        let mut dependent = Vec::new();
        let mut duplicate = Vec::new();
        let mut used: Vec<&str> = Vec::new();
        for p in &particles {
            let param = if width {
                p.width_param.as_str()
            } else {
                p.mass_param.as_str()
            };
            match free_at(param) {
                Some(code) if code.first() != Some(&(p.pdg_code as i32)) => {
                    duplicate.push((*p, param));
                    continue;
                }
                _ if used.contains(&param) => {
                    duplicate.push((*p, param));
                    continue;
                }
                Some(_) => used.push(param),
                None => {
                    if param != "ZERO" {
                        used.push(param);
                    }
                    dependent.push((*p, param));
                }
            }
        }
        dependent.sort_by_key(|(p, _)| p.pdg_code);
        dependent.extend(duplicate);
        dependent
    };
    let dependent_lines = |block: &str| -> String {
        let (width, prefix) = match block {
            "MASS" => (false, " "),
            "DECAY" => (true, "DECAY "),
            _ => return String::new(),
        };
        let mut text = String::new();
        for (p, param) in derived(width) {
            let what = if param == "ZERO" || params.zeros.contains(param) {
                "0.0"
            } else {
                param
            };
            text.push_str(&format!(
                "{prefix} {} {} # {} : {what} \n",
                p.pdg_code,
                c_exponential(value(param), 6, false, 0),
                p.name.to_lowercase()
            ));
        }
        if text.is_empty() {
            return text;
        }
        format!(
            "## Dependent parameters: computed from the ones above, or fixed by the\n\
             ## model's restriction. Written for programs that read masses and widths\n\
             ## from this card, such as a shower.\n{text}"
        )
    };
    let block_header = |name: &str| {
        let mut text = format!(
            "\n###################################\n## INFORMATION FOR {name}\n###################################\n"
        );
        if name != "DECAY" {
            text.push_str(&format!("Block {} \n", name.to_lowercase()));
        }
        text
    };

    let mut body = String::from(
        "######################################################################\n\
         ## PARAM_CARD OF THE RUN: THE PARAMETERS VIBEGRAPH EVALUATED AT     ##\n\
         ######################################################################\n",
    );
    let mut todo = vec!["MASS", "DECAY"];
    let mut current: Option<&str> = None;
    for (block, code, name) in &externals {
        if current != Some(block.as_str()) {
            if let Some(previous) = current {
                body.push_str(&dependent_lines(previous));
            }
            current = Some(block.as_str());
            todo.retain(|b| b != block);
            body.push_str(&block_header(block));
        }
        let lhacode = code
            .iter()
            .map(|k| format!("{k:>3}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut info = (*name).to_string();
        if block == "SMINPUTS" && *code == [3] {
            info.push_str(" (Note: this Parameter is not used if you use a PDF set)");
        }
        let number = slha_number(value(name));
        if block == "DECAY" {
            body.push_str(&format!("DECAY {lhacode} {number} # {info} \n"));
        } else {
            body.push_str(&format!("  {lhacode} {number} # {info} \n"));
        }
    }
    if let Some(previous) = current {
        body.push_str(&dependent_lines(previous));
    }
    for block in todo {
        body.push_str(&block_header(block));
        body.push_str(&dependent_lines(block));
    }

    let mut qnumbers = String::new();
    for p in particles.iter().filter(|p| !SM_PDG.contains(&p.pdg_code)) {
        let charge = (3.0 * p.charge * 1e6).round() / 1e6;
        qnumbers.push_str(&format!(
            "Block QNUMBERS {}  # {} \n        1 {}  # 3 times electric charge\n        \
             2 {}  # number of spin states (2S+1)\n        \
             3 {}  # colour rep (1: singlet, 3: triplet, 8: octet)\n        \
             4 {}  # Particle/Antiparticle distinction (0=own anti)\n",
            p.pdg_code,
            p.name,
            charge + 0.0,
            p.spin,
            p.color,
            if p.is_self_conjugate { 0 } else { 1 },
        ));
    }
    if !qnumbers.is_empty() {
        body.push_str(
            "#===========================================================\n\
             # QUANTUM NUMBERS OF NEW STATE(S) (NON SM PDG CODE)\n\
             #===========================================================\n\n",
        );
        body.push_str(&qnumbers);
    }

    format!(
        "<slha>\n{}\n</slha>",
        quick_xml::escape::partial_escape(body.trim_end())
    )
}

/// What a generation read besides the artifact's own contents, as the event
/// file's `<vibegraphInputs>` header block records it.
#[derive(Clone, Debug, PartialEq)]
pub struct InputsRead {
    /// The central PDF member the weights were computed with and the lookup tier
    /// its set was found in; `None` when no parton density is read.
    pub pdf: Option<(PdfInput, Tier)>,
    /// SHA-256 of the integrate artifact file, as read.
    pub grid_sha256: String,
}

impl InputsRead {
    /// The `<vibegraphInputs>` block: the central PDF member with each file's
    /// SHA-256 and the tier (named, never a path: the header travels between
    /// machines), then the artifact's digest.
    pub fn header_block(&self) -> String {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        let mut write = |event: XmlEvent<'_>| {
            writer
                .write_event(event)
                .expect("writing to a Vec cannot fail");
        };
        write(XmlEvent::Start(BytesStart::new("vibegraphInputs")));
        if let Some((
            PdfInput::Lhapdf {
                lhaid,
                set,
                member,
                files,
            },
            tier,
        )) = &self.pdf
        {
            let mut pdf = BytesStart::new("pdf");
            pdf.push_attribute(("use", "central"));
            pdf.push_attribute(("set", set.as_str()));
            pdf.push_attribute(("lhaid", lhaid.to_string().as_str()));
            pdf.push_attribute(("member", member.to_string().as_str()));
            pdf.push_attribute(("tier", tier.name()));
            write(XmlEvent::Start(pdf));
            for file in files {
                let mut entry = BytesStart::new("file");
                entry.push_attribute(("name", file.name.as_str()));
                entry.push_attribute(("sha256", file.sha256.as_str()));
                write(XmlEvent::Empty(entry));
            }
            write(XmlEvent::End(BytesEnd::new("pdf")));
        }
        let mut grid = BytesStart::new("grid");
        grid.push_attribute(("sha256", self.grid_sha256.as_str()));
        write(XmlEvent::Empty(grid));
        write(XmlEvent::End(BytesEnd::new("vibegraphInputs")));
        String::from_utf8(writer.into_inner()).expect("the block is text")
    }
}

/// The `<initrwgt>` block declaring a file's reweighting weights, one line per
/// entry, in the layout MadGraph's reweight module writes: one `<weightgroup>`
/// holding a `<weight id='…'>` per hypothesis, its description as the text.
///
/// The ids go into attributes unescaped, so they must be attribute-safe; the
/// descriptions are escaped.
pub(crate) fn initrwgt_block(weights: &[(String, String)]) -> Vec<String> {
    let mut lines = vec![
        "<initrwgt>".to_string(),
        "<weightgroup name='mg_reweighting' weight_name_strategy='includeIdInWeightName'>"
            .to_string(),
    ];
    for (id, info) in weights {
        lines.push(format!(
            "<weight id='{id}'> {} </weight>",
            quick_xml::escape::escape(info.as_str())
        ));
    }
    lines.push("</weightgroup>".to_string());
    lines.push("</initrwgt>".to_string());
    lines
}

/// One event's `<rwgt>` block: the event's weight under each hypothesis, in
/// `XWGTUP`'s own units, as `<wgt id='…'>` lines. Empty when there are none.
pub(crate) fn rwgt_block(ids: &[String], weights: &[f64]) -> Vec<String> {
    assert_eq!(ids.len(), weights.len(), "one weight per declared id");
    if ids.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::with_capacity(ids.len() + 2);
    lines.push("<rwgt>".to_string());
    for (id, &w) in ids.iter().zip(weights) {
        lines.push(format!(
            "<wgt id='{id}'> {} </wgt>",
            c_exponential(w, 7, true, 0)
        ));
    }
    lines.push("</rwgt>".to_string());
    lines
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
    pub fn begin(out: W, init: &LheInit, header: Option<&str>) -> io::Result<Self> {
        Self::begin_with_blocks(out, init, header, &[])
    }

    /// [`begin`](Self::begin), with `blocks` — complete XML elements such as
    /// [`mg_run_card`]'s — written inside `<header>` after the comment, each on
    /// its own line.
    pub(crate) fn begin_with_blocks(
        mut out: W,
        init: &LheInit,
        header: Option<&str>,
        blocks: &[String],
    ) -> io::Result<Self> {
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
            for block in blocks {
                events.push(XmlEvent::Text(BytesText::from_escaped(block.as_str())));
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
    pub(crate) fn events_written(&self) -> u64 {
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
    use crate::artifact::FileDigest;
    use crate::runcard::ParamValue;

    /// The `<slha>` block of the default SM, entry for entry, is the card
    /// MadGraph wrote into its event files for the same restriction
    /// (`dy13_param_card.dat` is the `<slha>` content of a banked MadGraph
    /// file): every data line, with its spacing and its comment up to a
    /// dependent entry's expression, in the same order. The one value that
    /// differs is the Higgs width, which MadGraph rounds to `%e` and the
    /// restriction gives to two digits more.
    #[test]
    fn the_slha_block_has_madgraphs_layout() {
        let evaluated = EvaluatedModel::from_model(crate::ufo::sm::sm_model(
            crate::ufo::sm::SMRestrict::Default,
        ));
        let ours = slha_block(&evaluated);
        assert!(ours.starts_with("<slha>\n######"), "{ours}");
        assert!(ours.ends_with("# a : 0.0\n</slha>"), "{ours}");
        let theirs = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/dy13_param_card.dat"
        ))
        .unwrap();
        // A data line up to its comment's `:`, the dependent entries' expression
        // being MadGraph's own.
        let entries = |text: &str| -> Vec<String> {
            text.lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with('<'))
                .map(|l| l.split(" : ").next().unwrap().trim_end().to_string())
                .collect()
        };
        let mut expected = entries(&theirs);
        let wh = expected
            .iter()
            .position(|l| l == "DECAY  25 6.382339e-03 # WH")
            .expect("MadGraph's card writes WH");
        expected[wh] = "DECAY  25 6.38233934e-03 # WH".to_string();
        assert_eq!(entries(&ours), expected);
        // The section headings and block lines too.
        let headings = |text: &str| -> Vec<String> {
            text.lines()
                .filter(|l| l.starts_with("## INFORMATION") || l.starts_with("Block"))
                .map(str::to_string)
                .collect()
        };
        assert_eq!(headings(&ours), headings(&theirs));
    }

    /// A moved value keeps every digit it was bound with; the computed W mass
    /// follows MZ; a model with new states declares their quantum numbers as
    /// MadGraph's writer does, and the SM declares none.
    #[test]
    fn the_slha_block_writes_moved_values_and_new_states() {
        use crate::ufo::slha::ParamCard;
        use crate::ufo::sm::{sm_model, SMRestrict};
        let card: ParamCard = "Block MASS\n 23 91.18765432\n".parse().unwrap();
        let evaluated = EvaluatedModel::from_model_card(sm_model(SMRestrict::Default), &card);
        let block = slha_block(&evaluated);
        assert!(block.contains("\n   23 9.118765432e+01 # MZ \n"), "{block}");
        let mw = c_exponential(evaluated.param_values["MW"].re, 6, false, 0);
        assert!(
            block.contains(&format!("\n  24 {mw} # w+ : MW \n")),
            "{block}"
        );
        assert!(!block.contains("QNUMBERS"));

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo");
        let toy = crate::ufo::UFOModel::load(
            &root.join("vibegraph_toy_UFO"),
            Some(&root.join("vibegraph_toy_UFO/restrict_all.dat")),
        )
        .unwrap();
        let block = slha_block(&EvaluatedModel::from_model(toy));
        assert!(
            block.contains(
                "Block QNUMBERS 9000004  # qt \n        1 2  # 3 times electric charge\n        \
                 2 2  # number of spin states (2S+1)\n        \
                 3 3  # colour rep (1: singlet, 3: triplet, 8: octet)\n        \
                 4 1  # Particle/Antiparticle distinction (0=own anti)\n"
            ),
            "{block}"
        );
    }

    /// The block names the central member, the tier by name, each file's
    /// digest in load order and the artifact's digest; a run that reads no
    /// density lists the artifact alone. Attribute values are escaped.
    #[test]
    fn the_inputs_block_lists_the_central_member_and_the_grid() {
        let read = InputsRead {
            pdf: Some((
                PdfInput::Lhapdf {
                    lhaid: 900002,
                    set: "Set&Co".to_string(),
                    member: 2,
                    files: vec![
                        FileDigest {
                            name: "Set&Co.info".to_string(),
                            sha256: "aa".to_string(),
                        },
                        FileDigest {
                            name: "Set&Co_0002.dat".to_string(),
                            sha256: "bb".to_string(),
                        },
                    ],
                },
                Tier::Cvmfs,
            )),
            grid_sha256: "cc".to_string(),
        };
        assert_eq!(
            read.header_block(),
            "<vibegraphInputs>\n  \
             <pdf use=\"central\" set=\"Set&amp;Co\" lhaid=\"900002\" member=\"2\" tier=\"cvmfs\">\n    \
             <file name=\"Set&amp;Co.info\" sha256=\"aa\"/>\n    \
             <file name=\"Set&amp;Co_0002.dat\" sha256=\"bb\"/>\n  \
             </pdf>\n  \
             <grid sha256=\"cc\"/>\n\
             </vibegraphInputs>"
        );
        let none = InputsRead {
            pdf: None,
            grid_sha256: "cc".to_string(),
        };
        assert_eq!(
            none.header_block(),
            "<vibegraphInputs>\n  <grid sha256=\"cc\"/>\n</vibegraphInputs>"
        );
    }

    #[test]
    fn reweight_markup_has_madgraphs_layout() {
        let ids = vec!["a".to_string(), "b_2".to_string()];
        assert_eq!(
            rwgt_block(&ids, &[1.5, -2.0e-12]),
            [
                "<rwgt>",
                "<wgt id='a'> +1.5000000e+00 </wgt>",
                "<wgt id='b_2'> -2.0000000e-12 </wgt>",
                "</rwgt>"
            ]
        );
        assert!(rwgt_block(&[], &[]).is_empty());
        let block = initrwgt_block(&[("a".to_string(), "set ymt 1 & <more>".to_string())]);
        assert_eq!(
            block[2],
            "<weight id='a'> set ymt 1 &amp; &lt;more&gt; </weight>"
        );
    }

    /// Header blocks are markup a reader walks past, and the events still parse.
    #[test]
    fn header_blocks_leave_the_file_readable() {
        let block = initrwgt_block(&[("x".to_string(), "info".to_string())]);
        let mut out = Vec::new();
        {
            let mut writer =
                LheWriter::begin_with_blocks(&mut out, &init(), Some("provenance"), &block)
                    .unwrap();
            let mut e = event();
            e.trailer = rwgt_block(&["x".to_string()], &[0.25]);
            writer.write_event(&e).unwrap();
            writer.finish().unwrap();
        }
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("<initrwgt>\n<weightgroup"), "{text}");
        let file = LheFile::parse(&text).unwrap();
        assert_eq!(file.events.len(), 1);
        assert_eq!(
            file.events[0].trailer,
            ["<rwgt>", "<wgt id='x'> +2.5000000e-01 </wgt>", "</rwgt>"]
        );
    }
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

    /// `<MGRunCard>` as Pythia's `MadgraphPar` reads it: the four fields
    /// `JetMatching:setMad` takes are there as numbers, every parameter appears
    /// once, and no written line holds a `#` (which that reader skips whole).
    /// The card is MadGraph's record of it, so the jet thresholds `setcuts.f`
    /// rewrites under `xqcut` keep the card's values.
    #[test]
    fn mg_run_card_carries_what_set_mad_reads() {
        let parsed = RunCard::parse(
            "1 = ickkw\n30 = xqcut\n45 = ptj\n0.4 = drjj\nTrue = use_syst\n2 = alpsfact\n",
        )
        .expect("card");
        assert_eq!(
            parsed.float("ptj"),
            30.0,
            "the resolved card is setcuts.f's"
        );
        let (card, settings) = parsed.clone().split();
        let block = mg_run_card(&card, &settings, 20261021);
        assert!(block.starts_with("<MGRunCard>\n\n  "));
        assert!(block.ends_with("\n\n</MGRunCard>"));
        assert!(!block.contains("CDATA"));
        let mut fields = std::collections::BTreeMap::new();
        for line in block.lines().filter(|l| l.contains('=')) {
            assert!(!line.contains('#'), "{line}");
            let (value, name) = line.split_once('=').expect("=");
            let previous = fields.insert(name.trim().to_string(), value.trim().to_string());
            assert!(previous.is_none(), "{} written twice", name.trim());
        }
        let number = |name: &str| -> f64 { fields[name].parse().expect(name) };
        assert_eq!(number("ickkw"), 1.0);
        assert_eq!(number("xqcut"), 30.0);
        assert_eq!(number("maxjetflavor"), 4.0);
        // banner.py's own edits: alpsfact under matching with use_syst, the
        // separations under xqcut; setcuts.f's ptj/mmjj rewrite is not one.
        assert_eq!(number("alpsfact"), 1.0);
        assert_eq!(number("drjj"), 0.0);
        assert_eq!(number("ptj"), 45.0);
        assert_eq!(number("mmjj"), 0.0);
        assert_eq!(fields["use_syst"], "True");
        assert_eq!(fields["nevents"], "10000");
        assert_eq!(fields["event_norm"], "average");
        assert_eq!(
            fields["iseed"], "20261021",
            "iseed is the generation's seed"
        );
        assert_eq!(fields.len(), parsed.iter().filter(|(_, v)| !matches!(v, ParamValue::Opaque(s) | ParamValue::Str(s) if s.trim().is_empty())).count());
    }

    /// The record joins the integration card with the generation's settings as
    /// used: a setting changed after the split is what the record says, every
    /// integration field keeps the card's value, and a seed beyond MadGraph's
    /// 32-bit `iseed` leaves the card's own.
    #[test]
    fn mg_run_card_records_the_generation_as_drawn() {
        let parsed = RunCard::parse("7 = iseed\n500 = nevents\n").expect("card");
        let (card, mut settings) = parsed.split();
        settings.nevents = 42;
        settings.event_norm = crate::runcard::EventNorm::Sum;
        settings.systematics.enabled = false;
        let field = |block: &str, name: &str| -> String {
            block
                .lines()
                .filter_map(|l| l.split_once('='))
                .find(|(_, n)| n.trim() == name)
                .map(|(v, _)| v.trim().to_string())
                .unwrap_or_else(|| panic!("{name} is not recorded"))
        };
        let block = mg_run_card(&card, &settings, 9);
        assert_eq!(field(&block, "nevents"), "42");
        assert_eq!(field(&block, "event_norm"), "sum");
        assert_eq!(field(&block, "use_syst"), "False");
        assert_eq!(field(&block, "iseed"), "9");
        assert_eq!(field(&block, "ebeam1"), "6500.0");
        assert_eq!(field(&block, "time_of_flight"), "-1.0");
        let wide = mg_run_card(&card, &settings, u64::from(u32::MAX));
        assert_eq!(field(&wide, "iseed"), "7");
    }

    /// A header with no blocks is the header [`LheWriter::begin`] always
    /// wrote, byte for byte; blocks land inside `<header>`, after the comment.
    #[test]
    fn header_blocks_land_inside_the_header() {
        let plain = rendered(|out| {
            LheWriter::begin(&mut *out, &init(), Some("provenance"))?.finish()?;
            Ok(())
        });
        let empty = rendered(|out| {
            LheWriter::begin_with_blocks(&mut *out, &init(), Some("provenance"), &[])?.finish()?;
            Ok(())
        });
        assert_eq!(plain, empty);
        let with = rendered(|out| {
            LheWriter::begin_with_blocks(
                &mut *out,
                &init(),
                Some("provenance"),
                &["<MGRunCard>\n<![CDATA[\n  1 = ickkw\n]]>\n</MGRunCard>".to_string()],
            )?
            .finish()?;
            Ok(())
        });
        assert!(
            with.contains("-->\n<MGRunCard>\n<![CDATA[\n  1 = ickkw\n]]>\n</MGRunCard>\n</header>")
        );
        assert!(LheFile::parse(&with).is_ok());
    }
}
