//! The matched (`ickkw = 1`) and pure-cut (`xqcut > 0`) scale path against
//! MadEvent's own, event by event.
//!
//! # The oracle
//!
//! An instrumented replay of each banked MLM run writes, per written event,
//! both `setclscales` calls of its point — the one `update_scale_coupling`
//! makes and, under matching, the one `rewgt` makes with `keepq2bck = .true.` —
//! with every intermediate `validate_kt_cluster` reads plus the matched
//! additions: the jet memo on entry, the overwrite of the central vertices
//! (`Q2OVR`), `q2bck`, the configuration colour and mothers are taken from
//! (`CFG`) and the header fields as written (`OUT`).
//! `validation/madgraph/mlm_dump_manifest.json` pins the files and
//! `gen_kt_cluster_dumps.py`'s module docstring is the record grammar.
//!
//! # Two readings of each event
//!
//! * **Engine replay.** Each call is replayed through `setclscales` with the
//!   state MadEvent entered it with (its momenta, its jet memo, its scales), and
//!   compared record by record. This isolates the port of the routine from
//!   everything around it.
//! * **Production path.** The event goes through
//!   [`ScaleChoice::cluster_history`] exactly as a sampled point does — one
//!   momentum set, the memo rule the prescription uses, the run card as this
//!   crate resolves it — and the scales it reports (both calls, `q2bck`, the
//!   clustered configuration, `SCALUP`, `AQCDUP`) are compared with the record.
//!
//! Beside them, the jet memo: MadEvent keeps one count per job and channel,
//! filled by the channel's first point. [`the_jet_memo_is_the_channel_s_restricted_jet_count`]
//! checks that the count a channel-restricted clustering yields is the same on
//! every dumped event, for every channel of every row, and equals the value the
//! census read off every job.
//!
//! The forests come from each process directory's own `configs.inc` (the dump's
//! tables carry no directory name), with masses and widths from the dump.
//!
//! # What this cannot see
//!
//! * **Rejected points.** Only written events are dumped, so the `xqcut`
//!   decision is checked in one direction: no written event is rejected.
//!
//! # `rewgt`, factor by factor
//!
//! On the matched rows each event's `rewgt` is recomputed from the production
//! path's clustering history for the flavour combination MadEvent drew
//! (`RWLEG`'s `idup`, `IPSEL`), at the momentum fractions `RWBEG` records, and
//! compared in order with `RWVX` (vertex class, codes, `ipart`, `kt²`,
//! `αs(alpsfact·kt)`, `asref`, the ratio), `RWPDF` per beam (vertex, flavour,
//! action, `x` after `z`, `q²_now`, `q²_prev`, both densities, the ratio),
//! `RWKILL` and `RWEND`. On `pp_to_llj_xqcut_only` the factor must be `1`.
//! `αs` and the densities come from this crate's reading of the PDF set's grid,
//! MadEvent's from LHAPDF; they agree to a few ulp, so every field is compared
//! at [`AGREEMENT`]'s scale.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use vibegraph::coupling::alphas::AlphaSSource;
use vibegraph::coupling::cluster::graph::{ChannelSet, ColorTable, ConfigForest, ForestLine};
use vibegraph::coupling::cluster::kt::Channel;
use vibegraph::coupling::cluster::rewgt::{rewgt, PdfOutcome, Rewgt, RewgtKill, VertexClass};
use vibegraph::coupling::cluster::setclscales::{
    setclscales, ClusterScales, JetMemo, MemoStep, ScaleSettings,
};
use vibegraph::coupling::scales::{ClusterInput, ScaleChoice, ScaleEvent};
use vibegraph::lhef::build::scalup;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;

mod common;

/// Relative agreement for every scale: both sides evaluate the same expressions
/// on the same inputs (see `validate_kt_cluster`).
const AGREEMENT: f64 = 1e-12;

/// Relative agreement for `AQCDUP`: this side's `αs` comes from the PDF set's
/// grid as this crate interpolates it, MadEvent's from LHAPDF's own, so the two
/// agree to the interpolation's precision, not to the last ulp.
const AQCDUP_AGREEMENT: f64 = 1e-6;

/// Relative agreement for `rewgt`'s densities, `αs` values and ratios. Both
/// sides read the same grid through the same log-bicubic interpolation (and the
/// same `αs` tabulation), and agree to a few ulp (worst 9e-16 measured on the
/// llj rows), so the bound is the scales' own.
const PDF_AGREEMENT: f64 = 1e-12;

/// The least σ shift dropping `rewgt`'s `αs` ratios must cause on a matched
/// row: ten times the ~0.1% the seeded σ references resolve.
const NEGATIVE_CONTROL_MIN_SHIFT: f64 = 0.01;

const ROWS: &[&str] = &[
    "pp_to_llj_mlm",
    "pp_to_llj_xqcut_only",
    "pp_to_llj_mlm_alps2",
    "pp_to_ttx_0j1j_mlm",
    "pp_to_ll_0j2j_mlm",
];

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

// ── dump records ─────────────────────────────────────────────────────────────

/// One tagged record, as the extraction driver's JSON array.
#[derive(Clone, Copy)]
pub struct Rec<'a>(&'a [Value]);

impl<'a> Rec<'a> {
    fn tag(&self) -> &'a str {
        self.0[0].as_str().expect("a record opens with its tag")
    }
    fn i(&self, k: usize) -> i64 {
        self.0[k].as_i64().unwrap_or_else(|| self.f(k) as i64)
    }
    fn u(&self, k: usize) -> usize {
        self.i(k).max(0) as usize
    }
    fn f(&self, k: usize) -> f64 {
        self.0[k].as_f64().expect("a numeric field")
    }
    fn b(&self, k: usize) -> bool {
        self.0[k].as_bool().expect("a logical field")
    }
    fn s(&self, k: usize) -> &'a str {
        self.0[k].as_str().expect("a string field")
    }
}

pub struct Event {
    index: usize,
    /// `<subprocess dir>/<channel dir>` of the job that wrote the event.
    directory: String,
    records: Vec<Vec<Value>>,
}

impl Event {
    fn recs(&self) -> impl Iterator<Item = Rec<'_>> {
        self.records.iter().map(|r| Rec(r.as_slice()))
    }

    fn subprocess_dir(&self) -> &str {
        self.directory.split('/').next().expect("a directory")
    }

    /// The records of each `setclscales` call, in order: the first opens at the
    /// `SCL` record with `keepq2bck = F`, the second (under matching) at the one
    /// with `keepq2bck = T`.
    fn calls(&self) -> Vec<CallRecords<'_>> {
        let starts: Vec<usize> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0].as_str() == Some("SCL"))
            .map(|(k, _)| k)
            .collect();
        starts
            .iter()
            .enumerate()
            .map(|(n, &start)| {
                let end = starts.get(n + 1).copied().unwrap_or(self.records.len());
                CallRecords {
                    records: self.records[start..end]
                        .iter()
                        .map(|r| Rec(r.as_slice()))
                        .collect(),
                }
            })
            .collect()
    }

    fn only(&self, tag: &str) -> Option<Rec<'_>> {
        self.recs().find(|r| r.tag() == tag)
    }
}

