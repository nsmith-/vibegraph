//! Which PDF set and member a run reads: chosen from the run card at
//! integrate, re-read from the artifact's record at generate.
//!
//! A run card names its densities by an LHAPDF id, `lhaid`. The set is the one
//! whose id range `SetIndex .. SetIndex + NumMembers` holds it and the member is
//! `lhaid − SetIndex`. A set this build pins carries its range, so its ids
//! need no index; any other id is looked up in LHAPDF's `pdfsets.index`,
//! through the same tiers as a set ([`crate::cache::lhapdf`]), falling through
//! to the next tier's index when one does not cover the id and, last, to the
//! current index fetched with consent.
//!
//! What is recorded ([`PdfInput::Lhapdf`]) is the id, the set, the member and
//! the SHA-256 of the two files the loader parsed. [`load_recorded`] finds the
//! set by name again, wherever it now is, and refuses it unless those files
//! hash as recorded.

use std::path::Path;

use thiserror::Error;

use crate::artifact::{FileDigest, PdfInput};
use crate::cache::index::SetIndexTable;
use crate::cache::lhapdf::{
    fetch_index, locate_set, read_index, FetchFactory, FoundSet, LookupError, Tier, Tiers,
};
use crate::cache::pinned::PINNED_PDF_SETS;
use crate::runcard::{PdLabelRefused, PdfChoice, RunCard};

use super::grid::GridError;
use super::{PdfMember, PdfSet};

/// A loaded set and member, what the run records of them, and where they were
/// found.
#[derive(Debug)]
pub struct LoadedPdf {
    pub set: PdfSet,
    pub member: PdfMember,
    /// The record of this choice: a [`PdfInput::Lhapdf`] whose files are the
    /// ones just parsed.
    pub input: PdfInput,
    pub tier: Tier,
}

#[derive(Debug, Error)]
pub enum SelectFailure {
    #[error(transparent)]
    Label(#[from] PdLabelRefused),
    #[error(transparent)]
    Lookup(#[from] LookupError),
    #[error("cannot load PDF set {set} from {dir}: {source}")]
    Load {
        set: String,
        dir: String,
        source: GridError,
    },
    #[error("{0}")]
    Index(String),
    #[error("{}", index_unavailable(*.lhaid, .indices, .source))]
    IndexUnavailable {
        lhaid: i64,
        indices: String,
        source: LookupError,
    },
    #[error("lhaid {lhaid} is below every set in {indices}")]
    BelowEverySet { lhaid: i64, indices: String },
    #[error(
        "lhaid {lhaid} is in no known set: the nearest, {set} (SetIndex {set_index}), has \
         {members} members (ids {set_index}..{last}); indices read: {indices}",
        last = .set_index + i64::from(*.members) - 1
    )]
    NoKnownSet {
        lhaid: i64,
        set: String,
        set_index: i64,
        members: u32,
        indices: String,
    },
    #[error(
        "PDF set {set} at {dir} ({tier}) has no SetIndex in its .info, so lhaid {lhaid} cannot \
         be checked against it"
    )]
    NoSetIndex {
        set: String,
        dir: String,
        tier: Tier,
        lhaid: i64,
    },
    #[error(
        "{listed_by} gives PDF set {set} SetIndex {listed}, but its .info at {dir} ({tier}) says \
         {found}"
    )]
    SetIndexMismatch {
        set: String,
        listed_by: String,
        listed: i64,
        found: i64,
        dir: String,
        tier: Tier,
    },
    #[error(
        "lhaid {lhaid} is member {member} of PDF set {set} (SetIndex {set_index}), which has \
         {members} members"
    )]
    MemberOutOfRange {
        lhaid: i64,
        set: String,
        set_index: i64,
        member: u32,
        members: u32,
    },
    #[error(
        "PDF set {set} at {dir} ({tier}) has SetIndex {set_index}, so member {member} is lhaid \
         {got}, but the artifact recorded lhaid {lhaid}; re-integrate"
    )]
    RecordedIdMismatch {
        set: String,
        dir: String,
        tier: Tier,
        set_index: i64,
        member: u32,
        got: i64,
        lhaid: i64,
    },
    #[error(
        "PDF set {set} member {member} at {dir} ({tier}) is not the one the grid was trained on: \
         {file} has sha256 {got}, the artifact recorded {want}. The set changed after vibegraph \
         integrate ran (a CVMFS or LHAPDF update does this); re-integrate, or point \
         LHAPDF_DATA_PATH at the original files."
    )]
    Changed {
        set: String,
        member: u32,
        dir: String,
        tier: Tier,
        file: String,
        got: String,
        want: String,
    },
    #[error(
        "the artifact records the PDF files [{recorded}], but reading set {set} member {member} \
         reads [{read}]; re-integrate"
    )]
    RecordedFiles {
        set: String,
        member: u32,
        recorded: String,
        read: String,
    },
}

