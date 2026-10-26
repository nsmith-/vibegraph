//! LHAPDF's `pdfsets.index`: the table from a set's first LHAPDF id to its name.
//!
//! Each non-blank line is `<SetIndex> <name> <version>`. A set's ids are the
//! range `SetIndex .. SetIndex + NumMembers`, and the ranges of different sets
//! are disjoint, so the set an id belongs to is the entry with the largest
//! `SetIndex` at or below it — provided the id is also below that set's
//! `SetIndex + NumMembers`, which only the set's own `.info` can say.

use std::path::Path;

/// The file name LHAPDF gives its index, in every directory that carries one.
pub const INDEX_FILENAME: &str = "pdfsets.index";

/// One line of an index: a set's first LHAPDF id and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub set_index: i64,
    pub name: String,
}

/// A line of an index that is not `<integer> <name> …`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}:{line}: not a pdfsets.index line (`<SetIndex> <name> <version>`): {text:?}")]
pub struct IndexError {
    pub path: String,
    pub line: usize,
    pub text: String,
}

/// A parsed index, its entries in increasing `SetIndex`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetIndexTable {
    entries: Vec<IndexEntry>,
}

impl SetIndexTable {
    /// Parse an index's text; `path` names it in an error. Blank lines and
    /// `#` comments are skipped, and the version column is not read.
    pub fn parse(text: &str, path: &Path) -> Result<Self, IndexError> {
        let mut entries = Vec::new();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let bad = || IndexError {
                path: path.display().to_string(),
                line: n + 1,
                text: raw.to_string(),
            };
            let mut fields = line.split_whitespace();
            let set_index = fields
                .next()
                .and_then(|id| id.parse::<i64>().ok())
                .ok_or_else(bad)?;
            let name = fields.next().ok_or_else(bad)?.to_string();
            entries.push(IndexEntry { set_index, name });
        }
        entries.sort_by_key(|e| e.set_index);
        Ok(SetIndexTable { entries })
    }

    /// The entry whose range could hold `lhaid`: the one with the largest
    /// `SetIndex ≤ lhaid`, or `None` when `lhaid` is below every entry.
    pub fn candidate(&self, lhaid: i64) -> Option<&IndexEntry> {
        let above = self.entries.partition_point(|e| e.set_index <= lhaid);
        above.checked_sub(1).map(|i| &self.entries[i])
    }

    /// The entry named `name`, if the index lists it.
    pub fn by_name(&self, name: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(text: &str) -> SetIndexTable {
        SetIndexTable::parse(text, Path::new("pdfsets.index")).unwrap()
    }

    /// The candidate is the largest `SetIndex` at or below the id, whatever
    /// order the file lists the sets in.
    #[test]
    fn the_candidate_is_the_largest_set_index_at_or_below_the_id() {
        let t = table("325300 NNPDF31_nnlo_as_0118_mc_hessian_pdfas 1\n\n# c\n247000 NNPDF23_lo_as_0130_qed 1\n306000 NNPDF31_nnlo_hessian_pdfas 1\n");
        let at = |id| t.candidate(id).map(|e| (e.set_index, e.name.as_str()));
        assert_eq!(at(246999), None);
        assert_eq!(at(247000), Some((247000, "NNPDF23_lo_as_0130_qed")));
        assert_eq!(at(247100), Some((247000, "NNPDF23_lo_as_0130_qed")));
        assert_eq!(at(305999), Some((247000, "NNPDF23_lo_as_0130_qed")));
        assert_eq!(at(306000), Some((306000, "NNPDF31_nnlo_hessian_pdfas")));
        assert_eq!(
            at(400000),
            Some((325300, "NNPDF31_nnlo_as_0118_mc_hessian_pdfas"))
        );
        assert_eq!(
            t.by_name("NNPDF31_nnlo_hessian_pdfas").map(|e| e.set_index),
            Some(306000)
        );
    }

    /// A line without an integer id and a name is refused naming the file and
    /// the line, rather than skipped: a skipped line would shift an id onto
    /// its neighbouring set.
    #[test]
    fn a_malformed_line_is_refused_with_its_position() {
        for text in ["247000 A 1\nnot-a-number B 1\n", "247000 A 1\n306000\n"] {
            let err = SetIndexTable::parse(text, Path::new("/x/pdfsets.index")).unwrap_err();
            assert_eq!((err.path.as_str(), err.line), ("/x/pdfsets.index", 2));
        }
    }
}