/// One `setclscales` call's records, and what the comparison reads of them.
pub struct CallRecords<'a> {
    records: Vec<Rec<'a>>,
}

impl<'a> CallRecords<'a> {
    fn all(&self, tag: &'a str) -> impl Iterator<Item = Rec<'a>> + '_ {
        self.records.iter().copied().filter(move |r| r.tag() == tag)
    }
    fn first(&self, tag: &str) -> Option<Rec<'a>> {
        self.records.iter().copied().find(|r| r.tag() == tag)
    }
    fn last(&self, tag: &str) -> Option<Rec<'a>> {
        self.records.iter().copied().rev().find(|r| r.tag() == tag)
    }
    fn scl(&self) -> Rec<'a> {
        self.records[0]
    }
    fn iconfig(&self) -> usize {
        self.scl().u(1)
    }
    fn iproc(&self) -> usize {
        self.scl().u(2)
    }
    fn keepq2bck(&self) -> bool {
        self.scl().b(5)
    }
    /// `(scale, [q2fact(1), q2fact(2)])` on entry.
    fn entry(&self) -> (f64, [f64; 2]) {
        let scl = self.scl();
        (scl.f(7), [scl.f(8), scl.f(9)])
    }
    /// `njetstore(iconfig)` on entry, `-1` while unset.
    fn memo_on_entry(&self) -> i64 {
        self.scl().i(10)
    }
    fn momenta(&self, n_external: usize) -> Vec<[f64; 4]> {
        let evt = self.first("EVT").expect("an EVT record");
        let attempt = evt.i(1);
        let mut p = vec![[0.0; 4]; n_external];
        for mom in self.all("MOM").filter(|m| m.i(1) == attempt) {
            p[mom.u(2) - 1] = [mom.f(3), mom.f(4), mom.f(5), mom.f(6)];
        }
        p
    }
    fn n_external(&self) -> usize {
        self.first("EVT").expect("an EVT record").u(6)
    }
    /// The subprocess's external flavours, off the single-leg line records.
    fn external(&self, n_external: usize) -> Vec<i64> {
        let mut external = vec![0; n_external];
        for line in self.all("LINE") {
            let mask = line.u(1);
            if mask.is_power_of_two() && mask.trailing_zeros() < n_external as u32 {
                external[mask.trailing_zeros() as usize] = line.i(2);
            }
        }
        external
    }
    fn memo_steps(&self) -> Vec<(&'a str, usize)> {
        self.all("MEMO").map(|m| (m.s(1), m.u(2))).collect()
    }
    fn reclustered(&self) -> bool {
        self.all("MEMO").any(|m| m.s(1) == "RESTRICTED_RECLUSTER")
    }
    /// `(μR, [q2fact(1), q2fact(2)])` on the way out.
    fn out(&self) -> (f64, [f64; 2]) {
        let out = self.last("SCLOUT").expect("an SCLOUT record");
        (out.f(1), [out.f(2), out.f(3)])
    }
}

/// A decompressor whose child is reaped when the stream it feeds is dropped.
struct Decompressed<I> {
    child: std::process::Child,
    events: I,
}

impl<I: Iterator> Iterator for Decompressed<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.events.next()
    }
}

impl<I> Drop for Decompressed<I> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Stream a run's dump: the header, then the events in the banked file's order.
pub fn read_dump(path: &Path) -> (Value, impl Iterator<Item = Event>) {
    let mut child = Command::new("gzip")
        .arg("-dc")
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("gzip -dc");
    let stdout = child.stdout.take().expect("piped stdout");
    let mut reader = BufReader::with_capacity(1 << 20, stdout);
    let mut head = String::new();
    reader.read_line(&mut head).expect("dump header");
    let header: Value = serde_json::from_str(&head).expect("dump header parses");
    let events = reader.lines().map(|line| {
        let line = line.expect("dump line");
        let value: Value = serde_json::from_str(&line).expect("dump event parses");
        Event {
            index: value["index"].as_u64().expect("event index") as usize,
            directory: value["directory"]
                .as_str()
                .expect("event directory")
                .to_string(),
            records: value["records"]
                .as_array()
                .expect("event records")
                .iter()
                .map(|r| r.as_array().expect("a record is an array").clone())
                .collect(),
        }
    });
    (header, Decompressed { child, events })
}

// ── one row: its card, its directories ───────────────────────────────────────

/// A process directory's channel forests, read from its own `configs.inc`.
pub struct Directory {
    n_external: usize,
    n_proc: usize,
    configs: Vec<ConfigForest>,
    /// `confsub`: which channels each subprocess of the group has a diagram in.
    contributes: Vec<Vec<bool>>,
}

/// A fixed-form source's statements, continuation lines (a non-blank sixth
/// column) joined onto the line they continue.
fn statements(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines() {
        let continued = raw.len() > 5
            && !raw.starts_with(['C', 'c', '!'])
            && raw[..5].trim().is_empty()
            && !raw[5..6].trim().is_empty();
        if continued {
            out.last_mut()
                .expect("a statement to continue")
                .push_str(raw[6..].trim());
        } else {
            out.push(raw.trim().to_string());
        }
    }
    out
}