/// Why the densities a run reads could not be chosen or re-read. Boxed, since
/// it is returned once per run and its reasons carry paths and digests.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct PdfSelectError(Box<SelectFailure>);

impl PdfSelectError {
    /// The reason.
    pub fn kind(&self) -> &SelectFailure {
        &self.0
    }
}

impl<E: Into<SelectFailure>> From<E> for PdfSelectError {
    fn from(e: E) -> Self {
        PdfSelectError(Box::new(e.into()))
    }
}

/// The densities `card` reads: `None` on fixed-energy beams, where no density
/// is read and the label is inert; the set and member its `lhaid` names under
/// `pdlabel = lhapdf`; refused for every other label.
pub fn load_for_run_card(
    card: &RunCard,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<Option<LoadedPdf>, PdfSelectError> {
    match card.pdf_choice()? {
        PdfChoice::None => Ok(None),
        PdfChoice::Lhapdf { lhaid } => load_for_lhaid(lhaid, tiers, fetch).map(Some),
    }
}

/// The set and member LHAPDF id `lhaid` names.
pub fn load_for_lhaid(
    lhaid: i64,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<LoadedPdf, PdfSelectError> {
    if let Some(pin) = PINNED_PDF_SETS
        .iter()
        .find(|pin| (pin.set_index..pin.set_index + i64::from(pin.num_members)).contains(&lhaid))
    {
        let found = locate_set(pin.name, tiers, fetch)?;
        return load_in_set(lhaid, pin.name, pin.set_index, "this build's pin", found);
    }

    let mut read: Vec<String> = Vec::new();
    let mut nearest: Option<(String, i64, u32)> = None;
    let mut try_index =
        |path: &Path, read: &mut Vec<String>| -> Result<Option<LoadedPdf>, PdfSelectError> {
            let shown = path.display().to_string();
            if !read.contains(&shown) {
                read.push(shown);
            }
            let table = read_index(path).map_err(SelectFailure::Index)?;
            try_table(lhaid, &table, path, tiers, fetch, &mut nearest)
        };
    for (path, _) in tiers.local_indices() {
        if let Some(loaded) = try_index(&path, &mut read)? {
            return Ok(loaded);
        }
    }
    let fetched = fetch_index(tiers, fetch).map_err(|source| SelectFailure::IndexUnavailable {
        lhaid,
        indices: read.join(", "),
        source,
    })?;
    if let Some(loaded) = try_index(&fetched, &mut read)? {
        return Ok(loaded);
    }
    let indices = read.join(", ");
    Err(PdfSelectError::from(match nearest {
        Some((set, set_index, members)) => SelectFailure::NoKnownSet {
            lhaid,
            set,
            set_index,
            members,
            indices,
        },
        None => SelectFailure::BelowEverySet { lhaid, indices },
    }))
}

fn index_unavailable(lhaid: i64, indices: &str, why: &LookupError) -> String {
    if indices.is_empty() {
        format!(
            "no LHAPDF set index is available locally to look lhaid {lhaid} up in, and the \
             current one cannot be fetched: {why}"
        )
    } else {
        format!(
            "lhaid {lhaid} is in no set the indices read list ({indices}), and the current index \
             cannot be fetched: {why}"
        )
    }
}

/// `lhaid` against one index: `None` when the index has no set whose range
/// holds it, recording the nearest set below it in `nearest`.
fn try_table(
    lhaid: i64,
    table: &SetIndexTable,
    path: &Path,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
    nearest: &mut Option<(String, i64, u32)>,
) -> Result<Option<LoadedPdf>, PdfSelectError> {
    let Some(entry) = table.candidate(lhaid) else {
        return Ok(None);
    };
    let found = locate_set(&entry.name, tiers, fetch)?;
    let listed_by = path.display().to_string();
    match load_in_set(lhaid, &entry.name, entry.set_index, &listed_by, found) {
        Err(e) => match e.kind() {
            SelectFailure::MemberOutOfRange {
                set,
                set_index,
                members,
                ..
            } => {
                if nearest.as_ref().is_none_or(|(_, s, _)| set_index >= s) {
                    *nearest = Some((set.clone(), *set_index, *members));
                }
                Ok(None)
            }
            _ => Err(e),
        },
        Ok(loaded) => Ok(Some(loaded)),
    }
}

fn open(name: &str, found: &FoundSet) -> Result<PdfSet, PdfSelectError> {
    PdfSet::load(&found.dir, name)
        .map_err(|source| SelectFailure::Load {
            set: name.to_string(),
            dir: found.dir.display().to_string(),
            source,
        })
        .map_err(PdfSelectError::from)
}

fn set_index_of(set: &PdfSet, found: &FoundSet, lhaid: i64) -> Result<i64, PdfSelectError> {
    set.info
        .set_index()
        .ok_or_else(|| SelectFailure::NoSetIndex {
            set: set.name().to_string(),
            dir: found.dir.display().to_string(),
            tier: found.tier,
            lhaid,
        })
        .map_err(PdfSelectError::from)
}

fn load_member(set: &PdfSet, member: u32, found: &FoundSet) -> Result<PdfMember, PdfSelectError> {
    set.member(member)
        .map_err(|source| SelectFailure::Load {
            set: set.name().to_string(),
            dir: found.dir.display().to_string(),
            source,
        })
        .map_err(PdfSelectError::from)
}

/// Load the member of set `name`, found at `found`, that `lhaid` names, given
/// that `listed_by` lists the set at `listed` (an index, or this build's pin).
/// The set's own `.info` has to agree with that listing, and `lhaid` has to be
/// inside its member range.
fn load_in_set(
    lhaid: i64,
    name: &str,
    listed: i64,
    listed_by: &str,
    found: FoundSet,
) -> Result<LoadedPdf, PdfSelectError> {
    let set = open(name, &found)?;
    let set_index = set_index_of(&set, &found, lhaid)?;
    if set_index != listed {
        return Err(PdfSelectError::from(SelectFailure::SetIndexMismatch {
            set: name.to_string(),
            listed_by: listed_by.to_string(),
            listed,
            found: set_index,
            dir: found.dir.display().to_string(),
            tier: found.tier,
        }));
    }
    load_member_of(lhaid, set, set_index, found)
}

/// The member `lhaid − set_index` of `set`, refused unless it is one of the
/// set's members.
fn load_member_of(
    lhaid: i64,
    set: PdfSet,
    set_index: i64,
    found: FoundSet,
) -> Result<LoadedPdf, PdfSelectError> {
    let name = set.name().to_string();
    let members = set.info.num_members;
    let member = u32::try_from(lhaid - set_index)
        .ok()
        .filter(|&m| m < members)
        .ok_or_else(|| SelectFailure::MemberOutOfRange {
            lhaid,
            set: name.clone(),
            set_index,
            member: u32::try_from(lhaid - set_index).unwrap_or(u32::MAX),
            members,
        })?;
    let pdf = load_member(&set, member, &found)?;
    let input = PdfInput::Lhapdf {
        lhaid,
        set: name,
        member,
        files: read_files(&set, &pdf),
    };
    Ok(LoadedPdf {
        set,
        member: pdf,
        input,
        tier: found.tier,
    })
}

/// The files a load parsed, in load order.
fn read_files(set: &PdfSet, member: &PdfMember) -> Vec<FileDigest> {
    std::iter::once(set.info_file().clone())
        .chain(member.file().cloned())
        .collect()
}

/// The densities `input` records, found again by set name and checked against
/// the record: `.info` `SetIndex + member` must be the recorded `lhaid`, and
/// every recorded file must hash as recorded. `None` for a run that read none.
pub fn load_recorded(
    input: &PdfInput,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<Option<LoadedPdf>, PdfSelectError> {
    let PdfInput::Lhapdf {
        lhaid,
        set: name,
        member,
        files,
    } = input
    else {
        return Ok(None);
    };
    let (lhaid, member) = (*lhaid, *member);
    let found = locate_set(name, tiers, fetch)?;
    let set = open(name, &found)?;
    let changed = |file: &FileDigest, want: &FileDigest| SelectFailure::Changed {
        set: name.clone(),
        member,
        dir: found.dir.display().to_string(),
        tier: found.tier,
        file: file.name.clone(),
        got: file.sha256.clone(),
        want: want.sha256.clone(),
    };
    let names_differ = |read: &[FileDigest]| SelectFailure::RecordedFiles {
        set: name.clone(),
        member,
        recorded: names(files),
        read: names(read),
    };
    let info = set.info_file();
    match files.first() {
        Some(want) if want.name == info.name => {
            if want.sha256 != info.sha256 {
                return Err(PdfSelectError::from(changed(info, want)));
            }
        }
        _ => {
            return Err(PdfSelectError::from(names_differ(std::slice::from_ref(
                info,
            ))))
        }
    }
    let set_index = set_index_of(&set, &found, lhaid)?;
    let got = set_index + i64::from(member);
    if got != lhaid {
        return Err(PdfSelectError::from(SelectFailure::RecordedIdMismatch {
            set: name.clone(),
            dir: found.dir.display().to_string(),
            tier: found.tier,
            set_index,
            member,
            got,
            lhaid,
        }));
    }
    let pdf = load_member(&set, member, &found)?;
    let read = read_files(&set, &pdf);
    if read.len() != files.len() || read.iter().zip(files).any(|(r, w)| r.name != w.name) {
        return Err(PdfSelectError::from(names_differ(&read)));
    }
    if let Some((file, want)) = read.iter().zip(files).find(|(r, w)| r.sha256 != w.sha256) {
        return Err(PdfSelectError::from(changed(file, want)));
    }
    Ok(Some(LoadedPdf {
        set,
        member: pdf,
        input: input.clone(),
        tier: found.tier,
    }))
}

fn names(files: &[FileDigest]) -> String {
    files
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::index::INDEX_FILENAME;
    use crate::cache::lhapdf::tests::{tar_gz, tiers, Factory, MapFetch};
    use crate::cache::lhapdf::LHAPDF_SETS_URL;
    use crate::cache::pinned::DEFAULT_PDF_SET;
    use crate::ufo::identity::digest_bytes;
    use std::path::PathBuf;

    const SET: &str = "VgSyntheticSet";

    fn info(set_index: Option<&str>, members: u32) -> String {
        let set_index = set_index
            .map(|s| format!("SetIndex: {s}\n"))
            .unwrap_or_default();
        format!(
            "SetDesc: \"synthetic\"\n{set_index}Format: lhagrid1\nNumMembers: {members}\n\
             Particle: 2212\nFlavors: [-1, 1, 21]\nOrderQCD: 0\nErrorType: replicas\n\
             XMin: 0.01\nXMax: 1\nQMin: 1\nQMax: 100\nAlphaS_MZ: 0.118\nAlphaS_OrderQCD: 0\n\
             AlphaS_Type: ipol\nAlphaS_Qs: [1.0, 10.0, 100.0]\nAlphaS_Vals: [0.3, 0.2, 0.1]\n\
             AlphaS_Lambda4: 0.3\nAlphaS_Lambda5: 0.2\n"
        )
    }

    /// Member `m`'s grid: every knot `m + 1` times member 0's, so a reading
    /// says which member was loaded.
    fn dat(m: u32) -> String {
        let k = f64::from(m + 1);
        let mut text =
            String::from("PdfType: replica\nFormat: lhagrid1\n---\n0.01 1\n1 100\n-1 1 21\n");
        for row in [
            [0.1, 0.2, 0.3],
            [0.4, 0.5, 0.6],
            [0.7, 0.8, 0.9],
            [1.0, 1.1, 1.2],
        ] {
            text.push_str(&format!("{} {} {}\n", k * row[0], k * row[1], k * row[2]));
        }
        text.push_str("---\n");
        text
    }

    /// `<base>/<name>/` holding `.info` text and `members` member files.
    fn write_set(base: &Path, name: &str, info_text: &str, members: u32) -> PathBuf {
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}.info")), info_text).unwrap();
        for m in 0..members {
            std::fs::write(dir.join(format!("{name}_{m:04}.dat")), dat(m)).unwrap();
        }
        dir
    }

    fn write_index(dir: &Path, text: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(INDEX_FILENAME);
        std::fs::write(&path, text).unwrap();
        path
    }

    fn data_tiers(root: &Path) -> (Tiers, PathBuf) {
        let data = root.join("data");
        let mut t = tiers(root);
        t.lhapdf_data_path = vec![data.clone()];
        (t, data)
    }

    fn sha(path: &Path) -> String {
        digest_bytes(&std::fs::read(path).unwrap())
    }

    /// The id picks the set from the index and the member as `lhaid −
    /// SetIndex`: member 0 at the set's first id, member 2 at its third, read
    /// from that member's file. A `SetIndex` written with a trailing space, as
    /// CVMFS's 325300 `.info` is, reads the same.
    #[test]
    fn the_lhaid_picks_the_set_and_a_nonzero_member() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        write_index(&data, "100 Other 1\n900000 VgSyntheticSet 1\n");
        let dir = write_set(&data, SET, &info(Some("900000 "), 3), 3);
        let factory = Factory::refusing();
        let fetch = factory.callback();

        let zero = load_for_lhaid(900000, &t, &fetch).unwrap();
        let two = load_for_lhaid(900002, &t, &fetch).unwrap();
        assert_eq!(two.tier, Tier::LhapdfDataPath);
        let PdfInput::Lhapdf {
            lhaid,
            set,
            member,
            files,
        } = &two.input
        else {
            panic!("{:?}", two.input)
        };
        assert_eq!((*lhaid, set.as_str(), *member), (900002, SET, 2));
        assert_eq!(
            files,
            &vec![
                FileDigest {
                    name: format!("{SET}.info"),
                    sha256: sha(&dir.join(format!("{SET}.info"))),
                },
                FileDigest {
                    name: format!("{SET}_0002.dat"),
                    sha256: sha(&dir.join(format!("{SET}_0002.dat"))),
                },
            ]
        );
        let at = |p: &LoadedPdf| p.member.xfx_q2(1, 0.01, 1.0);
        assert_eq!(at(&two), 3.0 * at(&zero));
        assert!(factory.asked().is_empty());
    }

    /// Ids outside every listed range, each refused with the ids and the
    /// indices that were read: below the first set, and past the nearest
    /// set's last member. Neither is fetched from the server without consent,
    /// and the index fetch is the last thing tried.
    #[test]
    fn an_lhaid_in_no_known_set_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        let index = write_index(&data, "900000 VgSyntheticSet 1\n");
        write_set(&data, SET, &info(Some("900000"), 3), 3);
        let url = format!("{LHAPDF_SETS_URL}/{INDEX_FILENAME}");
        let mut server = MapFetch::default();
        server
            .files
            .insert(url, b"900000 VgSyntheticSet 1\n".to_vec());
        let factory = Factory::granting(server);
        let fetch = factory.callback();
        let fetched = t.cache_pdf_dir().join(INDEX_FILENAME);

        let below = load_for_lhaid(899999, &t, &fetch).unwrap_err().to_string();
        assert_eq!(
            below,
            format!(
                "lhaid 899999 is below every set in {}, {}",
                index.display(),
                fetched.display()
            )
        );
        let past = load_for_lhaid(900003, &t, &fetch).unwrap_err().to_string();
        assert_eq!(
            past,
            format!(
                "lhaid 900003 is in no known set: the nearest, VgSyntheticSet (SetIndex 900000), \
                 has 3 members (ids 900000..900002); indices read: {}, {}",
                index.display(),
                fetched.display()
            )
        );
        let asked: Vec<String> = factory.asked().into_iter().map(|r| r.what).collect();
        assert_eq!(
            asked,
            vec![
                "the LHAPDF set index pdfsets.index".to_string(),
                "the LHAPDF set index pdfsets.index".to_string()
            ]
        );

        let refusing = Factory::refusing();
        let fetch = refusing.callback();
        std::fs::remove_file(&fetched).unwrap();
        let err = load_for_lhaid(900003, &t, &fetch).unwrap_err().to_string();
        assert_eq!(
            err,
            format!(
                "lhaid 900003 is in no set the indices read list ({}), and the current index \
                 cannot be fetched: refused: the LHAPDF set index pdfsets.index",
                index.display()
            )
        );
        std::fs::remove_file(&index).unwrap();
        let err = load_for_lhaid(900003, &t, &fetch).unwrap_err().to_string();
        assert!(
            err.starts_with("no LHAPDF set index is available locally to look lhaid 900003 up in"),
            "{err}"
        );
    }

    /// An index that does not cover the id falls through to the next tier's
    /// index; the first covering one decides, and later tiers are not read.
    #[test]
    fn an_index_that_does_not_cover_the_id_falls_through() {
        let root = tempfile::tempdir().unwrap();
        let (mut t, data) = data_tiers(root.path());
        let cvmfs = root.path().join("cvmfs");
        write_index(&data, "900000 VgSyntheticSet 1\n");
        write_set(&data, SET, &info(Some("900000"), 3), 3);
        write_index(&cvmfs, "900000 VgSyntheticSet 1\n900100 VgNewer 1\n");
        write_set(&cvmfs, "VgNewer", &info(Some("900100"), 2), 2);
        write_index(&t.cache_pdf_dir(), "not an index\n");
        t.cvmfs = Some(cvmfs);
        let factory = Factory::refusing();

        let got = load_for_lhaid(900101, &t, &factory.callback()).unwrap();
        assert_eq!(got.tier, Tier::Cvmfs);
        assert!(
            matches!(&got.input, PdfInput::Lhapdf { set, member: 1, .. } if set == "VgNewer"),
            "{:?}",
            got.input
        );
        assert!(factory.asked().is_empty());
    }

    /// With no local index covering the id, the current index is fetched with
    /// consent into the cache, replacing a stale copy, and decides the set,
    /// which is then itself downloaded with consent.
    #[test]
    fn the_fetched_index_is_the_last_tier() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        write_index(&t.cache_pdf_dir(), "800000 VgStale 1\n");
        write_set(&data, "VgStale", &info(Some("800000"), 2), 2);
        let set_archive = {
            let files = vec![
                (format!("{SET}.info"), info(Some("900000"), 3).into_bytes()),
                (format!("{SET}_0001.dat"), dat(1).into_bytes()),
            ];
            tar_gz(SET, &files)
        };
        let mut server = MapFetch::default();
        server.files.insert(
            format!("{LHAPDF_SETS_URL}/{INDEX_FILENAME}"),
            b"800000 VgStale 1\n900000 VgSyntheticSet 1\n".to_vec(),
        );
        server
            .files
            .insert(format!("{LHAPDF_SETS_URL}/{SET}.tar.gz"), set_archive);
        let factory = Factory::granting(server);

        let got = load_for_lhaid(900001, &t, &factory.callback()).unwrap();
        assert_eq!(got.tier, Tier::Download);
        assert!(matches!(&got.input, PdfInput::Lhapdf { member: 1, .. }));
        let asked = factory.asked();
        assert_eq!(asked.len(), 2);
        assert_eq!(asked[0].what, "the LHAPDF set index pdfsets.index");
        assert_eq!(asked[1].what, format!("PDF set {SET}"));
        assert_eq!((asked[1].bytes, asked[1].sha256.as_deref()), (None, None));
    }

    /// The set's own `.info` decides: one without `SetIndex`, and one whose
    /// `SetIndex` disagrees with the index that listed it, are refused rather
    /// than trusted to the index.
    #[test]
    fn a_set_index_absent_or_disagreeing_with_the_index_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        write_index(&data, "900000 VgSyntheticSet 1\n");
        let factory = Factory::refusing();
        let fetch = factory.callback();

        write_set(&data, SET, &info(None, 3), 3);
        let err = load_for_lhaid(900001, &t, &fetch).unwrap_err();
        assert!(
            matches!(err.kind(), SelectFailure::NoSetIndex { .. }),
            "{err}"
        );

        write_set(&data, SET, &info(Some("900010"), 3), 3);
        let err = load_for_lhaid(900001, &t, &fetch).unwrap_err().to_string();
        assert!(
            err.contains("gives PDF set VgSyntheticSet SetIndex 900000, but its .info")
                && err.ends_with("(lhapdf_data_path) says 900010"),
            "{err}"
        );
    }

    /// An id in a pinned set's range needs no index: none exists anywhere and
    /// none is fetched.
    #[test]
    fn a_pinned_sets_ids_need_no_index() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        write_set(&data, DEFAULT_PDF_SET, &info(Some("247000"), 101), 2);
        let factory = Factory::refusing();
        let got = load_for_lhaid(247001, &t, &factory.callback()).unwrap();
        assert!(matches!(
            &got.input,
            PdfInput::Lhapdf {
                lhaid: 247001,
                member: 1,
                ..
            }
        ));
        assert!(factory.asked().is_empty(), "{:?}", factory.asked());
    }

    /// A proton card's built-in label is refused before anything is looked
    /// up; a fixed-energy card reads no density whatever its label.
    #[test]
    fn the_run_card_decides_whether_and_what_to_load() {
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let card = |text: &str| RunCard::parse(text).unwrap();

        let err = load_for_run_card(&card("nn23lo1 = pdlabel\n"), &t, &fetch).unwrap_err();
        assert!(matches!(err.kind(), SelectFailure::Label(_)), "{err}");
        let fixed = card("0 = lpp1\n0 = lpp2\nnn23lo1 = pdlabel\n");
        assert!(load_for_run_card(&fixed, &t, &fetch).unwrap().is_none());
        assert!(factory.asked().is_empty());
    }

    /// What integrate recorded reads back: the same files, wherever the set
    /// now is. A `.dat` with a comment line added parses to the same grid
    /// and is refused by its checksum, naming the file and both digests; so is
    /// a changed `.info`. A `SetIndex` that moved is refused by id.
    #[test]
    fn a_recorded_set_is_reread_and_refused_when_a_file_changed() {
        let root = tempfile::tempdir().unwrap();
        let (t, data) = data_tiers(root.path());
        write_index(&data, "900000 VgSyntheticSet 1\n");
        let dir = write_set(&data, SET, &info(Some("900000"), 3), 3);
        let factory = Factory::refusing();
        let fetch = factory.callback();
        let recorded = load_for_lhaid(900001, &t, &fetch).unwrap().input;

        let elsewhere = root.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        std::fs::rename(&dir, elsewhere.join(SET)).unwrap();
        let mut moved = t.clone();
        moved.lhapdf_data_path = vec![elsewhere.clone()];
        moved.data_path_tier = Tier::Lhapath;
        let again = load_recorded(&recorded, &moved, &fetch).unwrap().unwrap();
        assert_eq!(again.tier, Tier::Lhapath);
        assert_eq!(again.input, recorded);

        let dat_path = elsewhere.join(SET).join(format!("{SET}_0001.dat"));
        let original = std::fs::read(&dat_path).unwrap();
        let mut edited = b"# a comment line\n".to_vec();
        edited.extend_from_slice(&original);
        std::fs::write(&dat_path, &edited).unwrap();
        let err = load_recorded(&recorded, &moved, &fetch)
            .unwrap_err()
            .to_string();
        assert_eq!(
            err,
            format!(
                "PDF set {SET} member 1 at {} (lhapath) is not the one the grid was trained on: \
                 {SET}_0001.dat has sha256 {}, the artifact recorded {}. The set changed after \
                 vibegraph integrate ran (a CVMFS or LHAPDF update does this); re-integrate, or \
                 point LHAPDF_DATA_PATH at the original files.",
                elsewhere.join(SET).display(),
                digest_bytes(&edited),
                digest_bytes(&original)
            )
        );
        std::fs::write(&dat_path, &original).unwrap();

        let info_path = elsewhere.join(SET).join(format!("{SET}.info"));
        std::fs::write(&info_path, info(Some("900000"), 4)).unwrap();
        let err = load_recorded(&recorded, &moved, &fetch)
            .unwrap_err()
            .to_string();
        assert!(err.contains(&format!("{SET}.info has sha256")), "{err}");
        // A `.info` whose SetIndex moved is a changed file too, reported as one
        // rather than as the id mismatch it also causes.
        std::fs::write(&info_path, info(Some("900001"), 3)).unwrap();
        let err = load_recorded(&recorded, &moved, &fetch)
            .unwrap_err()
            .to_string();
        assert!(err.contains(&format!("{SET}.info has sha256")), "{err}");

        let PdfInput::Lhapdf {
            set, member, files, ..
        } = recorded.clone()
        else {
            unreachable!()
        };
        std::fs::write(&info_path, info(Some("900000"), 3)).unwrap();
        let shifted = PdfInput::Lhapdf {
            lhaid: 900002,
            set,
            member,
            files,
        };
        let err = load_recorded(&shifted, &moved, &fetch).unwrap_err();
        assert!(
            matches!(
                err.kind(),
                SelectFailure::RecordedIdMismatch {
                    got: 900001,
                    lhaid: 900002,
                    ..
                }
            ),
            "{err}"
        );
        assert!(load_recorded(&PdfInput::None, &moved, &fetch)
            .unwrap()
            .is_none());
    }
}
