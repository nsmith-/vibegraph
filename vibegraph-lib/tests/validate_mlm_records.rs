//! The `<mgrwt>` record of the banked MLM runs, rebuilt event by event from
//! the banked event file alone.
//!
//! # The oracle
//!
//! Every banked MLM run written with `use_syst = T` (`pp_to_llj_mlm`,
//! `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`, and `pp_to_llj_xqcut_only` for
//! the unmatched path that clusters for `xqcut`) carries MadEvent's `<mgrwt>`
//! on each of its 10 000 events: the `<rscale>` power and scale, the `<asrwt>`
//! emission scales, each beam's `<pdfrwt>` density chain and `<totfact>`. This
//! test rebuilds each record the way generation does — the production
//! clustering ([`ScaleChoice::cluster_history`]) on this crate's own channel
//! forests of the event's subprocess, `rewgt` for its flavours, and
//! [`LoRecord::fill`] — and compares every field at the digits `E15.8` prints.
//!
//! # What the event file does not say, and how it is chosen
//!
//! The file names the event's flavours and momenta, not the integration
//! channel MadEvent clustered it in, nor (on its face) which beam the matrix
//! element's first leg came from. The beam order is read off the record's own
//! `<pdfrwt>` order (MadEvent writes the matrix element's first leg first).
//! The channel is searched: the event is clustered in each of its
//! subprocess's configurations, and the configurations whose scales reproduce
//! the event's header (`SCALUP` and `AQCDUP` at their printed digits) and,
//! under matching, its `<scales pt_clust_N>` line are the candidates. An event
//! passes a field when every candidate's record agrees with MadEvent's there;
//! one with no candidate is reported, not compared. The momentum fractions are
//! read off the record's first chain entries: the written energies are
//! MadEvent's `xbk·ebeam` only to about 2e-7.
//!
//! # What is gated
//!
//! The `<rscale>` power on every row, and every field on
//! `pp_to_llj_xqcut_only`, the row that clusters without matching. On the
//! matched rows the rest is reported: the replay clusters this crate's own
//! forests at momenta rebuilt from eleven printed digits and a channel chosen
//! by the header, and the measured disagreements (a few units in the eighth
//! digit of an emission scale, ties between a chain's scales, and on
//! `pp_to_ttx_0j1j_mlm` a seeding scale on about 9% of the events) are not yet
//! told apart from that. The record rule itself is gated on MadEvent's own
//! clustering inputs by `validate_mlm_dumps`.
//!
//! # What this cannot see
//!
//! * The beam order: it is an input here, never a result.
//! * Anything the header already fixes about the clustering: a wrong
//!   clustering that still reproduces `SCALUP`, `AQCDUP` and `pt_clust` is
//!   taken as a candidate. The fields compared are the ones the header does
//!   not carry: the QCD power, the emission scales, the chains' flavours,
//!   fractions and scales, and `<totfact>`.
//! * The integration channel's QCD power against the clustered graph's:
//!   MadEvent builds its clustering table from the configurations that share
//!   the channel's power (`cluster.f:360`), so the two are one number on every
//!   event, and this crate's table is built the same way.
//! * Events whose matrix-element momenta are a symmetry permutation of
//!   identical final-state legs: MadEvent's first call clusters the
//!   unpermuted point (the permuted-`P1` deviation), and the file does not say
//!   which events those are. They are among the events with no candidate.

use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vibegraph::coupling::alphas::AlphaSSource;
use vibegraph::coupling::cluster::configs::{derive_channels, DerivedChannels};
use vibegraph::coupling::cluster::graph::ColorTable;
use vibegraph::coupling::cluster::rewgt::rewgt;
use vibegraph::coupling::scales::{ClusterInput, ScaleChoice, ScaleEvent};
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::lhef::build::scalup;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_OUTGOING};
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::runcard::RunCard;
use vibegraph::systematics::record::{LoRecord, RecordInputs};
use vibegraph::ufo::particles::ParticleId;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

mod common;

/// The rows written with `use_syst = T` whose `<mgrwt>` carries the MLM
/// terms or the `xqcut` clustering.
const ROWS: [&str; 4] = [
    "pp_to_llj_mlm",
    "pp_to_llj_xqcut_only",
    "pp_to_ttx_0j1j_mlm",
    "pp_to_ll_0j2j_mlm",
];