impl Directory {
    /// Parse `configs.inc`, `config_nqcd.inc` and `nexternal.inc`, taking each
    /// line's mass and width from the dump's `IFOR` rows with the same
    /// structure.
    fn load(dir: &Path, ifor: &[Vec<Value>]) -> Self {
        let read = |name: &str| {
            std::fs::read_to_string(dir.join(name))
                .unwrap_or_else(|e| panic!("{}: {e}", dir.join(name).display()))
        };
        let param = |text: &str, name: &str| -> usize {
            let key = format!("{name}=");
            text.lines()
                .find_map(|l| {
                    let compact: String = l.chars().filter(|c| !c.is_whitespace()).collect();
                    let at = compact.find(&key)?;
                    let rest = &compact[at + key.len()..];
                    rest.trim_end_matches(')').parse().ok()
                })
                .unwrap_or_else(|| panic!("{name} in {}", dir.display()))
        };
        let nexternal = read("nexternal.inc");
        let n_external = param(&nexternal, "NEXTERNAL");

        // `DATA (IFOREST(I,-k,c),I=1,2)/a,b/`, `DATA (SPROP(I,-k,c),I=1,n)/.../`,
        // `DATA TPRID(-k,c)/t/`.
        let args = |line: &str, open: &str| -> Option<(i32, usize, Vec<i64>)> {
            let at = line.find(open)?;
            let inner = &line[at + open.len()..];
            let close = inner.find(')')?;
            let head: Vec<&str> = inner[..close].split(',').collect();
            let slash = line.find('/')?;
            let values: Vec<i64> = line[slash + 1..]
                .trim_end()
                .trim_end_matches('/')
                .split(',')
                .map(|v| v.trim().parse().expect("an integer DATA value"))
                .collect();
            let (k, c) = if head.len() == 3 {
                (head[1], head[2])
            } else {
                (head[0], head[1])
            };
            Some((k.trim().parse().ok()?, c.trim().parse().ok()?, values))
        };
        let mut lines: BTreeMap<(usize, i32), ForestLine> = BTreeMap::new();
        let mut n_proc = 0;
        let line_at = |lines: &mut BTreeMap<(usize, i32), ForestLine>, c: usize, k: i32| {
            lines.entry((c, k)).or_insert(ForestLine {
                index: k,
                daughters: [0, 0],
                tprid: 0,
                sprop: Vec::new(),
                mass: 0.0,
                width: 0.0,
                forced: false,
            });
        };
        for line in statements(&read("configs.inc")).iter().map(String::as_str) {
            if let Some((k, c, v)) = args(line, "(IFOREST(I,") {
                line_at(&mut lines, c, k);
                lines.get_mut(&(c, k)).expect("inserted").daughters = [v[0] as i32, v[1] as i32];
            } else if let Some((k, c, v)) = args(line, "(SPROP(I,") {
                line_at(&mut lines, c, k);
                n_proc = v.len();
                lines.get_mut(&(c, k)).expect("inserted").sprop = v;
            } else if let Some((k, c, v)) = args(line, "TPRID(") {
                line_at(&mut lines, c, k);
                lines.get_mut(&(c, k)).expect("inserted").tprid = v[0];
            }
        }
        let n_configs = lines.keys().map(|(c, _)| *c).max().expect("a config");
        let mut configs = vec![ConfigForest::default(); n_configs];
        for ((c, _), mut line) in lines {
            // The same line in the dump carries its numerical mass and width.
            let found = ifor.iter().find(|row| {
                row[1].as_u64() == Some(c as u64)
                    && row[2].as_i64() == Some(line.index as i64)
                    && row[3].as_i64() == Some(line.daughters[0] as i64)
                    && row[4].as_i64() == Some(line.daughters[1] as i64)
                    && row[5].as_i64() == Some(line.tprid)
                    && row.len() == 8 + line.sprop.len()
                    && row[8..]
                        .iter()
                        .zip(&line.sprop)
                        .all(|(v, s)| v.as_i64() == Some(*s))
            });
            let row = found.unwrap_or_else(|| {
                panic!(
                    "{}: config {c} line {} is in no dumped IFOR row",
                    dir.display(),
                    line.index
                )
            });
            line.mass = row[6].as_f64().expect("mass");
            line.width = row[7].as_f64().expect("width");
            configs[c - 1].lines.push(line);
        }
        for config in &mut configs {
            config.lines.sort_by_key(|line| -line.index);
        }
        for raw in read("config_nqcd.inc").lines() {
            // `DATA NQCD(c)/n/`
            let Some(at) = raw.find("NQCD(") else {
                continue;
            };
            let rest = &raw[at + 5..];
            let c: usize = rest[..rest.find(')').expect(")")].parse().expect("config");
            let n: i64 = rest[rest.find('/').expect("/") + 1..]
                .trim()
                .trim_end_matches('/')
                .parse()
                .expect("nqcd");
            configs[c - 1].nqcd = n;
        }
        // `DATA (CONFSUB(I,c),I=1,n)/.../`: the diagram of subprocess I in
        // channel c, zero where it has none.
        let mut contributes = vec![vec![false; n_configs]; n_proc];
        for line in statements(&read("config_subproc_map.inc")) {
            let Some(at) = line.find("(CONFSUB(I,") else {
                continue;
            };
            let rest = &line[at + "(CONFSUB(I,".len()..];
            let c: usize = rest[..rest.find(')').expect(")")]
                .trim()
                .parse()
                .expect("config");
            let values = &line[line.find('/').expect("/") + 1..];
            for (iproc, diagram) in values.trim_end_matches('/').split(',').enumerate() {
                contributes[iproc][c - 1] = diagram.trim() != "0";
            }
        }
        Directory {
            n_external,
            n_proc,
            configs,
            contributes,
        }
    }

    fn channel_set(&self, external: &[i64]) -> ChannelSet {
        ChannelSet {
            n_external: self.n_external,
            n_incoming: 2,
            configs: self.configs.clone(),
            // Only the event's own subprocess is ever asked for a table, so the
            // group's other flavour rows repeat it.
            external_pdg: vec![external.to_vec(); self.n_proc],
            contributes: self.contributes.clone(),
        }
    }
}

/// Everything one row's events are replayed against.
pub struct Row {
    name: String,
    card: RunCard,
    choice: ScaleChoice,
    settings: ScaleSettings,
    colors: ColorTable,
    alpha_s: AlphaSSource,
    pdf: PdfMember,
    directories: HashMap<String, Directory>,
    sets: HashMap<(String, Vec<i64>), ChannelSet>,
}

impl Row {
    fn load(name: &str, header: &Value) -> Self {
        let mg = madgraph_dir();
        let card = RunCard::parse_file(&mg.join(format!("{name}_run_card.dat")))
            .unwrap_or_else(|e| panic!("{name}: run card: {e}"));
        let choice = ScaleChoice::from_run_card(&card)
            .unwrap_or_else(|e| panic!("{name}: scale prescription: {e}"));
        let fixed = |beam: &str| card.fixed_fac_scale || card.get(beam).expect("known").as_bool();
        let settings = ScaleSettings {
            scalefact: card.float("scalefact"),
            fixed_ren: card.fixed_ren_scale,
            fixed_fac: [fixed("fixed_fac_scale1"), fixed("fixed_fac_scale2")],
            beam_has_pdf: [card.lpp1 != 0, card.lpp2 != 0],
            ickkw: card.int("ickkw"),
            xqcut: card.float("xqcut"),
            xmtc: card.float("xmtcentral"),
            pdfwgt: card.get("pdfwgt").expect("known").as_bool(),
        };
        let model = common::sm_model();
        let colors = ColorTable::new(
            model.particles.values().map(|p| (p.pdg_code, p.color)),
            card.maxjetflavor,
        );
        let params = ParamCard::from_file(&mg.join(format!("output/{name}/Cards/param_card.dat")))
            .expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let alpha_s = AlphaSSource::from_run_card(
            &card,
            a_s,
            common::pdfset::set_alpha_s_info(&card).as_ref(),
        )
        .unwrap_or_else(|e| panic!("{name}: alpha_s: {e}"));

        // The dumped constants must be the card's, as resolved here: the dump
        // keeps every variant a directory wrote, including passes that ran with
        // the defaults unread, so the one whose ickkw matches is the run's.
        let const2: Vec<&Vec<Value>> = header["directory"]["CONST2"]
            .as_array()
            .expect("CONST2")
            .iter()
            .map(|r| r.as_array().expect("row"))
            .filter(|r| r[0].as_i64() == Some(settings.ickkw))
            .collect();
        assert!(
            !const2.is_empty(),
            "{name}: no CONST2 variant at the card's ickkw"
        );
        for row in &const2 {
            assert_eq!(row[1].as_f64(), Some(settings.xqcut), "{name}: xqcut");
            assert_eq!(row[2].as_f64(), Some(choice.alpsfact()), "{name}: alpsfact");
            assert_eq!(
                row[3].as_i64(),
                Some(choice.asrwgtflavor()),
                "{name}: asrwgtflavor"
            );
            assert_eq!(
                row[4].as_i64(),
                Some(card.maxjetflavor),
                "{name}: maxjetflavor"
            );
            // `setrun.f:82` clears pdfwgt at ickkw = 0, where every reader of it
            // also tests ickkw > 0; the card keeps its value here, so the flag
            // that can act is what is compared.
            assert_eq!(
                row[5].as_bool(),
                Some(settings.pdfwgt && settings.ickkw > 0),
                "{name}: pdfwgt"
            );
            // setcuts' xqcut rewrite, against the card as this crate resolved it.
            for (k, cut) in [(10, "ptj"), (11, "mmjj"), (12, "drjj"), (13, "drjl")] {
                assert_eq!(row[k].as_f64(), Some(card.float(cut)), "{name}: {cut}");
            }
            assert_eq!(
                row[16].as_f64(),
                Some(settings.scalefact),
                "{name}: scalefact"
            );
        }

        let pdf_name = "NNPDF23_lo_as_0130_qed";
        assert_eq!(card.lhaid, 247000, "{name}: the PDF set");
        let pdf = PdfSet::load(&mg.join("../pdf").join(pdf_name), pdf_name)
            .unwrap_or_else(|e| panic!("{name}: PDF set: {e}"))
            .member(0)
            .expect("PDF member 0");

        Row {
            name: name.to_string(),
            card,
            choice,
            settings,
            colors,
            alpha_s,
            pdf,
            directories: HashMap::new(),
            sets: HashMap::new(),
        }
    }

