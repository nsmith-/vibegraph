//! A request resolved against the PDF sets it names, and the writer that
//! turns one event's record into its weights.
//!
//! Every `--pdf` entry resolves through the same tiers and consent as the
//! run's own set. Missing sets are collected first and asked for together, one
//! question for every download a round of lookups needs, and every requested
//! member is then loaded up front, in parallel: each is read by every event,
//! so loading on first use would save nothing after the first event and would
//! move a missing file into the middle of a run.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

use rayon::prelude::*;
use thiserror::Error;

use super::header::{banner_lines, expand, Arg, SetHeader, SlotId};
use super::record::{LoRecord, RecordKinematics};
use super::request::{MemberRef, PdfEntry, PdfRef, VariationRequest};
use super::weight::{lo_weight, Variation, VariationPdf};
use crate::artifact::FileDigest;
use crate::cache::lhapdf::{DownloadRequest, FetchFactory, Tier, Tiers};
use crate::cache::store::{Fetch, FetchError};
use crate::pdf::alphas::GridAlphaS;
use crate::pdf::select::{open_set_by_name, open_set_for_lhaid, LoadedPdf, OpenedSet};
use crate::pdf::{PdfMember, PdfSet};

/// Why a request could not be resolved against its PDF sets.
#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("systematics PDF entry {entry}: {why}")]
    Entry { entry: String, why: String },
    #[error(
        "the systematics --pdf list starts with {first} (lhaid {first_id}), not the run's own \
         lhaid {run}: MadGraph's module finds the nominal only in first place when the list \
         holds two or more PDFs, and fails otherwise. Put {run} (or errorset) first"
    )]
    NominalNotFirst {
        first: String,
        first_id: i64,
        run: i64,
    },
    #[error("PDF set {set} member {member}: {why}")]
    Member {
        set: String,
        member: u32,
        why: String,
    },
    #[error(
        "the systematics weights need {count} download(s) that were not allowed: {what}\n{why}"
    )]
    Declined {
        count: usize,
        what: String,
        why: String,
    },
    #[error(
        "PDF set {set}'s {file} changed while the run read it: sha256 {got}, the run's own \
         density read {want}"
    )]
    CentralChanged {
        set: String,
        file: String,
        got: String,
        want: String,
    },
}

/// A loaded member as the weights read it: its densities and its own
/// strong coupling.
pub struct MemberEval {
    member: PdfMember,
    alphas: GridAlphaS,
}

impl VariationPdf for MemberEval {
    fn alphas_q(&self, q: f64) -> Result<f64, String> {
        self.alphas.try_eval(q).map_err(|e| e.to_string())
    }

    fn xfx_q(&self, pdg: i64, x: f64, q: f64) -> Result<f64, String> {
        let pdg = i32::try_from(pdg).map_err(|e| e.to_string())?;
        self.member
            .try_xfx_q2(pdg, x, q * q)
            .map_err(|e| e.to_string())
    }
}

/// One set the systematics weights read, as the event file's header records
/// it: the set, its `SetIndex`, the tier it was found in, and every file read,
/// each once.
#[derive(Clone, Debug, PartialEq)]
pub struct SystematicsSet {
    pub set: String,
    pub set_index: i64,
    pub tier: Tier,
    pub files: Vec<FileDigest>,
}

/// A request's PDFs, resolved and loaded.
pub struct ResolvedPdfs {
    /// Every slot: the `--pdf` list's members in order, then the fresh
    /// nominal when the list does not start with it.
    slots: Vec<SlotId>,
    evals: Vec<Arc<MemberEval>>,
    list_len: usize,
    nominal: usize,
    sets: BTreeMap<i64, SetHeader>,
    inputs: Vec<SystematicsSet>,
}

impl ResolvedPdfs {
    /// What the header records of the sets read.
    pub fn inputs(&self) -> &[SystematicsSet] {
        &self.inputs
    }

    /// Distinct members loaded.
    pub fn members_loaded(&self) -> usize {
        self.inputs
            .iter()
            .map(|s| s.files.len().saturating_sub(1))
            .sum()
    }
}

/// Where an entry's set comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SetKey {
    Run,
    Id(i64),
    Name(String),
}