/// `<totfact>` against MadEvent's, relative. The momentum fractions are read
/// off the record at eight digits (half a unit is up to 5e-8), and the
/// densities amplify that by their slope in `x`: measured up to 1.9e-7 on the
/// unmatched row, where every other field agrees.
const TOTFACT_AGREEMENT: f64 = 1e-6;

/// `unwgt.f` writes `AQCDUP` as `g²/4/3.1415926`, `π` truncated.
#[allow(clippy::approx_constant)]
const TRUNCATED_PI: f64 = 3.1415926;

/// `SCALUP` and `AQCDUP` are printed with seven significant digits.
const HEADER_AGREEMENT: f64 = 1e-6;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn banked_text(row: &str) -> String {
    let path = root()
        .join("validation/madgraph/output")
        .join(row)
        .join("Events/run_01/unweighted_events.lhe.gz");
    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; fetch the banked runs with `pixi run fetch-refdata`",
            path.display()
        )
    });
    let mut text = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut text)
        .expect("a gzip LHE file");
    text
}

/// Whether two values printed at `E15.8` are equal or one unit of the last
/// digit apart.
fn within_one_unit(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    let scale = a.abs().max(b.abs());
    let unit = 10f64.powi(scale.log10().floor() as i32 - 7);
    (a - b).abs() <= unit * (1.0 + 1e-6)
}

fn rel(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        (a - b).abs() / a.abs().max(b.abs())
    }
}

/// `pt_clust_N="v"` of an event's `<scales>` line, as `(N, v)`.
fn pt_clust(event: &LheEvent) -> Option<Vec<(usize, String)>> {
    let line = event
        .trailer
        .iter()
        .find(|l| l.trim_start().starts_with("<scales"))?;
    let mut out = Vec::new();
    for chunk in line.split("pt_clust_").skip(1) {
        let (key, rest) = chunk.split_once("=\"")?;
        let (value, _) = rest.split_once('"')?;
        out.push((key.parse().ok()?, value.to_string()));
    }
    Some(out)
}

/// One row: its card as MadEvent ran it, the prescription this crate compiles
/// from it, and the channel forests of each subprocess its events name.
struct Row {
    card: RunCard,
    choice: ScaleChoice,
    model: std::sync::Arc<UFOModel>,
    evaluated: EvaluatedModel,
    colors: ColorTable,
    alpha_s: AlphaSSource,
    pdf: PdfMember,
    ebeam: [f64; 2],
}

impl Row {
    fn load(name: &str, banner: &str) -> Self {
        let card = RunCard::parse(banner).unwrap_or_else(|e| panic!("{name}: run card: {e}"));
        let choice = ScaleChoice::from_run_card(&card)
            .unwrap_or_else(|e| panic!("{name}: scale prescription: {e}"));
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let colors = ColorTable::new(
            model.particles.values().map(|p| (p.pdg_code, p.color)),
            card.maxjetflavor,
        );
        let params = ParamCard::from_file(&root().join(format!(
            "validation/madgraph/output/{name}/Cards/param_card.dat"
        )))
        .expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let alpha_s = AlphaSSource::from_run_card(
            &card,
            a_s,
            common::pdfset::set_alpha_s_info(&card).as_ref(),
        )
        .unwrap_or_else(|e| panic!("{name}: alpha_s: {e}"));
        let set = "NNPDF23_lo_as_0130_qed";
        assert_eq!(card.lhaid, 247000, "{name}: the PDF set");
        let pdf = PdfSet::load(&root().join("validation/pdf").join(set), set)
            .unwrap_or_else(|e| panic!("{name}: PDF set: {e}"))
            .member(0)
            .expect("PDF member 0");
        let ebeam = [card.ebeam1, card.ebeam2];
        Row {
            card,
            choice,
            model,
            evaluated,
            colors,
            alpha_s,
            pdf,
            ebeam,
        }
    }

    fn name_of(&self, pdg: i64) -> String {
        self.model
            .particles
            .values()
            .find(|p| p.pdg_code == pdg)
            .unwrap_or_else(|| panic!("no particle {pdg}"))
            .name
            .clone()
    }