    fn set_for(&mut self, subdir: &str, external: &[i64], header: &Value) -> &ChannelSet {
        if !self.directories.contains_key(subdir) {
            let ifor: Vec<Vec<Value>> = header["directory"]["IFOR"]
                .as_array()
                .expect("IFOR")
                .iter()
                .map(|r| r.as_array().expect("row").clone())
                .collect();
            let dir = madgraph_dir().join(format!("output/{}/SubProcesses/{subdir}", self.name));
            self.directories
                .insert(subdir.to_string(), Directory::load(&dir, &ifor));
        }
        let key = (subdir.to_string(), external.to_vec());
        if !self.sets.contains_key(&key) {
            let set = self.directories[subdir].channel_set(external);
            self.sets.insert(key.clone(), set);
        }
        &self.sets[&key]
    }
}

// ── the tally ────────────────────────────────────────────────────────────────

/// Agreement per compared field: `(agreeing, checked, first divergent event)`.
#[derive(Default)]
struct Tally {
    events: usize,
    fields: BTreeMap<&'static str, (usize, usize, Option<usize>)>,
    worst: BTreeMap<&'static str, f64>,
    /// Events on which a beam's record scale differs from its density scale;
    /// on which the clustered configuration differs from the integration
    /// channel; and on which `SCALUP` read off the density scales would differ
    /// from the written one — the only events on which `SCALUP` alone can see
    /// the split, since it is the larger of two scales of which matching lowers
    /// only the smaller.
    controls: (usize, usize, usize),
    /// Matched events whose `P1` is not the mirror of `PP`.
    permuted: usize,
    /// Over the events whose factor gates: `Σ 1/A`, with `A` the product of the
    /// `αs` ratios, this crate's and MadEvent's, and the count. The events are
    /// unweighted, so `⟨1/A⟩` is σ without the `αs` factor over σ with it.
    inverse_alpha: (f64, f64, usize),
    /// Mean `rewgt` over the gated events.
    rewgt_sum: f64,
    /// Permuted events whose first-call scales differ from this crate's:
    /// (event, subprocess directory, MadEvent's weight factor over this crate's).
    rescaled: Vec<(usize, String, f64)>,
}

impl Tally {
    fn check(&mut self, field: &'static str, event: usize, ok: bool) {
        let entry = self.fields.entry(field).or_insert((0, 0, None));
        entry.1 += 1;
        if ok {
            entry.0 += 1;
        } else if entry.2.is_none() {
            entry.2 = Some(event);
        }
    }

    fn close(&mut self, field: &'static str, event: usize, a: f64, b: f64, tol: f64) {
        self.within(field, event, rel(a, b), tol);
    }

    /// Record an already-formed relative deviation.
    fn within(&mut self, field: &'static str, event: usize, deviation: f64, tol: f64) {
        let worst = self.worst.entry(field).or_insert(0.0);
        *worst = worst.max(deviation);
        self.check(field, event, deviation < tol);
    }

    fn print(&self, name: &str) {
        println!("\n{name}: {} events", self.events);
        for (field, (ok, checked, first)) in &self.fields {
            let worst = self
                .worst
                .get(field)
                .map_or(String::new(), |w| format!("  worst {w:.2e}"));
            let first = first.map_or(String::new(), |e| {
                format!("  first divergence at event {e}")
            });
            println!("  {field:<44} {ok:>6}/{checked:<6}{worst}{first}");
        }
        println!(
            "  events whose P1 is a symmetry permutation of PP: {}",
            self.permuted
        );
        println!(
            "  controls: a beam's record scale differs from its density scale on {}, CFG \
             from the integration channel on {}, SCALUP from the density-scale reading on {}",
            self.controls.0, self.controls.1, self.controls.2
        );
        let (mine, theirs, n) = self.inverse_alpha;
        if n > 0 {
            println!(
                "  rewgt: mean {:.6} over {n} gated events; without the alpha_s factor sigma \
                 scales by <1/A> = {:.6} (MadEvent's own ratios: {:.6})",
                self.rewgt_sum / n as f64,
                mine / n as f64,
                theirs / n as f64
            );
        }
        if !self.rescaled.is_empty() {
            let mut by_n: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
            for (event, dir, ratio) in &self.rescaled {
                println!("  info, permuted P1 with other scales: event {event} {dir}: MadEvent/vibegraph weight factor {ratio:.6}");
                by_n.entry(&dir[..2]).or_default().push(*ratio);
            }
            for (n, ratios) in by_n {
                let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
                let lo = ratios.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                println!(
                    "  info, permuted P1 with other scales, {n}: {} events, weight factor ratio \
                     mean {mean:.6}, range {lo:.6}..{hi:.6}",
                    ratios.len()
                );
            }
        }
    }

    fn all_agree(&self) -> bool {
        self.fields
            .iter()
            .filter(|(field, _)| !field.starts_with("info"))
            .all(|(_, (ok, checked, _))| ok == checked)
    }
}

fn rel(a: f64, b: f64) -> f64 {
    if a == b {
        return 0.0;
    }
    let scale = a.abs().max(b.abs());
    if scale == 0.0 {
        0.0
    } else {
        (a - b).abs() / scale
    }
}

/// `unwgt.f` fills `AQCDUP` as `g*g/4d0/3.1415926d0`, `g` built with the full π.
fn aqcdup_from_alpha_s(alpha_s: f64) -> f64 {
    #[allow(clippy::approx_constant)]
    const PI: f64 = 3.141592653589793;
    #[allow(clippy::approx_constant)]
    const PI_TRUNCATED: f64 = 3.1415926;
    alpha_s * PI / PI_TRUNCATED
}