/// Which members of the set an entry reads.
#[derive(Clone, Copy, Debug)]
enum Members {
    All,
    Id(i64),
    Index(u32),
}

fn entry_text(entry: &PdfEntry) -> String {
    match entry {
        PdfEntry::ErrorSet => "errorset".to_string(),
        PdfEntry::Central => "central".to_string(),
        PdfEntry::Set(PdfRef::Id(id)) => id.to_string(),
        PdfEntry::Set(PdfRef::Name(name)) => name.clone(),
        PdfEntry::Member(MemberRef::Id(id)) => format!("{id}@0"),
        PdfEntry::Member(MemberRef::Named { set, member }) => format!("{set}@{member}"),
    }
}

fn target(entry: &PdfEntry, run_lhaid: i64) -> (SetKey, Members) {
    match entry {
        PdfEntry::ErrorSet => (SetKey::Run, Members::All),
        PdfEntry::Central => (SetKey::Run, Members::Id(run_lhaid)),
        PdfEntry::Set(PdfRef::Id(id)) => (SetKey::Id(*id), Members::All),
        PdfEntry::Set(PdfRef::Name(name)) => (SetKey::Name(name.clone()), Members::All),
        PdfEntry::Member(MemberRef::Id(id)) => (SetKey::Id(*id), Members::Id(*id)),
        PdfEntry::Member(MemberRef::Named { set, member }) => {
            (SetKey::Name(set.clone()), Members::Index(*member))
        }
    }
}

/// A fetch shared by every download one consent covered.
struct SharedFetch(Rc<dyn Fetch>);

impl Fetch for SharedFetch {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.0.fetch(url)
    }
}

/// The marker a lookup refused in a probing round carries.
const DEFERRED: &str = "deferred until every download is known";