    /// The channel forests of one concrete subprocess, its legs in the
    /// matrix element's order.
    fn channels(&self, legs: &[i64]) -> Option<DerivedChannels> {
        let names: Vec<String> = legs.iter().map(|&c| self.name_of(c)).collect();
        let line = format!(
            "generate {} {} > {}",
            names[0],
            names[1],
            names[2..].join(" ")
        );
        let parsed = parse_proc_card(&line, &ParsingOptions::default()).ok()?;
        let sets = generate_from_proc_card(&parsed, self.model.as_ref()).ok()?;
        let set = sets.iter().find(|s| {
            !s.diagrams.is_empty()
                && s.particles_in
                    .iter()
                    .chain(&s.particles_out)
                    .collect::<Vec<_>>()
                    == names.iter().collect::<Vec<_>>()
        })?;
        let externals: Vec<ParticleId> = names
            .iter()
            .map(|n| self.model.particle_id(n).expect("external in model"))
            .collect();
        derive_channels(
            &set.diagrams,
            &externals,
            2,
            self.model.as_ref(),
            &self.evaluated,
        )
        .ok()
    }
}

/// What one event's comparison found.
#[derive(Default)]
struct Outcome {
    /// Configurations whose scales reproduce the header.
    candidates: usize,
    /// Per field: whether every candidate's record agrees with MadEvent's.
    fields: Vec<(&'static str, bool)>,
    /// `<totfact>`'s worst relative deviation over the candidates.
    totfact: f64,
    /// Candidates disagreeing among themselves on any printed field.
    ambiguous: bool,
    /// The first candidate's record and MadEvent's, where a field differs.
    shown: Option<(Vec<String>, Vec<String>)>,
}

fn record_fields(ours: &LoRecord, theirs: &LoRecord) -> Vec<(&'static str, bool)> {
    let power = |r: &LoRecord| r.n_qcd as i64 - r.emission_scales.len() as i64;
    let chain = |beam: usize, field: usize| {
        let (a, b) = (&ours.beams[beam].entries, &theirs.beams[beam].entries);
        a.len() == b.len()
            && a.iter().zip(b).all(|(p, q)| match field {
                0 => p.0 == q.0,
                1 => within_one_unit(p.1, q.1),
                _ => within_one_unit(p.2, q.2),
            })
    };
    vec![
        ("<rscale> power", power(ours) == power(theirs)),
        ("<rscale> scale", within_one_unit(ours.mu_r, theirs.mu_r)),
        (
            "<asrwt> count and scales",
            ours.emission_scales.len() == theirs.emission_scales.len()
                && ours
                    .emission_scales
                    .iter()
                    .zip(&theirs.emission_scales)
                    .all(|(a, b)| within_one_unit(*a, *b)),
        ),
        (
            "<pdfrwt> beam labels in order",
            ours.beam_order == theirs.beam_order,
        ),
        ("<pdfrwt beam=1> count and codes", chain(0, 0)),
        ("<pdfrwt beam=1> x", chain(0, 1)),
        ("<pdfrwt beam=1> q", chain(0, 2)),
        ("<pdfrwt beam=2> count and codes", chain(1, 0)),
        ("<pdfrwt beam=2> x", chain(1, 1)),
        ("<pdfrwt beam=2> q", chain(1, 2)),
    ]
}

fn compare_event(
    row: &Row,
    sets: &HashMap<Vec<i64>, Option<DerivedChannels>>,
    event: &LheEvent,
) -> Option<Outcome> {
    let theirs = LoRecord::parse(&event.trailer).ok()??;
    let matched = row.choice.rewgt_settings().is_some();
    // The matrix element's first leg is the beam MadEvent writes first.
    let flipped = theirs.beam_order == [2, 1];
    let incoming: Vec<usize> = (0..event.particles.len())
        .filter(|&k| event.particles[k].status == STATUS_INCOMING)
        .collect();
    let outgoing: Vec<usize> = (0..event.particles.len())
        .filter(|&k| event.particles[k].status == STATUS_OUTGOING)
        .collect();
    let me_in = if flipped {
        [incoming[1], incoming[0]]
    } else {
        [incoming[0], incoming[1]]
    };
    let legs: Vec<i64> = me_in
        .iter()
        .chain(&outgoing)
        .map(|&k| i64::from(event.particles[k].pdg))
        .collect();
    let derived = sets.get(&legs)?.as_ref()?;

    // The partonic frame, the matrix element's first leg along +z.
    let x_lab = [0, 1].map(|b| event.particles[incoming[b]].momentum[0] / row.ebeam[b]);
    let beta = (x_lab[0] - x_lab[1]) / (x_lab[0] + x_lab[1]);
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    let to_cm = |p: [f64; 4]| {
        let e = gamma * (p[0] - beta * p[3]);
        let pz = gamma * (p[3] - beta * p[0]);
        if flipped {
            [e, p[1], -p[2], -pz]
        } else {
            [e, p[1], p[2], pz]
        }
    };
    let momenta: Vec<[f64; 4]> = me_in
        .iter()
        .chain(&outgoing)
        .map(|&k| to_cm(event.particles[k].momentum))
        .collect();
    let labels: [u8; 2] = if flipped { [2, 1] } else { [1, 2] };
    // The momentum fractions are MadEvent's `xbk`, as its record prints them:
    // the written energies are `xbk·ebeam` only to about 2e-7, opposite on
    // the two beams (a rapidity offset of the boost to the lab), which is more
    // than the eighth digit the record keeps.
    let x = labels.map(|l| theirs.beams[usize::from(l) - 1].entries[0].1);
    let printed_pt = if matched { pt_clust(event) } else { None };

    let mut out = Outcome::default();
    let mut seen: Option<Vec<String>> = None;
    for config in 1..=derived.set.configs.len() {
        let input = ClusterInput {
            set: &derived.set,
            colors: &row.colors,
            this_config: config,
            iproc: 1,
            tables: None,
        };
        let Ok(history) = row.choice.cluster_history(
            &ScaleEvent {
                incoming: [momenta[0], momenta[1]],
                outgoing: &momenta[2..],
            },
            &input,
        ) else {
            continue;
        };
        let scales = history.event_scales();
        if rel(scalup(&scales), event.scale) > HEADER_AGREEMENT
            || rel(
                row.alpha_s.eval(scales.mu_r) * TRUNCATED_PI / std::f64::consts::PI,
                event.alpha_qcd,
            ) > HEADER_AGREEMENT
        {
            continue;
        }
        if let Some(printed) = &printed_pt {
            let Some(record) = history.matched_record(&row.colors) else {
                continue;
            };
            // `pt_clust_N` names the event's line N; the outgoing legs keep
            // the file's order.
            let ok = printed.iter().all(|(n, v)| {
                outgoing
                    .iter()
                    .position(|&k| k + 1 == *n)
                    .is_some_and(|leg| format!("{:.5}", record.ptclus[leg + 2]) == *v)
            });
            if !ok {
                continue;
            }
        }
        let graph = scales.graph.expect("a clustered event");
        let n_qcd = u32::try_from(derived.set.configs[graph].nqcd).expect("a QCD power");
        let last = history.second.as_ref().unwrap_or(&history.first);
        let factor = match (row.choice.rewgt_settings(), history.rewgt_history()) {
            (Some(settings), Some(h)) => rewgt(
                &h,
                &row.colors,
                &settings,
                &legs,
                x,
                |q| row.alpha_s.eval(q),
                |pdg, x, q2| row.pdf.xfx_q2(pdg as i32, x, q2),
            )
            .ok(),
            _ => None,
        };
        if matched && factor.is_none() {
            continue;
        }
        let inputs = RecordInputs {
            n_qcd,
            mu_r: last.mu_r,
            incoming: [legs[0], legs[1]],
            x,
            mu_f: last.q2fact.map(f64::sqrt),
            labels,
            asref: if matched {
                row.alpha_s.eval(history.first.mu_r)
            } else {
                0.0
            },
            rewgt: factor.as_ref(),
        };
        let Ok(full) = LoRecord::fill(&inputs, |pdg, x, q2| row.pdf.try_xfx_q2(pdg as i32, x, q2))
        else {
            continue;
        };
        let ours = full.as_written();
        let lines = ours.lines();
        let printed: Vec<String> = lines[..lines.len() - 2].to_vec();
        if let Some(first) = &seen {
            out.ambiguous |= *first != printed;
        } else {
            seen = Some(printed);
        }
        out.candidates += 1;
        let fields = record_fields(&ours, &theirs);
        if out.shown.is_none()
            && (fields.iter().any(|(f, ok)| !ok && !f.ends_with(" x"))
                || rel(full.tot_fact, theirs.tot_fact) > 1e-5)
        {
            out.shown = Some((ours.lines(), theirs.lines()));
        }
        if out.fields.is_empty() {
            out.fields = fields;
        } else {
            for (into, (_, ok)) in out.fields.iter_mut().zip(fields) {
                into.1 &= ok;
            }
        }
        out.totfact = out.totfact.max(rel(full.tot_fact, theirs.tot_fact));
    }
    Some(out)
}

/// The `<MGRunCard>` text the banner records.
fn banner_card(text: &str) -> String {
    let start = text.find("<MGRunCard>").expect("an <MGRunCard>") + "<MGRunCard>".len();
    let end = text.find("</MGRunCard>").expect("</MGRunCard>");
    text[start..end].replace("<![CDATA[", "").replace("]]>", "")
}

#[test]
fn the_mgrwt_record_is_madevents_on_every_banked_mlm_event() {
    let mut failures = Vec::new();
    for name in ROWS {
        let text = banked_text(name);
        let file = LheFile::parse(&text).expect("the banked file parses");
        let row = Row::load(name, &banner_card(&text));
        assert!(row.card.get("use_syst").expect("known").as_bool());
        // Every subprocess the events name, in the matrix element's leg order.
        let mut keys: Vec<Vec<i64>> = Vec::new();
        for event in &file.events {
            let Ok(Some(record)) = LoRecord::parse(&event.trailer) else {
                continue;
            };
            let incoming: Vec<i64> = event
                .particles
                .iter()
                .filter(|p| p.status == STATUS_INCOMING)
                .map(|p| i64::from(p.pdg))
                .collect();
            let mut legs = if record.beam_order == [2, 1] {
                vec![incoming[1], incoming[0]]
            } else {
                incoming
            };
            legs.extend(
                event
                    .particles
                    .iter()
                    .filter(|p| p.status == STATUS_OUTGOING)
                    .map(|p| i64::from(p.pdg)),
            );
            if !keys.contains(&legs) {
                keys.push(legs);
            }
        }
        let sets: HashMap<Vec<i64>, Option<DerivedChannels>> = keys
            .par_iter()
            .map(|legs| (legs.clone(), row.channels(legs)))
            .collect();
        let outcomes: Vec<Option<Outcome>> = file
            .events
            .par_iter()
            .map(|event| compare_event(&row, &sets, event))
            .collect();

        let mut fields: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
        let (mut compared, mut no_candidate, mut ambiguous, mut no_set) = (0, 0, 0, 0);
        let mut worst_totfact = 0.0f64;
        let mut totfact_ok = 0usize;
        let mut shown: Vec<String> = Vec::new();
        for outcome in &outcomes {
            let Some(o) = outcome else {
                no_set += 1;
                continue;
            };
            if o.candidates == 0 {
                no_candidate += 1;
                continue;
            }
            compared += 1;
            ambiguous += usize::from(o.ambiguous);
            if let (Some((ours, theirs)), true) = (&o.shown, shown.len() < 6) {
                shown.push(format!(
                    "  differs:\n    ours   {}\n    theirs {}",
                    ours[1..6].join(" "),
                    theirs[1..6].join(" ")
                ));
            }
            for (field, ok) in &o.fields {
                let entry = fields.entry(field).or_insert((0, 0));
                entry.0 += usize::from(*ok);
                entry.1 += 1;
            }
            worst_totfact = worst_totfact.max(o.totfact);
            totfact_ok += usize::from(o.totfact <= TOTFACT_AGREEMENT);
        }
        println!(
            "\n{name}: {} events, {} subprocesses; {compared} compared, {no_candidate} with no \
             configuration reproducing the header, {no_set} without a record or a subprocess; \
             {ambiguous} with candidates disagreeing among themselves",
            file.events.len(),
            sets.len()
        );
        for line in &shown {
            println!("{line}");
        }
        // Gated: the QCD power on every row, and every field on the row that
        // clusters without matching. The matched rows' other fields are
        // informational (see the module documentation).
        let unmatched = row.choice.rewgt_settings().is_none();
        for (field, (ok, n)) in &fields {
            let gated = unmatched || *field == "<rscale> power";
            println!(
                "  {field:<36} {ok:>6}/{n}{}",
                if gated { "" } else { "  (info)" }
            );
            if gated && ok != n {
                failures.push(format!("{name}: {field} {ok}/{n}"));
            }
        }
        println!(
            "  {:<36} {totfact_ok:>6}/{compared}  worst {worst_totfact:.2e}{}",
            "<totfact>",
            if unmatched { "" } else { "  (info)" }
        );
        if unmatched && totfact_ok != compared {
            failures.push(format!("{name}: <totfact> {totfact_ok}/{compared}"));
        }
        if compared * 10 < file.events.len() * 9 {
            failures.push(format!(
                "{name}: only {compared} of {} events reached a comparison",
                file.events.len()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