// ── the replays ──────────────────────────────────────────────────────────────

/// Replay one call through `setclscales` from the state MadEvent entered it with.
fn replay_call(
    row: &Row,
    set: &ChannelSet,
    call: &CallRecords<'_>,
    n_external: usize,
) -> Result<ClusterScales, String> {
    let tables = set.merge_tables(call.iconfig());
    let channel = Channel {
        set,
        table: &tables[call.iproc() - 1],
        colors: &row.colors,
        this_config: call.iconfig(),
        iproc: call.iproc(),
    };
    let stored = call.memo_on_entry();
    let mut memo = JetMemo((stored >= 0).then_some(stored as usize));
    setclscales(
        &channel,
        row.choice.cluster_settings(),
        &row.settings,
        &call.momenta(n_external),
        &mut memo,
        row.card.get("chcluster").expect("known").as_bool(),
        &[],
        call.entry(),
        false,
    )
    .map_err(|e| format!("{e:?}"))
}

/// Compare one replayed call with its records, each field under its name in
/// `fields`. `fixed` names the beams whose factorisation scale the card fixes,
/// which carry no central value.
fn compare_call(
    tally: &mut Tally,
    fields: [&'static str; 6],
    event: usize,
    call: &CallRecords<'_>,
    mine: &ClusterScales,
    fixed: [bool; 2],
) {
    let [memo_f, pt2_f, branch_f, scale_f, central_f, ovr_f] = fields;
    let steps: Vec<(&str, usize)> = mine
        .attempts
        .iter()
        .map(|a| (a.memo.name(), a.jets))
        .collect();
    tally.check(memo_f, event, steps == call.memo_steps());

    let mut worst = 0.0f64;
    for stage in call.all("PT2").filter(|r| r.s(1) == "FINAL") {
        worst = worst.max(rel(mine.pt2[stage.u(2) - 1], stage.f(3)));
    }
    tally.within(pt2_f, event, worst, AGREEMENT);

    let mur = call.last("MUR").map(|r| r.s(1)).unwrap_or("NOT_ENTERED");
    let muf = call.last("MUF").map(|r| r.s(1)).unwrap_or("NONE");
    tally.check(
        branch_f,
        event,
        mine.mur_branch.name() == mur && mine.muf_branch.name() == muf,
    );

    let (mu_r, q2fact) = call.out();
    let deviation = rel(mine.mu_r, mu_r)
        .max(rel(mine.q2fact[0], q2fact[0]))
        .max(rel(mine.q2fact[1], q2fact[1]));
    tally.within(scale_f, event, deviation, AGREEMENT);

    if let Some(central) = call.all("Q2BCK").find(|r| r.s(1) == "CENTRAL") {
        let theirs = [central.f(5), central.f(6)];
        let mut deviation = 0.0f64;
        let mut present = true;
        for beam in 0..2 {
            if fixed[beam] {
                continue;
            }
            match mine.q2central[beam] {
                Some(q2) => deviation = deviation.max(rel(q2, theirs[beam])),
                None => present = false,
            }
        }
        tally.check(central_f, event, present && deviation < AGREEMENT);
    }

    if let Some(after) = call.all("Q2OVR").find(|r| r.s(1) == "AFTER") {
        let jcentral = [after.u(2), after.u(3)];
        let theirs = [after.f(4), after.f(5)];
        let mut deviation = 0.0f64;
        for beam in 0..2 {
            if jcentral[beam] > 0 {
                deviation = deviation.max(rel(mine.pt2[jcentral[beam] - 1], theirs[beam]));
            }
        }
        tally.check(
            ovr_f,
            event,
            mine.jcentral == jcentral && deviation < AGREEMENT,
        );
    }
}

/// A vertex as both sides are compared on: `n`, class, `[mother, d1, d2]`,
/// their codes, `ipart(1, mother)`.
type VertexKey<'a> = (usize, &'a str, [usize; 3], [i64; 3], usize);

/// MadEvent's name for a vertex class.
fn class_name(r: &Rewgt, n: usize, class: VertexClass) -> &'static str {
    if let Some(RewgtKill::AlphaSScale { n: at, .. }) = r.kill {
        if at == n {
            return "KILL_Q2";
        }
    }
    match class {
        VertexClass::Core => "CORE",
        VertexClass::Isr => "ISR",
        VertexClass::Fsr => "FSR",
        _ => "NONE",
    }
}

/// MadEvent's name for a density-chain action.
fn action_name(outcome: PdfOutcome, killed: bool) -> &'static str {
    match outcome {
        PdfOutcome::First => "FIRST",
        PdfOutcome::Ratio { .. } if killed => "KILL_PDF",
        PdfOutcome::Ratio { .. } => "RATIO",
        PdfOutcome::NoRise => "NOT_RISING",
        PdfOutcome::PastLast => "NONE",
    }
}