/// The one question for several downloads: what each is, every URL, and
/// their size where every one is known.
fn combined_request(requests: &[DownloadRequest]) -> DownloadRequest {
    if let [one] = requests {
        return one.clone();
    }
    let what = requests
        .iter()
        .map(|r| r.what.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    DownloadRequest {
        what: format!(
            "{} downloads for the systematics weights ({what})",
            requests.len()
        ),
        url: requests
            .iter()
            .map(|r| r.url.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        bytes: requests.iter().map(|r| r.bytes).sum(),
        sha256: None,
        destination: requests[0]
            .destination
            .parent()
            .map_or_else(|| requests[0].destination.clone(), |p| p.to_path_buf()),
        archive: true,
    }
}

/// Open every set `keys` names. Lookups run in rounds: a round's downloads
/// are recorded rather than made, asked for in one question, and the round
/// is run again with exactly those allowed.
fn open_sets(
    keys: &BTreeSet<SetKey>,
    run_set: &PdfSet,
    run_tier: Tier,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<BTreeMap<SetKey, OpenedSet>, ResolveError> {
    let mut allowed: BTreeSet<String> = BTreeSet::new();
    let mut fetcher: Option<Rc<dyn Fetch>> = None;
    loop {
        let deferred: RefCell<Vec<DownloadRequest>> = RefCell::new(Vec::new());
        let probe = |request: &DownloadRequest| -> Result<Box<dyn Fetch>, String> {
            match &fetcher {
                Some(shared) if allowed.contains(&request.url) => {
                    Ok(Box::new(SharedFetch(shared.clone())))
                }
                _ => {
                    deferred.borrow_mut().push(request.clone());
                    Err(DEFERRED.to_string())
                }
            }
        };
        let mut opened = BTreeMap::new();
        let mut failure = None;
        for key in keys {
            let result = match key {
                SetKey::Run => PdfSet::load(run_set.dir(), run_set.name())
                    .map_err(|e| e.to_string())
                    .and_then(|set| {
                        let set_index = set
                            .info
                            .set_index()
                            .ok_or_else(|| format!("{} has no SetIndex", run_set.name()))?;
                        Ok(OpenedSet {
                            set,
                            set_index,
                            found: crate::cache::lhapdf::FoundSet {
                                dir: run_set.dir().to_path_buf(),
                                tier: run_tier,
                            },
                        })
                    }),
                SetKey::Id(id) => open_set_for_lhaid(*id, tiers, &probe).map_err(|e| e.to_string()),
                SetKey::Name(name) => {
                    open_set_by_name(name, tiers, &probe).map_err(|e| e.to_string())
                }
            };
            match result {
                Ok(set) => {
                    opened.insert(key.clone(), set);
                }
                Err(why) => {
                    failure.get_or_insert((key.clone(), why));
                }
            }
        }
        let asked = deferred.into_inner();
        if asked.is_empty() {
            if let Some((key, why)) = failure {
                return Err(ResolveError::Entry {
                    entry: match key {
                        SetKey::Run => run_set.name().to_string(),
                        SetKey::Id(id) => id.to_string(),
                        SetKey::Name(name) => name,
                    },
                    why,
                });
            }
            return Ok(opened);
        }
        let mut unique: Vec<DownloadRequest> = Vec::new();
        for request in asked {
            if !unique.iter().any(|r| r.url == request.url) {
                unique.push(request);
            }
        }
        let question = combined_request(&unique);
        match fetch(&question) {
            Ok(granted) => {
                fetcher = Some(Rc::from(granted));
                allowed.extend(unique.into_iter().map(|r| r.url));
            }
            Err(why) => {
                return Err(ResolveError::Declined {
                    count: unique.len(),
                    what: unique
                        .iter()
                        .map(|r| r.what.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    why,
                })
            }
        }
    }
}

/// The run's own density, as a request's PDF list is resolved against it:
/// its set, the tier the set was found in, and the lhaid and member the run
/// reads.
pub struct RunSet<'a> {
    pub set: &'a PdfSet,
    pub tier: Tier,
    pub lhaid: i64,
    pub member: u32,
}

impl<'a> RunSet<'a> {
    /// The run's density as `central` loaded it.
    pub fn of(central: &'a LoadedPdf) -> Self {
        let crate::artifact::PdfInput::Lhapdf { lhaid, member, .. } = &central.input else {
            unreachable!("systematics weights are resolved only for a run that reads a density");
        };
        RunSet {
            set: &central.set,
            tier: central.tier,
            lhaid: *lhaid,
            member: *member,
        }
    }
}

/// A request's PDF list resolved to slots against the sets' `.info` files,
/// before any member is read: everything the `<initrwgt>` block needs.
pub struct OpenedRequest {
    opened: BTreeMap<SetKey, OpenedSet>,
    /// Each slot as (set key, member index), the list's in order and then
    /// the fresh nominal when the list does not start with it.
    wanted: Vec<(SetKey, u32)>,
    list_len: usize,
    nominal: usize,
    sets: BTreeMap<i64, SetHeader>,
    run_member: u32,
}

/// Resolve `request`'s PDF list against the sets it names and the run's own
/// set, reading only their `.info` files.
pub fn open_request(
    request: &VariationRequest,
    run: &RunSet<'_>,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<OpenedRequest, ResolveError> {
    let (run_lhaid, run_member) = (run.lhaid, run.member);
    let targets: Vec<(SetKey, Members)> =
        request.pdfs.iter().map(|e| target(e, run_lhaid)).collect();
    let keys: BTreeSet<SetKey> = targets.iter().map(|(k, _)| k.clone()).collect();
    let opened = open_sets(&keys, run.set, run.tier, tiers, fetch)?;

    let mut wanted: Vec<(SetKey, u32)> = Vec::new();
    let mut sets = BTreeMap::new();
    for (entry, (key, members)) in request.pdfs.iter().zip(&targets) {
        let opened_set = &opened[key];
        let num = opened_set.set.info.num_members();
        let bad = |why: String| ResolveError::Entry {
            entry: entry_text(entry),
            why,
        };
        match members {
            Members::All => {
                sets.insert(
                    opened_set.set_index,
                    SetHeader {
                        name: opened_set.set.name().to_string(),
                        error_type: opened_set.set.info.error_type(),
                        description: opened_set.set.info.lhapdf_description(),
                    },
                );
                wanted.extend((0..num).map(|m| (key.clone(), m)));
            }
            Members::Id(id) => {
                let m = u32::try_from(id - opened_set.set_index)
                    .ok()
                    .filter(|&m| m < num)
                    .ok_or_else(|| {
                        bad(format!(
                            "lhaid {id} is not a member of {} (SetIndex {}, {num} members)",
                            opened_set.set.name(),
                            opened_set.set_index
                        ))
                    })?;
                wanted.push((key.clone(), m));
            }
            Members::Index(m) => {
                if *m >= num {
                    return Err(bad(format!("{} has {num} members", opened_set.set.name())));
                }
                wanted.push((key.clone(), *m));
            }
        }
    }
    let list_len = wanted.len();
    let first = slot_of_opened(&opened)(&wanted[0]);
    let nominal = if first.lhapdf_id == run_lhaid {
        0
    } else if list_len == 1 {
        wanted.push((SetKey::Run, run_member));
        1
    } else {
        return Err(ResolveError::NominalNotFirst {
            first: entry_text(&request.pdfs[0]),
            first_id: first.lhapdf_id,
            run: run_lhaid,
        });
    };
    let opened = if opened.contains_key(&SetKey::Run) {
        opened
    } else {
        let mut more = opened;
        more.extend(open_sets(
            &[SetKey::Run].into_iter().collect(),
            run.set,
            run.tier,
            tiers,
            fetch,
        )?);
        more
    };
    Ok(OpenedRequest {
        opened,
        wanted,
        list_len,
        nominal,
        sets,
        run_member,
    })
}

impl OpenedRequest {
    fn slots(&self) -> Vec<SlotId> {
        self.wanted
            .iter()
            .map(slot_of_opened(&self.opened))
            .collect()
    }

    /// The `<initrwgt>` lines between its tags that `request` writes over
    /// these sets: what [`Writer::header_lines`] gives once the members are
    /// loaded.
    pub fn header_lines(&self, request: &VariationRequest) -> Result<Vec<String>, String> {
        Ok(expanded(
            request,
            &self.slots(),
            self.list_len,
            self.nominal,
            &self.sets,
        )?
        .header)
    }

    /// Load every member the slots read, in parallel, checking the run's own
    /// files against what `central` read.
    pub fn load(self, central: &LoadedPdf) -> Result<ResolvedPdfs, ResolveError> {
        let opened = self.opened;
        if let Some(run) = opened.get(&SetKey::Run) {
            let (got, want) = (run.set.info_file(), central.set.info_file());
            if got.sha256 != want.sha256 {
                return Err(ResolveError::CentralChanged {
                    set: central.set.name().to_string(),
                    file: got.name.clone(),
                    got: got.sha256.clone(),
                    want: want.sha256.clone(),
                });
            }
        }
        let wanted = self.wanted;
        let distinct: BTreeSet<(SetKey, u32)> = wanted.iter().cloned().collect();
        let distinct: Vec<(SetKey, u32)> = distinct.into_iter().collect();
        let loaded: Vec<Result<MemberEval, ResolveError>> = distinct
            .par_iter()
            .map(|(key, m)| {
                let set = &opened[key].set;
                let member_error = |why: String| ResolveError::Member {
                    set: set.name().to_string(),
                    member: *m,
                    why,
                };
                let member = set.member(*m).map_err(|e| member_error(e.to_string()))?;
                let info = member
                    .alpha_s_info()
                    .ok_or_else(|| member_error("no alpha_s metadata".to_string()))?;
                let alphas =
                    GridAlphaS::from_info(info).map_err(|e| member_error(e.to_string()))?;
                Ok(MemberEval { member, alphas })
            })
            .collect();
        let mut evals_by: BTreeMap<(SetKey, u32), Arc<MemberEval>> = BTreeMap::new();
        for (key, eval) in distinct.iter().zip(loaded) {
            evals_by.insert(key.clone(), Arc::new(eval?));
        }
        if let (Some(eval), Some(want)) = (
            evals_by.get(&(SetKey::Run, self.run_member)),
            central.member.file(),
        ) {
            if let Some(got) = eval.member.file() {
                if got.sha256 != want.sha256 {
                    return Err(ResolveError::CentralChanged {
                        set: central.set.name().to_string(),
                        file: got.name.clone(),
                        got: got.sha256.clone(),
                        want: want.sha256.clone(),
                    });
                }
            }
        }

        let mut inputs: Vec<SystematicsSet> = Vec::new();
        let mut seen: Vec<SetKey> = Vec::new();
        for (key, _) in &wanted {
            if seen.contains(key) {
                continue;
            }
            seen.push(key.clone());
            let o = &opened[key];
            let mut files = vec![o.set.info_file().clone()];
            for ((k, _), eval) in &evals_by {
                if k == key {
                    files.extend(eval.member.file().cloned());
                }
            }
            inputs.push(SystematicsSet {
                set: o.set.name().to_string(),
                set_index: o.set_index,
                tier: o.found.tier,
                files,
            });
        }

        Ok(ResolvedPdfs {
            slots: wanted.iter().map(slot_of_opened(&opened)).collect(),
            evals: wanted.iter().map(|k| evals_by[k].clone()).collect(),
            list_len: self.list_len,
            nominal: self.nominal,
            sets: self.sets,
            inputs,
        })
    }
}

/// Resolve `request`'s PDF list against the sets it names, the run's own
/// density `central` among them, and load every member it reads.
pub fn resolve(
    request: &VariationRequest,
    central: &LoadedPdf,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<ResolvedPdfs, ResolveError> {
    open_request(request, &RunSet::of(central), tiers, fetch)?.load(central)
}

/// `request` expanded over `slots`: its variations, their ids and the
/// `<initrwgt>` lines, with no member attached yet.
fn expanded(
    request: &VariationRequest,
    slots: &[SlotId],
    list_len: usize,
    nominal: usize,
    sets: &BTreeMap<i64, SetHeader>,
) -> Result<Writer, String> {
    let args = expand(request, slots, list_len, nominal)?;
    let first_id = request.start_id.unwrap_or(1);
    let header = banner_lines(&args, slots, sets, nominal, first_id);
    let ids = (0..args.len() - 1)
        .map(|k| (first_id + k as i64).to_string())
        .collect();
    Ok(Writer {
        args,
        ids,
        header,
        evals: Vec::new(),
    })
}

fn slot_of_opened(opened: &BTreeMap<SetKey, OpenedSet>) -> impl Fn(&(SetKey, u32)) -> SlotId + '_ {
    move |(key, m)| SlotId {
        lhapdf_id: opened[key].set_index + i64::from(*m),
        member: *m,
    }
}

/// The variations of one request, their header, and the members they read:
/// what turns an event's record into its weights.
pub struct Writer {
    args: Vec<Arg>,
    ids: Vec<String>,
    header: Vec<String>,
    evals: Vec<Arc<MemberEval>>,
}

impl std::fmt::Debug for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer")
            .field("weights", &self.ids.len())
            .field("slots", &self.evals.len())
            .finish()
    }
}

impl Writer {
    /// Expand `request` over `pdfs` and write its header.
    pub fn new(request: &VariationRequest, pdfs: &ResolvedPdfs) -> Result<Self, String> {
        let mut writer = expanded(
            request,
            &pdfs.slots,
            pdfs.list_len,
            pdfs.nominal,
            &pdfs.sets,
        )?;
        writer.evals = pdfs.evals.clone();
        Ok(writer)
    }

    /// The weight ids, in the order every event lists them.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// The `<initrwgt>` lines between its tags.
    pub fn header_lines(&self) -> &[String] {
        &self.header
    }

    fn variation(&self, arg: &Arg) -> Variation<'_> {
        Variation {
            mur: arg.mur,
            muf: arg.muf,
            alps: arg.alps,
            dyn_scale: arg.dyn_scale,
            pdf: self.evals[arg.pdf].as_ref(),
        }
    }

    /// Every variation, the nominal first and then one per weight id: the
    /// arguments [`lo_weight`] is evaluated at.
    pub fn variations(&self) -> Vec<Variation<'_>> {
        self.args.iter().map(|arg| self.variation(arg)).collect()
    }

    /// `W` of every variation on one event, nominal first: what a ratio is
    /// taken of.
    pub fn raw_weights(
        &self,
        record: &LoRecord,
        kinematics: &RecordKinematics,
    ) -> Result<Vec<f64>, String> {
        let record = record.as_written();
        self.args
            .iter()
            .map(|arg| lo_weight(&record, kinematics, &self.variation(arg)))
            .collect()
    }

    /// The event's weight under each variation, `XWGTUP × W(var)/W(nominal)`,
    /// with `XWGTUP` and the record read as they are written.
    pub fn weights(
        &self,
        record: &LoRecord,
        kinematics: &RecordKinematics,
        xwgtup: f64,
    ) -> Result<Vec<f64>, String> {
        let raw = self.raw_weights(record, kinematics)?;
        let nominal = raw[0];
        if nominal == 0.0 || !nominal.is_finite() {
            return Err(format!(
                "the event's nominal weight W is {nominal}, so no variation has a ratio"
            ));
        }
        let xwgtup: f64 = format!("{xwgtup:+.7e}")
            .parse()
            .expect("a finite weight's spelling parses");
        Ok(raw[1..].iter().map(|w| xwgtup * w / nominal).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::cache::lhapdf::tests::{tar_gz, tiers, Factory, MapFetch};
    use crate::cache::store::lhapdf_download_url;
    use crate::pdf::select::load_for_lhaid;
    use crate::systematics::request::RequestSource;

    fn info(set_index: i64, members: u32, alpha_type: &str) -> String {
        format!(
            "SetDesc: \"synthetic => test\"\nSetIndex: {set_index}\nFormat: lhagrid1\n\
             NumMembers: {members}\nParticle: 2212\nFlavors: [-1, 1, 21]\nOrderQCD: 0\n\
             ErrorType: replicas\nXMin: 0.01\nXMax: 1\nQMin: 1\nQMax: 100\nAlphaS_MZ: 0.118\n\
             AlphaS_OrderQCD: 0\nAlphaS_Type: {alpha_type}\nAlphaS_Qs: [1.0, 10.0, 100.0]\n\
             AlphaS_Vals: [0.3, 0.2, 0.1]\nAlphaS_Lambda4: 0.3\nAlphaS_Lambda5: 0.2\n"
        )
    }

    fn dat() -> String {
        "PdfType: replica\nFormat: lhagrid1\n---\n0.01 1\n1 100\n-1 1 21\n0.1 0.2 0.3\n\
         0.4 0.5 0.6\n0.7 0.8 0.9\n1.0 1.1 1.2\n---\n"
            .to_string()
    }

    fn files(name: &str, set_index: i64, members: u32) -> Vec<(String, Vec<u8>)> {
        let mut out = vec![(
            format!("{name}.info"),
            info(set_index, members, "ipol").into_bytes(),
        )];
        for m in 0..members {
            out.push((format!("{name}_{m:04}.dat"), dat().into_bytes()));
        }
        out
    }

    fn write_set(base: &Path, name: &str, set_index: i64, members: u32) {
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        for (file, bytes) in files(name, set_index, members) {
            std::fs::write(dir.join(file), bytes).unwrap();
        }
    }

    /// Two local sets, `VgSysA` (the run's, 900000, three members) and
    /// `VgSysB` (910000, two), with an index listing both.
    fn local(root: &Path) -> Tiers {
        let data = root.join("data");
        write_set(&data, "VgSysA", 900000, 3);
        write_set(&data, "VgSysB", 910000, 2);
        std::fs::write(
            data.join("pdfsets.index"),
            "900000 VgSysA 3\n910000 VgSysB 2\n",
        )
        .unwrap();
        let mut t = tiers(root);
        t.lhapdf_data_path = vec![data];
        t
    }

    fn request(pdfs: &str) -> VariationRequest {
        VariationRequest::from_arguments(&[format!("--pdf={pdfs}")], RequestSource::CommandLine)
            .unwrap()
    }

    /// `errorset` is the run's own set, every member, the nominal first; the
    /// header records the set's `.info` and each member read, once.
    #[test]
    fn errorset_reads_the_runs_whole_set_with_the_nominal_first() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let central = load_for_lhaid(900000, &t, &fetch).unwrap();
        let resolved = resolve(&request("errorset"), &central, &t, &fetch).unwrap();
        assert_eq!((resolved.list_len, resolved.nominal), (3, 0));
        assert_eq!(resolved.inputs().len(), 1);
        assert_eq!(resolved.inputs()[0].files.len(), 4);
        assert_eq!(resolved.inputs()[0].set_index, 900000);
        let writer = Writer::new(&request("errorset"), &resolved).unwrap();
        // 44 scale weights, the re-inserted nominal and members 1 and 2.
        assert_eq!(writer.ids().len(), 47);
        assert!(factory.asked().is_empty());
    }

    /// Refusal table: two or more PDFs whose first is not the run's own.
    #[test]
    fn a_list_not_starting_with_the_nominal_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let central = load_for_lhaid(900000, &t, &fetch).unwrap();
        let e = resolve(&request("VgSysB,errorset"), &central, &t, &fetch)
            .err()
            .expect("refused");
        assert!(
            matches!(
                e,
                ResolveError::NominalNotFirst {
                    first_id: 910000,
                    run: 900000,
                    ..
                }
            ),
            "{e}"
        );
    }

    /// A single PDF not holding the nominal is honoured against a fresh
    /// nominal, which no list entry equals.
    #[test]
    fn a_single_pdf_without_the_nominal_reads_a_fresh_nominal() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let central = load_for_lhaid(900000, &t, &fetch).unwrap();
        let req = request("VgSysB@1");
        let resolved = resolve(&req, &central, &t, &fetch).unwrap();
        assert_eq!((resolved.list_len, resolved.nominal), (1, 1));
        let writer = Writer::new(&req, &resolved).unwrap();
        let last = writer.header_lines().last().unwrap();
        assert!(
            last.contains("PDF=\"910001\"") && last.contains("PDF=910000 MemberID=1"),
            "{last}"
        );
    }

    /// Refusal table: a set found nowhere and its download refused; and the
    /// downloads two missing sets need are asked for in one question.
    #[test]
    fn missing_sets_are_asked_for_once_and_a_refusal_stops_the_run() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let mut server = MapFetch::default();
        for (name, index) in [("VgSysC", 920000), ("VgSysD", 930000)] {
            server.files.insert(
                lhapdf_download_url(name),
                tar_gz(name, &files(name, index, 1)),
            );
        }
        let refusing = Factory::refusing();
        let central = load_for_lhaid(900000, &t, &refusing.callback()).unwrap();
        let req = request("errorset,VgSysC,VgSysD");
        let e = resolve(&req, &central, &t, &refusing.callback())
            .err()
            .expect("refused");
        assert!(matches!(e, ResolveError::Declined { count: 2, .. }), "{e}");
        assert_eq!(refusing.asked().len(), 1, "one question");

        let granting = Factory::granting(server);
        let resolved = resolve(&req, &central, &t, &granting.callback()).unwrap();
        let asked = granting.asked();
        assert_eq!(asked.len(), 1, "one question for both: {asked:?}");
        assert!(
            asked[0].what.contains("VgSysC") && asked[0].what.contains("VgSysD"),
            "{}",
            asked[0].what
        );
        assert_eq!(resolved.list_len, 5);
        assert_eq!(resolved.inputs().len(), 3);
        assert_eq!(resolved.inputs()[1].tier, Tier::Download);
    }

    /// Refusal table: a member whose resolved `AlphaS_Type` is not `ipol`,
    /// named by set and member.
    #[test]
    fn a_member_whose_coupling_is_not_tabulated_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let dir = t.lhapdf_data_path[0].join("VgSysB");
        std::fs::write(
            dir.join("VgSysB_0001.dat"),
            dat().replace(
                "PdfType: replica",
                "PdfType: replica\nAlphaS_Type: analytic",
            ),
        )
        .unwrap();
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let central = load_for_lhaid(900000, &t, &fetch).unwrap();
        let e = resolve(&request("VgSysB@1"), &central, &t, &fetch)
            .err()
            .expect("refused");
        assert!(
            matches!(&e, ResolveError::Member { set, member: 1, .. } if set == "VgSysB")
                && e.to_string().contains("analytic"),
            "{e}"
        );
    }

    /// A set named by an id that is not its first: the whole set, as
    /// `mkPDF(id).set()` gives it.
    #[test]
    fn an_id_naming_a_later_member_still_means_the_whole_set() {
        let root = tempfile::tempdir().unwrap();
        let t = local(root.path());
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let central = load_for_lhaid(900000, &t, &fetch).unwrap();
        let resolved = resolve(&request("errorset,910001"), &central, &t, &fetch).unwrap();
        assert_eq!(resolved.list_len, 5);
        assert!(resolved.sets.contains_key(&910000));
    }
}