/// The matched factor of one event, recomputed from the production path's
/// history and compared field by field with `RWVX`, `RWPDF`, `RWKILL` and
/// `RWEND`. `f` names each field, prefixed as informational on a permuted event.
#[allow(clippy::too_many_arguments)]
fn compare_rewgt(
    tally: &mut Tally,
    row: &Row,
    event: &Event,
    history: &vibegraph::coupling::scales::ClusterHistory,
    p1: &[[f64; 4]],
    i: usize,
    gated: bool,
    f: &dyn Fn(&'static str) -> &'static str,
) -> Option<Rewgt> {
    let begin = event.only("RWBEG")?;
    let settings = row.choice.rewgt_settings().expect("a matched row");
    let mut legs: Vec<Rec<'_>> = event.recs().filter(|r| r.tag() == "RWLEG").collect();
    legs.sort_by_key(|r| r.u(1));
    let flavours: Vec<i64> = legs.iter().map(|r| r.i(2)).collect();
    let x = [begin.f(6), begin.f(7)];
    // The clustering's first beam takes the momentum fraction of the physical
    // beam its leg 1 arrives on: `ib(1)` is that beam.
    let from = if p1[0][3] > 0.0 { 1 } else { 2 };
    tally.check(
        "rewgt: beam 1's x is that of the beam its leg 1 arrives on",
        i,
        begin.u(8) == from,
    );
    let Some(input) = history.rewgt_history() else {
        tally.check(f("rewgt: product (RWEND)"), i, false);
        return None;
    };
    tally.check(
        f("rewgt: jlast"),
        i,
        input.jlast == [begin.u(16), begin.u(17)],
    );
    let pdf = &row.pdf;
    let mine = match rewgt(
        &input,
        &row.colors,
        &settings,
        &flavours,
        x,
        |q| row.alpha_s.eval(q),
        |pdg, x, q2| pdf.xfx_q2(pdg as i32, x, q2),
    ) {
        Ok(r) => r,
        Err(e) => {
            println!("{} event {i}: rewgt refused: {e}", row.name);
            tally.check(f("rewgt: product (RWEND)"), i, false);
            return None;
        }
    };
    tally.close(
        f("rewgt: asref"),
        i,
        row.alpha_s.eval(input.mu_r),
        begin.f(4),
        PDF_AGREEMENT,
    );

    // Vertices, in order.
    let vx: Vec<Rec<'_>> = event.recs().filter(|r| r.tag() == "RWVX").collect();
    let theirs: Vec<VertexKey<'_>> = vx
        .iter()
        .map(|r| {
            (
                r.u(1),
                r.s(2),
                [r.u(3), r.u(4), r.u(5)],
                [r.i(6), r.i(7), r.i(8)],
                r.u(9),
            )
        })
        .collect();
    let ours: Vec<VertexKey<'_>> = mine
        .vertices
        .iter()
        .map(|v| {
            (
                v.n,
                class_name(&mine, v.n, v.class),
                [
                    v.mother as usize,
                    v.daughters[0] as usize,
                    v.daughters[1] as usize,
                ],
                v.pdg,
                v.ipart,
            )
        })
        .collect();
    tally.check(
        f("rewgt: vertices (class, lines, codes, ipart)"),
        i,
        ours == theirs,
    );
    let (mut kt, mut num, mut ratio) = (0.0f64, 0.0f64, 0.0f64);
    for (v, r) in mine.vertices.iter().zip(&vx) {
        if let Some(a) = v.alpha_s {
            kt = kt.max(rel(a.q2, r.f(16)));
            num = num.max(rel(a.numerator, r.f(18)));
            ratio = ratio.max(rel(a.ratio, r.f(20)));
        }
    }
    tally.within(f("rewgt: reweighted vertex kt^2"), i, kt, AGREEMENT);
    tally.within(f("rewgt: alpha_s(alpsfact kt)"), i, num, PDF_AGREEMENT);
    tally.within(f("rewgt: alpha_s ratio"), i, ratio, PDF_AGREEMENT);

    // Density chains, per beam, in order.
    let killed_pdf = matches!(mine.kill, Some(RewgtKill::PdfDenominator { .. }));
    let (mut steps_ok, mut x_dev, mut q_dev, mut f_dev, mut r_dev) =
        (true, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for beam in 0..2 {
        let theirs: Vec<Rec<'_>> = event
            .recs()
            .filter(|r| r.tag() == "RWPDF" && r.u(2) == beam + 1)
            .collect();
        let ours = &mine.pdf_chain[beam];
        if theirs.len() != ours.len() {
            steps_ok = false;
            continue;
        }
        for (k, (s, r)) in ours.iter().zip(&theirs).enumerate() {
            let last = k + 1 == ours.len();
            let action = action_name(s.outcome, killed_pdf && last);
            if (s.n, s.flavour, action) != (r.u(1), r.i(5), r.s(10)) {
                steps_ok = false;
            }
            x_dev = x_dev.max(rel(s.x, r.f(7)));
            q_dev = q_dev.max(rel(s.q2_now, r.f(8)));
            if !s.q2_prev.is_nan() {
                q_dev = q_dev.max(rel(s.q2_prev, r.f(9)));
            }
            if let PdfOutcome::Ratio {
                numerator,
                denominator,
                ratio,
            } = s.outcome
            {
                f_dev = f_dev
                    .max(rel(numerator, r.f(11)))
                    .max(rel(denominator, r.f(12)));
                r_dev = r_dev.max(rel(ratio, r.f(13)));
            }
        }
    }
    tally.check(f("rewgt: PDF chain (vertex, flavour, action)"), i, steps_ok);
    tally.within(f("rewgt: PDF x after z"), i, x_dev, AGREEMENT);
    tally.within(f("rewgt: PDF q2_now, q2_prev"), i, q_dev, AGREEMENT);
    tally.within(f("rewgt: PDF densities"), i, f_dev, PDF_AGREEMENT);
    tally.within(f("rewgt: PDF ratio"), i, r_dev, PDF_AGREEMENT);

    let their_kill = event.only("RWKILL").map(|r| (r.s(1).to_string(), r.u(2)));
    let our_kill = mine.kill.map(|k| match k {
        RewgtKill::AlphaSScale { n, .. } => ("ALPHAS_Q2".to_string(), n),
        RewgtKill::PdfDenominator { n, .. } => ("PDF_DENOMINATOR".to_string(), n),
    });
    tally.check(f("rewgt: kill"), i, their_kill == our_kill);
    let end = event.only("RWEND").expect("an RWEND record");
    tally.close(
        f("rewgt: product (RWEND)"),
        i,
        mine.weight,
        end.f(1),
        PDF_AGREEMENT,
    );
    tally.check(
        f("rewgt: product = listed factors"),
        i,
        mine.product_of_factors().to_bits() == mine.weight.to_bits(),
    );
    if gated && mine.kill.is_none() {
        let a: f64 = mine
            .vertices
            .iter()
            .filter_map(|v| v.alpha_s)
            .map(|a| a.ratio)
            .product();
        let theirs: f64 = vx
            .iter()
            .filter(|r| matches!(r.s(2), "ISR" | "FSR"))
            .map(|r| r.f(20))
            .product();
        tally.inverse_alpha.0 += 1.0 / a;
        tally.inverse_alpha.1 += 1.0 / theirs;
        tally.inverse_alpha.2 += 1;
        tally.rewgt_sum += mine.weight;
    }
    Some(mine)
}

fn replay_row(name: &str) -> (Tally, BTreeMap<(String, usize), BTreeSet<usize>>) {
    let entry = dump_manifest()["runs"][name].clone();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(entry["path"].as_str().expect("dump path"));
    assert!(path.is_file(), "{name}: no dump at {}", path.display());
    let (header, events) = read_dump(&path);
    let mut row = Row::load(name, &header);
    let matched = row.settings.ickkw > 0;
    let mut tally = Tally::default();
    let mut memo_counts: BTreeMap<(String, usize), BTreeSet<usize>> = BTreeMap::new();

    for event in events {
        tally.events += 1;
        let i = event.index;
        let calls = event.calls();
        tally.check(
            "setclscales calls per event",
            i,
            calls.len() == if matched { 2 } else { 1 },
        );
        let first = &calls[0];
        assert!(
            !first.keepq2bck(),
            "{name} event {i}: first call keeps q2bck"
        );
        let n_external = first.n_external();
        let external = first.external(n_external);
        let subdir = event.subprocess_dir().to_string();
        let set = row.set_for(&subdir, &external, &header).clone();

        // The memo: a restricted clustering of every channel of the directory,
        // on this event's momenta, and the count it stores.
        let momenta = first.momenta(n_external);
        for config in 1..=set.configs.len() {
            let tables = set.merge_tables(config);
            let channel = Channel {
                set: &set,
                table: &tables[first.iproc() - 1],
                colors: &row.colors,
                this_config: config,
                iproc: first.iproc(),
            };
            let mut memo = JetMemo::default();
            let _ = setclscales(
                &channel,
                row.choice.cluster_settings(),
                &ScaleSettings::default(),
                &momenta,
                &mut memo,
                false,
                &[],
                (0.0, [0.0, 0.0]),
                false,
            );
            if let Some(count) = memo.0 {
                memo_counts
                    .entry((subdir.clone(), config))
                    .or_default()
                    .insert(count);
            }
        }

        // Engine replay, call by call.
        for (n, call) in calls.iter().enumerate() {
            let fields = if n == 0 {
                [
                    "engine 1st: memo steps",
                    "engine 1st: vertex scales",
                    "engine 1st: muR/muF branches",
                    "engine 1st: muR, q2fact",
                    "engine 1st: q2bck (central)",
                    "engine 1st: Q2OVR",
                ]
            } else {
                [
                    "engine 2nd: memo steps",
                    "engine 2nd: vertex scales",
                    "engine 2nd: muR/muF branches",
                    "engine 2nd: muR, q2fact",
                    "engine 2nd: q2fact (central)",
                    "engine 2nd: Q2OVR",
                ]
            };
            match replay_call(&row, &set, call, n_external) {
                Ok(mine) => {
                    compare_call(&mut tally, fields, i, call, &mine, row.settings.fixed_fac)
                }
                Err(_) => tally.check(fields[3], i, false),
            }
        }

        // The production path, on the momenta the matrix element is evaluated
        // at. MadEvent's first call clusters the sampled point `PP`; `rewgt`'s
        // clusters `P1`, the point after `DSIGPROC`'s symmetry permutation of
        // the configuration and its mirror, which is the one the matrix element
        // reads. A sampled term here is that `P1`, so both calls read it; the
        // unmatched row has only the first call, whose `PP` is what is dumped.
        let physical = calls.last().expect("a call").momenta(n_external);
        let incoming = [physical[0], physical[1]];
        let input = ClusterInput {
            set: &set,
            colors: &row.colors,
            this_config: first.iconfig(),
            iproc: first.iproc(),
            tables: None,
        };
        let history = row.choice.cluster_history(
            &ScaleEvent {
                incoming,
                outgoing: &physical[2..],
            },
            &input,
        );
        let history = match history {
            Ok(h) => {
                tally.check("production: xqcut passes a written event", i, true);
                h
            }
            Err(e) => {
                println!("{name} event {i}: production refused: {e}");
                tally.check("production: xqcut passes a written event", i, false);
                continue;
            }
        };
        // MadEvent's first call reads `PP`; where `P1` is not merely its mirror
        // (a symmetry permutation of identical-mass final legs), the first call
        // clustered a relabelled event, and every field built on its scales is
        // that relabelled event's. Those events are checked through `PP` below
        // and reported beside the production path, not gated on it.
        let mirrored = first.scl().u(4) == 2;
        let unmirror = |p: [f64; 4]| {
            if mirrored {
                [p[0], p[1], -p[2], -p[3]]
            } else {
                p
            }
        };
        let permuted = matched
            && physical
                .iter()
                .zip(&momenta)
                .any(|(b, a)| (0..4).any(|k| rel(unmirror(*b)[k], a[k]) > 1e-9));
        let f = |name: &'static str| -> &'static str {
            if permuted {
                Box::leak(format!("info, permuted P1: {name}").into_boxed_str())
            } else {
                name
            }
        };
        if permuted {
            tally.permuted += 1;
            let from_pp = row
                .choice
                .cluster_history(
                    &ScaleEvent {
                        incoming: [momenta[0], momenta[1]],
                        outgoing: &momenta[2..],
                    },
                    &input,
                )
                .map(|h| {
                    let (mu_r, q2fact) = first.out();
                    rel(h.first.mu_r, mu_r)
                        .max(rel(h.first.q2fact[0], q2fact[0]))
                        .max(rel(h.first.q2fact[1], q2fact[1]))
                })
                .unwrap_or(1.0);
            tally.within(
                "permuted P1: first call reproduced from PP",
                i,
                from_pp,
                AGREEMENT,
            );
        }
        let stored = history
            .first
            .attempts
            .iter()
            .find(|a| a.memo == MemoStep::Stored)
            .map(|a| a.jets as i64);
        tally.check(
            "production: memo value = njetstore on entry",
            i,
            stored == Some(first.memo_on_entry()),
        );
        let recl = |h: &ClusterScales| h.attempts.iter().any(|a| a.memo == MemoStep::Reclustered);
        tally.check(
            "production 1st: restricted re-cluster",
            i,
            recl(&history.first) == first.reclustered(),
        );
        let (mu_r, q2fact) = first.out();
        tally.within(
            f("production 1st: muR, q2fact"),
            i,
            rel(history.first.mu_r, mu_r)
                .max(rel(history.first.q2fact[0], q2fact[0]))
                .max(rel(history.first.q2fact[1], q2fact[1])),
            AGREEMENT,
        );
        if let (Some(second), Some(call)) = (&history.second, calls.get(1)) {
            tally.check(
                "production 2nd: restricted re-cluster",
                i,
                recl(second) == call.reclustered(),
            );
            let (mu_r, q2fact) = call.out();
            tally.within(
                f("production 2nd: muR, q2fact"),
                i,
                rel(second.mu_r, mu_r)
                    .max(rel(second.q2fact[0], q2fact[0]))
                    .max(rel(second.q2fact[1], q2fact[1])),
                AGREEMENT,
            );
            if let Some(after) = call.all("Q2OVR").find(|r| r.s(1) == "AFTER") {
                let jcentral = [after.u(2), after.u(3)];
                let mut deviation = 0.0f64;
                for (beam, &vertex) in jcentral.iter().enumerate() {
                    if vertex > 0 {
                        deviation = deviation.max(rel(second.pt2[vertex - 1], after.f(4 + beam)));
                    }
                }
                tally.check(
                    f("production 2nd: Q2OVR"),
                    i,
                    second.jcentral == jcentral && deviation < AGREEMENT,
                );
            }
            if let (Some(out), Some(q2bck)) =
                (call.all("Q2BCK").find(|r| r.s(1) == "OUT"), history.q2bck)
            {
                tally.within(
                    f("production: q2bck"),
                    i,
                    rel(q2bck[0], out.f(3)).max(rel(q2bck[1], out.f(4))),
                    AGREEMENT,
                );
            }
        }
        let scales = history.event_scales();
        if matched {
            if let Some(cfg) = event.only("CFG") {
                tally.check(
                    "production: clustered configuration (CFG)",
                    i,
                    scales.clustered_config.map(|c| c + 1) == Some(cfg.u(3)),
                );
            }
        }
        let out = event.only("OUT").expect("an OUT record");
        tally.close(
            f("production: SCALUP"),
            i,
            scalup(&scales),
            out.f(4),
            AGREEMENT,
        );
        if matched {
            // The controls: the record read at the densities' scale, and the
            // colour configuration read as the integration channel, are the two
            // readings the matched fields replace. Each must miss on some events,
            // or the fields above could not tell the readings apart.
            if (0..2).any(|beam| rel(scales.mu_f[beam], scales.mu_f_record[beam]) >= AGREEMENT) {
                tally.controls.0 += 1;
            }
            if rel(scales.mu_f[0].max(scales.mu_f[1]), out.f(4)) >= AGREEMENT {
                tally.controls.2 += 1;
            }
            if let Some(cfg) = event.only("CFG") {
                if cfg.u(3) != first.iconfig() {
                    tally.controls.1 += 1;
                }
            }
        }
        tally.close(
            f("production: AQCDUP"),
            i,
            aqcdup_from_alpha_s(row.alpha_s.eval(scales.mu_r)),
            out.f(5),
            AQCDUP_AGREEMENT,
        );

        if !matched {
            // `rewgt` returns `1` before reading anything at `ickkw = 0`.
            let end = event.only("RWEND").map_or(1.0, |r| r.f(1));
            tally.check(
                "rewgt = 1 without matching",
                i,
                end == 1.0 && row.choice.rewgt_settings().is_none(),
            );
            continue;
        }
        let Some(mine) = compare_rewgt(
            &mut tally, &row, &event, &history, &physical, i, !permuted, &f,
        ) else {
            continue;
        };
        // Where the first call's scales differ from this crate's, the weight
        // factor it drives differs too: `rewgt · αs(μR)^n · f₁ f₂` at the
        // densities' scales, each side at its own scales and momentum fractions'
        // flavours, with this crate's `αs` and densities on both.
        let (mu_r, q2fact) = first.out();
        let differs = rel(history.first.mu_r, mu_r)
            .max(rel(history.first.q2fact[0], q2fact[0]))
            .max(rel(history.first.q2fact[1], q2fact[1]))
            >= AGREEMENT;
        if permuted && differs {
            let begin = event.only("RWBEG").expect("RWBEG");
            let end = event.only("RWEND").expect("RWEND");
            let mut legs: Vec<Rec<'_>> = event.recs().filter(|r| r.tag() == "RWLEG").collect();
            legs.sort_by_key(|r| r.u(1));
            let x = [begin.f(6), begin.f(7)];
            let nqcd = row.directories[&subdir].configs[first.iconfig() - 1].nqcd as i32;
            let factor = |rw: f64, mu_r: f64, q2: [f64; 2]| {
                rw * row.alpha_s.eval(mu_r).powi(nqcd)
                    * (0..2)
                        .map(|j| row.pdf.xfx_q2(legs[j].i(2) as i32, x[j], q2[j]) / x[j])
                        .product::<f64>()
            };
            let theirs = factor(end.f(1), mu_r, [begin.f(12), begin.f(13)]);
            let ours = factor(mine.weight, history.first.mu_r, history.first.q2fact);
            tally.rescaled.push((i, subdir.clone(), theirs / ours));
        }
    }
    (tally, memo_counts)
}

fn dump_manifest() -> Value {
    let path = madgraph_dir().join("mlm_dump_manifest.json");
    serde_json::from_slice(&std::fs::read(&path).expect("mlm_dump_manifest.json"))
        .expect("manifest parses")
}

fn census_memo(name: &str) -> BTreeMap<(String, usize), BTreeSet<usize>> {
    let census: Value = serde_json::from_slice(
        &std::fs::read(madgraph_dir().join("mlm_census.json")).expect("mlm_census.json"),
    )
    .expect("census parses");
    let mut out: BTreeMap<(String, usize), BTreeSet<usize>> = BTreeMap::new();
    for (key, values) in census["rows"][name]["jet_memo"]["njetstore_on_entry_by_channel"]
        .as_object()
        .expect("jet_memo")
    {
        // `<subprocess dir>/<job dir>:<iconfig>`
        let (path, config) = key.rsplit_once(':').expect("a keyed channel");
        let subdir = path.split('/').next().expect("subprocess dir").to_string();
        for v in values.as_array().expect("values") {
            out.entry((subdir.clone(), config.parse().expect("iconfig")))
                .or_default()
                .insert(v.as_u64().expect("count") as usize);
        }
    }
    out
}

fn selected_rows() -> Vec<&'static str> {
    let only = std::env::var("MLM_ROW").unwrap_or_default();
    ROWS.iter()
        .copied()
        .filter(|r| only.is_empty() || r.contains(&only))
        .collect()
}

#[test]
#[ignore = "oracle layer: the MLM dumps are outside the reference bundle; `pixi run validate-mlm-dumps` runs this against them"]
fn matched_scales_reproduce_madevents_event_by_event() {
    let mut failed = Vec::new();
    for name in selected_rows() {
        let (tally, _) = replay_row(name);
        tally.print(name);
        let pinned = dump_manifest()["runs"][name]["n_events"]
            .as_u64()
            .expect("n_events") as usize;
        assert_eq!(tally.events, pinned, "{name}: event count");
        if !tally.all_agree() {
            failed.push(name);
        }
        let matched = !name.contains("xqcut_only");
        assert!(
            !matched || (tally.controls.0 > 0 && tally.controls.1 > 0),
            "{name}: a control never fired, so the matched fields cannot tell the readings apart"
        );
        // The negative control: without the `αs` ratios σ would move by `⟨1/A⟩ − 1`
        // over MadEvent's own (unweighted) events. The σ gates hold these rows to
        // about 0.1%, so the factor must be worth far more than that.
        if matched {
            let (inverse, _, n) = tally.inverse_alpha;
            let shift = (inverse / n as f64 - 1.0).abs();
            assert!(
                shift > NEGATIVE_CONTROL_MIN_SHIFT,
                "{name}: dropping the alpha_s factor moves sigma by only {shift:.4}"
            );
        }
    }
    assert!(failed.is_empty(), "rows with divergent fields: {failed:?}");
}

/// MadEvent's jet memo holds, per job and channel, the jet count its first point
/// stored from a clustering restricted to that channel. The rule this crate
/// uses — the restricted count of the event at hand — is that value exactly when
/// the restricted count is the same on every point, which is what is measured
/// here: every channel of every directory, on every dumped event of the
/// directory, against the census of every job.
#[test]
#[ignore = "oracle layer: the MLM dumps are outside the reference bundle; `pixi run validate-mlm-dumps` runs this against them"]
fn the_jet_memo_is_the_channel_s_restricted_jet_count() {
    let mut problems = Vec::new();
    for name in selected_rows() {
        let (_, counts) = replay_row(name);
        let census = census_memo(name);
        let mut single = 0;
        for (key, values) in &counts {
            if values.len() != 1 {
                problems.push(format!("{name} {key:?}: restricted counts {values:?}"));
            } else {
                single += 1;
            }
        }
        let mut agreeing = 0;
        for (key, expected) in &census {
            match counts.get(key) {
                Some(mine) if mine == expected => agreeing += 1,
                other => problems.push(format!(
                    "{name} {key:?}: census {expected:?}, restricted counts {other:?}"
                )),
            }
        }
        println!(
            "{name}: {} channels clustered, {single} with one restricted count over every \
             event of their directory; {agreeing}/{} census channels agree",
            counts.len(),
            census.len()
        );
    }
    for p in &problems {
        println!("  {p}");
    }
    assert!(problems.is_empty(), "{} memo discrepancies", problems.len());
}
